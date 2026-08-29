//! Text-only bridge between DoubaoWork's native main chat and an
//! OpenAI-compatible Chat Completions endpoint.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::live;
use crate::ws::WebSocket;

pub const DEFAULT_PORT: u16 = 18_766;
pub const BRIDGE_PATH: &str = "/v1/doubao/chat/completion";
const MARKER_HEADER: &str = "x-doubao-protocol-bridge";
const MARKER_VALUE: &str = "text-v1";
const ALLOWED_ORIGIN: &str = "chrome://doubaowork-chat";
const MAX_HEADER_BYTES: usize = 32 * 1024;
const MAX_BODY_BYTES: usize = 128 * 1024;

#[derive(Clone, Debug)]
pub struct Config {
    pub bridge_port: u16,
    pub cdp_port: u16,
    pub upstream: Option<String>,
    pub model: String,
    pub api_key_env: String,
    pub only_prompt: Option<String>,
    pub once: bool,
    pub ttl: Duration,
    pub mock_parts: Vec<String>,
    pub mock_delay: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bridge_port: DEFAULT_PORT,
            cdp_port: live::DEFAULT_PORT,
            upstream: None,
            model: "mock".into(),
            api_key_env: "DOUBAO_MODEL_API_KEY".into(),
            only_prompt: None,
            once: false,
            ttl: Duration::from_secs(180),
            mock_parts: vec!["PROTOCOL_".into(), "BRIDGE_".into(), "STREAM_OK".into()],
            mock_delay: Duration::from_millis(650),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunStats {
    pub intercepted: u64,
    pub accepted: u64,
    pub completed: u64,
    pub chunks: u64,
    pub content_chars: usize,
    pub first_delta_ms: Option<u128>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug)]
struct TurnContext {
    openai_request: Value,
    original_blocks: Vec<Value>,
    conversation_id: String,
    local_conversation_id: String,
    section_id: String,
    bot_id: String,
    query_local_id: String,
    user_message_index: i64,
    assistant_message_index: i64,
}

fn decode_request(request: &Value, model: &str) -> Result<TurnContext, String> {
    let root = request
        .as_object()
        .ok_or_else(|| "request body must be a JSON object".to_string())?;
    let messages = root
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| "text bridge requires exactly one message".to_string())?;
    if messages.len() != 1 {
        return Err("text bridge requires exactly one message".into());
    }
    let message = messages[0]
        .as_object()
        .ok_or_else(|| "message must be an object".to_string())?;
    let blocks = message
        .get("content_block")
        .and_then(Value::as_array)
        .ok_or_else(|| "message contains no content blocks".to_string())?;
    if blocks.is_empty() {
        return Err("message contains no content blocks".into());
    }

    let mut text_parts = Vec::with_capacity(blocks.len());
    let mut clean_blocks = Vec::with_capacity(blocks.len());
    for block in blocks {
        if block.get("block_type").and_then(Value::as_i64) != Some(10_000) {
            return Err("only plain-text content blocks are supported".into());
        }
        let text = block
            .pointer("/content/text_block/text")
            .and_then(Value::as_str)
            .ok_or_else(|| "plain-text block is empty".to_string())?;
        if text.trim().is_empty() {
            return Err("plain-text block is empty".into());
        }
        text_parts.push(text);
        clean_blocks.push(json!({
            "block_type": 10000,
            "block_id": block.get("block_id").and_then(Value::as_str)
                .map(str::to_owned).unwrap_or_else(new_numeric_id),
            "content": {"text_block": {"text": text}},
            "is_finish": false
        }));
    }
    if text_parts
        .iter()
        .map(|part| part.chars().count())
        .sum::<usize>()
        > 32_768
    {
        return Err("message text exceeds 32768 characters".into());
    }
    if !matches!(root.get("user_context"), None | Some(Value::Null))
        && root
            .get("user_context")
            .and_then(Value::as_array)
            .is_none_or(|v| !v.is_empty())
    {
        return Err("user_context is not supported".into());
    }
    let option = root.get("option").and_then(Value::as_object);
    if let Some(connectors) = option
        .and_then(|value| value.get("connector_info_list"))
        .and_then(Value::as_array)
    {
        if !connectors.is_empty() {
            return Err("connectors are not supported".into());
        }
    }
    let task = option
        .and_then(|value| value.get("general_task_param"))
        .and_then(Value::as_object);
    for key in [
        "selected_skills",
        "skill_selections",
        "attachments",
        "references",
    ] {
        if task
            .and_then(|value| value.get(key))
            .is_some_and(has_nonempty_value)
        {
            return Err(format!("{key} is not supported"));
        }
    }

    let client = root.get("client_meta").and_then(Value::as_object);
    let conversation_id = client
        .and_then(|value| value.get("conversation_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(new_numeric_id);
    let section_id = client
        .and_then(|value| value.get("last_section_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(new_numeric_id);
    let local_conversation_id = client
        .and_then(|value| value.get("local_conversation_id"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("local_{}", unix_millis()));
    let query_local_id = message
        .get("local_message_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "message local_message_id is missing".to_string())?
        .to_owned();
    let last_index = client
        .and_then(|value| value.get("last_message_index"))
        .and_then(Value::as_i64)
        .unwrap_or(-1);
    let text = text_parts.join("\n\n");

    Ok(TurnContext {
        openai_request: json!({
            "model": model,
            "messages": [{"role": "user", "content": text}],
            "stream": true
        }),
        original_blocks: clean_blocks,
        conversation_id,
        local_conversation_id,
        section_id,
        bot_id: client
            .and_then(|value| value.get("bot_id"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        query_local_id,
        user_message_index: last_index + 1,
        assistant_message_index: last_index + 2,
    })
}

fn has_nonempty_value(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
        Value::String(value) => !value.is_empty(),
        _ => true,
    }
}

struct DoubaoSseEncoder {
    turn: TurnContext,
    question_id: String,
    message_id: String,
    reply_unique_key: String,
    block_id: String,
    event_id: u64,
    answer: String,
}

impl DoubaoSseEncoder {
    fn new(turn: TurnContext) -> Self {
        Self {
            turn,
            question_id: new_numeric_id(),
            message_id: new_numeric_id(),
            reply_unique_key: format!("bridge-{}", new_numeric_id()),
            block_id: new_numeric_id(),
            event_id: 0,
            answer: String::new(),
        }
    }

    fn start(&mut self) -> Vec<String> {
        let now = unix_seconds();
        let version = unix_micros();
        let user_message = json!({
            "conversation_id": self.turn.conversation_id,
            "message_id": self.question_id,
            "message_body_version": 1,
            "sender_id": "",
            "user_type": 1,
            "content_type": 9999,
            "content": serde_json::to_string(&self.turn.original_blocks).unwrap_or_default(),
            "index_in_conv": self.turn.user_message_index,
            "create_time": now,
            "biz_content_type": "",
            "content_block": self.turn.original_blocks,
            "tts_content": "",
            "update_time": now,
            "ext": {
                "bot_id": self.turn.bot_id,
                "reply_unique_key": self.reply_unique_key,
                "use_content_block": "1",
                "is_finish": "0"
            },
            "local_message_id": self.turn.query_local_id,
            "section_id": self.turn.section_id,
            "bot_reply_message_id": "0",
            "fetch_token": self.question_id
        });
        vec![
            self.event("SSE_HEARTBEAT", json!({})),
            self.event(
                "SSE_ACK",
                json!({
                    "query_list": [{
                        "question_id": self.question_id,
                        "local_message_id": self.turn.query_local_id,
                        "message_index": self.turn.user_message_index
                    }],
                    "ack_client_meta": {
                        "conversation_id": self.turn.conversation_id,
                        "local_conversation_id": self.turn.local_conversation_id,
                        "conversation_type": 3,
                        "section_id": self.turn.section_id
                    },
                    "timeout_conf": {
                        "answer_first_pending_time": 180000,
                        "packet_interval_time": 120000,
                        "max_retry_count": 0,
                        "max_retry_duration_ms": 0,
                        "retry_interval_ms": 1000
                    }
                }),
            ),
            self.event(
                "FULL_MSG_NOTIFY",
                json!({
                    "message": user_message,
                    "message_attr": {
                        "badge_count": 1,
                        "read_badge_count": 1,
                        "read_conv_version": version,
                        "pre_read_conv_version": version
                    }
                }),
            ),
            self.event(
                "STREAM_TIMEOUT_CONTROL",
                json!({
                    "next_chunk_pending": 120000,
                    "max_retry_count": 0,
                    "max_retry_duration_ms": 0,
                    "retry_interval_ms": 1000
                }),
            ),
            self.event(
                "STREAM_MSG_NOTIFY",
                json!({
                    "content": {
                        "content_block": [{
                            "block_type": 10000,
                            "block_id": self.block_id,
                            "content": {"text_block": {"text": ""}},
                            "is_finish": false,
                            "patch_type": 2
                        }],
                        "content_status": 100,
                        "ext": {
                            "reply_unique_key": self.reply_unique_key,
                            "use_content_block": "1",
                            "is_general_task": "1"
                        },
                        "content_type": 9999
                    },
                    "meta": {
                        "message_id": self.message_id,
                        "conversation_id": self.turn.conversation_id,
                        "section_id": self.turn.section_id,
                        "sender_id": self.turn.bot_id,
                        "user_type": 2,
                        "create_time": now,
                        "index_in_conv": self.turn.assistant_message_index,
                        "bot_reply_message_id": self.question_id,
                        "local_conversation_id": self.turn.local_conversation_id
                    },
                    "attr": {"reply_unique_key": self.reply_unique_key}
                }),
            ),
        ]
    }

    fn delta(&mut self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }
        self.answer.push_str(text);
        vec![
            self.event(
                "STREAM_CHUNK",
                json!({
                    "message_id": self.message_id,
                    "patch_op": [{
                        "patch_object": 1,
                        "patch_type": 1,
                        "patch_value": {
                            "content_block": [{
                                "block_type": 10000,
                                "block_id": self.block_id,
                                "content": {"text_block": {"text": text}},
                                "is_finish": false,
                                "patch_type": 1
                            }]
                        }
                    }]
                }),
            ),
            self.event(
                "STREAM_CHUNK",
                json!({
                    "message_id": self.message_id,
                    "patch_op": [{
                        "patch_object": 111,
                        "patch_type": 1,
                        "patch_value": {"tts_content": text}
                    }]
                }),
            ),
        ]
    }

    fn finish(&mut self) -> Vec<String> {
        let version = unix_micros();
        let brief: String = self.answer.chars().take(160).collect();
        vec![
            self.event(
                "STREAM_CHUNK",
                json!({
                    "message_id": self.message_id,
                    "patch_op": [{
                        "patch_object": 1,
                        "patch_type": 1,
                        "patch_value": {
                            "content_block": [{
                                "block_type": 10000,
                                "block_id": self.block_id,
                                "content": {"text_block": {}},
                                "is_finish": true,
                                "patch_type": 1
                            }]
                        }
                    }]
                }),
            ),
            self.event(
                "STREAM_CHUNK",
                json!({
                    "message_id": self.message_id,
                    "patch_op": [
                        {"patch_object": 3, "patch_type": 2, "patch_value": {}},
                        {"patch_object": 50, "patch_type": 1,
                         "patch_value": {"ext": {"is_finish": "1"}}}
                    ]
                }),
            ),
            self.event(
                "SSE_REPLY_END",
                json!({
                    "end_type": 1,
                    "msg_finish_attr": {
                        "msgid": self.message_id,
                        "badge_count": 1,
                        "read_badge_count": 1,
                        "read_conv_version": version,
                        "pre_read_conv_version": version,
                        "brief": brief
                    }
                }),
            ),
            self.event(
                "SSE_REPLY_END",
                json!({"end_type": 2, "answer_finish_attr": {"has_suggest": false}}),
            ),
            self.event("SSE_REPLY_END", json!({"end_type": 3})),
        ]
    }

    fn event(&mut self, name: &str, data: Value) -> String {
        let id = self.event_id;
        self.event_id += 1;
        format!("id: {id}\nevent: {name}\ndata: {data}\n\n")
    }
}

fn stream_openai<R: Read, F: FnMut(&str) -> Result<(), String>>(
    reader: R,
    mut on_delta: F,
) -> Result<(), String> {
    let mut data_lines = Vec::new();
    let mut terminal = false;
    for line in BufReader::new(reader).lines() {
        let line = line.map_err(|_| "OpenAI stream is unavailable".to_string())?;
        let line = line.strip_suffix('\r').unwrap_or(&line);
        if line.is_empty() {
            process_openai_block(&data_lines, &mut terminal, &mut on_delta)?;
            data_lines.clear();
        } else if let Some(data) = line.strip_prefix("data:") {
            data_lines.push(data.trim_start().to_owned());
        }
    }
    if !data_lines.is_empty() {
        process_openai_block(&data_lines, &mut terminal, &mut on_delta)?;
    }
    if !terminal {
        return Err("OpenAI stream ended before a completion marker".into());
    }
    Ok(())
}

fn process_openai_block<F: FnMut(&str) -> Result<(), String>>(
    data_lines: &[String],
    terminal: &mut bool,
    on_delta: &mut F,
) -> Result<(), String> {
    if data_lines.is_empty() {
        return Ok(());
    }
    let raw = data_lines.join("\n");
    if raw == "[DONE]" {
        *terminal = true;
        return Ok(());
    }
    let Ok(payload) = serde_json::from_str::<Value>(&raw) else {
        return Ok(());
    };
    let Some(choice) = payload
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
    else {
        return Ok(());
    };
    if let Some(text) = choice.pointer("/delta/content").and_then(Value::as_str) {
        if !text.is_empty() {
            on_delta(text)?;
        }
    }
    if choice
        .get("finish_reason")
        .is_some_and(|value| !value.is_null())
    {
        *terminal = true;
    }
    Ok(())
}

#[derive(Default)]
struct Shared {
    stats: Mutex<RunStats>,
    done: AtomicBool,
    request_started: Mutex<Option<Instant>>,
}

struct ServerHandle {
    port: u16,
    stop: Arc<AtomicBool>,
    thread: thread::JoinHandle<()>,
}

impl ServerHandle {
    fn stop(self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.thread.join();
    }
}

fn start_server(config: Config, shared: Arc<Shared>) -> Result<ServerHandle, String> {
    let listener = TcpListener::bind(("127.0.0.1", config.bridge_port))
        .map_err(|error| format!("cannot bind protocol bridge: {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let thread = thread::spawn(move || {
        while !thread_stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Err(error) = handle_connection(stream, &config, &shared) {
                        if !shared.done.load(Ordering::Relaxed) {
                            set_error(&shared, error);
                        }
                    }
                    if config.once && shared.done.load(Ordering::Relaxed) {
                        break;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
                }
                Err(error) => {
                    set_error(&shared, format!("protocol bridge listener failed: {error}"));
                    break;
                }
            }
        }
    });
    Ok(ServerHandle { port, stop, thread })
}

fn handle_connection(
    mut stream: TcpStream,
    config: &Config,
    shared: &Arc<Shared>,
) -> Result<(), String> {
    stream.set_nodelay(true).ok();
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();
    let request = match read_http_request(&mut stream) {
        Ok(request) => request,
        Err(error) => {
            let _ = write_error(&mut stream, 400, &error);
            return Err(error);
        }
    };
    if request.method != "POST" || request.path != BRIDGE_PATH {
        write_error(&mut stream, 404, "not found")?;
        return Ok(());
    }
    let host = request
        .headers
        .get("host")
        .and_then(|value| value.split(':').next())
        .unwrap_or("");
    if !matches!(host, "127.0.0.1" | "localhost")
        || request.headers.get("origin").map(String::as_str) != Some(ALLOWED_ORIGIN)
        || request.headers.get(MARKER_HEADER).map(String::as_str) != Some(MARKER_VALUE)
    {
        write_error(&mut stream, 403, "request is not allowed")?;
        return Ok(());
    }
    let incoming: Value = match serde_json::from_slice(&request.body) {
        Ok(value) => value,
        Err(_) => {
            let error = "request body is not valid JSON".to_string();
            set_error(shared, error.clone());
            write_error(&mut stream, 400, &error)?;
            return Err(error);
        }
    };
    let turn = match decode_request(&incoming, &config.model) {
        Ok(turn) => turn,
        Err(error) => {
            set_error(shared, error.clone());
            write_error(&mut stream, 400, &error)?;
            return Err(error);
        }
    };
    {
        let mut stats = shared.stats.lock().unwrap();
        stats.accepted += 1;
        stats.content_chars = turn
            .openai_request
            .pointer("/messages/0/content")
            .and_then(Value::as_str)
            .map(str::chars)
            .map(Iterator::count)
            .unwrap_or(0);
    }
    write_sse_headers(&mut stream)?;
    let mut encoder = DoubaoSseEncoder::new(turn.clone());
    write_frames(&mut stream, encoder.start())?;
    let started = shared
        .request_started
        .lock()
        .unwrap()
        .unwrap_or_else(Instant::now);
    let mut chunks = 0u64;
    let mut first_delta_ms = None;
    let stream_result = {
        let mut emit = |text: &str| -> Result<(), String> {
            if first_delta_ms.is_none() {
                first_delta_ms = Some(started.elapsed().as_millis());
            }
            write_frames(&mut stream, encoder.delta(text))?;
            chunks += 1;
            Ok(())
        };
        if let Some(endpoint) = config.upstream.as_deref() {
            stream_upstream(endpoint, config, &turn.openai_request, &mut emit)
        } else {
            for part in &config.mock_parts {
                emit(part)?;
                thread::sleep(config.mock_delay);
            }
            Ok(())
        }
    };
    if let Err(error) = stream_result {
        set_error(shared, error.clone());
        return Err(error);
    }
    if chunks == 0 {
        let error = "OpenAI stream returned no text".to_string();
        set_error(shared, error.clone());
        return Err(error);
    }
    write_frames(&mut stream, encoder.finish())?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|error| error.to_string())?;
    {
        let mut stats = shared.stats.lock().unwrap();
        stats.completed += 1;
        stats.chunks = chunks;
        stats.first_delta_ms = first_delta_ms;
    }
    shared.done.store(true, Ordering::Relaxed);
    Ok(())
}

fn stream_upstream<F: FnMut(&str) -> Result<(), String>>(
    endpoint: &str,
    config: &Config,
    payload: &Value,
    emit: &mut F,
) -> Result<(), String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_recv_body(Some(Duration::from_secs(120)))
        .build()
        .into();
    let mut request = agent
        .post(endpoint)
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream");
    if let Ok(api_key) = std::env::var(&config.api_key_env) {
        if !api_key.is_empty() {
            request = request.header("Authorization", &format!("Bearer {api_key}"));
        }
    }
    let response = match request.send(payload.to_string()) {
        Ok(response) => response,
        Err(ureq::Error::StatusCode(code)) => {
            return Err(format!("OpenAI upstream returned HTTP {code}"));
        }
        Err(_) => {
            return Err("OpenAI upstream is unavailable".into());
        }
    };
    stream_openai(response.into_body().into_reader(), emit)
}

struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut raw = Vec::new();
    let header_end = loop {
        if let Some(position) = raw.windows(4).position(|window| window == b"\r\n\r\n") {
            break position;
        }
        if raw.len() >= MAX_HEADER_BYTES {
            return Err("request headers are too large".into());
        }
        let mut chunk = [0u8; 4096];
        let read = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("request ended during headers".into());
        }
        raw.extend_from_slice(&chunk[..read]);
    };
    let head = std::str::from_utf8(&raw[..header_end])
        .map_err(|_| "request headers are not UTF-8".to_string())?;
    let mut lines = head.split("\r\n");
    let first = lines
        .next()
        .ok_or_else(|| "request line is missing".to_string())?;
    let mut request_line = first.split_whitespace();
    let method = request_line.next().unwrap_or("").to_owned();
    let path = request_line
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .to_owned();
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    let content_length = headers
        .get("content-length")
        .ok_or_else(|| "content length is missing".to_string())?
        .parse::<usize>()
        .map_err(|_| "invalid content length".to_string())?;
    if content_length == 0 || content_length > MAX_BODY_BYTES {
        return Err("request body is empty or too large".into());
    }
    let mut body = raw[header_end + 4..].to_vec();
    while body.len() < content_length {
        let mut chunk = [0u8; 8192];
        let read = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("request ended during body".into());
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(content_length);
    Ok(HttpRequest {
        method,
        path,
        headers,
        body,
    })
}

fn write_sse_headers(stream: &mut TcpStream) -> Result<(), String> {
    stream
        .write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\n\
                 Cache-Control: no-cache\r\nX-Accel-Buffering: no\r\nConnection: close\r\n\
                 Access-Control-Allow-Origin: {ALLOWED_ORIGIN}\r\n\
                 Access-Control-Allow-Private-Network: true\r\nVary: Origin\r\n\r\n"
            )
            .as_bytes(),
        )
        .map_err(|error| error.to_string())
}

fn write_error(stream: &mut TcpStream, status: u16, message: &str) -> Result<(), String> {
    let body = json!({"error": {"message": message}}).to_string();
    let reason = match status {
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Error",
    };
    stream
        .write_all(
            format!(
                "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json; charset=utf-8\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\
                 Access-Control-Allow-Origin: {ALLOWED_ORIGIN}\r\n\r\n{body}",
                body.len()
            )
            .as_bytes(),
        )
        .map_err(|error| error.to_string())
}

fn write_frames(stream: &mut TcpStream, frames: Vec<String>) -> Result<(), String> {
    for frame in frames {
        stream
            .write_all(frame.as_bytes())
            .map_err(|error| error.to_string())?;
        stream.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn set_error(shared: &Shared, error: String) {
    shared.stats.lock().unwrap().last_error = Some(error);
    shared.done.store(true, Ordering::Relaxed);
}

pub fn run<F: FnMut(String)>(config: &Config, mut log: F) -> Result<RunStats, String> {
    validate_config(config)?;
    let mut server_config = config.clone();
    server_config.upstream = config
        .upstream
        .as_deref()
        .map(normalize_endpoint)
        .transpose()?;
    let shared = Arc::new(Shared::default());
    let server = start_server(server_config, Arc::clone(&shared))?;
    let local_url = format!("http://127.0.0.1:{}{BRIDGE_PATH}", server.port);
    log(format!(
        "protocol bridge ready: {local_url} (model={})",
        config.model
    ));
    if config.only_prompt.is_some() {
        log("protocol bridge is scoped to one exact prompt".into());
    }

    let result = run_cdp(config, &local_url, &shared, &mut log);
    server.stop();
    result
}

fn run_cdp<F: FnMut(String)>(
    config: &Config,
    local_url: &str,
    shared: &Arc<Shared>,
    log: &mut F,
) -> Result<RunStats, String> {
    let ws_url = chat_target(config.cdp_port)?;
    let mut ws = WebSocket::connect(&ws_url, Duration::from_secs(3))?;
    let mut command_id = 0u64;
    command(
        &mut ws,
        &mut command_id,
        "Fetch.enable",
        json!({
            "patterns": [{
                "urlPattern": "*://www.doubao.com/chat/completion*",
                "requestStage": "Request"
            }]
        }),
    )?;

    let started = Instant::now();
    let loop_result = (|| -> Result<(), String> {
        while started.elapsed() < config.ttl {
            if config.once && shared.done.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(500));
                break;
            }
            let text = match ws.recv_text(Duration::from_secs(1)) {
                Ok(text) => text,
                Err(error) if is_timeout(&error) => continue,
                Err(error) => return Err(error),
            };
            let message: Value = serde_json::from_str(&text).map_err(|error| error.to_string())?;
            if message.get("method").and_then(Value::as_str) != Some("Fetch.requestPaused") {
                continue;
            }
            let Some(params) = message.get("params") else {
                continue;
            };
            let Some(request_id) = params.get("requestId").and_then(Value::as_str) else {
                continue;
            };
            let request = params.get("request").unwrap_or(&Value::Null);
            let url = request.get("url").and_then(Value::as_str).unwrap_or("");
            command_id += 1;
            if !is_chat_completion_url(url)
                || config.only_prompt.as_deref().is_some_and(|prompt| {
                    !request_matches_prompt(
                        request.get("postData").and_then(Value::as_str),
                        prompt,
                        &config.model,
                    )
                })
            {
                send_command(
                    &mut ws,
                    command_id,
                    "Fetch.continueRequest",
                    json!({"requestId": request_id}),
                )?;
                continue;
            }
            *shared.request_started.lock().unwrap() = Some(Instant::now());
            send_command(
                &mut ws,
                command_id,
                "Fetch.continueRequest",
                json!({
                    "requestId": request_id,
                    "url": local_url,
                    "headers": [
                        {"name": "Content-Type", "value": "application/json"},
                        {"name": "Accept", "value": "text/event-stream"},
                        {"name": "Origin", "value": ALLOWED_ORIGIN},
                        {"name": "X-Doubao-Protocol-Bridge", "value": MARKER_VALUE}
                    ]
                }),
            )?;
            let mut stats = shared.stats.lock().unwrap();
            stats.intercepted += 1;
            log(format!(
                "protocol bridge: intercepted request {}",
                stats.intercepted
            ));
        }
        Ok(())
    })();

    command_id += 1;
    let _ = send_command(&mut ws, command_id, "Fetch.disable", json!({}));
    ws.close();
    loop_result?;

    let stats = shared.stats.lock().unwrap().clone();
    if let Some(error) = stats.last_error.clone() {
        return Err(error);
    }
    if stats.intercepted == 0 {
        return Err("protocol bridge timed out before a matching request".into());
    }
    if stats.accepted != stats.intercepted || stats.completed != stats.intercepted {
        return Err(format!(
            "protocol bridge incomplete: intercepted={}, accepted={}, completed={}",
            stats.intercepted, stats.accepted, stats.completed
        ));
    }
    log(format!(
        "protocol bridge: completed {} delta(s)",
        stats.chunks
    ));
    Ok(stats)
}

fn command(
    ws: &mut WebSocket,
    command_id: &mut u64,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    *command_id += 1;
    send_command(ws, *command_id, method, params)?;
    loop {
        let text = ws.recv_text(Duration::from_secs(5))?;
        let message: Value = serde_json::from_str(&text).map_err(|error| error.to_string())?;
        if message.get("id").and_then(Value::as_u64) == Some(*command_id) {
            if let Some(error) = message.get("error") {
                return Err(format!("{method}: {error}"));
            }
            return Ok(message.get("result").cloned().unwrap_or(Value::Null));
        }
    }
}

fn send_command(
    ws: &mut WebSocket,
    command_id: u64,
    method: &str,
    params: Value,
) -> Result<(), String> {
    ws.send_text(&json!({"id": command_id, "method": method, "params": params}).to_string())
}

fn chat_target(cdp_port: u16) -> Result<String, String> {
    let mut candidates = live::targets(cdp_port)?;
    candidates.retain(|target| {
        let url = target.get("url").and_then(Value::as_str).unwrap_or("");
        target.get("type").and_then(Value::as_str) == Some("page")
            && (url.starts_with("chrome://doubaowork-chat/chat")
                || url.starts_with("doubaowork://doubaowork-chat/chat"))
            && !url.contains("launcher")
            && target
                .get("webSocketDebuggerUrl")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty())
    });
    candidates.sort_by(|left, right| {
        left.get("url")
            .and_then(Value::as_str)
            .cmp(&right.get("url").and_then(Value::as_str))
    });
    candidates
        .first()
        .and_then(|target| target.get("webSocketDebuggerUrl"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "no active DoubaoWork chat page was found".to_string())
}

fn request_matches_prompt(post_data: Option<&str>, prompt: &str, model: &str) -> bool {
    post_data
        .and_then(|data| serde_json::from_str::<Value>(data).ok())
        .and_then(|request| decode_request(&request, model).ok())
        .and_then(|turn| {
            turn.openai_request
                .pointer("/messages/0/content")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .as_deref()
        == Some(prompt)
}

fn is_chat_completion_url(url: &str) -> bool {
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return false;
    };
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    authority.split(':').next() == Some("www.doubao.com")
        && path.split('?').next() == Some("chat/completion")
}

fn normalize_endpoint(base: &str) -> Result<String, String> {
    let base = base.trim();
    if base.contains('?') || base.contains('#') {
        return Err("upstream must not contain query or fragment".into());
    }
    let (scheme, rest) = if let Some(rest) = base.strip_prefix("https://") {
        ("https", rest)
    } else if let Some(rest) = base.strip_prefix("http://") {
        ("http", rest)
    } else {
        return Err("upstream must be an http(s) URL".into());
    };
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    if authority.is_empty() || authority.contains('@') {
        return Err("upstream must not contain credentials".into());
    }
    if scheme == "http"
        && !(authority == "localhost"
            || authority.starts_with("localhost:")
            || authority == "127.0.0.1"
            || authority.starts_with("127.0.0.1:")
            || authority == "[::1]"
            || authority.starts_with("[::1]:"))
    {
        return Err("non-loopback upstreams must use HTTPS".into());
    }
    let path = path.trim_end_matches('/');
    let path = if path.ends_with("chat/completions") {
        path.to_owned()
    } else if path.is_empty() {
        "chat/completions".into()
    } else {
        format!("{path}/chat/completions")
    };
    Ok(format!("{scheme}://{authority}/{path}"))
}

fn validate_config(config: &Config) -> Result<(), String> {
    if config.ttl < Duration::from_secs(15) || config.ttl > Duration::from_secs(3600) {
        return Err("ttl must be between 15 and 3600 seconds".into());
    }
    if config.model.is_empty() {
        return Err("model must not be empty".into());
    }
    if config.upstream.is_none() && config.mock_parts.is_empty() {
        return Err("mock response must contain at least one delta".into());
    }
    Ok(())
}

fn is_timeout(error: &str) -> bool {
    error.contains("timed out")
        || error.contains("WouldBlock")
        || error.contains("Resource temporarily unavailable")
}

static LAST_NUMERIC_ID: AtomicU64 = AtomicU64::new(0);

fn new_numeric_id() -> String {
    let base = unix_millis().saturating_mul(10_000);
    loop {
        let previous = LAST_NUMERIC_ID.load(Ordering::Relaxed);
        let next = base.max(previous.saturating_add(1));
        if LAST_NUMERIC_ID
            .compare_exchange(previous, next, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            return next.to_string();
        }
    }
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn unix_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::mpsc;

    fn request(text: &str) -> Value {
        json!({
            "client_meta": {
                "conversation_id": "conv-1",
                "local_conversation_id": "local-conv-1",
                "last_section_id": "section-1",
                "last_message_index": 6,
                "bot_id": "bot-1"
            },
            "messages": [{
                "local_message_id": "local-message-1",
                "content_block": [{
                    "block_type": 10000,
                    "block_id": "user-block-1",
                    "content": {"text_block": {"text": text}}
                }]
            }],
            "option": {
                "connector_info_list": [],
                "general_task_param": {"workspace_id": "must-not-leak"}
            },
            "user_context": []
        })
    }

    #[test]
    fn decodes_only_plain_text_without_private_context() {
        let turn = decode_request(&request("  hello  "), "gpt-test").unwrap();
        assert_eq!(
            turn.openai_request,
            json!({
                "model": "gpt-test",
                "messages": [{"role": "user", "content": "  hello  "}],
                "stream": true
            })
        );
        assert_eq!(turn.user_message_index, 7);
        assert_eq!(turn.assistant_message_index, 8);
        assert!(!turn.openai_request.to_string().contains("workspace_id"));
    }

    #[test]
    fn rejects_context_and_unknown_blocks() {
        let mut context = request("hello");
        context["user_context"] = json!([{"kind": "document"}]);
        assert!(decode_request(&context, "gpt-test").is_err());

        let mut connector = request("hello");
        connector["option"]["connector_info_list"] = json!([{"id": "drive"}]);
        assert!(decode_request(&connector, "gpt-test").is_err());

        let mut block = request("hello");
        block["messages"][0]["content_block"][0]["block_type"] = json!(20_000);
        assert!(decode_request(&block, "gpt-test").is_err());
    }

    #[test]
    fn encodes_native_stream_events_without_duplicate_delta_channel() {
        let turn = decode_request(&request("hello"), "gpt-test").unwrap();
        let mut encoder = DoubaoSseEncoder::new(turn);
        let start = encoder.start().join("");
        let delta = encoder.delta("外部").join("");
        let finish = encoder.finish().join("");
        for event in [
            "SSE_HEARTBEAT",
            "SSE_ACK",
            "FULL_MSG_NOTIFY",
            "STREAM_TIMEOUT_CONTROL",
            "STREAM_MSG_NOTIFY",
        ] {
            assert!(start.contains(&format!("event: {event}")));
        }
        assert_eq!(delta.matches("event: STREAM_CHUNK").count(), 2);
        assert!(!delta.contains("event: CHUNK_DELTA"));
        assert!(delta.contains("外部"));
        assert!(finish.contains("\"end_type\":1"));
        assert!(finish.contains("\"end_type\":2"));
        assert!(finish.contains("\"end_type\":3"));
    }

    #[test]
    fn parses_fragmented_openai_stream_and_terminal_text() {
        let stream = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"},",
            "\"finish_reason\":null}]}\r\n\r\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" world\"},",
            "\"finish_reason\":\"stop\"}]}\r\n\r\n"
        );
        let mut deltas = Vec::new();
        stream_openai(Cursor::new(stream.as_bytes()), |text| {
            deltas.push(text.to_owned());
            Ok(())
        })
        .unwrap();
        assert_eq!(deltas, ["hello", " world"]);
    }

    #[test]
    fn rejects_truncated_openai_stream() {
        let stream = b"data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n";
        let error = stream_openai(Cursor::new(stream), |_| Ok(())).unwrap_err();
        assert!(error.contains("completion marker"));
    }

    #[test]
    fn normalizes_and_secures_upstream_urls() {
        assert_eq!(
            normalize_endpoint("https://api.example.com/v1").unwrap(),
            "https://api.example.com/v1/chat/completions"
        );
        assert!(normalize_endpoint("http://api.example.com/v1").is_err());
        assert!(normalize_endpoint("https://token@api.example.com/v1").is_err());
    }

    #[test]
    fn generates_distinct_seventeen_digit_ids() {
        let ids: std::collections::HashSet<_> = (0..5).map(|_| new_numeric_id()).collect();
        assert_eq!(ids.len(), 5);
        assert!(ids
            .iter()
            .all(|value| value.len() == 17 && value.chars().all(|ch| ch.is_ascii_digit())));
    }

    #[test]
    fn converts_a_full_request_through_an_openai_compatible_upstream() {
        let upstream_listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let upstream_port = upstream_listener.local_addr().unwrap().port();
        let (request_tx, request_rx) = mpsc::channel();
        let upstream_thread = thread::spawn(move || {
            let (mut stream, _) = upstream_listener.accept().unwrap();
            let request = read_http_request(&mut stream).unwrap();
            request_tx.send(request.body).unwrap();
            stream
                .write_all(
                    concat!(
                        "HTTP/1.1 200 OK\r\n",
                        "Content-Type: text/event-stream\r\n",
                        "Connection: close\r\n\r\n",
                        "data: {\"choices\":[{\"delta\":{\"content\":\"UP_\"},",
                        "\"finish_reason\":null}]}\n\n",
                        "data: {\"choices\":[{\"delta\":{\"content\":\"OK\"},",
                        "\"finish_reason\":null}]}\n\n",
                        "data: [DONE]\n\n"
                    )
                    .as_bytes(),
                )
                .unwrap();
        });

        let config = Config {
            bridge_port: 0,
            upstream: Some(format!(
                "http://127.0.0.1:{upstream_port}/v1/chat/completions"
            )),
            model: "external-test".into(),
            once: true,
            mock_delay: Duration::ZERO,
            ..Config::default()
        };
        let shared = Arc::new(Shared::default());
        let server = start_server(config.clone(), Arc::clone(&shared)).unwrap();
        let mut client = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let body = request("question").to_string();
        client
            .write_all(
                format!(
                    "POST {BRIDGE_PATH} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\
                     Content-Type: application/json\r\nOrigin: {ALLOWED_ORIGIN}\r\n\
                     X-Doubao-Protocol-Bridge: {MARKER_VALUE}\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    server.port,
                    body.len()
                )
                .as_bytes(),
            )
            .unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server.stop();
        upstream_thread.join().unwrap();

        let upstream_body: Value = serde_json::from_slice(&request_rx.recv().unwrap()).unwrap();
        assert_eq!(
            upstream_body,
            json!({
                "model": "external-test",
                "messages": [{"role": "user", "content": "question"}],
                "stream": true
            })
        );
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"text\":\"UP_\""));
        assert!(response.contains("\"text\":\"OK\""));
        assert!(response.contains("\"end_type\":3"));
        assert_eq!(shared.stats.lock().unwrap().completed, 1);
    }
}

//! Minimal WebSocket client + Chrome DevTools Protocol caller
//! (port of `ws.py`). std-only, synchronous. Client frames are masked per
//! RFC 6455; no Origin header is sent (Chromium rejects CDP handshakes that
//! carry an unknown Origin).

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct WebSocket {
    sock: TcpStream,
    buf: Vec<u8>,
}

fn rand_bytes(n: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut buf)?;
    Ok(buf)
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
    }
    out
}

impl WebSocket {
    pub fn connect(url: &str, timeout: Duration) -> Result<Self, String> {
        let rest = url
            .strip_prefix("ws://")
            .ok_or_else(|| format!("not a ws:// url: {url}"))?;
        let (hostport, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i + 1..]),
            None => (rest, ""),
        };
        let host = hostport.split(':').next().unwrap_or(hostport);
        let port: u16 = hostport
            .split(':')
            .nth(1)
            .and_then(|p| p.parse().ok())
            .unwrap_or(80);
        let addrs: Vec<_> = (host, port)
            .to_socket_addrs()
            .map_err(|e| format!("resolve {host}:{port}: {e}"))?
            .collect();
        // try every resolved address (localhost may resolve to ::1 first
        // while the debugger only listens on 127.0.0.1)
        let mut last_err = String::new();
        let mut sock_opt = None;
        for addr in &addrs {
            match TcpStream::connect_timeout(addr, timeout) {
                Ok(s) => {
                    sock_opt = Some(s);
                    break;
                }
                Err(e) => last_err = e.to_string(),
            }
        }
        let sock = sock_opt.ok_or_else(|| format!("connect: {last_err}"))?;
        sock.set_read_timeout(Some(timeout)).ok();
        sock.set_nodelay(true).ok();

        let key = base64_encode(&rand_bytes(16).map_err(|e| e.to_string())?);
        let request = format!(
            "GET /{path} HTTP/1.1\r\nHost: {hostport}\r\n\
             Upgrade: websocket\r\nConnection: Upgrade\r\n\
             Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        let mut ws = WebSocket { sock, buf: Vec::new() };
        ws.sock
            .write_all(request.as_bytes())
            .map_err(|e| format!("handshake write: {e}"))?;
        let head = ws.read_head()?;
        let status_line = head.split("\r\n").next().unwrap_or("");
        if !status_line.contains(" 101 ") {
            return Err(format!("handshake failed: {}", &head[..head.len().min(200)]));
        }
        Ok(ws)
    }

    fn read_head(&mut self) -> Result<String, String> {
        let mut data = Vec::new();
        loop {
            if let Some(pos) = data
                .windows(4)
                .position(|w| w == b"\r\n\r\n")
            {
                let head = String::from_utf8_lossy(&data[..pos]).into_owned();
                self.buf = data[pos + 4..].to_vec();
                return Ok(head);
            }
            let mut chunk = [0u8; 4096];
            let n = self
                .sock
                .read(&mut chunk)
                .map_err(|e| format!("handshake read: {e}"))?;
            if n == 0 {
                return Err("closed during handshake".into());
            }
            data.extend_from_slice(&chunk[..n]);
        }
    }

    fn read_exact(&mut self, n: usize) -> Result<Vec<u8>, String> {
        while self.buf.len() < n {
            let mut chunk = [0u8; 65536];
            let read = self.sock.read(&mut chunk).map_err(|e| format!("read: {e}"))?;
            if read == 0 {
                return Err("connection closed".into());
            }
            self.buf.extend_from_slice(&chunk[..read]);
        }
        Ok(self.buf.drain(..n).collect())
    }

    pub fn send_text(&mut self, payload: &str) -> Result<(), String> {
        let data = payload.as_bytes();
        let n = data.len();
        let mut head = vec![0x81u8];
        if n < 126 {
            head.push(0x80 | n as u8);
        } else if n < 65536 {
            head.push(0x80 | 126);
            head.extend_from_slice(&(n as u16).to_be_bytes());
        } else {
            head.push(0x80 | 127);
            head.extend_from_slice(&(n as u64).to_be_bytes());
        }
        let mask = rand_bytes(4).map_err(|e| e.to_string())?;
        head.extend_from_slice(&mask);
        let masked: Vec<u8> =
            data.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]).collect();
        head.extend_from_slice(&masked);
        self.sock.write_all(&head).map_err(|e| format!("write: {e}"))
    }

    /// Receive the next text message. Answers pings with pongs, follows
    /// fragmented messages, errors on close frames.
    pub fn recv_text(&mut self, timeout: Duration) -> Result<String, String> {
        self.sock.set_read_timeout(Some(timeout)).ok();
        loop {
            let (opcode, payload) = self.read_message()?;
            match opcode {
                0x1 => {
                    return String::from_utf8(payload)
                        .map_err(|e| format!("invalid utf-8: {e}"));
                }
                0x9 => self.send_control(0xA, &payload)?, // ping -> pong
                0x8 => return Err("closed by peer".into()),
                _ => {}
            }
        }
    }

    fn send_control(&mut self, opcode: u8, payload: &[u8]) -> Result<(), String> {
        let mask = rand_bytes(4).map_err(|e| e.to_string())?;
        let mut head = vec![0x80 | opcode, 0x80 | payload.len() as u8];
        head.extend_from_slice(&mask);
        let masked: Vec<u8> =
            payload.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]).collect();
        head.extend_from_slice(&masked);
        self.sock.write_all(&head).map_err(|e| format!("write: {e}"))
    }

    /// Read one (possibly fragmented) message; returns its opcode and payload.
    fn read_message(&mut self) -> Result<(u8, Vec<u8>), String> {
        let mut payload = Vec::new();
        let mut first_opcode = 0u8;
        let mut first = true;
        loop {
            let h = self.read_exact(2)?;
            let fin = h[0] & 0x80 != 0;
            let opcode = h[0] & 0x0F;
            if first {
                first_opcode = opcode;
                first = false;
            }
            let masked = h[1] & 0x80 != 0;
            let mut ln = (h[1] & 0x7F) as u64;
            if ln == 126 {
                let e = self.read_exact(2)?;
                ln = u16::from_be_bytes([e[0], e[1]]) as u64;
            } else if ln == 127 {
                let e = self.read_exact(8)?;
                ln = u64::from_be_bytes(e.try_into().unwrap());
            }
            let mask = if masked { Some(self.read_exact(4)?) } else { None };
            let mut chunk = self.read_exact(ln as usize)?;
            if let Some(mask) = mask {
                for (i, b) in chunk.iter_mut().enumerate() {
                    *b ^= mask[i % 4];
                }
            }
            payload.extend_from_slice(&chunk);
            if fin {
                return Ok((first_opcode, payload));
            }
        }
    }

    pub fn close(self) {
        let _ = self.sock.shutdown(std::net::Shutdown::Both);
    }
}

/// Synchronous DevTools-protocol caller (events are ignored).
pub struct Cdp {
    ws: WebSocket,
    next_id: u64,
}

impl Cdp {
    pub fn connect(ws_url: &str, timeout: Duration) -> Result<Self, String> {
        Ok(Cdp { ws: WebSocket::connect(ws_url, timeout)?, next_id: 0 })
    }

    pub fn call(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        let msg = serde_json::json!({"id": id, "method": method, "params": params});
        self.ws.send_text(&msg.to_string())?;
        loop {
            let text = self.ws.recv_text(Duration::from_secs(10))?;
            let parsed: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| format!("bad json: {e}"))?;
            if parsed.get("id").and_then(|v| v.as_u64()) == Some(id) {
                if let Some(err) = parsed.get("error") {
                    return Err(format!("{method}: {err}"));
                }
                return Ok(parsed.get("result").cloned().unwrap_or(serde_json::Value::Null));
            }
        }
    }

    pub fn evaluate(&mut self, expression: &str) -> Result<serde_json::Value, String> {
        let result = self.call(
            "Runtime.evaluate",
            serde_json::json!({"expression": expression, "returnByValue": true}),
        )?;
        Ok(result.pointer("/result/value").cloned().unwrap_or(serde_json::Value::Null))
    }

    pub fn close(self) {
        self.ws.close();
    }
}

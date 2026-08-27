//! Live mode: reskin the ORIGINAL app at runtime via CDP — no file changes
//! (port of `live.py`).
//!
//! 1. (re)launch DoubaoWork with --remote-debugging-port
//! 2. poll the debugger's /json target list; for every embedded app page
//!    (doubaowork:// scheme, side-panel extension pages), connect over CDP
//!    and evaluate a small JS that forces dark theme + installs the theme CSS
//! 3. also register the same JS via Page.addScriptToEvaluateOnNewDocument so
//!    navigations keep the skin; keep watching for new pages

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::theme::Theme;
use crate::ws::Cdp;

pub const APP_BINARY: &str = "/Applications/DoubaoWork.app/Contents/MacOS/DoubaoWork";
pub const DEFAULT_PORT: u16 = 9222;

/// inject only into the app's own embedded pages, never into web tabs
const URL_PATTERNS: &[&str] = &[
    "doubaowork://",
    "chrome://doubaowork",
    "side_panel.html",
    "popup.html",
    "options.html",
];

const JS_TEMPLATE: &str = r#"(function(){
  var SKIN = %s, CSS = %s;
  function f(){
    var e=document.documentElement;
    if(e.getAttribute('data-theme')!=='dark')e.setAttribute('data-theme','dark');
    if(e.getAttribute('data-skin')!==SKIN)e.setAttribute('data-skin',SKIN);
    var b=document.body;
    if(b&&b.getAttribute('theme-mode')!=='dark')b.setAttribute('theme-mode','dark');
    var s=document.getElementById('doubao-skin-style');
    if(document.head){
      if(!s){s=document.createElement('style');s.id='doubao-skin-style';
        s.setAttribute('nonce','argus-csp-token');document.head.appendChild(s);}
      if(s.textContent!==CSS)s.textContent=CSS;
    }
  }
  f();
  new MutationObserver(f).observe(document.documentElement,
    {attributes:true,attributeFilter:['data-theme','data-skin']});
  document.addEventListener('DOMContentLoaded',f);
  if(window.__doubaoSkinTimer)clearInterval(window.__doubaoSkinTimer);
  window.__doubaoSkinTimer=setInterval(f,2000);
})();"#;

pub fn theme_js(theme: &Theme) -> String {
    let css = format!("html{{color-scheme:dark}}{}", theme.effective_css());
    let skin_json = serde_json::to_string(&theme.id).unwrap();
    let css_json = serde_json::to_string(&css).unwrap();
    JS_TEMPLATE
        .replacen("%s", &skin_json, 1)
        .replacen("%s", &css_json, 1)
}

/// Minimal HTTP GET over std TcpStream (localhost only).
fn http_get(port: u16, path: &str, timeout: Duration) -> Result<Vec<u8>, String> {
    let mut sock = TcpStream::connect(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    sock.set_read_timeout(Some(timeout)).ok();
    sock.write_all(
        format!("GET {path} HTTP/1.1\r\nHost: localhost:{port}\r\nConnection: close\r\n\r\n")
            .as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    // The CDP http server keeps the connection alive, so read headers first
    // and then exactly Content-Length body bytes.
    let mut raw = Vec::new();
    let header_end = loop {
        if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos;
        }
        let mut chunk = [0u8; 4096];
        let n = sock.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("connection closed during headers".into());
        }
        raw.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&raw[..header_end]).into_owned();
    let status = head.split("\r\n").next().unwrap_or("");
    if !status.contains(" 200") {
        return Err(format!("http {status}"));
    }
    let content_length: usize = head
        .split("\r\n")
        .find_map(|line| {
            let (k, v) = line.split_once(':')?;
            k.trim().eq_ignore_ascii_case("content-length").then(|| v.trim().parse().ok())?
        })
        .unwrap_or(0);
    let mut body = raw.split_off(header_end + 4);
    while body.len() < content_length {
        let mut chunk = [0u8; 65536];
        let n = sock.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(content_length);
    Ok(body)
}

fn port_up(port: u16) -> bool {
    http_get(port, "/json/version", Duration::from_secs(1)).is_ok()
}

fn targets(port: u16) -> Result<Vec<serde_json::Value>, String> {
    let body = http_get(port, "/json", Duration::from_secs(3))?;
    serde_json::from_slice(&body).map_err(|e| format!("bad /json: {e}"))
}

fn ensure_running<F: FnMut(String)>(port: u16, mut log: F) -> Result<(), String> {
    if port_up(port) {
        log("debug port already up — reusing the running instance".into());
        return Ok(());
    }
    if !Path::new(APP_BINARY).exists() {
        return Err(format!("app not found: {APP_BINARY}"));
    }
    // a running instance without the debug flag must be restarted first
    let _ = Command::new("osascript")
        .args(["-e", "tell application \"DoubaoWork\" to quit"])
        .output();
    for _ in 0..20 {
        let running = Command::new("pgrep")
            .args(["-f", "DoubaoWork.app/Contents/MacOS"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !running {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    // Launch through LaunchServices: a directly exec'd binary can end up in
    // a wedged state (no windows, unresponsive renderers) on some systems.
    log(format!("launching DoubaoWork --remote-debugging-port={port}"));
    Command::new("open")
        .arg("-a")
        .arg("/Applications/DoubaoWork.app")
        .arg("--args")
        .arg(format!("--remote-debugging-port={port}"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot launch app: {e}"))?;
    for _ in 0..60 {
        if port_up(port) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err("debug port did not come up".into())
}

/// Watch pages and inject the theme. Runs until `stop` is set, the debug
/// port goes away, or (with `once`) after a single pass.
pub fn run<F: FnMut(String)>(
    theme: &Theme,
    port: u16,
    once: bool,
    stop: Arc<AtomicBool>,
    mut log: F,
) -> Result<(), String> {
    ensure_running(port, &mut log)?;
    let js = theme_js(theme);
    let mut injected: std::collections::HashSet<String> = std::collections::HashSet::new();
    log(format!("live theme: {} ({}) — watching pages…", theme.name, theme.id));
    while !stop.load(Ordering::Relaxed) {
        let list = match targets(port) {
            Ok(l) => l,
            Err(_) => {
                log("debug port went away (app quit?) — exiting".into());
                return Ok(());
            }
        };
        for t in &list {
            if t.get("type").and_then(|v| v.as_str()) != Some("page") {
                continue;
            }
            let url = t.get("url").and_then(|v| v.as_str()).unwrap_or("");
            let tid = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if tid.is_empty()
                || injected.contains(tid)
                || !URL_PATTERNS.iter().any(|p| url.contains(p))
            {
                continue;
            }
            let ws_url = t.get("webSocketDebuggerUrl").and_then(|v| v.as_str()).unwrap_or("");
            match inject_target(ws_url, &js) {
                Ok(()) => {
                    injected.insert(tid.to_string());
                    log(format!("  injected: {}", &url[..url.len().min(70)]));
                }
                Err(e) => {
                    // page may be mid-navigation; retry next round
                    log(format!("  retry later: {} ({e})", &url[..url.len().min(60)]));
                }
            }
        }
        if once {
            log(format!("done — {} page(s) themed", injected.len()));
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    Ok(())
}

fn inject_target(ws_url: &str, js: &str) -> Result<(), String> {
    let mut cdp = Cdp::connect(ws_url, Duration::from_secs(10))?;
    let result = (|| {
        cdp.call("Page.enable", serde_json::json!({}))?;
        cdp.call(
            "Page.addScriptToEvaluateOnNewDocument",
            serde_json::json!({"source": js}),
        )?;
        cdp.evaluate(js)?;
        Ok(())
    })();
    cdp.close();
    result
}

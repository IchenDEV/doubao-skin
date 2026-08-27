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

fn ensure_running<F: FnMut(String)>(port: u16, mut log: F) -> Result<bool, String> {
    if port_up(port) {
        log("debug port already up — reusing the running instance".into());
        return Ok(false);
    }
    if !Path::new(APP_BINARY).exists() {
        return Err(format!("app not found: {APP_BINARY}"));
    }
    // a running instance without the debug flag must be restarted first:
    // graceful quit, then hard-kill whatever remains (a wedged instance may
    // never finish quitting, and a leftover process makes the next launch
    // restore a window-less session with renderers that never answer CDP)
    let _ = Command::new("osascript")
        .args(["-e", "tell application \"DoubaoWork\" to quit"])
        .output();
    std::thread::sleep(Duration::from_secs(3));
    kill_app();
    launch_app(port, &mut log)?;
    for _ in 0..60 {
        if port_up(port) {
            return Ok(true);
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err("debug port did not come up".into())
}

/// Marker file with the unix timestamp of our last app launch, shared
/// across watcher processes so a fresh watcher never restarts an app that
/// is still booting.
fn launch_marker() -> &'static str {
    "/tmp/doubao-work-skin-launched-at"
}

fn launched_recently() -> bool {
    std::fs::read_to_string(launch_marker())
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|t| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            now.saturating_sub(t) < 120
        })
        .unwrap_or(false)
}

fn mark_launched() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let _ = std::fs::write(launch_marker(), now.to_string());
}

/// Launch the app through LaunchServices with the debug flag: a directly
/// exec'd binary can end up in a wedged state (no windows, unresponsive
/// renderers) on some systems.
fn launch_app<F: FnMut(String)>(port: u16, mut log: F) -> Result<(), String> {
    log(format!("launching DoubaoWork --remote-debugging-port={port}"));
    mark_launched();
    Command::new("open")
        .arg("-a")
        .arg("/Applications/DoubaoWork.app")
        .arg("--args")
        .arg(format!("--remote-debugging-port={port}"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot launch app: {e}"))?;
    // nudge a window open (like clicking the Dock icon): when the previous
    // session was saved window-less, pages would boot without a window and
    // their renderers never answer CDP
    let _ = Command::new("osascript")
        .args(["-e", "tell application \"DoubaoWork\" to reopen"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    Ok(())
}

/// Kill the app. A wedged instance must be hard-killed: a graceful
/// AppleScript quit can persist a window-less session whose renderers then
/// never answer CDP on the next launch.
fn kill_app() {
    let _ = Command::new("pkill")
        .args(["-f", "DoubaoWork.app/Contents/MacOS"])
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
}

/// Watch pages and inject the theme. Runs until `stop` is set, the debug
/// port goes away, or (with `once`) after a single pass.
///
/// Dead targets: when the app's window is closed its renderers stop answering
/// CDP commands (the targets stay listed in /json). We probe each candidate
/// with a 1.5s evaluate before injecting; unresponsive targets are marked
/// dead and skipped quietly (one log line when marked, none while probing).
/// A dead target that answers again is re-injected; ids that vanish from
/// /json are dropped from the dead set.
///
/// Wedge recovery: this app permanently stops answering CDP on ALL windowed
/// targets once its window has been closed for a while (the browser process
/// and /json stay alive, so it is not detectable via the port). We restart
/// the app ONCE, only at watcher startup (i.e. when the user just picked a
/// theme) — never unprompted during steady state, so we don't reopen windows
/// the user deliberately closed.
pub fn run<F: FnMut(String)>(
    theme: &Theme,
    port: u16,
    once: bool,
    stop: Arc<AtomicBool>,
    mut log: F,
) -> Result<(), String> {
    let ensure_running_launched = ensure_running(port, &mut log)?;
    let js = theme_js(theme);
    let mut injected: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut dead: std::collections::HashSet<String> = std::collections::HashSet::new();
    log(format!("live theme: {} ({}) — watching pages…", theme.name, theme.id));
    let started = std::time::Instant::now();
    // when WE just (re)launched the app, its pages may take minutes to boot —
    // they look dead but are merely slow. Suppress the wedge restart for two
    // minutes after our own launch so we never restart a booting app.
    let mut launched_by_us_at: Option<std::time::Instant> =
        if ensure_running_launched { Some(std::time::Instant::now()) } else { None };
    let mut port_was_down = false;
    let mut down_ticks = 0u32;
    let mut all_dead_since: Option<std::time::Instant> = None;
    let mut heartbeat_tick = 0u32;
    let mut dead_probe_tick = 0u32;
    let mut wedge_restarted = false;
    let mut revive_attempted = false;
    while !stop.load(Ordering::Relaxed) {
        let list = match targets(port) {
            Ok(l) => l,
            Err(_) => {
                if once {
                    log("debug port went away (app quit?) — exiting".into());
                    return Ok(());
                }
                // app quit: keep watching; if it doesn't come back on its own,
                // relaunch it with the debug flag
                if !port_was_down {
                    log("debug port went away — waiting for the app to come back…".into());
                    port_was_down = true;
                }
                down_ticks += 1;
                if down_ticks >= 5 && !port_up(port) {
                    log("relaunching app with the debug port…".into());
                    if launch_app(port, &mut log).is_err() {
                        log("relaunch failed, will keep waiting".into());
                    } else {
                        launched_by_us_at = Some(std::time::Instant::now());
                    }
                    down_ticks = 0;
                }
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
        };
        if port_was_down {
            // fresh app instance: new target ids, inject everything again
            log("app is back — re-injecting pages…".into());
            injected.clear();
            dead.clear();
            port_was_down = false;
            all_dead_since = None;
        }
        // collect the matching page targets
        struct T<'a> { id: &'a str, url: &'a str, ws: &'a str }
        let mut pages = Vec::new();
        for t in &list {
            if t.get("type").and_then(|v| v.as_str()) != Some("page") {
                continue;
            }
            let url = t.get("url").and_then(|v| v.as_str()).unwrap_or("");
            let tid = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if tid.is_empty() || !URL_PATTERNS.iter().any(|p| url.contains(p)) {
                continue;
            }
            let ws_url = t.get("webSocketDebuggerUrl").and_then(|v| v.as_str()).unwrap_or("");
            pages.push(T { id: tid, url, ws: ws_url });
        }
        // forget dead ids that no longer exist
        let ids: std::collections::HashSet<&str> = pages.iter().map(|p| p.id).collect();
        dead.retain(|id| ids.contains(id.as_str()));
        injected.retain(|id| ids.contains(id.as_str()));

        // Wedge recovery (startup only): every WINDOWED target dead (the
        // background page stays responsive without a window, so it is
        // excluded) while this watcher is fresh => the app is frozen.
        // Two-stage recovery: first ACTIVATE the app (macOS freezes the
        // renderers of an occluded app; activation wakes them — the common
        // case); only if pages stay dead afterwards do we hard-kill and
        // relaunch. Never restart an app launched within the last two
        // minutes — a booting instance looks dead but is merely slow.
        let windowed: Vec<&T> =
            pages.iter().filter(|p| !p.url.contains("background")).collect();
        let startup = started.elapsed() < Duration::from_secs(120);
        let booting = launched_by_us_at
            .map(|t| t.elapsed() < Duration::from_secs(120))
            .unwrap_or(false)
            || launched_recently();
        if startup
            && !booting
            && !wedge_restarted
            && !windowed.is_empty()
            && windowed.iter().all(|p| dead.contains(p.id))
        {
            if revive_attempted {
                // stage 2: still all dead after a wake attempt — genuinely
                // wedged, hard-kill and relaunch (no extra waiting: these
                // targets already proved dead once before the wake)
                log("pages still unresponsive after wake — restarting the app…".into());
                kill_app();
                let _ = launch_app(port, &mut log);
                launched_by_us_at = Some(std::time::Instant::now());
                injected.clear();
                dead.clear();
                all_dead_since = None;
                wedge_restarted = true;
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
            let since = all_dead_since.get_or_insert_with(std::time::Instant::now);
            if since.elapsed() > Duration::from_secs(8) {
                // stage 1: occluded-app freeze — just wake it up
                log("pages unresponsive — waking the app (may be occluded)…".into());
                let _ = Command::new("osascript")
                    .args(["-e", "tell application \"DoubaoWork\" to reopen"])
                    .output();
                let _ = Command::new("osascript")
                    .args(["-e", "tell application \"DoubaoWork\" to activate"])
                    .output();
                revive_attempted = true;
                all_dead_since = None;
                // give the renderers a moment to wake, then force fresh
                // probes: the wedge check reads the `dead` set, which is
                // stale at this point
                std::thread::sleep(Duration::from_secs(5));
                dead.clear();
                continue;
            }
        } else {
            all_dead_since = None;
        }

        // low-frequency heartbeat for already-injected targets: if one stops
        // answering, demote it to dead so it gets re-probed (and re-injected
        // when it recovers)
        heartbeat_tick += 1;
        if heartbeat_tick >= 15 {
            heartbeat_tick = 0;
            for p in &pages {
                if injected.contains(p.id) && !probe(p.ws) {
                    injected.remove(p.id);
                    dead.insert(p.id.to_string());
                    log(format!(
                        "  target stopped responding, watching quietly: {}",
                        &p.url[..p.url.len().min(60)]
                    ));
                }
            }
        }

        // IMPORTANT: every CDP probe spins up a full DevTools session in the
        // renderer — probing every 2s keeps renderers at 100% CPU. Probe dead
        // targets only every 15th tick (~30s); that's plenty for recovery.
        dead_probe_tick += 1;
        let probe_dead_now = dead_probe_tick >= 15;
        if probe_dead_now {
            dead_probe_tick = 0;
        }

        for p in &pages {
            if injected.contains(p.id) {
                continue;
            }
            if dead.contains(p.id) {
                if !probe_dead_now {
                    continue;
                }
                // silent probe: back to life?
                if !probe(p.ws) {
                    continue;
                }
                dead.remove(p.id);
                log(format!("  target responsive again: {}", &p.url[..p.url.len().min(60)]));
            }
            match inject_target(p.ws, &js) {
                Ok(()) => {
                    injected.insert(p.id.to_string());
                    log(format!("  injected: {}", &p.url[..p.url.len().min(70)]));
                }
                Err(_) => {
                    // unresponsive or mid-navigation — mark dead, stay quiet
                    dead.insert(p.id.to_string());
                    log(format!(
                        "  target not responding, watching quietly: {}",
                        &p.url[..p.url.len().min(60)]
                    ));
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

/// Quick liveness probe: connect and evaluate a trivial expression with a
/// short timeout. Renderers of a closed-window app accept the WebSocket but
/// either never answer, or answer from the `cross-site-support` shell page
/// the app swaps in when it unloads the real UI — both count as dead.
fn probe(ws_url: &str) -> bool {
    let Ok(mut cdp) = Cdp::connect(ws_url, Duration::from_millis(4000)) else {
        return false;
    };
    let ok = cdp
        .evaluate_with_timeout("location.href", Duration::from_millis(4000))
        .map(|v| !v.as_str().unwrap_or("").contains("cross-site-support"))
        .unwrap_or(false);
    cdp.close();
    ok
}

fn inject_target(ws_url: &str, js: &str) -> Result<(), String> {
    let mut cdp = Cdp::connect(ws_url, Duration::from_secs(10))?;
    let result = (|| {
        // probe first: a wedged renderer accepts the socket but never answers
        // (or answers from the cross-site-support shell)
        let href = cdp.evaluate_with_timeout("location.href", Duration::from_millis(4000))?;
        if href.as_str().unwrap_or("").contains("cross-site-support") {
            return Err("shell page (real UI unloaded)".into());
        }
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

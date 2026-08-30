//! Live mode: reskin the ORIGINAL app at runtime via CDP — no file changes.
//!
//! 1. (re)launch the selected Doubao app with --remote-debugging-port
//! 2. poll the debugger's /json target list; for every embedded app page
//!    (doubao:// / doubaowork:// schemes, side-panel extension pages), connect over CDP
//!    and evaluate a small JS that forces dark theme + installs the theme CSS
//! 3. also register the same JS via Page.addScriptToEvaluateOnNewDocument so
//!    navigations keep the skin; keep watching for new pages

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::theme::Theme;
use crate::ws::Cdp;

mod platform;

pub const DEFAULT_PORT: u16 = 9222;
pub const DOUBAO_PORT: u16 = 9223;

const GENERIC_PAGE_PATTERNS: &[&str] = &["side_panel.html", "popup.html", "options.html"];
const INITIAL_INJECTION_TIMEOUT: Duration = Duration::from_secs(30);

fn ensure_live_supported(target_os: &str) -> Result<(), String> {
    if matches!(target_os, "macos" | "windows") {
        Ok(())
    } else {
        Err("实时应用主题仅支持 macOS 和 Windows".into())
    }
}

fn timed_out_injection_error(
    failure_elapsed: Option<Duration>,
    last_error: Option<&str>,
) -> Option<String> {
    failure_elapsed
        .is_some_and(|elapsed| elapsed >= INITIAL_INJECTION_TIMEOUT)
        .then(|| last_error.map(str::to_owned))
        .flatten()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetApp {
    Doubao,
    DoubaoWork,
}

impl TargetApp {
    pub const ALL: [Self; 2] = [Self::Doubao, Self::DoubaoWork];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Doubao => "doubao",
            Self::DoubaoWork => "doubao-work",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|target| target.id() == id)
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Doubao => "豆包",
            Self::DoubaoWork => "豆包工作",
        }
    }

    pub const fn bundle_id(self) -> &'static str {
        match self {
            Self::Doubao => "com.bot.pc.doubao",
            Self::DoubaoWork => "com.work.pc.doubao",
        }
    }

    pub fn port(self) -> u16 {
        let (override_name, fallback) = match self {
            Self::Doubao => ("DOUBAO_SKIN_DOUBAO_CDP_PORT", DOUBAO_PORT),
            Self::DoubaoWork => ("DOUBAO_SKIN_DOUBAO_WORK_CDP_PORT", DEFAULT_PORT),
        };
        std::env::var(override_name)
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .filter(|port| *port > 0)
            .unwrap_or(fallback)
    }

    fn launch_marker(self) -> PathBuf {
        match self {
            Self::Doubao => std::env::temp_dir().join("doubao-skin-doubao-launched-at"),
            Self::DoubaoWork => std::env::temp_dir().join("doubao-skin-doubao-work-launched-at"),
        }
    }

    pub fn is_installed(self) -> bool {
        self.installed_binary().is_some()
    }

    fn installed_binary(self) -> Option<PathBuf> {
        platform::installed_binary(self)
    }

    fn install_hint(self) -> String {
        platform::install_hint(self)
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn installed_app_bundle(self) -> Option<PathBuf> {
        platform::installed_app_bundle(self)
    }

    pub fn matches_identity_url(self, url: &str) -> bool {
        match self {
            Self::Doubao => url.starts_with("doubao://") || url.starts_with("chrome://doubao-"),
            Self::DoubaoWork => {
                url.starts_with("doubaowork://") || url.starts_with("chrome://doubaowork-")
            }
        }
    }

    fn matches_page_url(self, url: &str, identity_confirmed: bool) -> bool {
        self.matches_identity_url(url)
            || (identity_confirmed
                && GENERIC_PAGE_PATTERNS
                    .iter()
                    .any(|pattern| url.contains(pattern)))
    }
}

pub fn theme_js(theme: &Theme, target: TargetApp) -> String {
    let css = theme.injected_css();
    theme.bootstrap_js(Some(&css), Some(target.id()))
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
            k.trim()
                .eq_ignore_ascii_case("content-length")
                .then(|| v.trim().parse().ok())?
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

pub(crate) fn targets(port: u16) -> Result<Vec<serde_json::Value>, String> {
    let body = http_get(port, "/json", Duration::from_secs(3))?;
    serde_json::from_slice(&body).map_err(|e| format!("bad /json: {e}"))
}

fn targets_belong_to(target: TargetApp, list: &[serde_json::Value]) -> bool {
    list.iter().any(|entry| {
        entry.get("type").and_then(|value| value.as_str()) == Some("page")
            && entry
                .get("url")
                .and_then(|value| value.as_str())
                .is_some_and(|url| target.matches_identity_url(url))
    })
}

fn ensure_running<F: FnMut(String)>(target: TargetApp, mut log: F) -> Result<bool, String> {
    let port = target.port();
    if port_up(port) {
        let list = targets(port)?;
        if !targets_belong_to(target, &list) {
            return Err(format!(
                "{} 端口 {port} 已被其他程序占用，请关闭占用后再试",
                target.display_name()
            ));
        }
        log(format!(
            "{} debug port already up — reusing the running instance",
            target.display_name()
        ));
        return Ok(false);
    }
    if !target.is_installed() {
        return Err(format!(
            "未找到{}：{}",
            target.display_name(),
            target.install_hint()
        ));
    }
    // a running instance without the debug flag must be restarted first:
    // graceful quit, then hard-kill whatever remains (a wedged instance may
    // never finish quitting, and a leftover process makes the next launch
    // restore a window-less session with renderers that never answer CDP)
    platform::tell_app(target, "quit", false);
    std::thread::sleep(Duration::from_secs(3));
    platform::kill_app(target);
    platform::launch_app(target, &mut log)?;
    for _ in 0..60 {
        if port_up(port) {
            let list = targets(port)?;
            if targets_belong_to(target, &list) {
                return Ok(true);
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(format!("{} 的调试端口未正常启动", target.display_name()))
}

/// Marker file with the unix timestamp of our last app launch, shared
/// across watcher processes so a fresh watcher never restarts an app that
/// is still booting.
fn launched_recently(target: TargetApp) -> bool {
    std::fs::read_to_string(target.launch_marker())
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

fn mark_launched(target: TargetApp) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let _ = std::fs::write(target.launch_marker(), now.to_string());
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
    target: TargetApp,
    once: bool,
    stop: Arc<AtomicBool>,
    mut log: F,
) -> Result<(), String> {
    ensure_live_supported(std::env::consts::OS)?;
    let port = target.port();
    let ensure_running_launched = ensure_running(target, &mut log)?;
    let js = theme_js(theme, target);
    let mut injected: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut dead: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut sessions: std::collections::HashMap<String, Cdp> = std::collections::HashMap::new();
    log(format!(
        "live theme: {} ({}) → {} — watching pages…",
        theme.name,
        theme.id,
        target.display_name()
    ));
    let started = std::time::Instant::now();
    // when WE just (re)launched the app, its pages may take minutes to boot —
    // they look dead but are merely slow. Suppress the wedge restart for two
    // minutes after our own launch so we never restart a booting app.
    let mut launched_by_us_at: Option<std::time::Instant> = if ensure_running_launched {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let mut port_was_down = false;
    let mut down_ticks = 0u32;
    let mut all_dead_since: Option<std::time::Instant> = None;
    let mut dead_probe_tick = 0u32;
    let mut wedge_restarted = false;
    let mut revive_attempted = false;
    let mut applied_once = false;
    let mut first_injection_failure_at: Option<std::time::Instant> = None;
    let mut last_injection_error: Option<String> = None;
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
                    if platform::launch_app(target, &mut log).is_err() {
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
            sessions.clear();
            port_was_down = false;
            all_dead_since = None;
        }
        let identity_confirmed = targets_belong_to(target, &list);
        if !identity_confirmed
            && list.iter().any(|entry| {
                let url = entry
                    .get("url")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                TargetApp::ALL
                    .into_iter()
                    .filter(|candidate| *candidate != target)
                    .any(|candidate| candidate.matches_identity_url(url))
            })
        {
            return Err(format!(
                "{} 端口 {port} 连接到了错误的应用",
                target.display_name()
            ));
        }

        // collect the matching page targets
        struct T<'a> {
            id: &'a str,
            url: &'a str,
            ws: &'a str,
        }
        let mut pages = Vec::new();
        for t in &list {
            if t.get("type").and_then(|v| v.as_str()) != Some("page") {
                continue;
            }
            let url = t.get("url").and_then(|v| v.as_str()).unwrap_or("");
            let tid = t.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if tid.is_empty() || !target.matches_page_url(url, identity_confirmed) {
                continue;
            }
            let ws_url = t
                .get("webSocketDebuggerUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            pages.push(T {
                id: tid,
                url,
                ws: ws_url,
            });
        }
        // forget dead ids that no longer exist
        let ids: std::collections::HashSet<&str> = pages.iter().map(|p| p.id).collect();
        dead.retain(|id| ids.contains(id.as_str()));
        injected.retain(|id| ids.contains(id.as_str()));
        sessions.retain(|id, _session| ids.contains(id.as_str()));

        // Wedge recovery (startup only): every WINDOWED target dead (the
        // background page stays responsive without a window, so it is
        // excluded) while this watcher is fresh => the app is frozen.
        // Two-stage recovery: first ACTIVATE the app (macOS freezes the
        // renderers of an occluded app; activation wakes them — the common
        // case); only if pages stay dead afterwards do we hard-kill and
        // relaunch. Never restart an app launched within the last two
        // minutes — a booting instance looks dead but is merely slow.
        let windowed: Vec<&T> = pages
            .iter()
            .filter(|p| !p.url.contains("background"))
            .collect();
        let startup = started.elapsed() < Duration::from_secs(120);
        let booting = launched_by_us_at
            .map(|t| t.elapsed() < Duration::from_secs(120))
            .unwrap_or(false)
            || launched_recently(target);
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
                platform::kill_app(target);
                let _ = platform::launch_app(target, &mut log);
                launched_by_us_at = Some(std::time::Instant::now());
                injected.clear();
                dead.clear();
                sessions.clear();
                all_dead_since = None;
                wedge_restarted = true;
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
            let since = all_dead_since.get_or_insert_with(std::time::Instant::now);
            if since.elapsed() > Duration::from_secs(8) {
                // stage 1: occluded-app freeze — just wake it up
                log("pages unresponsive — waking the app (may be occluded)…".into());
                platform::tell_app(target, "reopen", false);
                platform::tell_app(target, "activate", false);
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

        // Reuse each retained DevTools session for a cheap marker check. This
        // catches same-target reloads where the privileged app scheme does not
        // execute Page.addScriptToEvaluateOnNewDocument. Opening a fresh CDP
        // session for this check every two seconds would be much more costly.
        for p in &pages {
            if !injected.contains(p.id) {
                continue;
            }
            enum SessionState {
                Alive,
                Reinjected,
                Dead,
            }
            let state = match sessions.get_mut(p.id) {
                Some(session) => match session.evaluate_with_timeout(
                    "({href:location.href,skin:document.documentElement&&document.documentElement.getAttribute('data-skin')})",
                    Duration::from_millis(4000),
                ) {
                    Ok(value)
                        if value
                            .get("href")
                            .and_then(|href| href.as_str())
                            .is_some_and(|href| !href.contains("cross-site-support")) =>
                    {
                        if value.get("skin").and_then(|skin| skin.as_str())
                            == Some(theme.id.as_str())
                        {
                            SessionState::Alive
                        } else if session
                            .evaluate_with_timeout(&js, Duration::from_secs(10))
                            .is_ok()
                        {
                            SessionState::Reinjected
                        } else {
                            SessionState::Dead
                        }
                    }
                    _ => SessionState::Dead,
                },
                None => SessionState::Dead,
            };
            match state {
                SessionState::Alive => {}
                SessionState::Reinjected => log(format!(
                    "  re-injected after navigation: {}",
                    &p.url[..p.url.len().min(60)]
                )),
                SessionState::Dead => {
                    if let Some(session) = sessions.remove(p.id) {
                        session.close();
                    }
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
                log(format!(
                    "  target responsive again: {}",
                    &p.url[..p.url.len().min(60)]
                ));
            }
            match inject_target(p.ws, &js) {
                Ok(session) => {
                    sessions.insert(p.id.to_string(), session);
                    injected.insert(p.id.to_string());
                    applied_once = true;
                    first_injection_failure_at = None;
                    last_injection_error = None;
                    log(format!("  injected: {}", &p.url[..p.url.len().min(70)]));
                }
                Err(error) => {
                    // Unresponsive or mid-navigation: keep watching, but
                    // retain the error until the first successful injection.
                    dead.insert(p.id.to_string());
                    if !applied_once {
                        first_injection_failure_at.get_or_insert_with(std::time::Instant::now);
                        last_injection_error = Some(error.clone());
                    }
                    log(format!(
                        "  target not responding, watching quietly: {} ({error})",
                        &p.url[..p.url.len().min(60)],
                    ));
                }
            }
        }
        if !applied_once {
            let failure_elapsed = first_injection_failure_at.map(|started| started.elapsed());
            if let Some(error) =
                timed_out_injection_error(failure_elapsed, last_injection_error.as_deref())
            {
                return Err(format!("{}主题注入失败：{error}", target.display_name()));
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

pub fn restore_js() -> &'static str {
    r#"(function(){
var runtime=window.__doubaoSkinRuntime;
if(runtime&&typeof runtime.destroy==='function')runtime.destroy();
var style=document.getElementById('doubao-skin-style');if(style)style.remove();
var backdrop=document.getElementById('doubao-skin-backdrop');if(backdrop)backdrop.remove();
document.querySelectorAll('[data-doubao-theme-icon]').forEach(function(el){el.removeAttribute('data-doubao-theme-icon');});
document.querySelectorAll('[data-doubao-theme-composer]').forEach(function(el){el.removeAttribute('data-doubao-theme-composer');});
})();"#
}

/// Remove the live skin from the selected app without touching its bundle or
/// persisted data. Success means at least one responsive page executed the
/// cleanup contract; a listening port or an already-closed app is not proof.
pub fn restore<F: FnMut(String)>(target: TargetApp, mut log: F) -> Result<usize, String> {
    ensure_live_supported(std::env::consts::OS)?;
    let port = target.port();
    if !port_up(port) {
        return Err(format!(
            "{}未开放本地调试端口，请打开应用后再试",
            target.display_name()
        ));
    }
    let list = targets(port)?;
    if !targets_belong_to(target, &list) {
        return Err(format!(
            "{} 端口 {port} 属于其他程序，未执行恢复",
            target.display_name()
        ));
    }
    let mut restored = 0usize;
    let mut failures = 0usize;
    for entry in &list {
        if entry.get("type").and_then(|value| value.as_str()) != Some("page") {
            continue;
        }
        let url = entry
            .get("url")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if !target.matches_page_url(url, true) {
            continue;
        }
        let ws_url = entry
            .get("webSocketDebuggerUrl")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if ws_url.is_empty() {
            continue;
        }
        match evaluate_target(ws_url, restore_js()) {
            Ok(()) => {
                restored += 1;
                log(format!("  restored: {}", &url[..url.len().min(70)]));
            }
            Err(_) => failures += 1,
        }
    }
    if restored == 0 {
        return Err(if failures > 0 {
            format!(
                "{}页面暂时无响应，请重新打开应用后再试",
                target.display_name()
            )
        } else {
            format!(
                "没有找到可恢复的{}页面，请打开应用后再试",
                target.display_name()
            )
        });
    }
    log(format!("done — {restored} page(s) restored"));
    Ok(restored)
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

fn inject_target(ws_url: &str, js: &str) -> Result<Cdp, String> {
    let mut cdp = Cdp::connect(ws_url, Duration::from_secs(10))?;
    let result = (|| {
        // probe first: a wedged renderer accepts the socket but never answers
        // (or answers from the cross-site-support shell)
        let href = cdp.evaluate_with_timeout("location.href", Duration::from_millis(4000))?;
        if href.as_str().unwrap_or("").contains("cross-site-support") {
            return Err("shell page (real UI unloaded)".into());
        }
        cdp.call(
            "Page.addScriptToEvaluateOnNewDocument",
            serde_json::json!({"source": js}),
        )?;
        cdp.evaluate(js)?;
        Ok(())
    })();
    match result {
        Ok(()) => Ok(cdp),
        Err(error) => {
            cdp.close();
            Err(error)
        }
    }
}

fn evaluate_target(ws_url: &str, js: &str) -> Result<(), String> {
    let mut cdp = Cdp::connect(ws_url, Duration::from_secs(10))?;
    let result = cdp.evaluate(js).map(|_| ());
    cdp.close();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_mode_reports_its_platform_boundary_before_using_app_paths() {
        assert!(ensure_live_supported("macos").is_ok());
        assert!(ensure_live_supported("windows").is_ok());
        assert_eq!(
            ensure_live_supported("linux").unwrap_err(),
            "实时应用主题仅支持 macOS 和 Windows"
        );
    }

    #[test]
    fn initial_injection_failure_times_out_with_the_last_error() {
        assert_eq!(
            timed_out_injection_error(
                Some(Duration::from_secs(29)),
                Some("platform random source unavailable")
            ),
            None
        );
        assert_eq!(
            timed_out_injection_error(
                Some(Duration::from_secs(30)),
                Some("platform random source unavailable")
            ),
            Some("platform random source unavailable".to_string())
        );
    }

    #[test]
    fn target_metadata_keeps_the_two_official_apps_isolated() {
        assert_eq!(TargetApp::Doubao.id(), "doubao");
        assert_eq!(TargetApp::Doubao.display_name(), "豆包");
        assert_eq!(TargetApp::Doubao.bundle_id(), "com.bot.pc.doubao");
        assert_eq!(TargetApp::Doubao.port(), 9223);

        assert_eq!(TargetApp::DoubaoWork.id(), "doubao-work");
        assert_eq!(TargetApp::DoubaoWork.display_name(), "豆包工作");
        assert_eq!(TargetApp::DoubaoWork.bundle_id(), "com.work.pc.doubao");
        assert_eq!(TargetApp::DoubaoWork.port(), DEFAULT_PORT);
        assert_ne!(
            TargetApp::Doubao.launch_marker(),
            TargetApp::DoubaoWork.launch_marker()
        );
    }

    #[test]
    fn target_urls_require_an_app_identity_before_generic_extension_pages() {
        assert!(TargetApp::Doubao.matches_identity_url("doubao://doubao-chat/chat"));
        assert!(TargetApp::Doubao.matches_identity_url("chrome://doubao-chat/chat"));
        assert!(!TargetApp::Doubao.matches_identity_url("doubaowork://doubaowork-chat/chat"));
        assert!(TargetApp::DoubaoWork.matches_identity_url("doubaowork://doubaowork-chat/chat"));
        assert!(TargetApp::DoubaoWork.matches_identity_url("chrome://doubaowork-chat/chat"));

        let side_panel = "chrome-extension://example/side_panel.html";
        assert!(!TargetApp::Doubao.matches_page_url(side_panel, false));
        assert!(TargetApp::Doubao.matches_page_url(side_panel, true));
    }

    #[test]
    fn port_ownership_rejects_generic_or_other_app_targets() {
        let generic = vec![serde_json::json!({
            "type": "page",
            "url": "chrome-extension://example/side_panel.html"
        })];
        let doubao = vec![serde_json::json!({
            "type": "page",
            "url": "chrome://doubao-chat/chat"
        })];
        let work = vec![serde_json::json!({
            "type": "page",
            "url": "chrome://doubaowork-chat/chat"
        })];
        assert!(!targets_belong_to(TargetApp::Doubao, &generic));
        assert!(targets_belong_to(TargetApp::Doubao, &doubao));
        assert!(!targets_belong_to(TargetApp::Doubao, &work));
        assert!(targets_belong_to(TargetApp::DoubaoWork, &work));
    }

    #[test]
    fn restore_script_only_removes_skin_owned_runtime_and_markers() {
        let js = restore_js();
        for owned_marker in [
            "__doubaoSkinRuntime",
            "doubao-skin-style",
            "doubao-skin-backdrop",
            "data-doubao-theme-icon",
            "data-doubao-theme-composer",
        ] {
            assert!(js.contains(owned_marker), "missing {owned_marker}");
        }
        assert!(js.contains("runtime.destroy"));
        assert!(!js.contains("document.documentElement.removeAttribute('data-skin"));
        assert!(!js.contains("localStorage"));
        assert!(!js.contains("indexedDB"));
        assert!(!js.contains("textContent"));
    }

    #[test]
    fn live_bootstrap_marks_and_isolates_the_selected_target() {
        let themes = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../themes");
        let theme =
            crate::theme::load(&themes, "doubao-snack-giggle").expect("doubao-snack-giggle");
        let doubao = theme_js(&theme, TargetApp::Doubao);
        let work = theme_js(&theme, TargetApp::DoubaoWork);

        assert!(doubao.contains("TARGET=\"doubao\""));
        assert!(work.contains("TARGET=\"doubao-work\""));
        assert!(doubao.contains("data-skin-target"));
        assert!(doubao.contains(
            "html[data-skin][data-skin-target=doubao] #chat-route-main{background-color:transparent!important;}"
        ));
        assert!(!work.contains("TARGET=\"doubao\","));
    }
}

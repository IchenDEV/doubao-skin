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
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::theme::Theme;
use crate::ws::Cdp;

mod platform;

pub const DEFAULT_PORT: u16 = 9222;
pub const DOUBAO_PORT: u16 = 9223;
pub const WORKBUDDY_PORT: u16 = 9224;

const WORKBUDDY_MACOS_RENDERER_URL: &str =
    "file:///Applications/WorkBuddy.app/Contents/Resources/app.asar/renderer/index.html";

const GENERIC_PAGE_PATTERNS: &[&str] = &["side_panel.html", "popup.html", "options.html"];
const INITIAL_INJECTION_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortLossPolicy {
    Relaunch,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MissingPortAction {
    Wait,
    Relaunch,
    Stop,
}

fn missing_port_action(once: bool, policy: PortLossPolicy, down_ticks: u32) -> MissingPortAction {
    if once || policy == PortLossPolicy::Stop {
        MissingPortAction::Stop
    } else if down_ticks >= 5 {
        MissingPortAction::Relaunch
    } else {
        MissingPortAction::Wait
    }
}

fn ensure_live_supported(target_os: &str, _target: TargetApp) -> Result<(), String> {
    if matches!(target_os, "macos" | "windows") {
        Ok(())
    } else {
        Err("实时应用主题仅支持 macOS 和 Windows".into())
    }
}

fn strict_percent_decode(value: &str) -> Option<String> {
    fn hex_value(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }

    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = hex_value(*bytes.get(index + 1)?)?;
            let low = hex_value(*bytes.get(index + 2)?)?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn windows_file_url_path(url: &str) -> Option<String> {
    let url = url.split(['?', '#']).next()?;
    let prefix = url.get(..8)?;
    if !prefix.eq_ignore_ascii_case("file:///") {
        return None;
    }
    let path = strict_percent_decode(&url[8..])?;
    if path.contains('\\')
        || path
            .split('/')
            .any(|segment| segment == "." || segment == "..")
    {
        return None;
    }
    Some(path)
}

fn windows_workbuddy_renderer_path(binary: &Path) -> Option<String> {
    let directory = binary.parent()?;
    let path = directory.join("resources/app.asar/renderer/index.html");
    Some(path.to_string_lossy().replace('\\', "/"))
}

fn matches_workbuddy_renderer_for_platform(
    target_os: &str,
    url: &str,
    installed_binary: Option<&Path>,
) -> bool {
    match target_os {
        "macos" => url.split(['?', '#']).next() == Some(WORKBUDDY_MACOS_RENDERER_URL),
        "windows" => {
            let Some(expected) = installed_binary.and_then(windows_workbuddy_renderer_path) else {
                return false;
            };
            windows_file_url_path(url).is_some_and(|actual| actual.eq_ignore_ascii_case(&expected))
        }
        _ => false,
    }
}

fn installed_workbuddy_binary_for_identity() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        platform::installed_binary(TargetApp::WorkBuddy)
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
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
    WorkBuddy,
}

impl TargetApp {
    pub const ALL: [Self; 3] = [Self::Doubao, Self::DoubaoWork, Self::WorkBuddy];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Doubao => "doubao",
            Self::DoubaoWork => "doubao-work",
            Self::WorkBuddy => "workbuddy",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|target| target.id() == id)
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Doubao => "豆包",
            Self::DoubaoWork => "豆包工作",
            Self::WorkBuddy => "WorkBuddy",
        }
    }

    pub const fn bundle_id(self) -> &'static str {
        match self {
            Self::Doubao => "com.bot.pc.doubao",
            Self::DoubaoWork => "com.work.pc.doubao",
            Self::WorkBuddy => "com.workbuddy.workbuddy",
        }
    }

    pub fn port(self) -> u16 {
        let (override_name, fallback) = match self {
            Self::Doubao => ("DOUBAO_SKIN_DOUBAO_CDP_PORT", DOUBAO_PORT),
            Self::DoubaoWork => ("DOUBAO_SKIN_DOUBAO_WORK_CDP_PORT", DEFAULT_PORT),
            Self::WorkBuddy => ("DOUBAO_SKIN_WORKBUDDY_CDP_PORT", WORKBUDDY_PORT),
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
            Self::WorkBuddy => std::env::temp_dir().join("doubao-skin-workbuddy-launched-at"),
        }
    }

    pub fn is_installed(self) -> bool {
        self.installed_binary().is_some()
    }

    pub fn is_running(self) -> bool {
        platform::app_is_running(self)
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
            Self::WorkBuddy => matches_workbuddy_renderer_for_platform(
                std::env::consts::OS,
                url,
                installed_workbuddy_binary_for_identity().as_deref(),
            ),
        }
    }

    fn matches_page_url(self, url: &str, identity_confirmed: bool) -> bool {
        match self {
            Self::WorkBuddy => self.matches_identity_url(url),
            Self::Doubao | Self::DoubaoWork => {
                self.matches_identity_url(url)
                    || (identity_confirmed
                        && GENERIC_PAGE_PATTERNS
                            .iter()
                            .any(|pattern| url.contains(pattern)))
            }
        }
    }

    pub const fn relaunch_after_port_loss(self) -> bool {
        !matches!(self, Self::WorkBuddy)
    }
}

/// Returns whether the exact executable path is currently running.
///
/// This is used by the bundled login item to yield ownership while the main
/// desktop app is open. Platform process commands stay behind the live
/// adapter rather than leaking into the helper binary.
pub fn executable_is_running(path: &std::path::Path) -> bool {
    platform::executable_is_running(path)
}

pub fn theme_js(theme: &Theme, target: TargetApp) -> String {
    theme.bootstrap_js_for_target(target)
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

fn port_listening(port: u16) -> bool {
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&address, Duration::from_millis(250)).is_ok()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrepareState {
    Ready,
    LaunchRequired,
    RestartConfirmationRequired,
    NotInstalled,
    WrongPortOwner,
}

pub(crate) fn prepare_state_for_observation(
    target: TargetApp,
    installed: bool,
    running: bool,
    port_listening: bool,
    port_owned: bool,
) -> PrepareState {
    if port_listening {
        return if port_owned {
            PrepareState::Ready
        } else {
            PrepareState::WrongPortOwner
        };
    }
    if !installed {
        return PrepareState::NotInstalled;
    }
    if target == TargetApp::WorkBuddy && running {
        PrepareState::RestartConfirmationRequired
    } else {
        PrepareState::LaunchRequired
    }
}

fn process_running(target: TargetApp) -> bool {
    platform::process_running(target)
}

pub fn prepare_state(target: TargetApp) -> Result<PrepareState, String> {
    let port = target.port();
    let listening = port_listening(port);
    let owned = if listening {
        targets(port)
            .map(|list| targets_belong_to(target, &list))
            .unwrap_or(false)
    } else {
        false
    };
    Ok(prepare_state_for_observation(
        target,
        target.is_installed(),
        process_running(target),
        listening,
        owned,
    ))
}

fn ensure_running<F: FnMut(String)>(
    target: TargetApp,
    allow_restart: bool,
    mut log: F,
) -> Result<bool, String> {
    let port = target.port();
    match prepare_state(target)? {
        PrepareState::Ready => {
            log(format!(
                "{} debug port already up — reusing the running instance",
                target.display_name()
            ));
            return Ok(false);
        }
        PrepareState::WrongPortOwner => {
            return Err(format!(
                "{} 端口 {port} 已被其他程序占用，请关闭占用后再试",
                target.display_name()
            ));
        }
        PrepareState::NotInstalled => {
            return Err(format!(
                "未找到{}：{}",
                target.display_name(),
                target.install_hint()
            ));
        }
        PrepareState::RestartConfirmationRequired if !allow_restart => {
            return Err("WorkBuddy 正在运行，需要明确确认重启后才能应用主题".into());
        }
        PrepareState::RestartConfirmationRequired => {
            platform::tell_app(target, "quit", false);
            for _ in 0..12 {
                if !process_running(target) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
            }
            if process_running(target) {
                platform::kill_app(target);
            }
        }
        PrepareState::LaunchRequired if target != TargetApp::WorkBuddy => {
            // Preserve the established Doubao behavior: a running instance
            // without the debug flag is restarted before launching.
            platform::tell_app(target, "quit", false);
            std::thread::sleep(Duration::from_secs(3));
            platform::kill_app(target);
        }
        PrepareState::LaunchRequired => {}
    }
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
    log: F,
) -> Result<(), String> {
    run_with_restart_permission(theme, target, once, stop, false, log)
}

pub fn run_with_restart_permission<F: FnMut(String)>(
    theme: &Theme,
    target: TargetApp,
    once: bool,
    stop: Arc<AtomicBool>,
    allow_restart: bool,
    log: F,
) -> Result<(), String> {
    let port_loss_policy = if once || !target.relaunch_after_port_loss() {
        PortLossPolicy::Stop
    } else {
        PortLossPolicy::Relaunch
    };
    run_with_options(
        theme,
        target,
        once,
        port_loss_policy,
        stop,
        allow_restart,
        log,
    )
}

pub fn run_with_policy<F: FnMut(String)>(
    theme: &Theme,
    target: TargetApp,
    once: bool,
    port_loss_policy: PortLossPolicy,
    stop: Arc<AtomicBool>,
    log: F,
) -> Result<(), String> {
    run_with_options(theme, target, once, port_loss_policy, stop, true, log)
}

fn run_with_options<F: FnMut(String)>(
    theme: &Theme,
    target: TargetApp,
    once: bool,
    port_loss_policy: PortLossPolicy,
    stop: Arc<AtomicBool>,
    allow_restart: bool,
    mut log: F,
) -> Result<(), String> {
    ensure_live_supported(std::env::consts::OS, target)?;
    if !theme.supports_target(target) {
        return Err(format!(
            "主题 {} 不支持 {}",
            theme.name,
            target.display_name()
        ));
    }
    let port = target.port();
    let ensure_running_launched = ensure_running(target, allow_restart, &mut log)?;
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
                // Keep the old CLI watcher behavior, but let user-facing
                // automatic theming stop when the target is deliberately quit.
                if !port_was_down {
                    log("debug port went away — waiting for the app to come back…".into());
                    port_was_down = true;
                }
                down_ticks = down_ticks.saturating_add(1);
                match missing_port_action(once, port_loss_policy, down_ticks) {
                    MissingPortAction::Stop => {
                        log("target app closed — automatic theme watcher stopped".into());
                        return Ok(());
                    }
                    MissingPortAction::Relaunch if !port_up(port) => {
                        log("relaunching app with the debug port…".into());
                        if platform::launch_app(target, &mut log).is_err() {
                            log("relaunch failed, will keep waiting".into());
                        } else {
                            launched_by_us_at = Some(std::time::Instant::now());
                        }
                        down_ticks = 0;
                    }
                    MissingPortAction::Relaunch | MissingPortAction::Wait => {}
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
        if target.relaunch_after_port_loss()
            && startup
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
    ensure_live_supported(std::env::consts::OS, target)?;
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
        assert!(ensure_live_supported("macos", TargetApp::WorkBuddy).is_ok());
        assert!(ensure_live_supported("windows", TargetApp::Doubao).is_ok());
        assert!(ensure_live_supported("windows", TargetApp::WorkBuddy).is_ok());
        assert_eq!(
            ensure_live_supported("linux", TargetApp::Doubao).unwrap_err(),
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
    fn stop_on_target_exit_never_relaunches() {
        assert_eq!(
            missing_port_action(false, PortLossPolicy::Stop, 1),
            MissingPortAction::Stop
        );
        assert_eq!(
            missing_port_action(false, PortLossPolicy::Relaunch, 4),
            MissingPortAction::Wait
        );
        assert_eq!(
            missing_port_action(false, PortLossPolicy::Relaunch, 5),
            MissingPortAction::Relaunch
        );
        assert_eq!(
            missing_port_action(true, PortLossPolicy::Relaunch, 5),
            MissingPortAction::Stop
        );
    }

    #[test]
    fn target_metadata_keeps_all_supported_apps_isolated() {
        assert_eq!(TargetApp::Doubao.id(), "doubao");
        assert_eq!(TargetApp::Doubao.display_name(), "豆包");
        assert_eq!(TargetApp::Doubao.bundle_id(), "com.bot.pc.doubao");
        assert_eq!(TargetApp::Doubao.port(), 9223);

        assert_eq!(TargetApp::DoubaoWork.id(), "doubao-work");
        assert_eq!(TargetApp::DoubaoWork.display_name(), "豆包工作");
        assert_eq!(TargetApp::DoubaoWork.bundle_id(), "com.work.pc.doubao");
        assert_eq!(TargetApp::DoubaoWork.port(), DEFAULT_PORT);

        assert_eq!(TargetApp::WorkBuddy.id(), "workbuddy");
        assert_eq!(TargetApp::WorkBuddy.display_name(), "WorkBuddy");
        assert_eq!(TargetApp::WorkBuddy.bundle_id(), "com.workbuddy.workbuddy");
        assert_eq!(TargetApp::WorkBuddy.port(), 9224);
        assert_ne!(
            TargetApp::Doubao.launch_marker(),
            TargetApp::DoubaoWork.launch_marker()
        );
        assert_ne!(
            TargetApp::DoubaoWork.launch_marker(),
            TargetApp::WorkBuddy.launch_marker()
        );
        assert_eq!(TargetApp::ALL.len(), 3);
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
    fn macos_workbuddy_only_matches_the_verified_main_renderer() {
        let root =
            "file:///Applications/WorkBuddy.app/Contents/Resources/app.asar/renderer/index.html";
        for url in [
            root.to_string(),
            format!("{root}#/home"),
            format!("{root}?source=launch"),
            format!("{root}?source=launch#/home"),
        ] {
            assert!(
                matches_workbuddy_renderer_for_platform("macos", &url, None),
                "{url}"
            );
        }
        for url in [
            "file:///Applications/WorkBuddy.app/Contents/Resources/app.asar/renderer/other.html",
            "file:///Applications/Other.app/Contents/Resources/app.asar/renderer/index.html",
            "file:///tmp/index.html",
            "https://www.workbuddy.cn/space/home",
            "devtools://devtools/bundled/inspector.html",
            "chrome-extension://example/side_panel.html",
        ] {
            assert!(
                !matches_workbuddy_renderer_for_platform("macos", url, None),
                "{url}"
            );
        }
    }

    #[test]
    fn windows_workbuddy_renderer_is_derived_from_the_installed_binary() {
        let binary =
            PathBuf::from("C:/Users/Wei Li/AppData/Local/Programs/WorkBuddy/WorkBuddy.exe");
        for url in [
            "file:///C:/Users/Wei%20Li/AppData/Local/Programs/WorkBuddy/resources/app.asar/renderer/index.html",
            "file:///c:/users/wei%20li/appdata/local/programs/workbuddy/resources/app.asar/renderer/index.html#/home",
            "file:///C:/Users/Wei%20Li/AppData/Local/Programs/WorkBuddy/resources/app.asar/renderer/index.html?source=launch#/home",
        ] {
            assert!(
                matches_workbuddy_renderer_for_platform("windows", url, Some(&binary)),
                "{url}"
            );
        }
        for url in [
            "file:///C:/Users/Other/AppData/Local/Programs/WorkBuddy/resources/app.asar/renderer/index.html",
            "file:///C:/Users/Wei%20Li/AppData/Local/Programs/Other/resources/app.asar/renderer/index.html",
            "file:///C:/Users/Wei%20Li/AppData/Local/Programs/WorkBuddy/resources/app.asar/renderer/other.html",
            "file:///C:/Users/Wei%20Li/AppData/Local/Programs/WorkBuddy/other/../resources/app.asar/renderer/index.html",
            "file:///C:/Users/Wei%ZZLi/AppData/Local/Programs/WorkBuddy/resources/app.asar/renderer/index.html",
            "https://www.workbuddy.cn/space/home",
            "devtools://devtools/bundled/inspector.html",
            "chrome-extension://example/side_panel.html",
        ] {
            assert!(
                !matches_workbuddy_renderer_for_platform("windows", url, Some(&binary)),
                "{url}"
            );
        }
        assert!(!matches_workbuddy_renderer_for_platform(
            "windows",
            "file:///C:/Users/Wei%20Li/AppData/Local/Programs/WorkBuddy/resources/app.asar/renderer/index.html",
            None,
        ));
    }

    #[test]
    fn windows_workbuddy_renderer_decodes_utf8_paths_strictly() {
        let binary = PathBuf::from("C:/Users/测试/AppData/Local/Programs/WorkBuddy/WorkBuddy.exe");
        assert!(matches_workbuddy_renderer_for_platform(
            "windows",
            "file:///C:/Users/%E6%B5%8B%E8%AF%95/AppData/Local/Programs/WorkBuddy/resources/app.asar/renderer/index.html",
            Some(&binary),
        ));
        assert!(!matches_workbuddy_renderer_for_platform(
            "windows",
            "file:///C:/Users/%FF/AppData/Local/Programs/WorkBuddy/resources/app.asar/renderer/index.html",
            Some(&binary),
        ));
    }

    #[test]
    fn workbuddy_lifecycle_requires_explicit_restart_and_never_relaunches_after_quit() {
        assert_eq!(
            prepare_state_for_observation(TargetApp::WorkBuddy, false, false, false, false),
            PrepareState::NotInstalled
        );
        assert_eq!(
            prepare_state_for_observation(TargetApp::WorkBuddy, true, false, false, false),
            PrepareState::LaunchRequired
        );
        assert_eq!(
            prepare_state_for_observation(TargetApp::WorkBuddy, true, true, false, false),
            PrepareState::RestartConfirmationRequired
        );
        assert_eq!(
            prepare_state_for_observation(TargetApp::WorkBuddy, true, true, true, true),
            PrepareState::Ready
        );
        assert_eq!(
            prepare_state_for_observation(TargetApp::WorkBuddy, true, true, true, false),
            PrepareState::WrongPortOwner
        );
        assert_eq!(
            prepare_state_for_observation(TargetApp::Doubao, true, true, false, false),
            PrepareState::LaunchRequired
        );
        assert_eq!(
            prepare_state_for_observation(TargetApp::DoubaoWork, true, true, false, false),
            PrepareState::LaunchRequired
        );
        assert!(!TargetApp::WorkBuddy.relaunch_after_port_loss());
        assert!(TargetApp::Doubao.relaunch_after_port_loss());
        assert!(TargetApp::DoubaoWork.relaunch_after_port_loss());
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
        let workbuddy = theme_js(&theme, TargetApp::WorkBuddy);

        assert!(doubao.contains("TARGET=\"doubao\""));
        assert!(work.contains("TARGET=\"doubao-work\""));
        assert!(workbuddy.contains("TARGET=\"workbuddy\""));
        assert!(doubao.contains("data-skin-target"));
        assert!(doubao.contains(
            "html[data-skin][data-skin-target=doubao] #chat-route-main{background-color:transparent!important;}"
        ));
        assert!(!work.contains("TARGET=\"doubao\","));
    }
}

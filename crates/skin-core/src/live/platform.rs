//! Operating-system adapters for locating and controlling the official apps.
//!
//! Keep filesystem conventions, registries, and process commands in this file
//! so the CDP/theme runtime stays platform-neutral.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::{mark_launched, TargetApp};

fn path_override_name(target: TargetApp) -> &'static str {
    match target {
        TargetApp::Doubao => "DOUBAO_SKIN_DOUBAO_PATH",
        TargetApp::DoubaoWork => "DOUBAO_SKIN_DOUBAO_WORK_PATH",
    }
}

fn explicit_binary(target: TargetApp) -> Option<PathBuf> {
    std::env::var_os(path_override_name(target))
        .map(PathBuf::from)
        .filter(|path| path.is_file())
}

#[cfg(target_os = "macos")]
fn macos_binary_relative_path(target: TargetApp) -> &'static str {
    match target {
        TargetApp::Doubao => "Contents/MacOS/Doubao",
        TargetApp::DoubaoWork => "Contents/MacOS/DoubaoWork",
    }
}

#[cfg(any(test, target_os = "macos"))]
fn macos_bundle_from_launch_services_output(output: &[u8]) -> Option<PathBuf> {
    let value = std::str::from_utf8(output).ok()?.trim();
    let path = PathBuf::from(value);
    (path.is_absolute() && path.extension().is_some_and(|extension| extension == "app"))
        .then_some(path)
}

#[cfg(target_os = "macos")]
fn registered_macos_bundle(target: TargetApp) -> Option<PathBuf> {
    let script = format!(
        "POSIX path of (path to application id \"{}\")",
        target.bundle_id()
    );
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| macos_bundle_from_launch_services_output(&output.stdout))?
}

#[cfg(target_os = "macos")]
fn bundle_from_explicit_binary(target: TargetApp) -> Option<PathBuf> {
    let binary = explicit_binary(target)?;
    binary
        .ancestors()
        .find(|path| path.extension().is_some_and(|extension| extension == "app"))
        .map(Path::to_path_buf)
}

#[cfg(target_os = "macos")]
pub(super) fn installed_app_bundle(target: TargetApp) -> Option<PathBuf> {
    bundle_from_explicit_binary(target).or_else(|| registered_macos_bundle(target))
}

#[cfg(not(target_os = "macos"))]
pub(super) fn installed_app_bundle(_target: TargetApp) -> Option<PathBuf> {
    None
}

#[cfg(target_os = "macos")]
pub(super) fn installed_binary(target: TargetApp) -> Option<PathBuf> {
    explicit_binary(target).or_else(|| {
        installed_app_bundle(target)
            .map(|bundle| bundle.join(macos_binary_relative_path(target)))
            .filter(|path| path.is_file())
    })
}

#[cfg(target_os = "windows")]
pub(super) fn installed_binary(target: TargetApp) -> Option<PathBuf> {
    windows_installed_binary(target)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn installed_binary(_target: TargetApp) -> Option<PathBuf> {
    None
}

#[cfg(target_os = "macos")]
pub(super) fn install_hint(target: TargetApp) -> String {
    format!("macOS bundle id {}", target.bundle_id())
}

#[cfg(target_os = "windows")]
pub(super) fn install_hint(target: TargetApp) -> String {
    format!(
        "Windows installed-app registry or {}",
        path_override_name(target)
    )
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn install_hint(_target: TargetApp) -> String {
    "unsupported platform".into()
}

#[cfg(any(test, target_os = "windows"))]
fn windows_relative_binary_paths(target: TargetApp) -> &'static [&'static str] {
    match target {
        TargetApp::Doubao => &[
            "Doubao/Application/Doubao.exe",
            "Doubao/Application/app/Doubao.exe",
            "Doubao/Doubao.exe",
        ],
        TargetApp::DoubaoWork => &[
            "DoubaoWork/Application/DoubaoWork.exe",
            "DoubaoWork/Application/app/DoubaoWork.exe",
            "DoubaoWork/DoubaoWork.exe",
        ],
    }
}

#[cfg(any(test, target_os = "windows"))]
fn windows_executable_names(target: TargetApp) -> &'static [&'static str] {
    match target {
        TargetApp::Doubao => &["Doubao.exe"],
        TargetApp::DoubaoWork => &["DoubaoWork.exe"],
    }
}

#[cfg(any(test, target_os = "windows"))]
fn windows_launch_working_directory(binary: &Path) -> Result<PathBuf, String> {
    binary
        .parent()
        .filter(|directory| !directory.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .ok_or_else(|| format!("cannot resolve launch directory for {}", binary.display()))
}

#[cfg(any(test, target_os = "windows"))]
fn windows_app_root(target: TargetApp, local_app_data: &Path) -> PathBuf {
    local_app_data.join(match target {
        TargetApp::Doubao => "Doubao",
        TargetApp::DoubaoWork => "DoubaoWork",
    })
}

#[cfg(any(test, target_os = "windows"))]
fn find_named_executable(root: &Path, names: &[&str], depth: u8) -> Option<PathBuf> {
    if !root.is_dir() {
        return None;
    }
    for name in names {
        let direct = root.join(name);
        if direct.is_file() {
            return Some(direct);
        }
    }
    if depth == 0 {
        return None;
    }
    for entry in std::fs::read_dir(root).ok()?.flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            if let Some(binary) = find_named_executable(&entry.path(), names, depth - 1) {
                return Some(binary);
            }
        }
    }
    None
}

#[cfg(any(test, target_os = "windows"))]
fn windows_binary_in_root(target: TargetApp, local_app_data: &Path) -> Option<PathBuf> {
    windows_relative_binary_paths(target)
        .iter()
        .map(|relative| local_app_data.join(relative))
        .find(|candidate| candidate.is_file())
        .or_else(|| {
            find_named_executable(
                &windows_app_root(target, local_app_data),
                windows_executable_names(target),
                4,
            )
        })
}

#[cfg(any(test, target_os = "windows"))]
fn registry_entry_matches(target: TargetApp, key_name: &str, display_name: &str) -> bool {
    let identity = format!("{key_name} {display_name}").to_ascii_lowercase();
    let is_doubao = identity.contains("doubao") || identity.contains("豆包");
    let is_work = identity.contains("work") || identity.contains("工作");
    is_doubao
        && match target {
            TargetApp::Doubao => !is_work,
            TargetApp::DoubaoWork => is_work,
        }
}

#[cfg(any(test, target_os = "windows"))]
fn display_icon_path(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    let path = if let Some(quoted) = value.strip_prefix('"') {
        quoted.split('"').next().unwrap_or(quoted)
    } else {
        value.split(',').next().unwrap_or(value).trim()
    };
    (!path.is_empty()).then(|| PathBuf::from(path))
}

#[cfg(target_os = "windows")]
fn windows_registry_install_paths(target: TargetApp) -> Vec<PathBuf> {
    use windows_registry::{CURRENT_USER, LOCAL_MACHINE};

    const UNINSTALL: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
    let mut paths = Vec::new();
    for hive in [CURRENT_USER, LOCAL_MACHINE] {
        let Ok(uninstall) = hive.open(UNINSTALL) else {
            continue;
        };
        let Ok(keys) = uninstall.keys() else {
            continue;
        };
        for key_name in keys {
            let Ok(entry) = uninstall.open(&key_name) else {
                continue;
            };
            let display_name = entry.get_string("DisplayName").unwrap_or_default();
            if !registry_entry_matches(target, &key_name, &display_name) {
                continue;
            }
            if let Ok(icon) = entry.get_string("DisplayIcon") {
                if let Some(path) = display_icon_path(&icon) {
                    paths.push(path);
                }
            }
            if let Ok(location) = entry.get_string("InstallLocation") {
                if !location.trim().is_empty() {
                    paths.push(PathBuf::from(location.trim()));
                }
            }
        }
    }
    paths
}

#[cfg(target_os = "windows")]
fn binary_from_install_path(target: TargetApp, path: &Path) -> Option<PathBuf> {
    if path.is_file()
        && path.file_name().is_some_and(|file_name| {
            windows_executable_names(target)
                .iter()
                .any(|name| file_name.to_string_lossy().eq_ignore_ascii_case(name))
        })
    {
        return Some(path.to_path_buf());
    }
    find_named_executable(path, windows_executable_names(target), 5)
}

#[cfg(target_os = "windows")]
fn windows_installed_binary_uncached(target: TargetApp) -> Option<PathBuf> {
    if let Some(binary) = explicit_binary(target) {
        return Some(binary);
    }
    if let Some(binary) =
        dirs::data_local_dir().and_then(|root| windows_binary_in_root(target, &root))
    {
        return Some(binary);
    }
    for path in windows_registry_install_paths(target) {
        if let Some(binary) = binary_from_install_path(target, &path) {
            return Some(binary);
        }
    }
    for root_name in ["ProgramFiles", "ProgramFiles(x86)"] {
        let Some(root) = std::env::var_os(root_name).map(PathBuf::from) else {
            continue;
        };
        for folder in ["ByteDance", "Doubao", "DoubaoWork"] {
            if let Some(binary) = binary_from_install_path(target, &root.join(folder)) {
                return Some(binary);
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn windows_installed_binary(target: TargetApp) -> Option<PathBuf> {
    use std::sync::OnceLock;

    static INSTALLATIONS: OnceLock<[Option<PathBuf>; 2]> = OnceLock::new();
    let installations = INSTALLATIONS.get_or_init(|| {
        [
            windows_installed_binary_uncached(TargetApp::Doubao),
            windows_installed_binary_uncached(TargetApp::DoubaoWork),
        ]
    });
    installations[match target {
        TargetApp::Doubao => 0,
        TargetApp::DoubaoWork => 1,
    }]
    .clone()
}

#[cfg(target_os = "macos")]
pub(super) fn tell_app(target: TargetApp, action: &str, spawn: bool) {
    let script = format!("tell application id \"{}\" to {action}", target.bundle_id());
    let mut command = Command::new("osascript");
    command
        .args(["-e", &script])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if spawn {
        let _ = command.spawn();
    } else {
        let _ = command.output();
    }
}

#[cfg(target_os = "windows")]
pub(super) fn tell_app(target: TargetApp, action: &str, _spawn: bool) {
    if action != "quit" {
        return;
    }
    let process_name = installed_binary(target)
        .and_then(|path| path.file_name().map(|name| name.to_owned()))
        .unwrap_or_else(|| windows_executable_names(target)[0].into());
    let _ = Command::new("taskkill")
        .arg("/IM")
        .arg(process_name)
        .arg("/T")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn tell_app(_target: TargetApp, _action: &str, _spawn: bool) {}

#[cfg(target_os = "macos")]
pub(super) fn launch_app<F: FnMut(String)>(target: TargetApp, mut log: F) -> Result<(), String> {
    let port = target.port();
    log(format!(
        "launching {} --remote-debugging-port={port}",
        target.display_name()
    ));
    mark_launched(target);
    Command::new("open")
        .arg("-b")
        .arg(target.bundle_id())
        .arg("--args")
        .arg(format!("--remote-debugging-port={port}"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|error| format!("cannot launch app: {error}"))?;
    tell_app(target, "reopen", true);
    Ok(())
}

#[cfg(target_os = "windows")]
pub(super) fn launch_app<F: FnMut(String)>(target: TargetApp, mut log: F) -> Result<(), String> {
    let port = target.port();
    let binary = installed_binary(target)
        .ok_or_else(|| format!("未找到{}：{}", target.display_name(), install_hint(target)))?;
    let working_directory = windows_launch_working_directory(&binary)?;
    log(format!(
        "launching {} --remote-debugging-port={port}",
        target.display_name()
    ));
    mark_launched(target);
    Command::new(&binary)
        .current_dir(working_directory)
        .arg(format!("--remote-debugging-port={port}"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|error| format!("cannot launch app: {error}"))?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn launch_app<F: FnMut(String)>(_target: TargetApp, _log: F) -> Result<(), String> {
    Err("live mode is supported only on macOS and Windows".into())
}

#[cfg(target_os = "macos")]
pub(super) fn kill_app(target: TargetApp) {
    let Some(binary) = installed_binary(target) else {
        return;
    };
    let process_pattern = binary.to_string_lossy();
    let _ = Command::new("pkill")
        .args(["-f", process_pattern.as_ref()])
        .output();
    for _ in 0..20 {
        let running = Command::new("pgrep")
            .args(["-f", process_pattern.as_ref()])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if !running {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

#[cfg(target_os = "windows")]
pub(super) fn kill_app(target: TargetApp) {
    let process_name = installed_binary(target)
        .and_then(|path| path.file_name().map(|name| name.to_owned()))
        .unwrap_or_else(|| windows_executable_names(target)[0].into());
    let _ = Command::new("taskkill")
        .arg("/F")
        .arg("/IM")
        .arg(&process_name)
        .arg("/T")
        .output();
    for _ in 0..20 {
        let running = Command::new("tasklist")
            .args([
                "/FI",
                &format!("IMAGENAME eq {}", process_name.to_string_lossy()),
                "/NH",
            ])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .to_ascii_lowercase()
                    .contains(&process_name.to_string_lossy().to_ascii_lowercase())
            })
            .unwrap_or(false);
        if !running {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn kill_app(_target: TargetApp) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_services_output_must_be_an_absolute_app_bundle() {
        assert_eq!(
            macos_bundle_from_launch_services_output(b"/Installed/DoubaoWork.app/\n"),
            Some(PathBuf::from("/Installed/DoubaoWork.app"))
        );
        assert_eq!(
            macos_bundle_from_launch_services_output(b"relative/DoubaoWork.app\n"),
            None
        );
        assert_eq!(
            macos_bundle_from_launch_services_output(b"/Installed/DoubaoWork\n"),
            None
        );
    }

    #[test]
    fn windows_install_detection_checks_both_per_user_targets() {
        let root = std::env::temp_dir().join(format!(
            "doubao-skin-windows-detection-{}",
            std::process::id()
        ));
        let doubao = root.join("Doubao/Application/Doubao.exe");
        let work = root.join("DoubaoWork/Application/DoubaoWork.exe");
        let internal_doubao = root.join("Doubao/Application/app/Doubao.exe");
        let internal_work = root.join("DoubaoWork/Application/app/DoubaoWork.exe");
        std::fs::create_dir_all(internal_doubao.parent().unwrap()).unwrap();
        std::fs::create_dir_all(internal_work.parent().unwrap()).unwrap();
        std::fs::write(&doubao, []).unwrap();
        std::fs::write(&work, []).unwrap();
        std::fs::write(&internal_doubao, []).unwrap();
        std::fs::write(&internal_work, []).unwrap();

        assert_eq!(
            windows_binary_in_root(TargetApp::Doubao, &root),
            Some(doubao)
        );
        assert_eq!(
            windows_binary_in_root(TargetApp::DoubaoWork, &root),
            Some(work)
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn windows_registry_detection_keeps_the_two_products_isolated() {
        assert!(registry_entry_matches(TargetApp::Doubao, "Doubao", "豆包"));
        assert!(!registry_entry_matches(
            TargetApp::DoubaoWork,
            "Doubao",
            "豆包"
        ));
        assert!(registry_entry_matches(
            TargetApp::DoubaoWork,
            "DoubaoWork",
            "豆包工作"
        ));
        assert!(!registry_entry_matches(
            TargetApp::Doubao,
            "DoubaoWork",
            "豆包工作"
        ));
    }

    #[test]
    fn windows_registry_display_icon_removes_quotes_and_resource_index() {
        assert_eq!(
            display_icon_path(r#""C:\Users\tester\Doubao\Doubao.exe",0"#),
            Some(PathBuf::from(r"C:\Users\tester\Doubao\Doubao.exe"))
        );
    }

    #[test]
    fn windows_launches_from_the_installed_binary_directory() {
        let binary = Path::new("C:/Users/tester/Doubao/Application/app/Doubao.exe");
        assert_eq!(
            windows_launch_working_directory(binary).unwrap(),
            PathBuf::from("C:/Users/tester/Doubao/Application/app")
        );
        assert!(windows_launch_working_directory(Path::new("Doubao.exe")).is_err());
    }
}

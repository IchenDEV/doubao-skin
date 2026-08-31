//! Persisted state for restoring the last successfully applied live theme.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::live::TargetApp;
use crate::theme;

const SCHEMA_VERSION: u32 = 1;
static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq)]
pub struct LastApplied {
    target: TargetApp,
    theme_id: String,
    surface_opacity: Option<f32>,
}

impl LastApplied {
    pub fn new(
        target: TargetApp,
        theme_id: impl Into<String>,
        surface_opacity: Option<f32>,
    ) -> Result<Self, String> {
        let value = Self {
            target,
            theme_id: theme_id.into(),
            surface_opacity,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn target(&self) -> TargetApp {
        self.target
    }

    pub fn theme_id(&self) -> &str {
        &self.theme_id
    }

    pub fn surface_opacity(&self) -> Option<f32> {
        self.surface_opacity
    }

    fn validate(&self) -> Result<(), String> {
        let id = self.theme_id.trim();
        if id.is_empty() || id.len() > 128 {
            return Err("自动恢复主题 ID 无效".into());
        }
        if self
            .surface_opacity
            .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err("自动恢复界面透明度无效".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AutoThemeSettings {
    last_applied: Option<LastApplied>,
    keep_requested: bool,
    open_at_login: bool,
}

impl AutoThemeSettings {
    pub fn last_applied(&self) -> Option<&LastApplied> {
        self.last_applied.as_ref()
    }

    pub fn set_last_applied(&mut self, last_applied: LastApplied) {
        self.last_applied = Some(last_applied);
    }

    pub fn clear_last_applied(&mut self) {
        self.last_applied = None;
    }

    pub fn keep_requested(&self) -> bool {
        self.keep_requested
    }

    pub fn set_keep_requested(&mut self, requested: bool) {
        self.keep_requested = requested;
        if !requested {
            self.open_at_login = false;
        }
    }

    pub fn open_at_login(&self) -> bool {
        self.open_at_login
    }

    pub fn set_open_at_login(&mut self, enabled: bool) {
        self.open_at_login = enabled && self.keep_requested;
    }

    pub fn clear_and_disable(&mut self) {
        self.last_applied = None;
        self.keep_requested = false;
        self.open_at_login = false;
    }

    fn validate(&self) -> Result<(), String> {
        if self.open_at_login && !self.keep_requested {
            return Err("登录时打开需要先开启自动保持主题".into());
        }
        if let Some(last_applied) = &self.last_applied {
            last_applied.validate()?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn test_only_invalid_login_dependency() -> Self {
        Self {
            last_applied: None,
            keep_requested: false,
            open_at_login: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedSettings {
    schema_version: u32,
    last_applied: Option<PersistedLastApplied>,
    keep_requested: bool,
    open_at_login: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedLastApplied {
    target: String,
    theme_id: String,
    surface_opacity: Option<f32>,
}

impl TryFrom<PersistedSettings> for AutoThemeSettings {
    type Error = String;

    fn try_from(value: PersistedSettings) -> Result<Self, Self::Error> {
        if value.schema_version != SCHEMA_VERSION {
            return Err(format!(
                "不支持的自动主题设置版本：{}",
                value.schema_version
            ));
        }
        let last_applied = value
            .last_applied
            .map(|last| {
                let target = TargetApp::from_id(&last.target)
                    .ok_or_else(|| format!("未知的目标应用：{}", last.target))?;
                LastApplied::new(target, last.theme_id, last.surface_opacity)
            })
            .transpose()?;
        let settings = Self {
            last_applied,
            keep_requested: value.keep_requested,
            open_at_login: value.open_at_login,
        };
        settings.validate()?;
        Ok(settings)
    }
}

impl From<&AutoThemeSettings> for PersistedSettings {
    fn from(value: &AutoThemeSettings) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            last_applied: value
                .last_applied
                .as_ref()
                .map(|last| PersistedLastApplied {
                    target: last.target.id().to_string(),
                    theme_id: last.theme_id.clone(),
                    surface_opacity: last.surface_opacity,
                }),
            keep_requested: value.keep_requested,
            open_at_login: value.open_at_login,
        }
    }
}

pub fn settings_path() -> PathBuf {
    theme::app_data_dir().join("auto-theme.json")
}

pub fn load() -> Result<AutoThemeSettings, String> {
    load_from(&settings_path())
}

pub fn save(settings: &AutoThemeSettings) -> Result<(), String> {
    save_to(&settings_path(), settings)
}

pub fn login_session_path() -> PathBuf {
    theme::app_data_dir().join("auto-theme-session.json")
}

pub fn consume_login_open(
    login_session_id: u64,
    main_app_running: bool,
    open_at_login: bool,
) -> Result<bool, String> {
    consume_login_open_from(
        &login_session_path(),
        login_session_id,
        main_app_running,
        open_at_login,
    )
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoginSessionMarker {
    schema_version: u32,
    #[serde(alias = "audit_session_id")]
    login_session_id: u64,
}

fn consume_login_open_from(
    path: &Path,
    login_session_id: u64,
    main_app_running: bool,
    open_at_login: bool,
) -> Result<bool, String> {
    if login_session_id == 0 {
        return Err("无法识别当前登录会话".into());
    }
    match fs::read(path) {
        Ok(bytes) => {
            let marker: LoginSessionMarker = serde_json::from_slice(&bytes)
                .map_err(|error| format!("自动主题会话标记已损坏：{error}"))?;
            if marker.schema_version != SCHEMA_VERSION {
                return Err(format!(
                    "不支持的自动主题会话版本：{}",
                    marker.schema_version
                ));
            }
            if marker.login_session_id == login_session_id {
                return Ok(false);
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(format!("无法读取自动主题会话标记：{error}")),
    }
    let marker = LoginSessionMarker {
        schema_version: SCHEMA_VERSION,
        login_session_id,
    };
    let bytes = serde_json::to_vec_pretty(&marker)
        .map_err(|error| format!("无法编码自动主题会话标记：{error}"))?;
    write_atomic(path, &bytes)?;
    Ok(open_at_login && !main_app_running)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisorAction {
    Exit,
    StopWatcher,
    StartWatcher,
    Wait,
}

#[derive(Debug, Clone, Copy)]
pub struct SupervisorSnapshot {
    pub keep_requested: bool,
    pub main_app_running: bool,
    pub target_running: bool,
    pub watcher_finished: bool,
}

impl SupervisorSnapshot {
    #[cfg(test)]
    fn running(main_app_running: bool) -> Self {
        Self {
            keep_requested: true,
            main_app_running,
            target_running: true,
            watcher_finished: false,
        }
    }

    #[cfg(test)]
    fn stopped() -> Self {
        Self {
            keep_requested: true,
            main_app_running: false,
            target_running: false,
            watcher_finished: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct SupervisorState {
    login_start_pending: bool,
    watcher_active: bool,
    yielded_to_main: bool,
    waiting_for_new_launch: bool,
    observed_stopped: bool,
}

impl SupervisorState {
    pub fn new(login_start_pending: bool) -> Self {
        Self {
            login_start_pending,
            ..Self::default()
        }
    }

    pub fn next(&mut self, snapshot: SupervisorSnapshot) -> SupervisorAction {
        if !snapshot.keep_requested {
            return SupervisorAction::Exit;
        }
        if snapshot.main_app_running {
            self.yielded_to_main = true;
            if self.watcher_active {
                self.watcher_active = false;
                return SupervisorAction::StopWatcher;
            }
            return SupervisorAction::Wait;
        }
        if self.watcher_active {
            if snapshot.watcher_finished {
                self.watcher_active = false;
                self.waiting_for_new_launch = true;
                self.observed_stopped = !snapshot.target_running;
            }
            return SupervisorAction::Wait;
        }
        if self.yielded_to_main {
            self.yielded_to_main = false;
            if snapshot.target_running {
                self.watcher_active = true;
                return SupervisorAction::StartWatcher;
            }
            self.waiting_for_new_launch = true;
            self.observed_stopped = true;
            return SupervisorAction::Wait;
        }
        if self.login_start_pending {
            self.login_start_pending = false;
            self.watcher_active = true;
            return SupervisorAction::StartWatcher;
        }
        if self.waiting_for_new_launch {
            if !snapshot.target_running {
                self.observed_stopped = true;
                return SupervisorAction::Wait;
            }
            if self.observed_stopped {
                self.waiting_for_new_launch = false;
                self.observed_stopped = false;
                self.watcher_active = true;
                return SupervisorAction::StartWatcher;
            }
            return SupervisorAction::Wait;
        }
        if snapshot.target_running {
            self.watcher_active = true;
            SupervisorAction::StartWatcher
        } else {
            self.waiting_for_new_launch = true;
            self.observed_stopped = true;
            SupervisorAction::Wait
        }
    }
}

fn load_from(path: &Path) -> Result<AutoThemeSettings, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(AutoThemeSettings::default())
        }
        Err(error) => return Err(format!("无法读取自动主题设置：{error}")),
    };
    let persisted: PersistedSettings =
        serde_json::from_slice(&bytes).map_err(|error| format!("自动主题设置已损坏：{error}"))?;
    persisted.try_into()
}

fn save_to(path: &Path, settings: &AutoThemeSettings) -> Result<(), String> {
    settings.validate()?;
    let parent = path
        .parent()
        .ok_or_else(|| "无法确定自动主题设置目录".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建自动主题设置目录：{error}"))?;
    let persisted = PersistedSettings::from(settings);
    let bytes = serde_json::to_vec_pretty(&persisted)
        .map_err(|error| format!("无法编码自动主题设置：{error}"))?;
    write_atomic(path, &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "无法确定自动主题设置目录".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建自动主题设置目录：{error}"))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("auto-theme.json");
    let temporary = parent.join(format!(
        ".{file_name}.tmp-{}-{}",
        std::process::id(),
        NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| format!("无法创建自动主题临时设置：{error}"))?;
        file.write_all(bytes)
            .and_then(|_| file.write_all(b"\n"))
            .and_then(|_| file.sync_all())
            .map_err(|error| format!("无法保存自动主题设置：{error}"))?;
        replace_file(&temporary, path)?;
        sync_directory(parent);
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(not(target_os = "windows"))]
fn replace_file(temporary: &Path, path: &Path) -> Result<(), String> {
    fs::rename(temporary, path).map_err(|error| format!("无法提交自动主题设置：{error}"))
}

#[cfg(target_os = "windows")]
fn replace_file(temporary: &Path, path: &Path) -> Result<(), String> {
    if !path.exists() {
        return fs::rename(temporary, path)
            .map_err(|error| format!("无法提交自动主题设置：{error}"));
    }
    let backup = path.with_extension(format!("backup-{}", std::process::id()));
    fs::rename(path, &backup).map_err(|error| format!("无法备份自动主题设置：{error}"))?;
    if let Err(error) = fs::rename(temporary, path) {
        let _ = fs::rename(&backup, path);
        return Err(format!("无法提交自动主题设置：{error}"));
    }
    let _ = fs::remove_file(backup);
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) {
    if let Ok(directory) = fs::File::open(path) {
        let _ = directory.sync_all();
    }
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) {}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::live::TargetApp;

    use super::{
        consume_login_open_from, load_from, save_to, AutoThemeSettings, LastApplied,
        SupervisorAction, SupervisorSnapshot, SupervisorState,
    };

    static NEXT_DIR: AtomicU64 = AtomicU64::new(1);

    fn test_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "doubao-skin-auto-theme-{name}-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn missing_settings_default_to_disabled() {
        let path = test_dir("missing").join("auto-theme.json");
        assert_eq!(load_from(&path).unwrap(), AutoThemeSettings::default());
    }

    #[test]
    fn settings_round_trip_the_last_successful_theme() {
        let path = test_dir("round-trip").join("auto-theme.json");
        let mut settings = AutoThemeSettings::default();
        settings.set_last_applied(
            LastApplied::new(TargetApp::DoubaoWork, "gallery-whale-maid", Some(0.62)).unwrap(),
        );
        settings.set_keep_requested(true);
        settings.set_open_at_login(true);

        save_to(&path, &settings).unwrap();
        assert_eq!(load_from(&path).unwrap(), settings);
    }

    #[test]
    fn disabling_keep_also_disables_login_launch() {
        let mut settings = AutoThemeSettings::default();
        settings.set_keep_requested(true);
        settings.set_open_at_login(true);
        settings.set_keep_requested(false);
        assert!(!settings.keep_requested());
        assert!(!settings.open_at_login());
    }

    #[test]
    fn invalid_last_applied_values_are_rejected() {
        assert!(LastApplied::new(TargetApp::Doubao, "", None).is_err());
        assert!(LastApplied::new(TargetApp::Doubao, "theme", Some(f32::NAN)).is_err());
        assert!(LastApplied::new(TargetApp::Doubao, "theme", Some(1.1)).is_err());
    }

    #[test]
    fn corrupt_and_unknown_settings_fail_without_deleting_the_file() {
        let directory = test_dir("corrupt");
        let path = directory.join("auto-theme.json");
        fs::write(&path, b"{not-json").unwrap();
        assert!(load_from(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{not-json");

        fs::write(
            &path,
            br#"{"schema_version":99,"last_applied":null,"keep_requested":false,"open_at_login":false}"#,
        )
        .unwrap();
        assert!(load_from(&path).is_err());
        assert!(path.exists());
    }

    #[test]
    fn invalid_new_state_does_not_replace_valid_settings() {
        let path = test_dir("preserve").join("auto-theme.json");
        let mut valid = AutoThemeSettings::default();
        valid
            .set_last_applied(LastApplied::new(TargetApp::Doubao, "pure-dark", Some(0.8)).unwrap());
        save_to(&path, &valid).unwrap();
        let before = fs::read(&path).unwrap();

        let invalid = AutoThemeSettings::test_only_invalid_login_dependency();
        assert!(save_to(&path, &invalid).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    #[test]
    fn login_open_is_consumed_once_per_login_session() {
        let path = test_dir("login-session").join("auto-theme-session.json");
        let first = u64::from(u32::MAX) + 41;
        assert!(consume_login_open_from(&path, first, false, true).unwrap());
        assert!(!consume_login_open_from(&path, first, false, true).unwrap());
        assert!(!consume_login_open_from(&path, first + 1, true, true).unwrap());
        assert!(!consume_login_open_from(&path, first + 1, false, true).unwrap());
        assert!(consume_login_open_from(&path, first + 2, false, true).unwrap());
        assert!(!consume_login_open_from(&path, first + 3, false, false).unwrap());
    }

    #[test]
    fn legacy_audit_session_marker_is_read_and_rewritten_portably() {
        let path = test_dir("legacy-session").join("auto-theme-session.json");
        fs::write(&path, br#"{"schema_version":1,"audit_session_id":41}"#).unwrap();

        assert!(!consume_login_open_from(&path, 41, false, true).unwrap());
        assert!(consume_login_open_from(&path, u64::from(u32::MAX) + 42, false, true).unwrap());
        let rewritten = fs::read_to_string(&path).unwrap();
        assert!(rewritten.contains("\"login_session_id\""));
        assert!(!rewritten.contains("\"audit_session_id\""));
    }

    #[test]
    fn zero_login_session_is_rejected_without_writing_a_marker() {
        let path = test_dir("zero-session").join("auto-theme-session.json");
        assert!(consume_login_open_from(&path, 0, false, true).is_err());
        assert!(!path.exists());
    }

    #[test]
    fn supervisor_never_starts_when_keep_is_disabled() {
        let mut state = SupervisorState::new(false);
        assert_eq!(
            state.next(SupervisorSnapshot {
                keep_requested: false,
                main_app_running: false,
                target_running: true,
                watcher_finished: false,
            }),
            SupervisorAction::Exit
        );
    }

    #[test]
    fn supervisor_yields_to_the_main_app_then_takes_over() {
        let mut state = SupervisorState::new(false);
        assert_eq!(
            state.next(SupervisorSnapshot::running(false)),
            SupervisorAction::StartWatcher
        );
        assert_eq!(
            state.next(SupervisorSnapshot::running(true)),
            SupervisorAction::StopWatcher
        );
        assert_eq!(
            state.next(SupervisorSnapshot::running(true)),
            SupervisorAction::Wait
        );
        assert_eq!(
            state.next(SupervisorSnapshot::running(false)),
            SupervisorAction::StartWatcher
        );
    }

    #[test]
    fn supervisor_waits_for_a_complete_restart_after_target_exit() {
        let mut state = SupervisorState::new(false);
        assert_eq!(
            state.next(SupervisorSnapshot::running(false)),
            SupervisorAction::StartWatcher
        );
        assert_eq!(
            state.next(SupervisorSnapshot {
                watcher_finished: true,
                ..SupervisorSnapshot::running(false)
            }),
            SupervisorAction::Wait
        );
        assert_eq!(
            state.next(SupervisorSnapshot::running(false)),
            SupervisorAction::Wait
        );
        assert_eq!(
            state.next(SupervisorSnapshot::stopped()),
            SupervisorAction::Wait
        );
        assert_eq!(
            state.next(SupervisorSnapshot::running(false)),
            SupervisorAction::StartWatcher
        );
    }

    #[test]
    fn login_start_can_launch_once_without_a_running_target() {
        let mut state = SupervisorState::new(true);
        assert_eq!(
            state.next(SupervisorSnapshot::stopped()),
            SupervisorAction::StartWatcher
        );
        assert_eq!(
            state.next(SupervisorSnapshot {
                watcher_finished: true,
                ..SupervisorSnapshot::stopped()
            }),
            SupervisorAction::Wait
        );
        assert_eq!(
            state.next(SupervisorSnapshot::stopped()),
            SupervisorAction::Wait
        );
    }
}

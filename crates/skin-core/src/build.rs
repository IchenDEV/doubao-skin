//! Offline application cloning.
//!
//! The current clone format is a macOS application bundle. Other platforms
//! fail at this module boundary without compiling or invoking macOS tooling.

#[cfg(not(target_os = "macos"))]
use std::path::PathBuf;

#[cfg(not(target_os = "macos"))]
use crate::theme::Theme;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{apply, remove, skin_app};

#[cfg(any(test, not(target_os = "macos")))]
fn unsupported_offline_build(target_os: &str) -> String {
    match target_os {
        "windows" => "离线克隆仅支持 macOS；Windows 请使用实时应用主题".into(),
        _ => format!("离线克隆仅支持 macOS；当前平台 {target_os} 只支持主题创作与打包命令"),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn skin_app() -> Result<PathBuf, String> {
    Err(unsupported_offline_build(std::env::consts::OS))
}

#[cfg(not(target_os = "macos"))]
pub fn apply<F: FnMut(String)>(_theme: &Theme, _log: F) -> Result<PathBuf, String> {
    Err(unsupported_offline_build(std::env::consts::OS))
}

#[cfg(not(target_os = "macos"))]
pub fn remove<F: FnMut(String)>(_log: F) -> Result<(), String> {
    Err(unsupported_offline_build(std::env::consts::OS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_clone_reports_the_platform_capability_boundary() {
        assert_eq!(
            unsupported_offline_build("windows"),
            "离线克隆仅支持 macOS；Windows 请使用实时应用主题"
        );
        assert_eq!(
            unsupported_offline_build("linux"),
            "离线克隆仅支持 macOS；当前平台 linux 只支持主题创作与打包命令"
        );
    }
}

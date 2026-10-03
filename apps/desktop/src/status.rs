//! Pure rules for what the user is told about a theme operation and how a
//! store theme relates to the local copy. No GPUI types live here so the rules
//! can be tested without a window.

use skin_core::live::TargetApp;

use crate::i18n::t;

/// Turn a backend error into something a user can act on. Errors from
/// `skin_core::live` are already Chinese sentences and pass through; low-level
/// English errors (sockets, CDP) are summarised, the original stays in logs.
pub fn describe_failure(reason: &str, target: TargetApp) -> String {
    let reason = reason.trim();
    let readable = reason
        .chars()
        .any(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch));
    if readable {
        reason.to_string()
    } else {
        t().format_cannot_connect(target.display_name())
    }
}

/// How a store theme relates to the copy installed locally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreInstall {
    NotInstalled,
    Installed,
    UpdateAvailable,
}

/// `installed_version` is `None` when the theme is not installed at all.
/// Unparseable versions never trigger an update prompt.
pub fn store_install_state(installed_version: Option<&str>, store_version: &str) -> StoreInstall {
    match installed_version {
        None => StoreInstall::NotInstalled,
        Some(installed) if skin_core::theme::is_newer_version(store_version, installed) => {
            StoreInstall::UpdateAvailable
        }
        Some(_) => StoreInstall::Installed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_install_state_offers_updates_only_for_strictly_newer_versions() {
        assert_eq!(
            store_install_state(None, "1.0.0"),
            StoreInstall::NotInstalled
        );
        assert_eq!(
            store_install_state(Some("1.0.0"), "1.0.0"),
            StoreInstall::Installed
        );
        assert_eq!(
            store_install_state(Some("1.0.0"), "1.1.0"),
            StoreInstall::UpdateAvailable
        );
        assert_eq!(
            store_install_state(Some("2.0.0"), "1.9.0"),
            StoreInstall::Installed
        );
        assert_eq!(
            store_install_state(Some("1.0.0"), ""),
            StoreInstall::Installed
        );
        assert_eq!(
            store_install_state(Some("dev"), "1.0.0"),
            StoreInstall::Installed
        );
    }

    #[test]
    fn non_chinese_errors_are_summarised_for_the_user() {
        let text = describe_failure("Connection refused (os error 61)", TargetApp::DoubaoWork);
        assert!(text.contains("豆包工作"), "{text}");
        assert!(!text.contains("os error"), "{text}");
        assert_eq!(
            describe_failure("豆包的调试端口未正常启动", TargetApp::Doubao),
            "豆包的调试端口未正常启动"
        );
    }
}

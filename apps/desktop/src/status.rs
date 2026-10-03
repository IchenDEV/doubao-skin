//! Pure UI state for applying and restoring themes: what the user is told,
//! and which phase the live injection is in. No GPUI types live here so the
//! rules can be tested without a window.

use skin_core::live::TargetApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Info,
    Success,
    Error,
}

/// Where a notice applies. A notice about one theme on one target app must
/// not leak onto other themes or the other app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoticeScope {
    Global,
    Theme { theme_id: String, target: TargetApp },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub scope: NoticeScope,
    pub tone: Tone,
    pub text: String,
}

impl Notice {
    pub fn new(scope: NoticeScope, tone: Tone, text: impl Into<String>) -> Self {
        Self {
            scope,
            tone,
            text: text.into(),
        }
    }

    pub fn global(tone: Tone, text: impl Into<String>) -> Self {
        Self::new(NoticeScope::Global, tone, text)
    }

    pub fn for_theme(
        theme_id: &str,
        target: TargetApp,
        tone: Tone,
        text: impl Into<String>,
    ) -> Self {
        Self::new(
            NoticeScope::Theme {
                theme_id: theme_id.to_string(),
                target,
            },
            tone,
            text,
        )
    }

    pub fn visible_for(&self, theme_id: Option<&str>, target: TargetApp) -> bool {
        match &self.scope {
            NoticeScope::Global => true,
            NoticeScope::Theme {
                theme_id: scoped_theme,
                target: scoped_target,
            } => theme_id == Some(scoped_theme.as_str()) && *scoped_target == target,
        }
    }
}

/// Lifecycle of the live injection for the selected target app.
#[derive(Clone, Debug, PartialEq)]
pub enum ApplyPhase {
    Idle,
    Applying {
        target: TargetApp,
        theme_id: String,
        opacity: Option<f32>,
    },
    Active {
        target: TargetApp,
        theme_id: String,
        opacity: Option<f32>,
    },
    Restoring {
        target: TargetApp,
    },
}

/// How a background apply/restore thread ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finished<'a> {
    Restored,
    RestoreFailed(&'a str),
    ApplyFailed(&'a str),
    /// The watcher stopped on its own (for example the app quit).
    WatchEnded,
}

impl ApplyPhase {
    /// An apply or restore is in flight; new apply requests are ignored.
    pub fn is_busy(&self) -> bool {
        matches!(self, Self::Applying { .. } | Self::Restoring { .. })
    }

    pub fn is_restoring(&self) -> bool {
        matches!(self, Self::Restoring { .. })
    }

    pub fn is_applying(&self, target: TargetApp, theme_id: &str) -> bool {
        matches!(self, Self::Applying { target: t, theme_id: id, .. } if *t == target && id == theme_id)
    }

    pub fn is_active(&self, target: TargetApp, theme_id: &str) -> bool {
        matches!(self, Self::Active { target: t, theme_id: id, .. } if *t == target && id == theme_id)
    }

    pub fn active_theme_id(&self) -> Option<&str> {
        match self {
            Self::Active { theme_id, .. } => Some(theme_id.as_str()),
            _ => None,
        }
    }

    pub fn active_opacity(&self) -> Option<f32> {
        match self {
            Self::Active { opacity, .. } => *opacity,
            _ => None,
        }
    }

    /// Target app and theme id the current phase belongs to (empty theme id
    /// while restoring).
    pub fn target_and_theme(&self) -> Option<(TargetApp, &str)> {
        match self {
            Self::Applying {
                target, theme_id, ..
            }
            | Self::Active {
                target, theme_id, ..
            } => Some((*target, theme_id.as_str())),
            Self::Restoring { target } => Some((*target, "")),
            Self::Idle => None,
        }
    }

    pub fn begin_apply(&mut self, target: TargetApp, theme_id: &str, opacity: Option<f32>) {
        *self = Self::Applying {
            target,
            theme_id: theme_id.to_string(),
            opacity,
        };
    }

    pub fn begin_restore(&mut self, target: TargetApp) {
        *self = Self::Restoring { target };
    }

    /// The injection reported success. Only a pending apply can become active.
    pub fn confirm_applied(&mut self) {
        if let Self::Applying {
            target,
            theme_id,
            opacity,
        } = self
        {
            *self = Self::Active {
                target: *target,
                theme_id: std::mem::take(theme_id),
                opacity: *opacity,
            };
        }
    }

    /// Forget any applied/pending state (the watcher was stopped on purpose)
    /// and report which target app still carries an injection to clean up.
    pub fn stop(&mut self) -> Option<TargetApp> {
        let previous = match self {
            Self::Applying { target, .. } | Self::Active { target, .. } => Some(*target),
            Self::Idle | Self::Restoring { .. } => None,
        };
        *self = Self::Idle;
        previous
    }

    /// Settle the phase after a background thread ended and return what the
    /// user should be told.
    pub fn finish(
        &mut self,
        outcome: Finished<'_>,
        target: TargetApp,
        theme_id: &str,
    ) -> Option<Notice> {
        *self = Self::Idle;
        Some(match outcome {
            Finished::Restored => Notice::global(Tone::Success, "已恢复默认"),
            Finished::RestoreFailed(reason) => Notice::global(
                Tone::Error,
                format!("恢复失败：{}", describe_failure(reason, target)),
            ),
            Finished::ApplyFailed(reason) => Notice::for_theme(
                theme_id,
                target,
                Tone::Error,
                format!("应用失败：{}", describe_failure(reason, target)),
            ),
            Finished::WatchEnded => Notice::for_theme(
                theme_id,
                target,
                Tone::Info,
                format!("{}已关闭，主题不再生效", target.display_name()),
            ),
        })
    }
}

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
        format!(
            "无法连接到{}，请确认应用已打开后重试",
            target.display_name()
        )
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

    fn theme_scope(id: &str, target: TargetApp) -> NoticeScope {
        NoticeScope::Theme {
            theme_id: id.to_string(),
            target,
        }
    }

    #[test]
    fn theme_notice_is_hidden_for_other_themes_and_targets() {
        let notice = Notice::new(
            theme_scope("violet-night", TargetApp::Doubao),
            Tone::Error,
            "应用失败：未找到豆包",
        );
        assert!(notice.visible_for(Some("violet-night"), TargetApp::Doubao));
        assert!(!notice.visible_for(Some("other"), TargetApp::Doubao));
        assert!(!notice.visible_for(Some("violet-night"), TargetApp::DoubaoWork));
        assert!(!notice.visible_for(None, TargetApp::Doubao));
    }

    #[test]
    fn global_notice_is_visible_everywhere() {
        let notice = Notice::new(NoticeScope::Global, Tone::Info, "正在安装主题…");
        assert!(notice.visible_for(Some("a"), TargetApp::Doubao));
        assert!(notice.visible_for(None, TargetApp::DoubaoWork));
    }

    #[test]
    fn applying_is_not_active_until_the_injection_is_confirmed() {
        let mut phase = ApplyPhase::Idle;
        phase.begin_apply(TargetApp::Doubao, "violet-night", Some(0.8));
        assert!(phase.is_busy());
        assert!(phase.is_applying(TargetApp::Doubao, "violet-night"));
        assert_eq!(phase.active_theme_id(), None);
        assert!(!phase.is_active(TargetApp::Doubao, "violet-night"));

        phase.confirm_applied();
        assert!(!phase.is_busy());
        assert_eq!(phase.active_theme_id(), Some("violet-night"));
        assert!(phase.is_active(TargetApp::Doubao, "violet-night"));
        assert!(!phase.is_active(TargetApp::DoubaoWork, "violet-night"));
        assert_eq!(phase.active_opacity(), Some(0.8));
    }

    #[test]
    fn confirming_without_a_pending_apply_changes_nothing() {
        let mut phase = ApplyPhase::Idle;
        phase.confirm_applied();
        assert_eq!(phase, ApplyPhase::Idle);
        let mut phase = ApplyPhase::Restoring {
            target: TargetApp::Doubao,
        };
        phase.confirm_applied();
        assert!(phase.is_busy());
    }

    #[test]
    fn failed_apply_returns_to_idle_without_ever_being_active() {
        let mut phase = ApplyPhase::Idle;
        phase.begin_apply(TargetApp::Doubao, "violet-night", None);
        let notice = phase.finish(
            Finished::ApplyFailed("未找到豆包：/Applications/Doubao.app"),
            TargetApp::Doubao,
            "violet-night",
        );
        assert_eq!(phase, ApplyPhase::Idle);
        let notice = notice.expect("a failure must be explained");
        assert_eq!(notice.tone, Tone::Error);
        assert!(notice.text.contains("未找到豆包"), "{}", notice.text);
        assert_eq!(notice.scope, theme_scope("violet-night", TargetApp::Doubao));
    }

    #[test]
    fn restore_failure_never_reports_an_apply_failure() {
        let mut phase = ApplyPhase::Idle;
        phase.begin_restore(TargetApp::DoubaoWork);
        assert!(phase.is_busy());
        let notice = phase
            .finish(
                Finished::RestoreFailed("豆包工作未开放本地调试端口，请打开应用后再试"),
                TargetApp::DoubaoWork,
                "violet-night",
            )
            .unwrap();
        assert_eq!(phase, ApplyPhase::Idle);
        assert_eq!(notice.tone, Tone::Error);
        assert!(!notice.text.contains("应用失败"), "{}", notice.text);
        assert!(notice.text.contains("请打开应用后再试"), "{}", notice.text);
    }

    #[test]
    fn restore_success_is_reported_and_clears_the_phase() {
        let mut phase = ApplyPhase::Idle;
        phase.begin_restore(TargetApp::Doubao);
        let notice = phase
            .finish(Finished::Restored, TargetApp::Doubao, "x")
            .unwrap();
        assert_eq!(phase, ApplyPhase::Idle);
        assert_eq!(notice.tone, Tone::Success);
        assert_eq!(notice.text, "已恢复默认");
    }

    #[test]
    fn watcher_ending_on_its_own_clears_the_active_state() {
        let mut phase = ApplyPhase::Idle;
        phase.begin_apply(TargetApp::Doubao, "violet-night", None);
        phase.confirm_applied();
        let notice = phase
            .finish(Finished::WatchEnded, TargetApp::Doubao, "violet-night")
            .unwrap();
        assert_eq!(phase, ApplyPhase::Idle);
        assert_eq!(phase.active_theme_id(), None);
        assert_eq!(notice.tone, Tone::Info);
        assert!(notice.text.contains("豆包已关闭"), "{}", notice.text);
    }

    #[test]
    fn stop_forgets_the_phase_and_reports_the_previous_target() {
        let mut phase = ApplyPhase::Idle;
        phase.begin_apply(TargetApp::Doubao, "violet-night", None);
        phase.confirm_applied();
        assert_eq!(phase.stop(), Some(TargetApp::Doubao));
        assert_eq!(phase, ApplyPhase::Idle);
        assert_eq!(phase.stop(), None);
    }

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

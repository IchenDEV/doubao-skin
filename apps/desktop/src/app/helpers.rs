//! Small state and target-selection helpers.

use std::path::PathBuf;

use skin_core::theme_package::{SupportDeclaration, SupportLevel, TargetSupport};
use skin_core::{live, theme};

use crate::i18n::t;

pub fn target_preference_path() -> PathBuf {
    theme::app_data_dir().join("target-app")
}

pub fn read_target_preference() -> Option<String> {
    std::fs::read_to_string(target_preference_path())
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn save_target_preference(target: live::TargetApp) {
    let path = target_preference_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, target.id());
}

pub fn initial_target(
    saved: Option<&str>,
    doubao_installed: bool,
    work_installed: bool,
    workbuddy_installed: bool,
) -> live::TargetApp {
    if let Some(saved) = saved.and_then(live::TargetApp::from_id) {
        let installed = match saved {
            live::TargetApp::Doubao => doubao_installed,
            live::TargetApp::DoubaoWork => work_installed,
            live::TargetApp::WorkBuddy => workbuddy_installed,
        };
        if installed {
            return saved;
        }
    }
    if work_installed {
        live::TargetApp::DoubaoWork
    } else if doubao_installed {
        live::TargetApp::Doubao
    } else if workbuddy_installed {
        live::TargetApp::WorkBuddy
    } else {
        live::TargetApp::DoubaoWork
    }
}

pub fn uses_short_compact_layout(compact: bool, height: gpui::Pixels) -> bool {
    use gpui::px;
    compact && height <= px(600.)
}

pub fn preview_identity(target: live::TargetApp) -> (&'static str, &'static str) {
    let l = t();
    match target {
        live::TargetApp::Doubao => (l.target_doubao, l.target_doubao_greeting),
        live::TargetApp::DoubaoWork => (l.target_doubao_work, l.target_doubao_work_greeting),
        live::TargetApp::WorkBuddy => (l.target_workbuddy, l.target_workbuddy_greeting),
    }
}

pub fn target_shortcut(target: live::TargetApp) -> &'static str {
    match target {
        live::TargetApp::Doubao => "Command-1",
        live::TargetApp::DoubaoWork => "Command-2",
        live::TargetApp::WorkBuddy => "Command-3",
    }
}

pub fn support_label(support: TargetSupport) -> &'static str {
    if !support.is_supported() {
        return "不支持";
    }
    if support.declaration == SupportDeclaration::LegacyInferred {
        return "兼容模式";
    }
    match support.level {
        SupportLevel::Tailored => "专属适配",
        SupportLevel::Shared => "共享适配",
        SupportLevel::Unsupported => "不支持",
    }
}

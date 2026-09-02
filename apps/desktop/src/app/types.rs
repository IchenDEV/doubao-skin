//! Shared desktop state types.

use std::path::PathBuf;

use skin_core::auto_theme::AutoThemeSettings;
use skin_core::live::TargetApp;
use skin_core::theme;

use crate::app::platform::AutoThemeServiceStatus;

#[derive(Clone, Copy)]
pub(crate) struct TargetInstallations([bool; 3]);

impl TargetInstallations {
    pub(crate) fn detect() -> Self {
        Self::detect_with(TargetApp::is_installed)
    }

    pub(crate) fn detect_with(mut detect: impl FnMut(TargetApp) -> bool) -> Self {
        Self(TargetApp::ALL.map(&mut detect))
    }

    pub(crate) fn is_installed(self, target: TargetApp) -> bool {
        self.0[match target {
            TargetApp::Doubao => 0,
            TargetApp::DoubaoWork => 1,
            TargetApp::WorkBuddy => 2,
        }]
    }
}

pub enum Msg {
    Log(String),
    Applied {
        target: TargetApp,
        generation: u64,
    },
    Done {
        target: TargetApp,
        generation: u64,
        ok: bool,
        restoring: bool,
    },
    StoreLoaded(Result<Vec<StoreRow>, String>),
    InstallStarted,
    Installed {
        ids: Vec<String>,
        error: Option<String>,
        open_library: bool,
    },
    OpenUrl(String),
    AutoThemeServiceChanged {
        status: AutoThemeServiceStatus,
        error: Option<String>,
        rollback_settings: Option<AutoThemeSettings>,
    },
}

pub struct ThemeRow {
    pub theme: theme::Theme,
    pub preview: theme::PreviewStyle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SourceView {
    Library,
    Store,
}

pub struct StoreRow {
    pub theme: theme::StoreTheme,
    pub preview: Option<PathBuf>,
}

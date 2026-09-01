//! Shared desktop state types.

use std::path::PathBuf;

use skin_core::auto_theme::AutoThemeSettings;
use skin_core::theme;

use crate::app::platform::AutoThemeServiceStatus;

pub enum Msg {
    Log(String),
    Applied(u64),
    Done {
        generation: Option<u64>,
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

//! Shared desktop state types.

use std::path::PathBuf;

use skin_core::live::TargetApp;
use skin_core::theme;

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

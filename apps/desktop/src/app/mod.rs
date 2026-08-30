//! Application state and background message handling.

pub(crate) mod actions;
pub(crate) mod helpers;
mod input;
mod install;
pub(crate) mod platform;
pub(crate) mod theme_ops;
pub(crate) mod types;

use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use gpui::{Context, FocusHandle, Window};

use skin_core::{live, theme};

pub use self::helpers::{
    initial_target, preview_identity, read_target_preference, save_target_preference,
    theme_is_active, uses_short_compact_layout,
};
use crate::app::types::{Msg, SourceView, StoreRow, ThemeRow};
use crate::i18n::t;
use crate::ui::constants::MAX_INTERNAL_LOGS;
use crate::ui::palette::UiPalette;

pub struct SkinApp {
    pub(crate) colors: UiPalette,
    pub(crate) tx: mpsc::Sender<Msg>,
    pub(crate) themes: Vec<ThemeRow>,
    pub(crate) source_view: SourceView,
    pub(crate) store_rows: Vec<StoreRow>,
    pub(crate) store_loading: bool,
    pub(crate) store_error: Option<String>,
    pub(crate) installing_package: bool,
    pub(crate) installing_store_theme: Option<String>,
    pub(crate) selected: usize,
    pub(crate) store_selected: usize,
    pub(crate) query: String,
    pub(crate) search_active: bool,
    pub(crate) internal_logs: VecDeque<String>,
    pub(crate) message: String,
    pub(crate) applying: bool,
    pub(crate) selected_target: live::TargetApp,
    pub(crate) active_target: Option<live::TargetApp>,
    pub(crate) active_theme: Option<String>,
    pub(crate) active_surface_opacity: Option<f32>,
    pub(crate) surface_opacity: f32,
    pub(crate) opacity_drag_start: Option<(gpui::Pixels, f32)>,
    pub(crate) live_stop: Option<Arc<AtomicBool>>,
    pub(crate) live_thread: Option<std::thread::JoinHandle<()>>,
    pub(crate) generation: u64,
    pub(crate) focus_handle: FocusHandle,
    pub(crate) url_buffer: Arc<Mutex<Vec<String>>>,
}

impl SkinApp {
    pub fn new(
        tx: mpsc::Sender<Msg>,
        rx: mpsc::Receiver<Msg>,
        url_buffer: Arc<Mutex<Vec<String>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let colors = UiPalette::for_appearance(window.appearance());
        cx.observe_window_appearance(window, |this, window, cx| {
            let colors = UiPalette::for_appearance(window.appearance());
            if this.colors != colors {
                this.colors = colors;
                cx.notify();
            }
        })
        .detach();
        let themes: Vec<ThemeRow> = theme::list_installed()
            .into_iter()
            .map(|theme| ThemeRow {
                preview: theme.preview_style(),
                theme,
            })
            .collect();
        let surface_opacity = themes
            .first()
            .map(|row| row.preview.surface_opacity)
            .unwrap_or(1.0);
        let selected_target = initial_target(
            read_target_preference().as_deref(),
            live::TargetApp::Doubao.is_installed(),
            live::TargetApp::DoubaoWork.is_installed(),
        );
        let url_buf = url_buffer.clone();
        cx.spawn(async move |this, cx| loop {
            while let Ok(msg) = rx.try_recv() {
                if this
                    .update(cx, |this, cx| {
                        this.handle_msg(msg);
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
            }
            let urls: Vec<String> = url_buf
                .lock()
                .ok()
                .map(|mut buf| buf.drain(..).collect())
                .unwrap_or_default();
            for url in urls {
                if this
                    .update(cx, |this, cx| {
                        this.handle_open_url(&url, cx);
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
            }
            cx.background_executor()
                .timer(Duration::from_millis(120))
                .await;
        })
        .detach();
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        Self {
            colors,
            tx,
            themes,
            source_view: SourceView::Library,
            store_rows: Vec::new(),
            store_loading: false,
            store_error: None,
            installing_package: false,
            installing_store_theme: None,
            selected: 0,
            store_selected: 0,
            query: String::new(),
            search_active: false,
            internal_logs: VecDeque::new(),
            message: String::new(),
            applying: false,
            selected_target,
            active_target: None,
            active_theme: None,
            active_surface_opacity: None,
            surface_opacity,
            opacity_drag_start: None,
            live_stop: None,
            live_thread: None,
            generation: 0,
            focus_handle,
            url_buffer,
        }
    }
    pub(crate) fn handle_msg(&mut self, msg: Msg) {
        let l = t();
        match msg {
            Msg::Log(line) => {
                self.internal_logs.push_front(line);
                self.internal_logs.truncate(MAX_INTERNAL_LOGS);
            }
            Msg::Applied(generation) if generation == self.generation => {
                self.applying = false;
                self.message = l.action_applied.into();
            }
            Msg::Applied(_) => {}
            Msg::Done {
                generation,
                ok,
                restoring,
            } => {
                if generation.is_none() || generation == Some(self.generation) {
                    self.applying = false;
                    if restoring && ok {
                        self.message = l.action_restored.into();
                    } else if !ok {
                        self.message = l.action_apply_failed.into();
                        self.active_target = None;
                        self.active_theme = None;
                        self.active_surface_opacity = None;
                        self.live_stop = None;
                    }
                }
            }
            Msg::StoreLoaded(result) => {
                self.store_loading = false;
                match result {
                    Ok(rows) => {
                        self.store_rows = rows;
                        self.store_error = None;
                        if self.store_selected >= self.store_rows.len() {
                            self.store_selected = 0;
                        }
                    }
                    Err(error) => self.store_error = Some(error),
                }
            }
            Msg::InstallStarted => {
                self.installing_package = true;
                self.message = l.install_installing.into();
            }
            Msg::Installed {
                ids,
                error,
                open_library,
            } => {
                self.installing_package = false;
                self.installing_store_theme = None;
                let selected = ids.last().cloned();
                self.reload_themes(selected.as_deref());
                if open_library && !ids.is_empty() {
                    self.source_view = SourceView::Library;
                    self.query.clear();
                }
                self.message = if let Some(error) = error {
                    if ids.is_empty() {
                        l.format_install_fail(&error)
                    } else {
                        l.format_install_partial(ids.len(), &error)
                    }
                } else if ids.len() == 1 {
                    l.install_one_done.into()
                } else {
                    l.format_install_count(ids.len())
                };
            }
            Msg::OpenUrl(url) => {
                if let Ok(mut buf) = self.url_buffer.lock() {
                    buf.push(url);
                }
            }
        }
    }
    pub(crate) fn reload_themes(&mut self, preferred_id: Option<&str>) {
        let selected_id = preferred_id.map(ToOwned::to_owned).or_else(|| {
            self.themes
                .get(self.selected)
                .map(|row| row.theme.id.clone())
        });
        self.themes = theme::list_installed()
            .into_iter()
            .map(|theme| ThemeRow {
                preview: theme.preview_style(),
                theme,
            })
            .collect();
        self.selected = selected_id
            .as_deref()
            .and_then(|id| self.themes.iter().position(|row| row.theme.id == id))
            .unwrap_or(0);
        self.surface_opacity = self
            .themes
            .get(self.selected)
            .map(|row| row.preview.surface_opacity)
            .unwrap_or(1.0);
    }
}

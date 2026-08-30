//! Local and store theme installation flows.

use std::path::PathBuf;
use std::sync::mpsc;

use gpui::{Context, PathPromptOptions, Window};

use skin_core::theme;

use crate::app::types::{Msg, SourceView, StoreRow};
use crate::app::SkinApp;
use crate::i18n::t;

impl SkinApp {
    pub(crate) fn handle_open_url(&mut self, url: &str, cx: &mut Context<Self>) {
        let theme_id = url
            .strip_prefix("doubao-skin://apply/")
            .or_else(|| url.strip_prefix("doubao-skin://theme/"))
            .map(|id| id.trim_end_matches('/'));
        let Some(theme_id) = theme_id else {
            return;
        };
        if let Some(index) = self.themes.iter().position(|row| row.theme.id == theme_id) {
            self.source_view = SourceView::Library;
            self.selected = index;
            self.query.clear();
            self.apply_selected(cx);
            return;
        }
        if let Some(index) = self
            .store_rows
            .iter()
            .position(|row| row.theme.id == theme_id)
        {
            self.source_view = SourceView::Store;
            self.store_selected = index;
            self.query.clear();
            self.install_store_theme(index, cx);
            return;
        }
        let id_owned = theme_id.to_string();
        let tx = self.tx.clone();
        self.message = t().format_searching_theme(&id_owned);
        std::thread::spawn(move || {
            let catalog_url = theme::theme_store_url();
            match theme::fetch_store_catalog(&catalog_url) {
                Ok(catalog) => {
                    let _ = tx.send(Msg::StoreLoaded(Ok(catalog
                        .themes
                        .into_iter()
                        .map(|store_theme| {
                            let cache_dir = theme::theme_store_cache_dir();
                            let preview =
                                theme::cache_store_preview(&catalog_url, &store_theme, &cache_dir)
                                    .ok()
                                    .flatten();
                            StoreRow {
                                theme: store_theme,
                                preview,
                            }
                        })
                        .collect())));
                    let _ = tx.send(Msg::OpenUrl(format!("doubao-skin://apply/{id_owned}")));
                }
                Err(error) => {
                    let _ = tx.send(Msg::StoreLoaded(Err(error)));
                }
            }
        });
        cx.notify();
    }

    pub(crate) fn switch_source(&mut self, source: SourceView, cx: &mut Context<Self>) {
        if self.source_view == source {
            return;
        }
        self.source_view = source;
        self.query.clear();
        self.search_active = false;
        if source == SourceView::Store && self.store_rows.is_empty() && !self.store_loading {
            self.load_store(cx);
        }
        cx.notify();
    }

    pub(crate) fn load_store(&mut self, cx: &mut Context<Self>) {
        if self.store_loading {
            return;
        }
        self.store_loading = true;
        self.store_error = None;
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let catalog_url = theme::theme_store_url();
            let result = theme::fetch_store_catalog(&catalog_url).map(|catalog| {
                let cache_dir = theme::theme_store_cache_dir();
                catalog
                    .themes
                    .into_iter()
                    .map(|store_theme| {
                        let preview =
                            theme::cache_store_preview(&catalog_url, &store_theme, &cache_dir)
                                .ok()
                                .flatten();
                        StoreRow {
                            theme: store_theme,
                            preview,
                        }
                    })
                    .collect()
            });
            let _ = tx.send(Msg::StoreLoaded(result));
        });
        cx.notify();
    }

    pub(crate) fn choose_package(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.installing_package {
            return;
        }
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some(t().install_prompt_title.into()),
        });
        let tx = self.tx.clone();
        window
            .spawn(cx, async move |_cx| {
                let paths = match receiver.await {
                    Ok(Ok(Some(paths))) => paths,
                    _ => return Ok::<(), anyhow::Error>(()),
                };
                let _ = tx.send(Msg::InstallStarted);
                std::thread::spawn(move || install_paths(paths, true, tx));
                Ok::<(), anyhow::Error>(())
            })
            .detach();
    }

    pub(crate) fn install_dropped_paths(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        if self.installing_package || paths.is_empty() {
            return;
        }
        self.installing_package = true;
        self.message = t().install_installing.into();
        let tx = self.tx.clone();
        let paths = paths.to_vec();
        std::thread::spawn(move || install_paths(paths, true, tx));
        cx.notify();
    }

    pub(crate) fn install_store_theme(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(row) = self.store_rows.get(index) else {
            return;
        };
        if self.installing_store_theme.is_some()
            || self
                .themes
                .iter()
                .any(|theme| theme.theme.id == row.theme.id)
        {
            return;
        }
        let item = row.theme.clone();
        self.installing_store_theme = Some(item.id.clone());
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let result = theme::download_and_install_store_theme(
                &theme::theme_store_url(),
                &item,
                &theme::user_themes_dir(),
            );
            match result {
                Ok(installed) => {
                    let _ = tx.send(Msg::Installed {
                        ids: vec![installed.id],
                        error: None,
                        open_library: false,
                    });
                }
                Err(error) => {
                    let _ = tx.send(Msg::Installed {
                        ids: Vec::new(),
                        error: Some(error),
                        open_library: false,
                    });
                }
            }
        });
        cx.notify();
    }
}

pub fn install_paths(paths: Vec<PathBuf>, open_library: bool, tx: mpsc::Sender<Msg>) {
    let l = t();
    let installed_dir = theme::user_themes_dir();
    let mut ids = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match theme::install_theme_package(&path, &installed_dir) {
            Ok(installed) => ids.push(installed.id),
            Err(error) => {
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or(l.install_package_fallback_name);
                errors.push(format!("{name}：{error}"));
            }
        }
    }
    let error = (!errors.is_empty()).then(|| errors.join("；"));
    let _ = tx.send(Msg::Installed {
        ids,
        error,
        open_library,
    });
}

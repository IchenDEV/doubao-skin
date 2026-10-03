//! Keyboard input handling.

use gpui::{Context, Focusable, KeyDownEvent, Window};

use skin_core::live;

use crate::app::types::SourceView;
use crate::app::SkinApp;
use crate::ui::{about_key_action, AboutKeyAction};

impl SkinApp {
    pub(crate) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        match about_key_action(self.about_open, key) {
            AboutKeyAction::Close => {
                self.close_about(window, cx);
                cx.stop_propagation();
                return;
            }
            AboutKeyAction::Consume => {
                cx.stop_propagation();
                return;
            }
            AboutKeyAction::Ignore => {}
        }

        if key == "escape" && self.confirm_delete.is_some() {
            self.confirm_delete = None;
            cx.notify();
            cx.stop_propagation();
            return;
        }

        // Text entry, cursor movement and clipboard shortcuts belong to the
        // search input (key bindings + input handler); only list navigation
        // and leaving the box are handled here.
        if self.search.focus_handle(cx).is_focused(window) {
            match key {
                "escape" | "tab" => {
                    self.focus_handle.focus(window, cx);
                    cx.notify();
                }
                "up" if self.source_view == SourceView::Library => self.select_filtered(-1, cx),
                "down" if self.source_view == SourceView::Library => self.select_filtered(1, cx),
                "enter" | "return" if self.source_view == SourceView::Library => {
                    self.apply_selected(cx)
                }
                _ => return,
            }
            cx.stop_propagation();
            return;
        }

        if self.source_view != SourceView::Library {
            return;
        }
        match key {
            "up" | "left" => self.select_filtered(-1, cx),
            "down" | "right" => self.select_filtered(1, cx),
            "enter" | "return" => self.apply_selected(cx),
            _ => return,
        }
        cx.stop_propagation();
    }

    /// Window-level shortcuts arrive as real actions so the menu bar can show
    /// them; the About modal swallows them like every other key.
    pub(crate) fn focus_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.about_open {
            return;
        }
        self.search.focus_handle(cx).focus(window, cx);
        cx.notify();
    }

    pub(crate) fn switch_target_by_shortcut(
        &mut self,
        target: live::TargetApp,
        cx: &mut Context<Self>,
    ) {
        if !self.about_open {
            self.switch_target(target, cx);
        }
    }

    pub(crate) fn import_package_by_shortcut(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.about_open {
            self.choose_package(window, cx);
        }
    }

    pub(crate) fn filtered_indices(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        self.themes
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                (row.theme.supports_target(self.selected_target)
                    && (query.is_empty()
                        || row.theme.name.to_lowercase().contains(&query)
                        || row.theme.id.to_lowercase().contains(&query)
                        || row.theme.description.to_lowercase().contains(&query)
                        || row.theme.author.to_lowercase().contains(&query)
                        || row
                            .theme
                            .store_category
                            .as_deref()
                            .is_some_and(|category| category.to_lowercase().contains(&query))
                        || row
                            .theme
                            .store_tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query))))
                .then_some(index)
            })
            .collect()
    }

    pub(crate) fn filtered_store_indices(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        self.store_rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                (row.theme.supports_target(self.selected_target)
                    && (query.is_empty()
                        || row.theme.name.to_lowercase().contains(&query)
                        || row.theme.id.to_lowercase().contains(&query)
                        || row.theme.description.to_lowercase().contains(&query)
                        || row.theme.author.to_lowercase().contains(&query)
                        || row.theme.category.to_lowercase().contains(&query)
                        || row
                            .theme
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query))))
                .then_some(index)
            })
            .collect()
    }

    pub(crate) fn ensure_selected_match(&mut self) {
        let indices = self.filtered_indices();
        if indices.contains(&self.selected) {
            return;
        }
        if let Some(index) = indices.first().copied() {
            self.selected = index;
            self.surface_opacity = self.themes[index].preview.surface_opacity;
            self.message.clear();
            self.confirm_delete = None;
        }
    }

    pub(crate) fn ensure_store_selected_match(&mut self) {
        let indices = self.filtered_store_indices();
        if !indices.contains(&self.store_selected) {
            self.store_selected = indices.first().copied().unwrap_or(0);
        }
    }

    pub(crate) fn select_filtered(&mut self, delta: isize, cx: &mut Context<Self>) {
        let indices = self.filtered_indices();
        if indices.is_empty() {
            return;
        }
        let position = indices
            .iter()
            .position(|index| *index == self.selected)
            .unwrap_or(0);
        let next = if delta < 0 {
            position.saturating_sub(1)
        } else {
            (position + 1).min(indices.len() - 1)
        };
        self.selected = indices[next];
        self.surface_opacity = self.themes[self.selected].preview.surface_opacity;
        self.message.clear();
        self.confirm_delete = None;
        cx.notify();
    }

    pub(crate) fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.themes.len() {
            self.selected = index;
            self.surface_opacity = self.themes[index].preview.surface_opacity;
            self.confirm_delete = None;
            self.message.clear();
            self.restart_confirmation_target = None;
            cx.notify();
        }
    }
}

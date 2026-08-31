//! Keyboard input handling.

use gpui::{Context, KeyDownEvent, Window};

use skin_core::live;

use crate::app::types::SourceView;
use crate::app::SkinApp;

impl SkinApp {
    pub(crate) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;

        if modifiers.platform && key.eq_ignore_ascii_case("f") {
            self.search_active = true;
            self.focus_handle.focus(window, cx);
            cx.notify();
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && key.eq_ignore_ascii_case("o") {
            self.choose_package(window, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && key == "1" {
            self.switch_target(live::TargetApp::Doubao, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && key == "2" {
            self.switch_target(live::TargetApp::DoubaoWork, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && key == "3" {
            self.switch_target(live::TargetApp::WorkBuddy, cx);
            cx.stop_propagation();
            return;
        }

        if self.search_active {
            match key {
                "escape" => {
                    self.search_active = false;
                    cx.notify();
                }
                "backspace" => {
                    self.query.pop();
                    if self.source_view == SourceView::Library {
                        self.ensure_selected_match();
                    }
                    cx.notify();
                }
                "up" if self.source_view == SourceView::Library => self.select_filtered(-1, cx),
                "down" if self.source_view == SourceView::Library => self.select_filtered(1, cx),
                "enter" | "return" if self.source_view == SourceView::Library => {
                    self.apply_selected(cx)
                }
                "tab" => {
                    self.search_active = false;
                    cx.notify();
                }
                _ if !modifiers.platform && !modifiers.control && !modifiers.function => {
                    if let Some(text) = event.keystroke.key_char.as_deref() {
                        if !text.chars().any(char::is_control) {
                            self.query.push_str(text);
                            if self.source_view == SourceView::Library {
                                self.ensure_selected_match();
                            }
                            cx.notify();
                        }
                    }
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
        cx.notify();
    }

    pub(crate) fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.themes.len() {
            self.selected = index;
            self.surface_opacity = self.themes[index].preview.surface_opacity;
            self.search_active = false;
            self.message.clear();
            self.restart_confirmation_target = None;
            cx.notify();
        }
    }
}

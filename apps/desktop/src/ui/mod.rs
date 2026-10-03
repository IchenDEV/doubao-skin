//! Top-level desktop layout and rendering.

mod about;
pub(crate) mod assets;
mod composer;
pub(crate) mod constants;
mod detail;
mod layout_body;
pub(crate) mod palette;
mod sidebar;
mod widgets;

use gpui::{
    div, prelude::*, px, rgb, svg, Context, ExternalPaths, Focusable, FontWeight, IntoElement,
    MouseButton, Render, Role, Window,
};

use skin_core::live;

use crate::app::actions::{
    FocusSearch, ImportPackage, SwitchToDoubao, SwitchToDoubaoWork, SwitchToWorkBuddy,
};
use crate::app::{uses_short_compact_layout, SkinApp};
use crate::i18n::t;
use crate::ui::constants::{HEADER_HEIGHT, WINDOW_TITLE_X};

#[cfg(test)]
pub(crate) use about::about_version;
pub(crate) use about::{about_key_action, shows_about_entry, AboutKeyAction};

pub(crate) fn header_brand_padding(target_os: &str, compact: bool) -> f32 {
    if target_os == "macos" {
        WINDOW_TITLE_X - if compact { 16.0 } else { 24.0 }
    } else {
        0.0
    }
}

pub(crate) fn shows_auto_theme_controls(target_os: &str) -> bool {
    matches!(target_os, "macos" | "windows")
}

impl Render for SkinApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let l = t();
        self.sync_search_input(window, cx);
        let compact = window.viewport_size().width < px(900.);
        let short = uses_short_compact_layout(compact, window.viewport_size().height);
        let header = self.render_header(compact, cx);
        let body = self.render_body(compact, short, cx);
        div()
            .id("theme-picker")
            .role(Role::Application)
            .aria_label(l.aria_app)
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::key_down))
            .on_action(
                cx.listener(|this, _: &FocusSearch, window, cx| this.focus_search(window, cx)),
            )
            .on_action(cx.listener(|this, _: &ImportPackage, window, cx| {
                this.import_package_by_shortcut(window, cx)
            }))
            .on_action(cx.listener(|this, _: &SwitchToDoubao, _window, cx| {
                this.switch_target_by_shortcut(live::TargetApp::Doubao, cx)
            }))
            .on_action(cx.listener(|this, _: &SwitchToDoubaoWork, _window, cx| {
                this.switch_target_by_shortcut(live::TargetApp::DoubaoWork, cx)
            }))
            .on_action(cx.listener(|this, _: &SwitchToWorkBuddy, _window, cx| {
                this.switch_target_by_shortcut(live::TargetApp::WorkBuddy, cx)
            }))
            .drag_over::<ExternalPaths>(move |style, _, _, _| style.bg(rgb(colors.drop_hover)))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.install_dropped_paths(paths.paths(), cx)
            }))
            .bg(rgb(colors.shell))
            .relative()
            .flex()
            .flex_col()
            .child(header)
            .child(body)
            .when(self.about_open, |root| {
                root.child(self.render_about_modal(cx))
            })
    }
}

impl SkinApp {
    fn render_header(&self, compact: bool, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let brand_padding = header_brand_padding(std::env::consts::OS, compact);
        let brand = div()
            .flex()
            .items_center()
            .text_size(px(17.))
            .font_weight(FontWeight::BOLD)
            .text_color(rgb(colors.text))
            .child(l.app_name);
        let target_switch = self.render_target_switch(cx);
        let about_entry = div()
            .flex_1()
            .min_w(px(0.))
            .flex()
            .justify_end()
            .when(shows_about_entry(std::env::consts::OS), |right| {
                right.child(self.render_about_entry(cx))
            });
        if compact {
            div()
                .h(px(HEADER_HEIGHT))
                .px_4()
                .border_b_1()
                .border_color(rgb(colors.border))
                .flex()
                .items_center()
                .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                    window.start_window_move()
                })
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .child(div().pl(px(brand_padding)).child(brand)),
                )
                .child(target_switch)
                .child(about_entry)
                .into_any_element()
        } else {
            div()
                .h(px(HEADER_HEIGHT))
                .px_6()
                .border_b_1()
                .border_color(rgb(colors.border))
                .flex()
                .items_center()
                .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                    window.start_window_move()
                })
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .child(div().pl(px(brand_padding)).child(brand)),
                )
                .child(target_switch)
                .child(about_entry)
                .into_any_element()
        }
    }

    /// Keep the input in step with app-driven query changes (clearing on view
    /// switches, installs, deep links) and with the palette, and remember
    /// whether it has focus for this frame's border.
    fn sync_search_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_colors = self.colors.input_colors();
        let (input_text, composing) = {
            let input = self.search.read(cx);
            (input.text().to_string(), input.is_composing())
        };
        if !composing && input_text != self.query {
            let query = self.query.clone();
            self.search
                .update(cx, |input, cx| input.sync_text(&query, cx));
        }
        self.search.update(cx, |input, _cx| {
            if input.colors != input_colors {
                input.colors = input_colors;
            }
        });
        self.search_focused = self.search.focus_handle(cx).is_focused(window);
    }

    pub(crate) fn render_search_bar(
        &self,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .id("search")
            .role(Role::Group)
            .aria_label(l.search_placeholder)
            .when(compact, |view| view.flex_1().min_w(px(0.)))
            .when(!compact, |view| view.w_full())
            .h(px(36.))
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(8.))
            .border_1()
            .border_color(rgb(if self.search_focused {
                colors.focus_border
            } else {
                colors.border
            }))
            .bg(rgb(colors.control).opacity(0.92))
            .cursor_pointer()
            .on_click(cx.listener(|this, _event, window, cx| {
                this.search.focus_handle(cx).focus(window, cx);
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/search.svg")
                    .size(px(15.))
                    .text_color(rgb(colors.muted)),
            )
            .child(self.search.clone())
            .when(!self.query.is_empty(), |view| {
                view.child(
                    div()
                        .id("clear-search")
                        .role(Role::Button)
                        .aria_label(l.search_clear_label)
                        .size(px(20.))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(14.))
                        .text_color(rgb(colors.muted))
                        .hover(|style| style.bg(rgb(colors.hover)))
                        .child("×")
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.search.update(cx, |input, cx| input.clear(cx));
                            cx.stop_propagation();
                            cx.notify();
                        })),
                )
            })
            .into_any_element()
    }
}

/// Placeholder for the search box, with the platform's find shortcut.
pub(crate) fn search_placeholder_with_shortcut() -> String {
    let shortcut = if cfg!(target_os = "macos") {
        "⌘F"
    } else {
        "Ctrl+F"
    };
    format!("{}  {shortcut}", t().search_placeholder)
}

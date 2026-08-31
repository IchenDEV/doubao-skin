//! Top-level desktop layout and rendering.

pub(crate) mod assets;
mod composer;
pub(crate) mod constants;
mod detail;
mod layout_body;
pub(crate) mod palette;
mod sidebar;
mod widgets;

use gpui::{
    div, prelude::*, px, rgb, svg, Context, ExternalPaths, FontWeight, IntoElement, Render, Role,
    Window,
};

use crate::app::types::SourceView;
use crate::app::{uses_short_compact_layout, SkinApp};
use crate::i18n::t;
use crate::ui::constants::{HEADER_HEIGHT, WINDOW_TITLE_X};

pub(crate) fn header_brand_padding(target_os: &str, compact: bool) -> f32 {
    if target_os == "macos" {
        WINDOW_TITLE_X - if compact { 16.0 } else { 24.0 }
    } else {
        0.0
    }
}

impl Render for SkinApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let l = t();
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
            .drag_over::<ExternalPaths>(move |style, _, _, _| style.bg(rgb(colors.drop_hover)))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.install_dropped_paths(paths.paths(), cx)
            }))
            .bg(rgb(colors.shell))
            .flex()
            .flex_col()
            .child(header)
            .child(body)
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
        if compact {
            div()
                .h(px(HEADER_HEIGHT))
                .px_4()
                .border_b_1()
                .border_color(rgb(colors.border))
                .flex()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .child(div().pl(px(brand_padding)).child(brand)),
                )
                .child(target_switch)
                .child(div().flex_1().min_w(px(0.)))
                .into_any_element()
        } else {
            div()
                .h(px(HEADER_HEIGHT))
                .px_6()
                .border_b_1()
                .border_color(rgb(colors.border))
                .flex()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .child(div().pl(px(brand_padding)).child(brand)),
                )
                .child(target_switch)
                .child(div().flex_1().min_w(px(0.)))
                .into_any_element()
        }
    }

    pub(crate) fn render_search_bar(
        &self,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let query = self.query.clone();
        let search_active = self.search_active;
        div()
            .id("search")
            .role(Role::Button)
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
            .border_color(rgb(if search_active {
                colors.focus_border
            } else {
                colors.border
            }))
            .bg(rgb(colors.control).opacity(0.92))
            .cursor_pointer()
            .on_click(cx.listener(|this, _event, window, cx| {
                this.search_active = true;
                this.focus_handle.focus(window, cx);
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/search.svg")
                    .size(px(15.))
                    .text_color(rgb(colors.muted)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .text_sm()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(rgb(if query.is_empty() {
                        colors.muted
                    } else {
                        colors.text
                    }))
                    .child(if query.is_empty() {
                        l.search_placeholder.to_string()
                    } else {
                        query
                    }),
            )
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
                            this.query.clear();
                            if this.source_view == SourceView::Library {
                                this.ensure_selected_match();
                            }
                            cx.stop_propagation();
                            cx.notify();
                        })),
                )
            })
            .into_any_element()
    }
}

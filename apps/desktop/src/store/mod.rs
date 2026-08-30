//! Theme-store page composition.

mod detail;
mod grid;
mod ui;

use gpui::{div, prelude::*, px, rgb, svg, Context, FontWeight, Role};

use crate::app::SkinApp;
use crate::i18n::t;

impl SkinApp {
    pub(crate) fn render_store(&self, compact: bool, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let mut body = div()
            .flex_1()
            .min_h(px(0.))
            .p(if compact { px(16.) } else { px(24.) })
            .flex()
            .flex_col()
            .gap_4()
            .child(self.render_store_header(cx));
        if self.store_loading {
            return body
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child(l.store_loading),
                )
                .into_any_element();
        }
        if let Some(error) = self.store_error.as_ref() {
            return body
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child(l.store_connect_full_failed)
                        .child(div().text_xs().child(error.clone())),
                )
                .into_any_element();
        }
        let indices = self.filtered_store_indices();
        if indices.is_empty() {
            return body
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child(if self.query.is_empty() {
                            l.store_no_themes
                        } else {
                            l.search_no_match
                        }),
                )
                .into_any_element();
        }
        let mut grid = div()
            .id("theme-store-grid")
            .role(Role::List)
            .aria_label(l.store_title)
            .flex_1()
            .min_h(px(0.))
            .overflow_scroll()
            .flex()
            .flex_wrap()
            .content_start()
            .gap_4()
            .pb_4();
        for index in indices {
            grid = grid.child(self.render_store_card(index, compact, cx));
        }
        body = body.child(grid);
        body.into_any_element()
    }

    fn render_store_header(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(colors.text))
                    .child(l.store_title),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .when(!self.message.is_empty(), |view| {
                        view.child(
                            div()
                                .text_xs()
                                .text_color(rgb(if self.message.contains(l.error_keyword) {
                                    colors.danger
                                } else {
                                    colors.muted
                                }))
                                .child(self.message.clone()),
                        )
                    })
                    .child(
                        div()
                            .id("refresh-store")
                            .role(Role::Button)
                            .aria_label(l.store_refresh_label)
                            .h(px(32.))
                            .px_3()
                            .rounded(px(7.))
                            .border_1()
                            .border_color(rgb(colors.border))
                            .bg(rgb(colors.control))
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_sm()
                            .text_color(rgb(colors.text))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(colors.hover)))
                            .child(
                                svg()
                                    .path("icons/refresh.svg")
                                    .size(px(15.))
                                    .text_color(rgb(colors.muted)),
                            )
                            .child(l.store_refresh)
                            .on_click(cx.listener(|this, _event, _window, cx| this.load_store(cx))),
                    ),
            )
            .into_any_element()
    }
}

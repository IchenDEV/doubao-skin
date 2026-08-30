//! Theme-store sidebar controls.

use gpui::{div, prelude::*, px, rgb, Context, FontWeight, Role};

use crate::app::theme_ops::parse_store_accent;
use crate::app::SkinApp;
use crate::i18n::t;

impl SkinApp {
    pub(crate) fn render_store_sidebar_item(
        &self,
        index: usize,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let row = &self.store_rows[index];
        let selected = index == self.store_selected;
        let installed = self
            .themes
            .iter()
            .any(|theme| theme.theme.id == row.theme.id);
        let accent = parse_store_accent(row.theme.accent.as_deref());
        div()
            .id(("store-sidebar", index))
            .role(Role::Button)
            .aria_label(row.theme.name.clone())
            .h(px(42.))
            .flex_shrink_0()
            .px_3()
            .flex()
            .items_center()
            .gap_3()
            .rounded(px(8.))
            .bg(if selected {
                rgb(accent).opacity(0.16)
            } else {
                rgb(colors.sidebar)
            })
            .cursor_pointer()
            .hover(|style| style.bg(rgb(colors.hover)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.store_selected = index;
                cx.notify();
            }))
            .child(
                div()
                    .size(px(28.))
                    .rounded(px(6.))
                    .bg(rgb(accent).opacity(0.24))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(div().size(px(12.)).rounded_full().bg(rgb(accent))),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_sm()
                    .font_weight(if selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(colors.text))
                    .child(row.theme.name.clone()),
            )
            .when(installed, |item| {
                item.child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors.muted))
                        .child(l.install_button_done),
                )
            })
            .into_any_element()
    }

    pub(crate) fn render_store_sidebar_list(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let mut list = div()
            .id("store-themes-sidebar")
            .role(Role::List)
            .aria_label(l.aria_store_themes)
            .flex_1()
            .min_h(px(0.))
            .overflow_scroll()
            .flex_col()
            .px_3()
            .pb_4();
        if self.store_loading {
            return list
                .child(
                    div()
                        .px_3()
                        .py_4()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child(l.store_connecting),
                )
                .into_any_element();
        }
        if let Some(error) = self.store_error.as_ref() {
            return list
                .child(
                    div()
                        .px_3()
                        .py_4()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child(l.store_connect_failed)
                        .child(div().text_xs().child(error.clone())),
                )
                .into_any_element();
        }
        let indices = self.filtered_store_indices();
        if indices.is_empty() {
            return list
                .child(
                    div()
                        .px_3()
                        .py_4()
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
        for index in indices {
            list = list.child(self.render_store_sidebar_item(index, cx));
        }
        list.into_any_element()
    }
}

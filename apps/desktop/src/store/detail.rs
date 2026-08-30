//! Theme-store detail panel.

use gpui::{div, prelude::*, px, rgb, Context, FontWeight, Role};

use crate::app::theme_ops::parse_store_accent;
use crate::app::SkinApp;
use crate::i18n::{self, t};

impl SkinApp {
    pub(crate) fn render_store_detail(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let Some(row) = self.store_rows.get(self.store_selected) else {
            return div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(colors.muted))
                .child(l.store_select_hint)
                .into_any_element();
        };
        let installed = self
            .themes
            .iter()
            .any(|theme| theme.theme.id == row.theme.id);
        let installing = self.installing_store_theme.as_deref() == Some(row.theme.id.as_str());
        let accent = parse_store_accent(row.theme.accent.as_deref());
        let store_selected = self.store_selected;
        div()
            .id("store-detail")
            .flex_1()
            .min_h(px(0.))
            .overflow_scroll()
            .p(px(32.))
            .flex()
            .flex_col()
            .items_center()
            .gap_6()
            .child(
                div()
                    .w(px(480.))
                    .max_w_full()
                    .overflow_hidden()
                    .rounded(px(12.))
                    .border_1()
                    .border_color(rgb(colors.border))
                    .bg(rgb(colors.control))
                    .child(self.render_store_preview(row, 280.)),
            )
            .child(
                div()
                    .w(px(480.))
                    .max_w_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(22.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(colors.text))
                            .child(row.theme.name.clone()),
                    )
                    .child(div().text_sm().text_color(rgb(colors.muted)).child(
                        if row.theme.description.is_empty() {
                            row.theme.category.clone()
                        } else {
                            row.theme.description.clone()
                        },
                    ))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .text_xs()
                            .text_color(rgb(colors.muted))
                            .when(!row.theme.category.is_empty(), |d| {
                                d.child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded(px(4.))
                                        .bg(rgb(colors.control))
                                        .child(
                                            i18n::store_category_label(&row.theme.category)
                                                .to_string(),
                                        ),
                                )
                            })
                            .when(!row.theme.version.is_empty(), |d| {
                                d.child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded(px(4.))
                                        .bg(rgb(colors.control))
                                        .child(format!("v{}", row.theme.version)),
                                )
                            })
                            .when(!row.theme.author.is_empty(), |d| {
                                d.child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded(px(4.))
                                        .bg(rgb(colors.control))
                                        .child(row.theme.author.clone()),
                                )
                            }),
                    )
                    .child(
                        div()
                            .id("install-store-detail")
                            .role(Role::Button)
                            .aria_label(if installed {
                                l.install_button_done
                            } else if installing {
                                l.install_button_busy
                            } else {
                                l.action_apply_theme
                            })
                            .mt_2()
                            .h(px(40.))
                            .px_6()
                            .rounded(px(8.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .cursor_pointer()
                            .when(installed, |btn| {
                                btn.bg(rgb(colors.installed_control))
                                    .text_color(rgb(colors.muted))
                                    .child(l.install_button_done)
                            })
                            .when(!installed && !installing, |btn| {
                                btn.bg(rgb(accent))
                                    .text_color(rgb(0xffffff))
                                    .hover(|style| style.opacity(0.88))
                                    .child(l.action_apply_theme)
                                    .on_click(cx.listener(move |this, _event, _window, cx| {
                                        this.install_store_theme(store_selected, cx)
                                    }))
                            })
                            .when(installing, |btn| {
                                btn.bg(rgb(colors.control))
                                    .text_color(rgb(colors.muted))
                                    .child(l.install_button_busy)
                            }),
                    ),
            )
            .into_any_element()
    }
}

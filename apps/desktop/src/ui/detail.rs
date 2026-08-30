//! Installed-theme detail panel.

use gpui::{div, prelude::*, px, rgb, Context, FontWeight, Role};

use crate::app::SkinApp;
use crate::i18n::t;
use crate::preview::preview_rgba;

impl SkinApp {
    pub(crate) fn render_theme_detail(
        &self,
        compact: bool,
        short: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let Some(row) = self.themes.get(self.selected) else {
            return div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(colors.muted))
                .child(l.empty_library)
                .into_any_element();
        };
        let active = self.selected_settings_are_active(row);
        let target_installed = self.selected_target.is_installed();
        let detail_message = if !target_installed {
            l.format_please_install(self.selected_target.display_name())
        } else if self.message == l.action_applied {
            String::new()
        } else {
            self.message.clone()
        };
        div()
            .flex_1()
            .min_w(px(0.))
            .p(if short {
                px(12.)
            } else if compact {
                px(16.)
            } else {
                px(24.)
            })
            .flex()
            .flex_col()
            .gap(if short {
                px(8.)
            } else if compact {
                px(12.)
            } else {
                px(20.)
            })
            .child(self.render_preview(row, compact, short))
            .child(self.render_detail_actions(
                row,
                active,
                target_installed,
                &detail_message,
                compact,
                short,
                cx,
            ))
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_detail_actions(
        &self,
        row: &crate::app::types::ThemeRow,
        active: bool,
        target_installed: bool,
        detail_message: &str,
        compact: bool,
        short: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .min_h(if short || !compact { px(72.) } else { px(80.) })
            .flex()
            .when(compact && !short, |view| view.flex_col().items_start())
            .when(!compact || short, |view| view.items_center())
            .justify_between()
            .gap(if compact { px(12.) } else { px(20.) })
            .child(
                div()
                    .min_w(px(0.))
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .h(px(24.))
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_size(px(20.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(colors.text))
                            .child(row.theme.name.clone())
                            .when(active, |title| {
                                title.child(
                                    div()
                                        .h(px(20.))
                                        .px_2()
                                        .rounded(px(6.))
                                        .border_1()
                                        .border_color(preview_rgba(row.preview.colors.accent, 0.4))
                                        .bg(preview_rgba(row.preview.colors.accent, 0.14))
                                        .flex()
                                        .items_center()
                                        .text_size(px(10.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(preview_rgba(row.preview.colors.accent, 1.0))
                                        .child(l.action_applied),
                                )
                            }),
                    )
                    .child(
                        div()
                            .h(px(20.))
                            .text_sm()
                            .text_color(rgb(colors.muted))
                            .child(row.theme.description.clone()),
                    )
                    .child(
                        div()
                            .h(px(16.))
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_xs()
                            .text_color(rgb(if detail_message.contains(l.error_keyword) {
                                colors.danger
                            } else {
                                colors.muted
                            }))
                            .child(detail_message.to_string()),
                    ),
            )
            .child(self.render_detail_buttons(row, active, target_installed, compact, short, cx))
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_detail_buttons(
        &self,
        row: &crate::app::types::ThemeRow,
        active: bool,
        target_installed: bool,
        compact: bool,
        short: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .flex()
            .items_center()
            .gap_3()
            .when(compact && !short, |view| view.w_full().justify_end())
            .when(row.preview.has_background, |view| {
                view.child(self.render_opacity_control(cx))
            })
            .child(
                div()
                    .id("restore")
                    .role(Role::Button)
                    .aria_label(l.action_restore_default)
                    .h(px(36.))
                    .px_4()
                    .flex()
                    .items_center()
                    .rounded(px(7.))
                    .border_1()
                    .border_color(rgb(colors.border))
                    .bg(rgb(colors.control))
                    .text_sm()
                    .text_color(rgb(colors.text))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(colors.hover)))
                    .child(l.action_restore_default)
                    .on_click(cx.listener(|this, _event, _window, cx| this.restore_default(cx))),
            )
            .child(
                div()
                    .id("apply")
                    .role(Role::Button)
                    .aria_label(if !target_installed {
                        l.not_installed_target
                    } else if active {
                        l.action_in_use
                    } else {
                        l.action_apply_theme
                    })
                    .h(px(36.))
                    .px_5()
                    .flex()
                    .items_center()
                    .rounded(px(7.))
                    .bg(preview_rgba(row.preview.colors.accent, 1.0))
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0xffffff))
                    .opacity(if self.applying || active || !target_installed {
                        0.72
                    } else {
                        1.0
                    })
                    .when(!self.applying && !active && target_installed, |button| {
                        button
                            .cursor_pointer()
                            .hover(|style| style.opacity(0.88))
                            .on_click(
                                cx.listener(|this, _event, _window, cx| this.apply_selected(cx)),
                            )
                    })
                    .child(if !target_installed {
                        l.not_installed_target
                    } else if self.applying {
                        l.action_applying
                    } else if active {
                        l.action_in_use
                    } else {
                        l.action_apply_theme
                    }),
            )
            .into_any_element()
    }
}

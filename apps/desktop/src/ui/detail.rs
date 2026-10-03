//! Installed-theme detail panel.

use gpui::{div, prelude::*, px, rgb, Context, FontWeight, Role};

use crate::app::types::SourceView;
use crate::app::{support_label, SkinApp};
use crate::i18n::t;
use crate::preview::preview_rgba;
use crate::ui::shows_auto_theme_controls;

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
            let button = |id: &'static str, label: &'static str| {
                div()
                    .id(id)
                    .role(Role::Button)
                    .aria_label(label)
                    .h(px(32.))
                    .px_4()
                    .rounded(px(7.))
                    .border_1()
                    .border_color(rgb(colors.border))
                    .bg(rgb(colors.control))
                    .flex()
                    .items_center()
                    .text_sm()
                    .text_color(rgb(colors.text))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(colors.hover)))
                    .child(label)
            };
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_3()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(colors.muted))
                .child(l.empty_library)
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .child(button("empty-browse-store", l.empty_browse_store).on_click(
                            cx.listener(|this, _event, _window, cx| {
                                this.switch_source(SourceView::Store, cx)
                            }),
                        ))
                        .child(
                            button("empty-choose-package", l.empty_choose_package).on_click(
                                cx.listener(|this, _event, window, cx| {
                                    this.choose_package(window, cx)
                                }),
                            ),
                        ),
                )
                .into_any_element();
        };
        let active = self.selected_settings_are_active(row);
        let target_installed = self.target_installations.is_installed(self.selected_target);
        let theme_supported = row.theme.supports_target(self.selected_target);
        let restart_confirmation = self.restart_confirmation_target == Some(self.selected_target);
        let detail_message = if !target_installed {
            l.format_please_install(self.selected_target.display_name())
        } else if !theme_supported {
            format!("这个主题不支持{}", self.selected_target.display_name())
        } else if self.message == l.action_applied {
            format!(
                "已应用 · {} · {}",
                support_label(row.theme.target_support(self.selected_target)),
                self.selected_target.display_name()
            )
        } else if self.message.is_empty() {
            format!(
                "{} · {}",
                support_label(row.theme.target_support(self.selected_target)),
                self.selected_target.display_name()
            )
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
            .when(shows_auto_theme_controls(std::env::consts::OS), |view| {
                view.child(self.render_auto_theme_controls(cx))
            })
            .child(self.render_detail_actions(
                row,
                active,
                target_installed,
                theme_supported,
                restart_confirmation,
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
        theme_supported: bool,
        restart_confirmation: bool,
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
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                // Not `flex_1`: the actions follow the message
                                // directly instead of floating at the far edge.
                                div()
                                    .min_w(px(0.))
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_xs()
                                    .text_color(rgb(
                                        if !target_installed
                                            || !theme_supported
                                            || detail_message.contains(l.error_keyword)
                                        {
                                            colors.danger
                                        } else {
                                            colors.muted
                                        },
                                    ))
                                    .child(detail_message.to_string()),
                            )
                            .child(self.render_theme_actions(row.theme.id.as_str(), cx)),
                    ),
            )
            .child(self.render_detail_buttons(
                row,
                active,
                target_installed,
                theme_supported,
                restart_confirmation,
                compact,
                short,
                cx,
            ))
            .into_any_element()
    }

    /// Low-key text actions on the info line: reveal in the file manager for
    /// every theme, delete (with inline confirmation) for user-installed ones.
    fn render_theme_actions(&self, theme_id: &str, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let confirming = self.confirm_delete.as_deref() == Some(theme_id);
        let busy = self.theme_sessions.is_busy(self.selected_target);
        let action = |id: &'static str, label: &'static str, color: u32, enabled: bool| {
            div()
                .id(id)
                .role(Role::Button)
                .aria_label(label)
                .flex_shrink_0()
                .text_xs()
                .whitespace_nowrap()
                .text_color(rgb(color))
                .when(enabled, |view| {
                    view.cursor_pointer().hover(|style| style.opacity(0.72))
                })
                .child(label)
        };
        if confirming {
            return div()
                .flex()
                .flex_shrink_0()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .whitespace_nowrap()
                        .text_color(rgb(colors.muted))
                        .child(l.delete_prompt),
                )
                .child(
                    action("delete-confirm", l.action_confirm, colors.danger, !busy).on_click(
                        cx.listener(|this, _event, _window, cx| this.delete_selected(cx)),
                    ),
                )
                .child(
                    action("delete-cancel", l.action_cancel, colors.link, true)
                        .on_click(cx.listener(|this, _event, _window, cx| this.cancel_delete(cx))),
                )
                .into_any_element();
        }
        let mut row = div().flex().flex_shrink_0().items_center().gap_3().child(
            action(
                "reveal-theme",
                l.reveal_in_file_manager(),
                colors.link,
                true,
            )
            .on_click(cx.listener(|this, _event, _window, cx| this.reveal_selected_theme(cx))),
        );
        if self.can_delete_selected() {
            row = row.child(if self.selected_theme_in_use() {
                action("delete-theme", l.action_delete_blocked, colors.muted, false)
                    .into_any_element()
            } else {
                action("delete-theme", l.action_delete, colors.link, true)
                    .on_click(
                        cx.listener(|this, _event, _window, cx| this.request_delete_selected(cx)),
                    )
                    .into_any_element()
            });
        }
        row.into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_detail_buttons(
        &self,
        row: &crate::app::types::ThemeRow,
        active: bool,
        target_installed: bool,
        theme_supported: bool,
        restart_confirmation: bool,
        compact: bool,
        short: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let busy = self.theme_sessions.is_busy(self.selected_target);
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
                    .opacity(if busy { 0.72 } else { 1.0 })
                    .child(l.action_restore_default)
                    .when(!busy, |button| {
                        button
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(colors.hover)))
                            .on_click(
                                cx.listener(|this, _event, _window, cx| this.restore_default(cx)),
                            )
                    }),
            )
            .child(
                div()
                    .id("apply")
                    .role(Role::Button)
                    .aria_label(if !target_installed {
                        l.not_installed_target
                    } else if !theme_supported {
                        "此主题不支持当前应用"
                    } else if active {
                        l.action_in_use
                    } else if restart_confirmation {
                        "重启 WorkBuddy 并应用"
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
                    .opacity(if busy || active || !target_installed || !theme_supported {
                        0.72
                    } else {
                        1.0
                    })
                    .when(
                        !busy && !active && target_installed && theme_supported,
                        |button| {
                            button
                                .cursor_pointer()
                                .hover(|style| style.opacity(0.88))
                                .on_click(
                                    cx.listener(|this, _event, _window, cx| {
                                        this.apply_selected(cx)
                                    }),
                                )
                        },
                    )
                    .child(if !target_installed {
                        l.not_installed_target
                    } else if !theme_supported {
                        "主题不兼容"
                    } else if busy {
                        l.action_applying
                    } else if active {
                        l.action_in_use
                    } else if restart_confirmation {
                        "重启并应用"
                    } else {
                        l.action_apply_theme
                    }),
            )
            .into_any_element()
    }
}

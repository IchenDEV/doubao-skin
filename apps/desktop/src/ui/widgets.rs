//! Reusable theme controls.

use gpui::{
    div, img, prelude::*, px, rgb, Context, FontWeight, MouseButton, MouseDownEvent,
    MouseMoveEvent, ObjectFit, Role, Toggled,
};

use skin_core::live;

use crate::app::types::ThemeRow;
use crate::app::{auto_theme::control_state, platform::AutoThemeServiceStatus};
use crate::app::{target_shortcut, SkinApp};
use crate::i18n::t;
use crate::preview::preview_rgba;
use crate::ui::assets::local_image_source;
use crate::ui::constants::{MIN_SURFACE_OPACITY, OPACITY_TRACK_WIDTH, SURFACE_OPACITY_RANGE};

impl SkinApp {
    pub(crate) fn render_auto_theme_controls(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let state = control_state(
            &self.auto_theme_settings,
            self.auto_theme_service_status,
            self.auto_theme_busy,
        );
        let saved_target = self
            .auto_theme_settings
            .last_applied()
            .map(|saved| saved.target())
            .unwrap_or(self.selected_target);
        let status = if self.auto_theme_busy {
            self.message.clone()
        } else if self.auto_theme_service_status == AutoThemeServiceStatus::Unsupported {
            l.auto_theme_unsupported.into()
        } else if self.auto_theme_settings.last_applied().is_none() {
            l.auto_theme_apply_first.into()
        } else if !self.auto_theme_settings.keep_requested()
            && self.auto_theme_service_status == AutoThemeServiceStatus::Enabled
        {
            l.auto_theme_cleanup_pending.into()
        } else if self.auto_theme_settings.keep_requested() {
            match self.auto_theme_service_status {
                AutoThemeServiceStatus::Enabled => l.auto_theme_enabled.into(),
                AutoThemeServiceStatus::RequiresApproval => l.auto_theme_approval_required.into(),
                AutoThemeServiceStatus::NotFound => l.auto_theme_missing_service.into(),
                _ => l.auto_theme_not_ready.into(),
            }
        } else {
            l.auto_theme_disabled.into()
        };

        div()
            .w_full()
            .px_3()
            .py_2()
            .rounded(px(9.))
            .border_1()
            .border_color(rgb(colors.border))
            .bg(rgb(colors.control).opacity(0.54))
            .flex()
            .flex_col()
            .gap_1()
            .child(self.render_auto_theme_row(
                "auto-theme-keep",
                l.auto_theme_keep_title,
                l.auto_theme_keep_description,
                state.keep_requested,
                state.keep_enabled,
                true,
                cx,
            ))
            .child(self.render_auto_theme_row(
                "auto-theme-login",
                l.auto_theme_login_title,
                &l.format_auto_theme_login_description(saved_target.display_name()),
                state.login_requested,
                state.login_enabled,
                false,
                cx,
            ))
            .child(
                div()
                    .h(px(16.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_xs()
                    .text_color(rgb(colors.muted))
                    .child(status)
                    .when(
                        self.auto_theme_service_status == AutoThemeServiceStatus::RequiresApproval,
                        |view| {
                            view.child(
                                div()
                                    .id("auto-theme-open-settings")
                                    .role(Role::Button)
                                    .aria_label(l.auto_theme_open_settings)
                                    .focusable()
                                    .tab_stop(true)
                                    .text_color(rgb(colors.link))
                                    .cursor_pointer()
                                    .hover(|style| style.opacity(0.72))
                                    .child(l.auto_theme_open_settings)
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.open_auto_theme_settings(cx)
                                    })),
                            )
                        },
                    ),
            )
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_auto_theme_row(
        &self,
        id: &'static str,
        title: &'static str,
        description: &str,
        toggled: bool,
        enabled: bool,
        keep_switch: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .id(id)
            .role(Role::Switch)
            .aria_label(l.format_switch_aria(title, enabled))
            .aria_toggled(if toggled {
                Toggled::True
            } else {
                Toggled::False
            })
            .h(px(34.))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .opacity(if enabled { 1.0 } else { 0.48 })
            .when(enabled, |row| {
                row.focusable()
                    .tab_stop(true)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        if keep_switch {
                            this.toggle_auto_theme_keep(cx);
                        } else {
                            this.toggle_open_at_login(cx);
                        }
                    }))
            })
            .child(
                div()
                    .min_w(px(0.))
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.text))
                            .child(title),
                    )
                    .child(
                        div()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_size(px(10.))
                            .text_color(rgb(colors.muted))
                            .child(description.to_string()),
                    ),
            )
            .child(
                div()
                    .w(px(34.))
                    .h(px(20.))
                    .p(px(2.))
                    .rounded_full()
                    .bg(rgb(if toggled {
                        colors.slider_accent
                    } else {
                        colors.border
                    }))
                    .flex()
                    .when(toggled, |track| track.justify_end())
                    .child(div().size(px(16.)).rounded_full().bg(rgb(colors.control))),
            )
            .into_any_element()
    }

    pub(crate) fn render_target_switch(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let mut segments = div()
            .w(px(336.))
            .h(px(36.))
            .p(px(2.))
            .rounded(px(8.))
            .border_1()
            .border_color(rgb(colors.border))
            .bg(rgb(colors.segmented_track))
            .flex()
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation()
            });
        for (index, target) in live::TargetApp::ALL.into_iter().enumerate() {
            let installed = target.is_installed();
            let selected = self.selected_target == target;
            let shortcut = target_shortcut(target);
            let label = l.format_target_label(target.display_name(), !installed);
            let aria = l.format_target_aria(target.display_name(), installed, selected, shortcut);
            segments = segments.child(
                div()
                    .id(("target-app", index))
                    .role(gpui::Role::Button)
                    .aria_label(aria)
                    .flex_1()
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgb(if selected {
                        colors.segmented_selected
                    } else {
                        colors.segmented_track
                    }))
                    .text_size(if installed { px(12.) } else { px(11.) })
                    .font_weight(if selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(colors.text))
                    .opacity(if installed { 1.0 } else { 0.46 })
                    .child(label)
                    .when(installed, |button| {
                        button
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(colors.control).opacity(0.72)))
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                this.switch_target(target, cx)
                            }))
                    }),
            );
        }
        segments.into_any_element()
    }

    pub(crate) fn render_opacity_control(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let percent = (self.surface_opacity * 100.0).round() as u32;
        let progress =
            ((self.surface_opacity - MIN_SURFACE_OPACITY) / SURFACE_OPACITY_RANGE).clamp(0.0, 1.0);
        div()
            .w(px(OPACITY_TRACK_WIDTH))
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_xs()
                    .text_color(rgb(colors.muted))
                    .child(l.opacity_label)
                    .child(format!("{percent}%")),
            )
            .child(
                div()
                    .id("opacity-slider")
                    .role(gpui::Role::Slider)
                    .aria_label(l.format_opacity_aria(percent))
                    .relative()
                    .w_full()
                    .h(px(24.))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                            this.opacity_drag_start =
                                Some((event.position.x, this.surface_opacity));
                            cx.stop_propagation();
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                        let Some((start_x, start_opacity)) = this.opacity_drag_start else {
                            return;
                        };
                        if !event.dragging() {
                            this.opacity_drag_start = None;
                            return;
                        }
                        let progress_delta = (event.position.x - start_x) / px(OPACITY_TRACK_WIDTH);
                        this.set_surface_opacity(
                            start_opacity + SURFACE_OPACITY_RANGE * progress_delta,
                            cx,
                        );
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, _cx| {
                            this.opacity_drag_start = None;
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, _cx| {
                            this.opacity_drag_start = None;
                        }),
                    )
                    .on_click(|_event, _window, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .relative()
                            .w_full()
                            .h(px(4.))
                            .rounded_full()
                            .bg(rgb(colors.border))
                            .child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top_0()
                                    .h_full()
                                    .w(px(OPACITY_TRACK_WIDTH * progress))
                                    .rounded_full()
                                    .bg(rgb(colors.slider_accent)),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .top(px(-4.))
                                    .left(px((OPACITY_TRACK_WIDTH * progress - 6.0)
                                        .clamp(0.0, OPACITY_TRACK_WIDTH - 12.0)))
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(rgb(colors.control))
                                    .border_1()
                                    .border_color(rgb(colors.slider_accent)),
                            ),
                    ),
            )
            .into_any_element()
    }

    pub(crate) fn render_theme_thumbnail(&self, row: &ThemeRow, size: f32) -> gpui::AnyElement {
        let accent = row.preview.colors.accent;
        if let Some(path) = row.theme.preview_image.as_ref() {
            return img(local_image_source(path))
                .size(px(size))
                .rounded(px(8.))
                .object_fit(ObjectFit::Cover)
                .border_1()
                .border_color(preview_rgba(accent, 0.24))
                .into_any_element();
        }
        if let Some(path) = row.preview.background.as_ref() {
            return img(local_image_source(path))
                .size(px(size))
                .rounded(px(8.))
                .object_fit(ObjectFit::Cover)
                .border_1()
                .border_color(preview_rgba(accent, 0.24))
                .into_any_element();
        }
        if row.preview.icons.main.is_some() {
            return div()
                .size(px(size))
                .rounded(px(8.))
                .bg(preview_rgba(row.preview.colors.main, 1.0))
                .border_1()
                .border_color(preview_rgba(accent, 0.28))
                .flex()
                .items_center()
                .justify_center()
                .child(crate::preview::preview_main_icon(
                    row.preview.icons.main.as_ref(),
                    size * 0.58,
                    accent,
                    row.preview.text,
                ))
                .into_any_element();
        }
        div()
            .size(px(size))
            .rounded(px(8.))
            .bg(preview_rgba(row.preview.colors.main, 1.0))
            .border_1()
            .border_color(preview_rgba(accent, 0.35))
            .child(
                div()
                    .m(px(size * 0.34))
                    .size(px(size * 0.32))
                    .rounded_full()
                    .bg(preview_rgba(accent, 1.0)),
            )
            .into_any_element()
    }
}

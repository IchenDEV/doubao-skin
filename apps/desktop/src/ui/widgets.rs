//! Reusable theme controls.

use gpui::{
    div, img, prelude::*, px, rgb, Context, FontWeight, MouseButton, MouseDownEvent,
    MouseMoveEvent, ObjectFit,
};

use skin_core::live;

use crate::app::types::ThemeRow;
use crate::app::SkinApp;
use crate::i18n::t;
use crate::preview::preview_rgba;
use crate::ui::assets::local_image_source;
use crate::ui::constants::{MIN_SURFACE_OPACITY, OPACITY_TRACK_WIDTH, SURFACE_OPACITY_RANGE};

impl SkinApp {
    pub(crate) fn render_target_switch(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let mut segments = div()
            .w(px(224.))
            .h(px(36.))
            .p(px(2.))
            .rounded(px(8.))
            .border_1()
            .border_color(rgb(colors.border))
            .bg(rgb(colors.segmented_track))
            .flex();
        for (index, target) in live::TargetApp::ALL.into_iter().enumerate() {
            let installed = target.is_installed();
            let selected = self.selected_target == target;
            let shortcut = if target == live::TargetApp::Doubao {
                "Command-1"
            } else {
                "Command-2"
            };
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

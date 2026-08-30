//! Installed-theme preview rendering.

mod main;
mod sidebar;
mod utils;

pub use self::utils::{preview_icon, preview_main_icon, preview_rgba};

use gpui::{div, img, prelude::*, px, rgb, ObjectFit};

use skin_core::theme;

use crate::app::types::ThemeRow;
use crate::app::{preview_identity, SkinApp};
use crate::ui::assets::local_image_source;
use crate::ui::constants::{PREVIEW_CONTENT_RADIUS, PREVIEW_FRAME_RADIUS};

impl SkinApp {
    pub(crate) fn render_preview(
        &self,
        row: &ThemeRow,
        compact: bool,
        short: bool,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let (app_name, greeting) = preview_identity(self.selected_target);
        let style = &row.preview;
        let accent = style.colors.accent;
        let surface = if style.has_background {
            self.surface_opacity
        } else {
            1.0
        };
        let opacity = theme::surface_opacity_profile(surface);
        let sidebar_opacity = if style.has_background {
            opacity.preview_sidebar
        } else {
            0.91
        };
        let main_opacity = if style.has_background {
            opacity.preview_page
        } else {
            0.56
        };
        let input_opacity = if style.has_background {
            opacity.input
        } else {
            0.94
        };
        let layer_opacity = if style.has_background {
            opacity.layer
        } else {
            0.34
        };
        let density_scale: f32 = match style.density.as_str() {
            "compact" => 0.88,
            "spacious" => 1.08,
            _ => 1.0,
        };
        let sidebar_width = (style.sidebar_width * if compact { 0.43 } else { 0.61 }).clamp(
            if compact { 94.0 } else { 126.0 },
            if compact { 132.0 } else { 190.0 },
        );
        let nav_row_height = (23.0 * density_scale).clamp(20.0, 27.0);
        let action_row_height = (24.0 * density_scale).clamp(21.0, 29.0);
        let nav_icon_size = (13.0 * density_scale).clamp(12.0, 15.0);
        let action_icon_size = (14.0 * density_scale).clamp(12.0, 16.0);
        let chat_margin = (style.chat_margin * if compact { 0.44 } else { 0.88 }).clamp(
            if compact { 8.0 } else { 16.0 },
            if compact { 24.0 } else { 48.0 },
        );
        let composer_scale = if compact { 0.82 } else { 1.08 };
        let composer_height = (style.composer_min_height * composer_scale).clamp(
            if compact { 42.0 } else { 52.0 },
            if compact { 64.0 } else { 88.0 },
        );
        let composer_padding = (style.composer_padding * if compact { 0.54 } else { 0.72 })
            .clamp(5.0, if compact { 10.0 } else { 14.0 });
        let composer_gap =
            (style.composer_gap * if compact { 0.62 } else { 0.76 }).clamp(3.0, 12.0);
        let composer_icon_size =
            (style.composer_icon_size * if compact { 0.62 } else { 0.72 }).clamp(11.0, 18.0);
        let composer_radius =
            (style.composer_radius * style.radius_scale * if compact { 0.82 } else { 1.0 })
                .clamp(4.0, 28.0);

        let canvas = self.render_preview_canvas(
            style,
            accent,
            sidebar_width,
            sidebar_opacity,
            main_opacity,
            input_opacity,
            layer_opacity,
            nav_row_height,
            nav_icon_size,
            action_row_height,
            action_icon_size,
            chat_margin,
            composer_height,
            composer_padding,
            composer_gap,
            composer_icon_size,
            composer_radius,
            app_name,
            greeting,
            compact,
        );

        div()
            .w_full()
            .flex_1()
            .min_h(if short {
                px(188.)
            } else if compact {
                px(236.)
            } else {
                px(320.)
            })
            .max_h(if short {
                px(188.)
            } else if compact {
                px(360.)
            } else {
                px(520.)
            })
            .when(!compact, |frame| frame.aspect_ratio(16.0 / 9.0))
            .overflow_hidden()
            .rounded(px(PREVIEW_FRAME_RADIUS))
            .border_1()
            .border_color(rgb(colors.border))
            .bg(preview_rgba(style.colors.main, 1.0))
            .flex()
            .child(canvas)
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_preview_canvas(
        &self,
        style: &theme::PreviewStyle,
        accent: theme::PreviewColor,
        sidebar_width: f32,
        sidebar_opacity: f32,
        main_opacity: f32,
        input_opacity: f32,
        layer_opacity: f32,
        nav_row_height: f32,
        nav_icon_size: f32,
        action_row_height: f32,
        action_icon_size: f32,
        chat_margin: f32,
        composer_height: f32,
        composer_padding: f32,
        composer_gap: f32,
        composer_icon_size: f32,
        composer_radius: f32,
        app_name: &'static str,
        greeting: &'static str,
        compact: bool,
    ) -> gpui::AnyElement {
        let mut canvas = div()
            .relative()
            .size_full()
            .overflow_hidden()
            .rounded(px(PREVIEW_CONTENT_RADIUS))
            .bg(preview_rgba(style.colors.main, 1.0));
        if let Some(path) = &style.background {
            let fit = match style.background_fit.as_str() {
                "contain" => ObjectFit::Contain,
                "fill" => ObjectFit::Fill,
                "none" => ObjectFit::None,
                "scale-down" => ObjectFit::ScaleDown,
                _ => ObjectFit::Cover,
            };
            canvas = canvas
                .child(
                    img(local_image_source(path))
                        .absolute()
                        .inset_0()
                        .size_full()
                        .rounded(px(PREVIEW_CONTENT_RADIUS))
                        .object_fit(fit)
                        .opacity(style.background_opacity),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(px(PREVIEW_CONTENT_RADIUS))
                        .bg(rgb(style.background_base)
                            .opacity(style.background_veil * style.background_opacity)),
                );
        } else {
            canvas = canvas
                .child(
                    div()
                        .absolute()
                        .top(px(-80.))
                        .right(px(-40.))
                        .size(px(260.))
                        .rounded_full()
                        .bg(preview_rgba(accent, 0.16)),
                )
                .child(
                    div()
                        .absolute()
                        .bottom(px(-120.))
                        .left(px(80.))
                        .size(px(300.))
                        .rounded_full()
                        .bg(preview_rgba(style.colors.sidebar, 0.34)),
                );
        }
        canvas
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .child(self.render_preview_sidebar(
                        style,
                        sidebar_width,
                        sidebar_opacity,
                        nav_row_height,
                        nav_icon_size,
                        accent,
                        app_name,
                        compact,
                    ))
                    .child(self.render_preview_main(
                        style,
                        accent,
                        main_opacity,
                        input_opacity,
                        layer_opacity,
                        chat_margin,
                        composer_height,
                        composer_padding,
                        composer_gap,
                        composer_icon_size,
                        composer_radius,
                        nav_icon_size,
                        action_row_height,
                        action_icon_size,
                        app_name,
                        greeting,
                        compact,
                    )),
            )
            .into_any_element()
    }
}

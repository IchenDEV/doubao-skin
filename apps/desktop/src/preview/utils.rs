//! Preview icons, colors, and image helpers.

use std::path::PathBuf;

use gpui::{div, img, prelude::*, px, rgb, svg, FontWeight, ObjectFit, Rgba, SharedString};

use skin_core::theme;

use crate::ui::assets::local_image_source;

pub fn preview_rgba(color: theme::PreviewColor, layer_opacity: f32) -> Rgba {
    let mut painted = rgb(color.rgb);
    painted.a = color.alpha * layer_opacity.clamp(0.0, 1.0);
    painted
}

pub fn preview_icon(
    path: Option<&PathBuf>,
    icon_size: f32,
    color: theme::PreviewColor,
) -> gpui::AnyElement {
    if let Some(path) = path {
        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
        {
            svg()
                .path(SharedString::from(path.to_string_lossy().into_owned()))
                .size(px(icon_size))
                .text_color(preview_rgba(color, 1.0))
                .into_any_element()
        } else {
            img(local_image_source(path))
                .size(px(icon_size))
                .object_fit(ObjectFit::Contain)
                .into_any_element()
        }
    } else {
        div()
            .size(px(icon_size))
            .rounded(px(icon_size * 0.32))
            .bg(preview_rgba(color, 0.18))
            .into_any_element()
    }
}

pub fn preview_main_icon(
    path: Option<&PathBuf>,
    icon_size: f32,
    accent: theme::PreviewColor,
    text_color: theme::PreviewColor,
) -> gpui::AnyElement {
    if let Some(path) = path {
        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
        {
            return svg()
                .path(SharedString::from(path.to_string_lossy().into_owned()))
                .size(px(icon_size))
                .text_color(preview_rgba(accent, 1.0))
                .into_any_element();
        }
        return img(local_image_source(path))
            .size(px(icon_size))
            .object_fit(ObjectFit::Contain)
            .into_any_element();
    }
    div()
        .size(px(icon_size))
        .rounded(px(icon_size * 0.34))
        .bg(preview_rgba(accent, 1.0))
        .border_1()
        .border_color(rgb(0xffffff).opacity(0.82))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(px(icon_size * 0.68))
                .rounded_full()
                .bg(rgb(0xfff8ed))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px((icon_size * 0.23).max(7.)))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(preview_rgba(text_color, 1.0))
                .child("•ᴗ•"),
        )
        .into_any_element()
}

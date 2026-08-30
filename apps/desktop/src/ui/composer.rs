//! Composer preview surface.

use gpui::{div, prelude::*, px};

use skin_core::theme;

use crate::app::SkinApp;
use crate::i18n::t;
use crate::preview::{preview_icon, preview_rgba};

impl SkinApp {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_composer(
        &self,
        style: &theme::PreviewStyle,
        chat_margin: f32,
        composer_height: f32,
        composer_padding: f32,
        composer_gap: f32,
        composer_icon_size: f32,
        composer_radius: f32,
        input_opacity: f32,
    ) -> gpui::AnyElement {
        let l = t();
        div()
            .absolute()
            .left(px(chat_margin))
            .right(px(chat_margin))
            .bottom(if composer_height > 52.0 {
                px(16.)
            } else {
                px(10.)
            })
            .min_h(px(composer_height))
            .p(px(composer_padding))
            .rounded(px(composer_radius))
            .bg(preview_rgba(style.input, input_opacity))
            .border_1()
            .border_color(preview_rgba(style.input_border, 0.72))
            .flex()
            .flex_col()
            .gap(px(composer_gap))
            .child(
                div()
                    .text_size(px(10.))
                    .text_color(preview_rgba(style.composer_placeholder, 0.78))
                    .child(l.nav_composer_placeholder),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(composer_gap))
                    .child(preview_icon(
                        style.icons.new_task.as_ref(),
                        composer_icon_size,
                        style.composer_icon,
                    ))
                    .child(preview_icon(
                        style.icons.project.as_ref(),
                        composer_icon_size,
                        style.composer_icon,
                    ))
                    .child(preview_icon(
                        style.icons.confirm.as_ref(),
                        composer_icon_size,
                        style.composer_icon,
                    ))
                    .child(preview_icon(
                        style.icons.knowledge.as_ref(),
                        composer_icon_size,
                        style.composer_icon,
                    ))
                    .child(preview_icon(
                        style.icons.more_skills.as_ref(),
                        composer_icon_size,
                        style.composer_icon,
                    ))
                    .child(preview_icon(
                        style.icons.connector.as_ref(),
                        composer_icon_size,
                        style.composer_icon,
                    ))
                    .child(div().flex_1())
                    .child(preview_icon(
                        style.icons.voice.as_ref(),
                        composer_icon_size + 2.0,
                        style.composer_icon,
                    )),
            )
            .into_any_element()
    }
}

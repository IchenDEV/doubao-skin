//! Main conversation preview content.

use gpui::{div, prelude::*, px, FontWeight};

use skin_core::theme;

use crate::app::SkinApp;
use crate::i18n::t;
use crate::preview::{preview_icon, preview_main_icon, preview_rgba};
use crate::ui::constants::PREVIEW_CONTENT_RADIUS;

fn preview_action_item(
    label: &'static str,
    path: Option<&std::path::PathBuf>,
    icon_color: theme::PreviewColor,
    text_color: theme::PreviewColor,
    background_color: theme::PreviewColor,
    background_opacity: f32,
    geometry: (f32, f32),
) -> gpui::AnyElement {
    let (row_height, icon_size) = geometry;
    div()
        .w_full()
        .h(px(row_height))
        .px_2()
        .flex()
        .items_center()
        .gap_2()
        .rounded(px(7.))
        .bg(preview_rgba(background_color, background_opacity))
        .child(preview_icon(path, icon_size, icon_color))
        .child(
            div()
                .text_size(px(10.))
                .text_color(preview_rgba(text_color, 0.9))
                .child(label),
        )
        .into_any_element()
}

impl SkinApp {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_preview_main(
        &self,
        style: &theme::PreviewStyle,
        accent: theme::PreviewColor,
        main_opacity: f32,
        input_opacity: f32,
        layer_opacity: f32,
        chat_margin: f32,
        composer_height: f32,
        composer_padding: f32,
        composer_gap: f32,
        composer_icon_size: f32,
        composer_radius: f32,
        nav_icon_size: f32,
        action_row_height: f32,
        action_icon_size: f32,
        app_name: &'static str,
        greeting: &'static str,
        compact: bool,
    ) -> gpui::AnyElement {
        let l = t();
        let action_geom = (action_row_height, action_icon_size);
        div()
            .flex_1()
            .h_full()
            .relative()
            .rounded_r(px(PREVIEW_CONTENT_RADIUS))
            .bg(preview_rgba(style.colors.main, main_opacity))
            .flex()
            .flex_col()
            .child(self.render_preview_top_bar(style, app_name, nav_icon_size, compact))
            .child(
                div()
                    .flex_1()
                    .px(px(chat_margin))
                    .pt(if compact { px(8.) } else { px(13.) })
                    .pb(if compact { px(64.) } else { px(86.) })
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(preview_main_icon(
                        style.icons.main.as_ref(),
                        if compact { 38. } else { 52. },
                        accent,
                        style.text,
                    ))
                    .child(
                        div()
                            .mt_2()
                            .text_size(if compact { px(15.) } else { px(19.) })
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(preview_rgba(style.text, 1.0))
                            .child(greeting),
                    )
                    .child(
                        div()
                            .mt(if compact { px(8.) } else { px(12.) })
                            .w(if compact { px(210.) } else { px(270.) })
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(9.))
                                    .text_color(preview_rgba(style.text, 0.5))
                                    .child(l.section_recommended),
                            )
                            .child(preview_action_item(
                                l.nav_daily_work,
                                style.icons.daily_work.as_ref(),
                                theme::PreviewColor::opaque(0x4f83d6),
                                style.text,
                                style.surface,
                                layer_opacity,
                                action_geom,
                            ))
                            .child(preview_action_item(
                                l.nav_content_creation,
                                style.icons.content_creation.as_ref(),
                                theme::PreviewColor::opaque(0x43a873),
                                style.text,
                                style.surface,
                                layer_opacity,
                                action_geom,
                            ))
                            .child(preview_action_item(
                                l.nav_research,
                                style.icons.research.as_ref(),
                                theme::PreviewColor::opaque(0x9a67d8),
                                style.text,
                                style.surface,
                                layer_opacity,
                                action_geom,
                            ))
                            .child(preview_action_item(
                                l.nav_design,
                                style.icons.design.as_ref(),
                                theme::PreviewColor::opaque(0xdf648d),
                                style.text,
                                style.surface,
                                layer_opacity,
                                action_geom,
                            )),
                    ),
            )
            .child(self.render_composer(
                style,
                chat_margin,
                composer_height,
                composer_padding,
                composer_gap,
                composer_icon_size,
                composer_radius,
                input_opacity,
            ))
            .into_any_element()
    }

    fn render_preview_top_bar(
        &self,
        style: &theme::PreviewStyle,
        app_name: &'static str,
        nav_icon_size: f32,
        compact: bool,
    ) -> gpui::AnyElement {
        div()
            .h(if compact { px(30.) } else { px(36.) })
            .px_3()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(preview_rgba(style.text, 0.08))
            .child(
                div()
                    .text_size(px(9.))
                    .text_color(preview_rgba(style.text, 0.48))
                    .child(app_name),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(preview_icon(
                        style.icons.read_aloud.as_ref(),
                        nav_icon_size,
                        style.text,
                    ))
                    .child(preview_icon(
                        style.icons.copy.as_ref(),
                        nav_icon_size,
                        style.text,
                    ))
                    .child(preview_icon(
                        style.icons.sidebar.as_ref(),
                        nav_icon_size,
                        style.text,
                    )),
            )
            .into_any_element()
    }
}

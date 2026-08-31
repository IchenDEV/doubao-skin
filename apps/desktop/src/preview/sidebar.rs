//! Preview sidebar content.

use gpui::{div, prelude::*, px, rgb, FontWeight};

use skin_core::theme;

use crate::app::SkinApp;
use crate::i18n::t;
use crate::preview::{preview_icon, preview_main_icon, preview_rgba};
use crate::ui::constants::PREVIEW_CONTENT_RADIUS;

pub fn preview_nav_item(
    label: &'static str,
    path: Option<&std::path::PathBuf>,
    color: theme::PreviewColor,
    selected: bool,
    row_height: f32,
    icon_size: f32,
) -> gpui::AnyElement {
    div()
        .w_full()
        .h(px(row_height))
        .px_2()
        .flex()
        .items_center()
        .gap_2()
        .rounded(px(6.))
        .when(selected, |row| row.bg(rgb(0xffffff).opacity(0.58)))
        .child(preview_icon(path, icon_size, color))
        .child(
            div()
                .text_size(px(10.))
                .text_color(preview_rgba(color, 0.92))
                .child(label),
        )
        .into_any_element()
}

impl SkinApp {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_preview_sidebar(
        &self,
        style: &theme::PreviewStyle,
        sidebar_width: f32,
        sidebar_opacity: f32,
        nav_row_height: f32,
        nav_icon_size: f32,
        accent: theme::PreviewColor,
        app_name: &'static str,
        compact: bool,
    ) -> gpui::AnyElement {
        let l = t();
        div()
            .w(px(sidebar_width))
            .h_full()
            .rounded_l(px(PREVIEW_CONTENT_RADIUS))
            .bg(preview_rgba(style.colors.sidebar, sidebar_opacity))
            .border_r_1()
            .border_color(preview_rgba(style.text, 0.09))
            .p(if compact { px(8.) } else { px(11.) })
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .h(px(25.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_size(px(12.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(preview_rgba(style.text, 1.0))
                    .child(l.nav_work_header)
                    .child(div().text_size(px(13.)).child("⌕")),
            )
            .child(preview_nav_item(
                l.nav_new_task,
                style.icons.new_task.as_ref(),
                style.text,
                false,
                nav_row_height,
                nav_icon_size,
            ))
            .child(preview_nav_item(
                l.nav_scheduled,
                style.icons.scheduled.as_ref(),
                style.text,
                false,
                nav_row_height,
                nav_icon_size,
            ))
            .child(preview_nav_item(
                l.nav_skills,
                style.icons.skills.as_ref(),
                style.text,
                false,
                nav_row_height,
                nav_icon_size,
            ))
            .child(preview_nav_item(
                l.nav_cloud,
                style.icons.cloud.as_ref(),
                style.text,
                false,
                nav_row_height,
                nav_icon_size,
            ))
            .child(preview_nav_item(
                l.nav_remote,
                style.icons.remote.as_ref(),
                style.text,
                false,
                nav_row_height,
                nav_icon_size,
            ))
            .child(
                div()
                    .mt_2()
                    .text_size(px(9.))
                    .text_color(preview_rgba(style.text, 0.48))
                    .child(l.section_pinned),
            )
            .child(preview_nav_item(
                l.nav_main_conversation,
                style.icons.conversation.as_ref(),
                accent,
                true,
                nav_row_height,
                nav_icon_size,
            ))
            .child(
                div()
                    .mt_1()
                    .text_size(px(9.))
                    .text_color(preview_rgba(style.text, 0.48))
                    .child(l.section_projects),
            )
            .child(preview_nav_item(
                l.nav_look,
                style.icons.project.as_ref(),
                style.text,
                false,
                nav_row_height,
                nav_icon_size,
            ))
            .child(div().flex_1())
            .child(
                div()
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(preview_main_icon(
                        style.icons.main.as_ref(),
                        20.,
                        accent,
                        style.text,
                    ))
                    .child(
                        div()
                            .text_size(px(9.))
                            .text_color(preview_rgba(style.text, 0.76))
                            .child(app_name),
                    ),
            )
            .into_any_element()
    }
}

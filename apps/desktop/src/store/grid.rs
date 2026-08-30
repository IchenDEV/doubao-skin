//! Theme-store grid.

use gpui::{
    div, img, prelude::*, px, rgb, svg, Context, FontWeight, ObjectFit, Role, SharedString,
};

use crate::app::theme_ops::parse_store_accent;
use crate::app::types::StoreRow;
use crate::app::SkinApp;
use crate::i18n::{self, t};
use crate::ui::assets::local_image_source;

impl SkinApp {
    pub(crate) fn render_store_preview(&self, row: &StoreRow, height: f32) -> gpui::AnyElement {
        let colors = self.colors;
        let accent = parse_store_accent(row.theme.accent.as_deref());
        if let Some(path) = row.preview.as_ref() {
            let shared = SharedString::from(path.to_string_lossy().into_owned());
            if path
                .extension()
                .and_then(|v| v.to_str())
                .is_some_and(|v| v.eq_ignore_ascii_case("svg"))
            {
                return div()
                    .w_full()
                    .h(px(height))
                    .bg(rgb(colors.preview_placeholder))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(svg().path(shared).size(px(64.)).text_color(rgb(accent)))
                    .into_any_element();
            }
            return img(local_image_source(path))
                .w_full()
                .h(px(height))
                .object_fit(ObjectFit::Cover)
                .into_any_element();
        }
        div()
            .w_full()
            .h(px(height))
            .bg(rgb(colors.preview_placeholder))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .size(px(66.))
                    .rounded(px(16.))
                    .bg(rgb(accent).opacity(0.15))
                    .border_1()
                    .border_color(rgb(accent).opacity(0.32))
                    .child(
                        div()
                            .m(px(22.))
                            .size(px(22.))
                            .rounded_full()
                            .bg(rgb(accent)),
                    ),
            )
            .into_any_element()
    }

    pub(crate) fn render_store_card(
        &self,
        index: usize,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let row = &self.store_rows[index];
        let installed = self
            .themes
            .iter()
            .any(|theme| theme.theme.id == row.theme.id);
        let installing = self.installing_store_theme.as_deref() == Some(row.theme.id.as_str());
        let accent = parse_store_accent(row.theme.accent.as_deref());
        div()
            .id(("store-theme", index))
            .w(px(if compact { 212. } else { 244. }))
            .overflow_hidden()
            .rounded(px(10.))
            .border_1()
            .border_color(rgb(colors.border))
            .bg(rgb(colors.control))
            .hover(|style| style.border_color(rgb(colors.card_hover_border)))
            .child(self.render_store_preview(row, if compact { 118. } else { 138. }))
            .child(
                div()
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.text))
                            .child(row.theme.name.clone()),
                    )
                    .child(
                        div()
                            .h(px(18.))
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_xs()
                            .text_color(rgb(colors.muted))
                            .child(row.theme.description.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(div().text_xs().text_color(rgb(colors.muted)).child(
                                if row.theme.version.is_empty() {
                                    i18n::store_category_label(&row.theme.category).to_string()
                                } else {
                                    format!(
                                        "{} · {}",
                                        i18n::store_category_label(&row.theme.category),
                                        row.theme.version
                                    )
                                },
                            ))
                            .child(
                                div()
                                    .id(("install-store-theme", index))
                                    .role(Role::Button)
                                    .aria_label(
                                        l.format_store_item_aria(&row.theme.name, installed),
                                    )
                                    .h(px(30.))
                                    .min_w(px(78.))
                                    .px_4()
                                    .rounded(px(7.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(rgb(if installed {
                                        colors.installed_control
                                    } else {
                                        accent
                                    }))
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(if installed {
                                        colors.muted
                                    } else {
                                        0xffffff
                                    }))
                                    .when(!installed && !installing, |btn| {
                                        btn.cursor_pointer().hover(|style| style.opacity(0.86))
                                    })
                                    .child(if installing {
                                        l.install_button_busy
                                    } else if installed {
                                        l.install_button_done
                                    } else {
                                        l.install_button
                                    })
                                    .on_click(cx.listener(move |this, _event, _window, cx| {
                                        this.install_store_theme(index, cx)
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }
}

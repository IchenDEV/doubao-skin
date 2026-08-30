//! Primary navigation sidebar.

use gpui::{div, prelude::*, px, rgb, svg, Context, ExternalPaths, FontWeight, Role};

use crate::app::types::SourceView;
use crate::app::SkinApp;
use crate::i18n::t;
use crate::preview::preview_rgba;

impl SkinApp {
    pub(crate) fn render_source_switch(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .w_full()
            .h(px(36.))
            .p(px(2.))
            .rounded(px(8.))
            .border_1()
            .border_color(rgb(colors.border))
            .bg(rgb(colors.segmented_track))
            .flex()
            .child(
                div()
                    .id("source-library")
                    .role(Role::Button)
                    .aria_label(l.source_library)
                    .flex_1()
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgb(if self.source_view == SourceView::Library {
                        colors.segmented_selected
                    } else {
                        colors.segmented_track
                    }))
                    .text_sm()
                    .font_weight(if self.source_view == SourceView::Library {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(colors.text))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(colors.control).opacity(0.72)))
                    .child(l.source_library)
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.switch_source(SourceView::Library, cx)
                    })),
            )
            .child(
                div()
                    .id("source-store")
                    .role(Role::Button)
                    .aria_label(l.source_store)
                    .flex_1()
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgb(if self.source_view == SourceView::Store {
                        colors.segmented_selected
                    } else {
                        colors.segmented_track
                    }))
                    .text_sm()
                    .font_weight(if self.source_view == SourceView::Store {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(colors.text))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(colors.control).opacity(0.72)))
                    .child(l.source_store)
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.switch_source(SourceView::Store, cx)
                    })),
            )
            .into_any_element()
    }

    pub(crate) fn render_drop_target(
        &self,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .id(if compact {
                "drop-compact"
            } else {
                "drop-sidebar"
            })
            .role(Role::Button)
            .aria_label(l.drop_label)
            .mx(if compact { px(16.) } else { px(12.) })
            .h(if compact { px(42.) } else { px(86.) })
            .px_3()
            .rounded(px(10.))
            .border_1()
            .border_dashed()
            .border_color(rgb(colors.drop_border))
            .bg(rgb(colors.control).opacity(0.62))
            .flex()
            .items_center()
            .justify_center()
            .gap_3()
            .drag_over::<ExternalPaths>(move |style, _, _, _| {
                style
                    .bg(rgb(colors.drop_hover))
                    .border_color(rgb(colors.drop_accent))
            })
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                cx.stop_propagation();
                this.install_dropped_paths(paths.paths(), cx)
            }))
            .child(
                svg()
                    .path("icons/install.svg")
                    .size(px(if compact { 20. } else { 28. }))
                    .text_color(rgb(colors.muted)),
            )
            .child(
                div()
                    .flex()
                    .when(!compact, |view| view.flex_col())
                    .items_start()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(colors.text))
                            .child(if self.installing_package {
                                l.install_installing
                            } else {
                                l.install_drop_hint
                            }),
                    )
                    .when(!compact, |view| {
                        view.child(
                            div()
                                .id("choose-package-sidebar")
                                .role(Role::Button)
                                .aria_label(l.install_prompt_title)
                                .text_xs()
                                .text_color(rgb(colors.link))
                                .cursor_pointer()
                                .hover(|style| style.opacity(0.72))
                                .child(l.install_choose_file)
                                .on_click(cx.listener(|this, _event, window, cx| {
                                    this.choose_package(window, cx)
                                })),
                        )
                    }),
            )
            .into_any_element()
    }

    pub(crate) fn render_theme_item(
        &self,
        index: usize,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let row = &self.themes[index];
        let selected = index == self.selected;
        let active = self.active_theme.as_deref() == Some(row.theme.id.as_str());
        let accent = row.preview.colors.accent;
        let item = div()
            .id(("theme", index))
            .role(Role::Button)
            .aria_label(row.theme.name.clone())
            .h(px(50.))
            .flex_shrink_0()
            .min_w(if compact { px(132.) } else { px(0.) })
            .px_3()
            .flex()
            .items_center()
            .gap_3()
            .rounded(px(8.))
            .bg(if selected {
                preview_rgba(accent, 0.16)
            } else {
                rgb(colors.sidebar)
            })
            .cursor_pointer()
            .hover(|style| style.bg(rgb(colors.hover)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.select(index, cx);
            }))
            .child(self.render_theme_thumbnail(row, if compact { 34. } else { 38. }))
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .font_weight(if selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(colors.text))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(row.theme.name.clone()),
            )
            .when(active, |item| {
                item.child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(preview_rgba(accent, 1.0))
                        .child("✓"),
                )
            });
        item.into_any_element()
    }
}

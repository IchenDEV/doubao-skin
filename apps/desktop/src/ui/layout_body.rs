//! Main content area layout.

use gpui::{div, prelude::*, px, rgb, Context, FontWeight, Role};

use crate::app::types::SourceView;
use crate::app::SkinApp;
use crate::i18n::t;

impl SkinApp {
    pub(crate) fn render_body(
        &self,
        compact: bool,
        short: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        if compact {
            if self.source_view == SourceView::Store {
                return div()
                    .flex_1()
                    .min_h(px(0.))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .px_4()
                            .py_3()
                            .bg(rgb(colors.sidebar))
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(div().w(px(244.)).child(self.render_source_switch(cx)))
                            .child(self.render_search_bar(compact, cx)),
                    )
                    .child(self.render_store(true, cx))
                    .into_any_element();
            }
            let content = self.render_theme_detail(compact, short, cx);
            return div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .flex_col()
                .bg(rgb(colors.sidebar))
                .child(
                    div()
                        .px_4()
                        .pt_3()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(div().w(px(244.)).child(self.render_source_switch(cx)))
                        .child(self.render_search_bar(compact, cx)),
                )
                .child(self.render_drop_target(true, cx))
                .child(self.render_theme_list(true, cx))
                .child(
                    div()
                        .flex_1()
                        .min_h(px(0.))
                        .bg(rgb(colors.shell))
                        .child(content),
                )
                .into_any_element();
        }
        let detail = if self.source_view == SourceView::Store {
            self.render_store_detail(cx)
        } else {
            self.render_theme_detail(compact, short, cx)
        };
        div()
            .flex_1()
            .min_h(px(0.))
            .flex()
            .child(self.render_wide_sidebar(cx))
            .child(detail)
            .into_any_element()
    }

    fn render_wide_sidebar(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .w(px(276.))
            .h_full()
            .bg(rgb(colors.sidebar))
            .border_r_1()
            .border_color(rgb(colors.border))
            .flex()
            .flex_col()
            .child(
                div()
                    .px_4()
                    .pt_4()
                    .pb_3()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(self.render_source_switch(cx))
                    .child(self.render_search_bar(false, cx)),
            )
            .when(self.source_view == SourceView::Library, |sidebar| {
                sidebar
                    .child(
                        div()
                            .px_5()
                            .pb_3()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.muted))
                            .child(l.source_library)
                            .child(self.filtered_indices().len().to_string()),
                    )
                    .child(self.render_drop_target(false, cx))
                    .child(div().h(px(12.)))
                    .child(self.render_theme_list(false, cx))
            })
            .when(self.source_view == SourceView::Store, |sidebar| {
                sidebar
                    .child(
                        div()
                            .px_5()
                            .pb_3()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(colors.muted))
                            .child(l.source_store)
                            .child(
                                div()
                                    .id("refresh-store-sidebar")
                                    .role(Role::Button)
                                    .aria_label(l.store_refresh)
                                    .cursor_pointer()
                                    .hover(|style| style.opacity(0.72))
                                    .child(if self.store_loading {
                                        l.store_loading
                                    } else {
                                        l.store_refresh
                                    })
                                    .on_click(
                                        cx.listener(|this, _event, _window, cx| {
                                            this.load_store(cx)
                                        }),
                                    ),
                            ),
                    )
                    .child(self.render_store_sidebar_list(cx))
            })
            .into_any_element()
    }

    pub(crate) fn render_theme_list(
        &self,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        let mut list = div()
            .id("themes")
            .role(Role::List)
            .aria_label(l.aria_all_themes)
            .flex()
            .gap_1()
            .overflow_scroll();
        list = if compact {
            list.flex_row().w_full().h(px(52.)).px_4()
        } else {
            list.flex_col().flex_1().px_3().pb_4()
        };
        let indices = self.filtered_indices();
        if indices.is_empty() {
            list = list.child(
                div()
                    .px_3()
                    .py_4()
                    .text_xs()
                    .text_color(rgb(colors.muted))
                    .child(l.search_no_match),
            );
        }
        for index in indices {
            list = list.child(self.render_theme_item(index, compact, cx));
        }
        list.into_any_element()
    }
}

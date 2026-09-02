//! Windows About entry and lightweight in-window dialog.

use gpui::{div, prelude::*, px, rgb, Context, FontWeight, MouseButton, Role, Window};

use crate::app::actions::{OFFICIAL_REPOSITORY_URL, OPEN_SOURCE_NOTICE};
use crate::app::SkinApp;
use crate::i18n::t;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AboutKeyAction {
    Ignore,
    Consume,
    Close,
}

pub(crate) fn shows_about_entry(target_os: &str) -> bool {
    target_os == "windows"
}

pub(crate) fn about_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub(crate) fn about_key_action(open: bool, key: &str) -> AboutKeyAction {
    if !open {
        AboutKeyAction::Ignore
    } else if key.eq_ignore_ascii_case("escape") {
        AboutKeyAction::Close
    } else {
        AboutKeyAction::Consume
    }
}

impl SkinApp {
    pub(crate) fn close_about(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.about_open = false;
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    pub(crate) fn render_about_entry(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .id("open-about")
            .role(Role::Button)
            .aria_label(l.about_dialog_aria)
            .px_2()
            .py_1()
            .rounded(px(6.))
            .text_xs()
            .text_color(rgb(colors.muted))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(colors.hover)).text_color(rgb(colors.text)))
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .on_click(cx.listener(|this, _event, window, cx| {
                this.about_open = true;
                this.about_focus_handle.focus(window, cx);
                cx.notify();
                cx.stop_propagation();
            }))
            .child(l.about_open)
            .into_any_element()
    }

    pub(crate) fn render_about_modal(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let l = t();
        div()
            .id("about-overlay")
            .absolute()
            .inset_0()
            .occlude()
            .bg(gpui::black().opacity(0.32))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .child(
                div()
                    .id("about-dialog")
                    .role(Role::Dialog)
                    .aria_label(l.about_dialog_aria)
                    .track_focus(&self.about_focus_handle)
                    .tab_index(0)
                    .w(px(430.))
                    .max_w(px(430.))
                    .p_6()
                    .rounded(px(14.))
                    .border_1()
                    .border_color(rgb(colors.border))
                    .bg(rgb(colors.shell))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .text_color(rgb(colors.text))
                    .child(
                        div()
                            .id("about-heading")
                            .role(Role::Heading)
                            .w_full()
                            .text_center()
                            .text_size(px(20.))
                            .font_weight(FontWeight::BOLD)
                            .child(l.app_name),
                    )
                    .child(
                        div()
                            .w_full()
                            .text_center()
                            .text_sm()
                            .text_color(rgb(colors.muted))
                            .child(format!("{} {}", l.about_version, about_version())),
                    )
                    .child(
                        div()
                            .w_full()
                            .text_center()
                            .text_sm()
                            .line_height(px(22.))
                            .child(OPEN_SOURCE_NOTICE),
                    )
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(colors.muted))
                                    .child(l.about_github),
                            )
                            .child(
                                div()
                                    .id("about-repository")
                                    .role(Role::Link)
                                    .aria_label(OFFICIAL_REPOSITORY_URL)
                                    .tab_index(0)
                                    .text_sm()
                                    .text_color(rgb(colors.link))
                                    .cursor_pointer()
                                    .hover(|style| style.opacity(0.72))
                                    .on_click(|_event, _window, cx| {
                                        cx.open_url(OFFICIAL_REPOSITORY_URL);
                                        cx.stop_propagation();
                                    })
                                    .child(OFFICIAL_REPOSITORY_URL),
                            ),
                    )
                    .child(
                        div().flex().justify_end().child(
                            div()
                                .id("close-about")
                                .role(Role::Button)
                                .aria_label(l.about_close)
                                .tab_index(0)
                                .px_4()
                                .py_2()
                                .rounded(px(8.))
                                .bg(rgb(colors.control))
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(colors.hover)))
                                .on_click(cx.listener(|this, _event, window, cx| {
                                    this.close_about(window, cx);
                                    cx.stop_propagation();
                                }))
                                .child(l.about_close),
                        ),
                    ),
            )
            .into_any_element()
    }
}

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod i18n;
mod preview;
mod store;
mod ui;

use std::sync::{mpsc, Arc, Mutex};

#[cfg(target_os = "macos")]
use gpui::KeyBinding;
use gpui::{point, prelude::*, px, size, App, Bounds, QuitMode, WindowBounds, WindowOptions};
use gpui_platform::application;

use crate::app::actions;
#[cfg(target_os = "macos")]
use crate::app::actions::{HideApplication, HideOthers, QuitApplication};
use crate::app::platform;
use crate::app::SkinApp;
use crate::i18n::t;
use crate::ui::assets::Assets;
use crate::ui::constants::{
    MAIN_WINDOW_HEIGHT, MAIN_WINDOW_WIDTH, TRAFFIC_LIGHT_X, TRAFFIC_LIGHT_Y,
};

fn main_window_options(bounds: Bounds<gpui::Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(MAIN_WINDOW_WIDTH), px(MAIN_WINDOW_HEIGHT))),
        is_resizable: false,
        titlebar: Some(titlebar_options(std::env::consts::OS)),
        ..Default::default()
    }
}

fn titlebar_options(target_os: &str) -> gpui::TitlebarOptions {
    if target_os == "macos" {
        gpui::TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some(point(px(TRAFFIC_LIGHT_X), px(TRAFFIC_LIGHT_Y))),
        }
    } else {
        gpui::TitlebarOptions {
            title: None,
            appears_transparent: false,
            traffic_light_position: None,
        }
    }
}

fn main() {
    platform::init_logger();
    let args: Vec<String> = std::env::args().collect();
    let live_arg = args
        .windows(2)
        .find(|pair| pair[0] == "--live")
        .map(|pair| pair[1].clone());
    let url_buffer: Arc<Mutex<Vec<String>>> = Arc::default();
    let url_buffer_for_callback = url_buffer.clone();
    let app = application()
        .with_assets(Assets)
        .with_quit_mode(QuitMode::LastWindowClosed);
    app.on_open_urls(move |urls| {
        if let Ok(mut buf) = url_buffer_for_callback.lock() {
            buf.extend(urls);
        }
    });
    app.run(move |cx: &mut App| {
        #[cfg(target_os = "macos")]
        platform::set_development_icon();
        #[cfg(target_os = "macos")]
        {
            cx.bind_keys([
                KeyBinding::new("cmd-h", HideApplication, None),
                KeyBinding::new("cmd-alt-h", HideOthers, None),
                KeyBinding::new("cmd-q", QuitApplication, None),
            ]);
            cx.set_menus([actions::application_menu()]);
        }
        cx.on_action(actions::show_about);
        cx.on_action(actions::hide_application);
        cx.on_action(actions::hide_others);
        cx.on_action(actions::show_all);
        cx.on_action(actions::quit_application);
        let (tx, rx) = mpsc::channel();
        let bounds = Bounds::centered(
            None,
            size(px(MAIN_WINDOW_WIDTH), px(MAIN_WINDOW_HEIGHT)),
            cx,
        );
        let url_buf = url_buffer.clone();
        let window = cx
            .open_window(main_window_options(bounds), move |window, cx| {
                cx.new(move |cx| SkinApp::new(tx, rx, url_buf, window, cx))
            })
            .unwrap();
        if let Some(id) = live_arg {
            let _ = window.update(cx, move |view, _window, cx| {
                if let Some(index) = view.themes.iter().position(|row| row.theme.id == id) {
                    view.selected = index;
                    view.apply_selected(cx);
                } else {
                    view.message = t().theme_unavailable.into();
                    cx.notify();
                }
            });
        }
        cx.activate(true);
    });
}

#[cfg(test)]
mod ui_regression_tests;

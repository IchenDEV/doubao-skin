//! Native application menu and lifecycle actions.

use gpui::{actions, App, Menu, MenuItem, SystemMenuType};

use crate::i18n::t;

actions!(
    doubao_skin,
    [About, HideApplication, HideOthers, ShowAll, QuitApplication]
);

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn application_menu() -> Menu {
    let l = t();
    Menu::new(l.app_name).items([
        MenuItem::action(l.menu_about, About),
        MenuItem::separator(),
        MenuItem::os_submenu(l.menu_services, SystemMenuType::Services),
        MenuItem::separator(),
        MenuItem::action(l.menu_hide, HideApplication),
        MenuItem::action(l.menu_hide_others, HideOthers),
        MenuItem::action(l.menu_show_all, ShowAll),
        MenuItem::separator(),
        MenuItem::action(l.menu_quit, QuitApplication),
    ])
}

pub fn show_about(_: &About, _: &mut App) {
    show_about_panel();
}

pub fn hide_application(_: &HideApplication, cx: &mut App) {
    cx.hide();
}

pub fn hide_others(_: &HideOthers, cx: &mut App) {
    cx.hide_other_apps();
}

pub fn show_all(_: &ShowAll, cx: &mut App) {
    cx.unhide_other_apps();
}

pub fn quit_application(_: &QuitApplication, cx: &mut App) {
    cx.quit();
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
fn show_about_panel() {
    use cocoa::appkit::NSApp;
    use cocoa::base::nil;
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let _: () = msg_send![NSApp(), orderFrontStandardAboutPanel: nil];
    }
}

#[cfg(not(target_os = "macos"))]
fn show_about_panel() {}

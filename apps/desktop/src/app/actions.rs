//! Native application menu and lifecycle actions.

use gpui::{actions, App, Menu, MenuItem, SystemMenuType};

use crate::i18n::t;

pub const OFFICIAL_REPOSITORY_URL: &str = "https://github.com/IchenDEV/doubao-skin";
pub const OPEN_SOURCE_NOTICE: &str =
    "本软件开源，官方版本永久免费。\n\n如遇冒充官方收费，请勿购买。\n\n请从 GitHub 官方仓库核验并下载。";

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
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSRange, NSString};
    use objc::{class, msg_send, sel, sel_impl};

    #[link(name = "AppKit", kind = "framework")]
    extern "C" {
        static NSAboutPanelOptionCredits: id;
        static NSLinkAttributeName: id;
        static NSParagraphStyleAttributeName: id;
    }

    unsafe {
        let credits_text = format!("{OPEN_SOURCE_NOTICE}\n\n{OFFICIAL_REPOSITORY_URL}");
        let credits_string = NSString::alloc(nil).init_str(&credits_text);
        let repository_string = NSString::alloc(nil).init_str(OFFICIAL_REPOSITORY_URL);
        let repository_url: id = msg_send![class!(NSURL), URLWithString: repository_string];
        let credits: id = msg_send![class!(NSMutableAttributedString), alloc];
        let credits: id = msg_send![credits, initWithString: credits_string];
        let paragraph_style: id = msg_send![class!(NSMutableParagraphStyle), new];
        let center_alignment: isize = 1;
        let _: () = msg_send![paragraph_style, setAlignment: center_alignment];
        let full_range = NSRange::new(0, credits_text.encode_utf16().count() as _);
        let _: () = msg_send![credits,
            addAttribute: NSParagraphStyleAttributeName
            value: paragraph_style
            range: full_range
        ];
        let link_start = credits_text
            .strip_suffix(OFFICIAL_REPOSITORY_URL)
            .map(|prefix| prefix.encode_utf16().count())
            .unwrap_or_default();
        let link_range = NSRange::new(
            link_start as _,
            OFFICIAL_REPOSITORY_URL.encode_utf16().count() as _,
        );
        let _: () = msg_send![credits,
            addAttribute: NSLinkAttributeName
            value: repository_url
            range: link_range
        ];
        let options: id = msg_send![class!(NSDictionary),
            dictionaryWithObject: credits
            forKey: NSAboutPanelOptionCredits
        ];
        let _: () = msg_send![NSApp(), orderFrontStandardAboutPanelWithOptions: options];
        let _: () = msg_send![paragraph_style, release];
        let _: () = msg_send![credits, release];
        let _: () = msg_send![repository_string, release];
        let _: () = msg_send![credits_string, release];
    }
}

#[cfg(not(target_os = "macos"))]
fn show_about_panel() {}

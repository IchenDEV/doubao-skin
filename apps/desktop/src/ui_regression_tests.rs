use std::path::Path;

use gpui::{point, px, size, ImageSource, Resource, WindowBounds};

use skin_core::theme_package::{SupportDeclaration, SupportLevel, TargetSupport};
use skin_core::{live, theme};

use crate::app::actions::application_menu;
use crate::app::{
    initial_target, preview_identity, support_label, target_shortcut, theme_is_active,
    uses_short_compact_layout,
};
use crate::i18n::t;
use crate::preview::preview_rgba;
use crate::ui::assets::local_image_source;
use crate::ui::constants::*;

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
}

#[test]
fn opacity_preview_composites_the_same_nested_surfaces_users_see() {
    let profile = theme::surface_opacity_profile(0.8);
    assert_close(
        profile.preview_page,
        theme::composite_alpha(profile.page, profile.page),
    );
    assert_close(
        profile.preview_sidebar,
        theme::composite_alpha(profile.page, profile.sidebar),
    );
}

#[test]
fn preview_paint_multiplies_theme_alpha_by_layer_opacity() {
    let color = theme::PreviewColor {
        rgb: 0xbd9999,
        alpha: 0.16,
    };
    let painted = preview_rgba(color, 0.6);
    assert_eq!(u32::from(painted) >> 8, 0xbd9999);
    assert_close(painted.a, 0.096);
}

#[test]
fn main_window_is_fixed_at_the_approved_size() {
    let bounds = gpui::Bounds::new(point(px(20.), px(30.)), size(px(1120.), px(720.)));
    let options = super::main_window_options(bounds);
    assert_eq!(options.window_bounds, Some(WindowBounds::Windowed(bounds)));
    assert_eq!(options.window_min_size, Some(size(px(1120.), px(720.))));
    assert!(!options.is_resizable);
    assert!(options.is_movable);
    assert!(options.is_minimizable);
}

#[test]
fn windows_keeps_the_native_titlebar_controls() {
    let windows = super::titlebar_options("windows");
    assert!(!windows.appears_transparent);
    assert_eq!(windows.title, None);
    assert_eq!(windows.traffic_light_position, None);

    let macos = super::titlebar_options("macos");
    assert!(macos.appears_transparent);
    assert!(macos.traffic_light_position.is_some());
}

#[test]
fn windows_theme_images_remain_filesystem_resources() {
    let source = local_image_source(Path::new(
        r"C:\Users\tester\Doubao-Skin\themes\gallery-whale-maid\preview.png",
    ));
    assert!(matches!(source, ImageSource::Resource(Resource::Path(_))));
}

#[test]
fn application_menu_contains_the_native_about_and_lifecycle_items() {
    let l = t();
    let menu = application_menu();
    assert_eq!(menu.name.as_ref(), l.app_name);
    let names = menu
        .items
        .iter()
        .filter_map(|item| match item {
            gpui::MenuItem::Action { name, .. } => Some(name.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            l.menu_about,
            l.menu_hide,
            l.menu_hide_others,
            l.menu_show_all,
            l.menu_quit
        ]
    );
    assert!(menu.items.iter().any(|item| matches!(
        item,
        gpui::MenuItem::SystemMenu(system)
            if system.menu_type == gpui::SystemMenuType::Services
    )));
}

#[test]
fn traffic_lights_leave_the_default_corner_and_align_with_custom_header() {
    assert_eq!(TRAFFIC_LIGHT_X, 14.0);
    assert_close(
        TRAFFIC_LIGHT_Y + TRAFFIC_LIGHT_DIAMETER / 2.0,
        HEADER_HEIGHT / 2.0,
    );
    assert_close(
        WINDOW_TITLE_X - (TRAFFIC_LIGHT_X + TRAFFIC_LIGHT_STEP * 2.0 + TRAFFIC_LIGHT_DIAMETER),
        WINDOW_TITLE_GAP,
    );
}

#[test]
fn preview_profile_uses_the_app_identity_instead_of_one_theme_name() {
    let l = t();
    assert_eq!(preview_identity(live::TargetApp::Doubao).0, l.target_doubao);
    assert_eq!(
        preview_identity(live::TargetApp::DoubaoWork).0,
        l.target_doubao_work
    );
    assert_eq!(
        preview_identity(live::TargetApp::WorkBuddy).0,
        l.target_workbuddy
    );
    assert_eq!(target_shortcut(live::TargetApp::WorkBuddy), "Command-3");
}

#[test]
fn support_badges_distinguish_target_capability() {
    assert_eq!(
        support_label(TargetSupport {
            level: SupportLevel::Tailored,
            declaration: SupportDeclaration::Explicit,
        }),
        "专属适配"
    );
    assert_eq!(
        support_label(TargetSupport {
            level: SupportLevel::Shared,
            declaration: SupportDeclaration::Explicit,
        }),
        "共享适配"
    );
    assert_eq!(
        support_label(TargetSupport {
            level: SupportLevel::Shared,
            declaration: SupportDeclaration::LegacyInferred,
        }),
        "兼容模式"
    );
}

#[test]
fn target_default_respects_installation_and_saved_preference() {
    assert_eq!(
        initial_target(Some("doubao"), true, true, true),
        live::TargetApp::Doubao
    );
    assert_eq!(
        initial_target(Some("unknown"), true, true, true),
        live::TargetApp::DoubaoWork
    );
    assert_eq!(
        initial_target(Some("doubao-work"), true, false, true),
        live::TargetApp::Doubao
    );
    assert_eq!(
        initial_target(Some("doubao"), false, true, true),
        live::TargetApp::DoubaoWork
    );
    assert_eq!(
        initial_target(Some("workbuddy"), true, true, true),
        live::TargetApp::WorkBuddy
    );
    assert_eq!(
        initial_target(None, false, false, true),
        live::TargetApp::WorkBuddy
    );
    assert_eq!(
        initial_target(None, false, false, false),
        live::TargetApp::DoubaoWork
    );
}

#[test]
fn active_theme_is_scoped_to_its_target() {
    assert!(theme_is_active(
        Some(live::TargetApp::Doubao),
        Some("violet-night"),
        live::TargetApp::Doubao,
        "violet-night"
    ));
    assert!(!theme_is_active(
        Some(live::TargetApp::DoubaoWork),
        Some("violet-night"),
        live::TargetApp::Doubao,
        "violet-night"
    ));
    assert!(theme_is_active(
        Some(live::TargetApp::WorkBuddy),
        Some("violet-night"),
        live::TargetApp::WorkBuddy,
        "violet-night"
    ));
}

#[test]
fn minimum_window_uses_the_short_compact_layout() {
    assert!(uses_short_compact_layout(true, px(560.)));
    assert!(!uses_short_compact_layout(true, px(720.)));
    assert!(!uses_short_compact_layout(false, px(560.)));
}

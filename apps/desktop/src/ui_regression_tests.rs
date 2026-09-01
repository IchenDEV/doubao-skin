use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use gpui::{point, px, size, ImageSource, Resource, WindowBounds};

use skin_core::theme_package::{SupportDeclaration, SupportLevel, TargetSupport};
use skin_core::{live, theme};

use crate::app::actions::application_menu;
use crate::app::helpers::target_shortcut_for_platform;
use crate::app::theme_sessions::{TargetSession, ThemeSessions};
use crate::app::{
    auto_theme::control_state, initial_target, platform::AutoThemeServiceStatus, preview_identity,
    support_label, target_shortcut, uses_short_compact_layout,
};
use crate::i18n::t;
use crate::preview::preview_rgba;
use crate::ui::assets::local_image_source;
use crate::ui::constants::*;
use crate::ui::{header_brand_padding, shows_auto_theme_controls};

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
fn windows_header_does_not_reserve_macos_traffic_light_space() {
    assert_close(header_brand_padding("windows", true), 0.0);
    assert_close(header_brand_padding("windows", false), 0.0);
    assert_close(header_brand_padding("macos", true), WINDOW_TITLE_X - 16.0);
    assert_close(header_brand_padding("macos", false), WINDOW_TITLE_X - 24.0);
}

#[test]
fn automatic_theme_controls_are_visible_on_supported_desktop_platforms() {
    assert!(shows_auto_theme_controls("macos"));
    assert!(shows_auto_theme_controls("windows"));
    assert!(!shows_auto_theme_controls("linux"));
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
    assert_eq!(
        target_shortcut(live::TargetApp::WorkBuddy),
        target_shortcut_for_platform(std::env::consts::OS, live::TargetApp::WorkBuddy)
    );
    assert_eq!(
        target_shortcut_for_platform("windows", live::TargetApp::WorkBuddy),
        "Ctrl-3"
    );
    assert_eq!(
        target_shortcut_for_platform("macos", live::TargetApp::WorkBuddy),
        "Command-3"
    );
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
fn applying_to_a_second_target_preserves_the_first_target_session() {
    let mut sessions = ThemeSessions::default();
    let workbuddy_stop = Arc::new(AtomicBool::new(false));
    let doubao_stop = Arc::new(AtomicBool::new(false));

    sessions.begin_applying(
        live::TargetApp::WorkBuddy,
        TargetSession::for_test("gallery-whale-maid", Some(0.68), 1, workbuddy_stop.clone()),
    );
    assert!(sessions.mark_applied(live::TargetApp::WorkBuddy, 1));
    sessions.begin_applying(
        live::TargetApp::Doubao,
        TargetSession::for_test("pure-dark", None, 2, doubao_stop),
    );
    assert!(sessions.mark_applied(live::TargetApp::Doubao, 2));

    assert!(sessions.is_active(live::TargetApp::WorkBuddy, "gallery-whale-maid", Some(0.68),));
    assert!(sessions.is_active(live::TargetApp::Doubao, "pure-dark", None));
    assert!(
        !workbuddy_stop.load(Ordering::Relaxed),
        "applying to 豆包 must not stop the WorkBuddy watcher"
    );
}

#[test]
fn replacing_one_target_stops_only_its_previous_generation() {
    let mut sessions = ThemeSessions::default();
    let old_workbuddy_stop = Arc::new(AtomicBool::new(false));
    let current_workbuddy_stop = Arc::new(AtomicBool::new(false));
    let doubao_stop = Arc::new(AtomicBool::new(false));

    sessions.begin_applying(
        live::TargetApp::WorkBuddy,
        TargetSession::for_test(
            "gallery-whale-maid",
            Some(0.68),
            1,
            old_workbuddy_stop.clone(),
        ),
    );
    assert!(sessions.mark_applied(live::TargetApp::WorkBuddy, 1));
    sessions.begin_applying(
        live::TargetApp::Doubao,
        TargetSession::for_test("pure-dark", None, 2, doubao_stop.clone()),
    );
    assert!(sessions.mark_applied(live::TargetApp::Doubao, 2));
    sessions.begin_applying(
        live::TargetApp::WorkBuddy,
        TargetSession::for_test("qq-light-blue", None, 3, current_workbuddy_stop.clone()),
    );
    assert!(sessions.mark_applied(live::TargetApp::WorkBuddy, 3));

    assert!(old_workbuddy_stop.load(Ordering::Relaxed));
    assert!(!current_workbuddy_stop.load(Ordering::Relaxed));
    assert!(!doubao_stop.load(Ordering::Relaxed));
    assert!(!sessions.complete_if_generation(live::TargetApp::WorkBuddy, 1));
    assert!(sessions.is_active(live::TargetApp::WorkBuddy, "qq-light-blue", None));
    assert!(sessions.is_active(live::TargetApp::Doubao, "pure-dark", None));
}

#[test]
fn restoring_one_target_blocks_only_that_target_until_completion() {
    let mut sessions = ThemeSessions::default();
    let workbuddy_stop = Arc::new(AtomicBool::new(false));

    sessions.begin_applying(
        live::TargetApp::WorkBuddy,
        TargetSession::for_test("gallery-whale-maid", Some(0.68), 1, workbuddy_stop.clone()),
    );
    assert!(sessions.mark_applied(live::TargetApp::WorkBuddy, 1));

    let previous = sessions.begin_restoring(live::TargetApp::WorkBuddy, 2);
    assert!(previous.is_some());
    assert!(workbuddy_stop.load(Ordering::Relaxed));
    assert!(sessions.is_busy(live::TargetApp::WorkBuddy));
    assert!(!sessions.is_busy(live::TargetApp::Doubao));
    assert!(!sessions.is_active(live::TargetApp::WorkBuddy, "gallery-whale-maid", Some(0.68),));
}

#[test]
fn completion_generation_is_scoped_to_its_target() {
    let mut sessions = ThemeSessions::default();

    sessions.begin_applying(
        live::TargetApp::WorkBuddy,
        TargetSession::for_test(
            "gallery-whale-maid",
            Some(0.68),
            1,
            Arc::new(AtomicBool::new(false)),
        ),
    );
    assert!(sessions.mark_applied(live::TargetApp::WorkBuddy, 1));
    sessions.begin_applying(
        live::TargetApp::Doubao,
        TargetSession::for_test("pure-dark", None, 2, Arc::new(AtomicBool::new(false))),
    );
    assert!(sessions.mark_applied(live::TargetApp::Doubao, 2));

    assert!(sessions.complete_if_generation(live::TargetApp::WorkBuddy, 1));
    assert!(!sessions.is_active(live::TargetApp::WorkBuddy, "gallery-whale-maid", Some(0.68),));
    assert!(sessions.is_active(live::TargetApp::Doubao, "pure-dark", None));
}

#[test]
fn minimum_window_uses_the_short_compact_layout() {
    assert!(uses_short_compact_layout(true, px(560.)));
    assert!(!uses_short_compact_layout(true, px(720.)));
    assert!(!uses_short_compact_layout(false, px(560.)));
}

#[test]
fn automatic_theme_controls_have_exactly_one_dependent_switch() {
    let mut settings = skin_core::auto_theme::AutoThemeSettings::default();
    settings.set_last_applied(
        skin_core::auto_theme::LastApplied::new(live::TargetApp::DoubaoWork, "pure-dark", None)
            .unwrap(),
    );
    settings.set_keep_requested(true);
    let state = control_state(&settings, AutoThemeServiceStatus::Enabled, false);
    assert!(state.keep_enabled);
    assert!(state.login_enabled);
    assert!(state.keep_requested);
    assert!(!state.login_requested);

    settings.set_keep_requested(false);
    settings.set_open_at_login(true);
    let state = control_state(&settings, AutoThemeServiceStatus::Enabled, false);
    assert!(!state.login_enabled);
    assert!(!state.login_requested);
}

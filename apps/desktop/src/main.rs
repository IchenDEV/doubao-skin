//! A novice-friendly theme picker: choose, preview, apply, restore.

use std::borrow::Cow;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use gpui::{
    actions, div, img, point, prelude::*, px, rgb, size, svg, App, AssetSource, Bounds, Context,
    ExternalPaths, FocusHandle, FontWeight, KeyBinding, KeyDownEvent, Menu, MenuItem, MouseButton,
    MouseDownEvent, MouseMoveEvent, ObjectFit, PathPromptOptions, Pixels, QuitMode, Rgba, Role,
    SharedString, SystemMenuType, TitlebarOptions, Window, WindowAppearance, WindowBounds,
    WindowOptions,
};
use gpui_platform::application;

use skin_core::theme_package::{SupportDeclaration, SupportLevel, TargetSupport};
use skin_core::{live, theme};

const MAX_INTERNAL_LOGS: usize = 300;
const HEADER_HEIGHT: f32 = 72.0;
const TRAFFIC_LIGHT_X: f32 = 14.0;
const TRAFFIC_LIGHT_DIAMETER: f32 = 14.0;
const TRAFFIC_LIGHT_STEP: f32 = 20.0;
const TRAFFIC_LIGHT_Y: f32 = (HEADER_HEIGHT - TRAFFIC_LIGHT_DIAMETER) / 2.0;
const WINDOW_TITLE_GAP: f32 = 24.0;
const WINDOW_TITLE_X: f32 =
    TRAFFIC_LIGHT_X + TRAFFIC_LIGHT_STEP * 2.0 + TRAFFIC_LIGHT_DIAMETER + WINDOW_TITLE_GAP;
const PREVIEW_FRAME_RADIUS: f32 = 12.0;
const PREVIEW_CONTENT_RADIUS: f32 = PREVIEW_FRAME_RADIUS - 1.0;
const MIN_SURFACE_OPACITY: f32 = 0.35;
const SURFACE_OPACITY_RANGE: f32 = 0.65;
const OPACITY_TRACK_WIDTH: f32 = 180.0;
const MAIN_WINDOW_WIDTH: f32 = 1120.0;
const MAIN_WINDOW_HEIGHT: f32 = 720.0;
const SEARCH_ICON_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none"><circle cx="7" cy="7" r="4.25" stroke="currentColor" stroke-width="1.5"/><path d="M10.25 10.25 14 14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>"##;
const INSTALL_ICON_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 18 18" fill="none"><path d="M3.25 6.25 9 3l5.75 3.25v6.5L9 16l-5.75-3.25v-6.5Z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/><path d="M9 3v6.4m0 0 2.15-2.1M9 9.4 6.85 7.3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;
const REFRESH_ICON_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none"><path d="M13.1 6A5.3 5.3 0 1 0 13 10.3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M10.7 3.6h2.7v2.7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;

actions!(
    doubao_skin,
    [About, HideApplication, HideOthers, ShowAll, QuitApplication]
);

fn application_menu() -> Menu {
    Menu::new("豆皮").items([
        MenuItem::action("关于豆皮", About),
        MenuItem::separator(),
        MenuItem::os_submenu("服务", SystemMenuType::Services),
        MenuItem::separator(),
        MenuItem::action("隐藏豆皮", HideApplication),
        MenuItem::action("隐藏其他", HideOthers),
        MenuItem::action("全部显示", ShowAll),
        MenuItem::separator(),
        MenuItem::action("退出豆皮", QuitApplication),
    ])
}

fn main_window_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(MAIN_WINDOW_WIDTH), px(MAIN_WINDOW_HEIGHT))),
        is_resizable: false,
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some(point(px(TRAFFIC_LIGHT_X), px(TRAFFIC_LIGHT_Y))),
        }),
        ..Default::default()
    }
}

fn uses_short_compact_layout(compact: bool, height: Pixels) -> bool {
    compact && height <= px(600.)
}

fn target_preference_path() -> PathBuf {
    theme::user_themes_dir()
        .parent()
        .map(|directory| directory.join("target-app"))
        .unwrap_or_else(|| PathBuf::from("target-app"))
}

fn read_target_preference() -> Option<String> {
    std::fs::read_to_string(target_preference_path())
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn save_target_preference(target: live::TargetApp) {
    let path = target_preference_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, target.id());
}

fn initial_target(
    saved: Option<&str>,
    doubao_installed: bool,
    work_installed: bool,
    workbuddy_installed: bool,
) -> live::TargetApp {
    if let Some(saved) = saved.and_then(live::TargetApp::from_id) {
        let installed = match saved {
            live::TargetApp::Doubao => doubao_installed,
            live::TargetApp::DoubaoWork => work_installed,
            live::TargetApp::WorkBuddy => workbuddy_installed,
        };
        if installed {
            return saved;
        }
    }
    if work_installed {
        live::TargetApp::DoubaoWork
    } else if doubao_installed {
        live::TargetApp::Doubao
    } else if workbuddy_installed {
        live::TargetApp::WorkBuddy
    } else {
        live::TargetApp::DoubaoWork
    }
}

fn preview_identity(target: live::TargetApp) -> (&'static str, &'static str) {
    match target {
        live::TargetApp::Doubao => ("豆包", "有什么我能帮你的？"),
        live::TargetApp::DoubaoWork => ("豆包工作", "今天有什么工作要处理？"),
        live::TargetApp::WorkBuddy => ("WorkBuddy", "今天想一起完成什么？"),
    }
}

fn target_shortcut(target: live::TargetApp) -> &'static str {
    match target {
        live::TargetApp::Doubao => "Command-1",
        live::TargetApp::DoubaoWork => "Command-2",
        live::TargetApp::WorkBuddy => "Command-3",
    }
}

fn support_label(support: TargetSupport) -> &'static str {
    if !support.is_supported() {
        return "不支持";
    }
    if support.declaration == SupportDeclaration::LegacyInferred {
        return "兼容模式";
    }
    match support.level {
        SupportLevel::Tailored => "专属适配",
        SupportLevel::Shared => "共享适配",
        SupportLevel::Unsupported => "不支持",
    }
}

fn theme_is_active(
    active_target: Option<live::TargetApp>,
    active_theme: Option<&str>,
    selected_target: live::TargetApp,
    theme_id: &str,
) -> bool {
    active_target == Some(selected_target) && active_theme == Some(theme_id)
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct UiPalette {
    shell: u32,
    sidebar: u32,
    control: u32,
    text: u32,
    muted: u32,
    border: u32,
    hover: u32,
    danger: u32,
    focus_border: u32,
    segmented_track: u32,
    segmented_selected: u32,
    drop_border: u32,
    drop_hover: u32,
    drop_accent: u32,
    link: u32,
    preview_placeholder: u32,
    installed_control: u32,
    card_hover_border: u32,
    slider_accent: u32,
}

impl UiPalette {
    fn for_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self {
                shell: 0x1c1c1e,
                sidebar: 0x242426,
                control: 0x2c2c2e,
                text: 0xf2f2f7,
                muted: 0xa7a7ad,
                border: 0x3a3a3c,
                hover: 0x363638,
                danger: 0xff7b79,
                focus_border: 0xc88d70,
                segmented_track: 0x29292b,
                segmented_selected: 0x48484a,
                drop_border: 0x555558,
                drop_hover: 0x3a2c28,
                drop_accent: 0xe0926e,
                link: 0xe0926e,
                preview_placeholder: 0x262628,
                installed_control: 0x3a3a3c,
                card_hover_border: 0x66666a,
                slider_accent: 0xc58b70,
            },
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self {
                shell: 0xf8f8f7,
                sidebar: 0xf1f2f3,
                control: 0xffffff,
                text: 0x242321,
                muted: 0x74726e,
                border: 0xdadbdc,
                hover: 0xe7e8e9,
                danger: 0xa84b4b,
                focus_border: 0x9c7b6b,
                segmented_track: 0xeeeeed,
                segmented_selected: 0xffffff,
                drop_border: 0xb9b9b7,
                drop_hover: 0xf1e8e3,
                drop_accent: 0xa64e24,
                link: 0x9d4a24,
                preview_placeholder: 0xf0f1f2,
                installed_control: 0xeeeeed,
                card_hover_border: 0xbcbdbc,
                slider_accent: 0x8f6b5b,
            },
        }
    }
}

fn preview_rgba(color: theme::PreviewColor, layer_opacity: f32) -> Rgba {
    let mut painted = rgb(color.rgb);
    painted.a = color.alpha * layer_opacity.clamp(0.0, 1.0);
    painted
}

fn preview_icon(
    path: Option<&PathBuf>,
    icon_size: f32,
    color: theme::PreviewColor,
) -> gpui::AnyElement {
    if let Some(path) = path {
        let path_string = SharedString::from(path.to_string_lossy().into_owned());
        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
        {
            svg()
                .path(path_string)
                .size(px(icon_size))
                .text_color(preview_rgba(color, 1.0))
                .into_any_element()
        } else {
            img(path_string)
                .size(px(icon_size))
                .object_fit(ObjectFit::Contain)
                .into_any_element()
        }
    } else {
        div()
            .size(px(icon_size))
            .rounded(px(icon_size * 0.32))
            .bg(preview_rgba(color, 0.18))
            .into_any_element()
    }
}

fn preview_main_icon(
    path: Option<&PathBuf>,
    icon_size: f32,
    accent: theme::PreviewColor,
    text_color: theme::PreviewColor,
) -> gpui::AnyElement {
    if let Some(path) = path {
        let path_string = SharedString::from(path.to_string_lossy().into_owned());
        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
        {
            return svg()
                .path(path_string)
                .size(px(icon_size))
                .text_color(preview_rgba(accent, 1.0))
                .into_any_element();
        }
        return img(path_string)
            .size(px(icon_size))
            .object_fit(ObjectFit::Contain)
            .into_any_element();
    }
    div()
        .size(px(icon_size))
        .rounded(px(icon_size * 0.34))
        .bg(preview_rgba(accent, 1.0))
        .border_1()
        .border_color(rgb(0xffffff).opacity(0.82))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(px(icon_size * 0.68))
                .rounded_full()
                .bg(rgb(0xfff8ed))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px((icon_size * 0.23).max(7.)))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(preview_rgba(text_color, 1.0))
                .child("•ᴗ•"),
        )
        .into_any_element()
}

fn preview_nav_item(
    label: &'static str,
    path: Option<&PathBuf>,
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

fn preview_action_item(
    label: &'static str,
    path: Option<&PathBuf>,
    icon_color: theme::PreviewColor,
    text_color: theme::PreviewColor,
    background_color: theme::PreviewColor,
    background_opacity: f32,
    geometry: (f32, f32),
) -> gpui::AnyElement {
    let (row_height, icon_size) = geometry;
    div()
        .w_full()
        .h(px(row_height))
        .px_2()
        .flex()
        .items_center()
        .gap_2()
        .rounded(px(7.))
        .bg(preview_rgba(background_color, background_opacity))
        .child(preview_icon(path, icon_size, icon_color))
        .child(
            div()
                .text_size(px(10.))
                .text_color(preview_rgba(text_color, 0.9))
                .child(label),
        )
        .into_any_element()
}

struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let embedded = match path {
            "icons/search.svg" => Some(SEARCH_ICON_SVG),
            "icons/install.svg" => Some(INSTALL_ICON_SVG),
            "icons/refresh.svg" => Some(REFRESH_ICON_SVG),
            _ => None,
        };
        if let Some(bytes) = embedded {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        Ok(Some(std::fs::read(path)?.into()))
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| {
                Some(SharedString::from(
                    entry.ok()?.path().to_string_lossy().into_owned(),
                ))
            })
            .collect())
    }
}

enum Msg {
    Log(String),
    Applied(u64),
    Done {
        generation: Option<u64>,
        ok: bool,
        restoring: bool,
    },
    StoreLoaded(Result<Vec<StoreRow>, String>),
    InstallStarted,
    Installed {
        ids: Vec<String>,
        error: Option<String>,
        open_library: bool,
    },
    OpenUrl(String),
}

struct ThemeRow {
    theme: theme::Theme,
    preview: theme::PreviewStyle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceView {
    Library,
    Store,
}

struct StoreRow {
    theme: theme::StoreTheme,
    preview: Option<PathBuf>,
}

struct SkinApp {
    colors: UiPalette,
    tx: mpsc::Sender<Msg>,
    themes: Vec<ThemeRow>,
    source_view: SourceView,
    store_rows: Vec<StoreRow>,
    store_loading: bool,
    store_error: Option<String>,
    installing_package: bool,
    installing_store_theme: Option<String>,
    selected: usize,
    store_selected: usize,
    query: String,
    search_active: bool,
    internal_logs: VecDeque<String>,
    message: String,
    applying: bool,
    restart_confirmation_target: Option<live::TargetApp>,
    selected_target: live::TargetApp,
    active_target: Option<live::TargetApp>,
    active_theme: Option<String>,
    active_surface_opacity: Option<f32>,
    surface_opacity: f32,
    opacity_drag_start: Option<(Pixels, f32)>,
    live_stop: Option<Arc<AtomicBool>>,
    live_thread: Option<std::thread::JoinHandle<()>>,
    generation: u64,
    focus_handle: FocusHandle,
    url_buffer: Arc<Mutex<Vec<String>>>,
}

impl SkinApp {
    fn new(
        tx: mpsc::Sender<Msg>,
        rx: mpsc::Receiver<Msg>,
        url_buffer: Arc<Mutex<Vec<String>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let colors = UiPalette::for_appearance(window.appearance());
        cx.observe_window_appearance(window, |this, window, cx| {
            let colors = UiPalette::for_appearance(window.appearance());
            if this.colors != colors {
                this.colors = colors;
                cx.notify();
            }
        })
        .detach();
        let selected_target = initial_target(
            read_target_preference().as_deref(),
            live::TargetApp::Doubao.is_installed(),
            live::TargetApp::DoubaoWork.is_installed(),
            live::TargetApp::WorkBuddy.is_installed(),
        );
        let themes: Vec<ThemeRow> = theme::list_installed()
            .into_iter()
            .map(|theme| ThemeRow {
                preview: theme.preview_style_for(selected_target),
                theme,
            })
            .collect();
        let surface_opacity = themes
            .first()
            .map(|row| row.preview.surface_opacity)
            .unwrap_or(1.0);
        let url_buf = url_buffer.clone();
        cx.spawn(async move |this, cx| loop {
            while let Ok(msg) = rx.try_recv() {
                if this
                    .update(cx, |this, cx| {
                        this.handle_msg(msg);
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
            }
            let urls: Vec<String> = url_buf
                .lock()
                .ok()
                .map(|mut buf| buf.drain(..).collect())
                .unwrap_or_default();
            for url in urls {
                if this
                    .update(cx, |this, cx| {
                        this.handle_open_url(&url, cx);
                        cx.notify();
                    })
                    .is_err()
                {
                    return;
                }
            }
            cx.background_executor()
                .timer(Duration::from_millis(120))
                .await;
        })
        .detach();
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        Self {
            colors,
            tx,
            themes,
            source_view: SourceView::Library,
            store_rows: Vec::new(),
            store_loading: false,
            store_error: None,
            installing_package: false,
            installing_store_theme: None,
            selected: 0,
            store_selected: 0,
            query: String::new(),
            search_active: false,
            internal_logs: VecDeque::new(),
            message: String::new(),
            applying: false,
            restart_confirmation_target: None,
            selected_target,
            active_target: None,
            active_theme: None,
            active_surface_opacity: None,
            surface_opacity,
            opacity_drag_start: None,
            live_stop: None,
            live_thread: None,
            generation: 0,
            focus_handle,
            url_buffer,
        }
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;

        if modifiers.platform && key.eq_ignore_ascii_case("f") {
            self.search_active = true;
            self.focus_handle.focus(window, cx);
            cx.notify();
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && key.eq_ignore_ascii_case("o") {
            self.choose_package(window, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && key == "1" {
            self.switch_target(live::TargetApp::Doubao, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && key == "2" {
            self.switch_target(live::TargetApp::DoubaoWork, cx);
            cx.stop_propagation();
            return;
        }
        if modifiers.platform && key == "3" {
            self.switch_target(live::TargetApp::WorkBuddy, cx);
            cx.stop_propagation();
            return;
        }

        if self.search_active {
            match key {
                "escape" => {
                    self.search_active = false;
                    cx.notify();
                }
                "backspace" => {
                    self.query.pop();
                    if self.source_view == SourceView::Library {
                        self.ensure_selected_match();
                    }
                    cx.notify();
                }
                "up" if self.source_view == SourceView::Library => self.select_filtered(-1, cx),
                "down" if self.source_view == SourceView::Library => self.select_filtered(1, cx),
                "enter" | "return" if self.source_view == SourceView::Library => {
                    self.apply_selected(cx)
                }
                "tab" => {
                    self.search_active = false;
                    cx.notify();
                }
                _ if !modifiers.platform && !modifiers.control && !modifiers.function => {
                    if let Some(text) = event.keystroke.key_char.as_deref() {
                        if !text.chars().any(char::is_control) {
                            self.query.push_str(text);
                            if self.source_view == SourceView::Library {
                                self.ensure_selected_match();
                            }
                            cx.notify();
                        }
                    }
                }
                _ => return,
            }
            cx.stop_propagation();
            return;
        }

        if self.source_view != SourceView::Library {
            return;
        }
        match key {
            "up" | "left" => self.select_filtered(-1, cx),
            "down" | "right" => self.select_filtered(1, cx),
            "enter" | "return" => self.apply_selected(cx),
            _ => return,
        }
        cx.stop_propagation();
    }

    fn filtered_indices(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        self.themes
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                (row.theme.supports_target(self.selected_target)
                    && (query.is_empty()
                        || row.theme.name.to_lowercase().contains(&query)
                        || row.theme.id.to_lowercase().contains(&query)
                        || row.theme.description.to_lowercase().contains(&query)
                        || row.theme.author.to_lowercase().contains(&query)
                        || row
                            .theme
                            .store_category
                            .as_deref()
                            .is_some_and(|category| category.to_lowercase().contains(&query))
                        || row
                            .theme
                            .store_tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query))))
                .then_some(index)
            })
            .collect()
    }

    fn filtered_store_indices(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        self.store_rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                (row.theme.supports_target(self.selected_target)
                    && (query.is_empty()
                        || row.theme.name.to_lowercase().contains(&query)
                        || row.theme.id.to_lowercase().contains(&query)
                        || row.theme.description.to_lowercase().contains(&query)
                        || row.theme.author.to_lowercase().contains(&query)
                        || row.theme.category.to_lowercase().contains(&query)
                        || row
                            .theme
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query))))
                .then_some(index)
            })
            .collect()
    }

    fn ensure_selected_match(&mut self) {
        let indices = self.filtered_indices();
        if indices.contains(&self.selected) {
            return;
        }
        if let Some(index) = indices.first().copied() {
            self.selected = index;
            self.surface_opacity = self.themes[index].preview.surface_opacity;
            self.restart_confirmation_target = None;
            self.message.clear();
        }
    }

    fn ensure_store_selected_match(&mut self) {
        let indices = self.filtered_store_indices();
        if !indices.contains(&self.store_selected) {
            self.store_selected = indices.first().copied().unwrap_or(0);
        }
    }

    fn select_filtered(&mut self, delta: isize, cx: &mut Context<Self>) {
        let indices = self.filtered_indices();
        if indices.is_empty() {
            return;
        }
        let position = indices
            .iter()
            .position(|index| *index == self.selected)
            .unwrap_or(0);
        let next = if delta < 0 {
            position.saturating_sub(1)
        } else {
            (position + 1).min(indices.len() - 1)
        };
        self.selected = indices[next];
        self.surface_opacity = self.themes[self.selected].preview.surface_opacity;
        self.restart_confirmation_target = None;
        self.message.clear();
        cx.notify();
    }

    fn handle_msg(&mut self, msg: Msg) {
        match msg {
            Msg::Log(line) => {
                self.internal_logs.push_front(line);
                self.internal_logs.truncate(MAX_INTERNAL_LOGS);
            }
            Msg::Applied(generation) if generation == self.generation => {
                self.applying = false;
                self.restart_confirmation_target = None;
                self.message = "已应用".into();
            }
            Msg::Applied(_) => {}
            Msg::Done {
                generation,
                ok,
                restoring,
            } => {
                if generation.is_none() || generation == Some(self.generation) {
                    self.applying = false;
                    if restoring && ok {
                        self.message = "已恢复默认".into();
                    } else if ok && self.active_target == Some(live::TargetApp::WorkBuddy) {
                        self.message = "WorkBuddy 已退出，主题监听已停止".into();
                        self.active_target = None;
                        self.active_theme = None;
                        self.active_surface_opacity = None;
                        self.live_stop = None;
                    } else if !ok {
                        self.message = "应用失败，请再试一次".into();
                        self.active_target = None;
                        self.active_theme = None;
                        self.active_surface_opacity = None;
                        self.live_stop = None;
                    }
                }
            }
            Msg::StoreLoaded(result) => {
                self.store_loading = false;
                match result {
                    Ok(rows) => {
                        self.store_rows = rows;
                        self.store_error = None;
                        if self.store_selected >= self.store_rows.len() {
                            self.store_selected = 0;
                        }
                    }
                    Err(error) => self.store_error = Some(error),
                }
            }
            Msg::InstallStarted => {
                self.installing_package = true;
                self.message = "正在安装主题…".into();
            }
            Msg::Installed {
                ids,
                error,
                open_library,
            } => {
                self.installing_package = false;
                self.installing_store_theme = None;
                let selected = ids.last().cloned();
                self.reload_themes(selected.as_deref());
                if open_library && !ids.is_empty() {
                    self.source_view = SourceView::Library;
                    self.query.clear();
                }
                self.message = if let Some(error) = error {
                    if ids.is_empty() {
                        format!("安装失败：{error}")
                    } else {
                        format!("已安装 {} 个主题；{error}", ids.len())
                    }
                } else if ids.len() == 1 {
                    "主题已安装".into()
                } else {
                    format!("已安装 {} 个主题", ids.len())
                };
            }
            Msg::OpenUrl(url) => {
                if let Ok(mut buf) = self.url_buffer.lock() {
                    buf.push(url);
                }
            }
        }
    }

    fn handle_open_url(&mut self, url: &str, cx: &mut Context<Self>) {
        let theme_id = url
            .strip_prefix("doubao-skin://apply/")
            .or_else(|| url.strip_prefix("doubao-skin://theme/"))
            .map(|id| id.trim_end_matches('/'));
        let Some(theme_id) = theme_id else {
            return;
        };
        if let Some(index) = self.themes.iter().position(|row| row.theme.id == theme_id) {
            self.source_view = SourceView::Library;
            self.selected = index;
            self.query.clear();
            self.apply_selected(cx);
            return;
        }
        if let Some(index) = self
            .store_rows
            .iter()
            .position(|row| row.theme.id == theme_id)
        {
            self.source_view = SourceView::Store;
            self.store_selected = index;
            self.query.clear();
            self.install_store_theme(index, cx);
            return;
        }
        let id_owned = theme_id.to_string();
        let tx = self.tx.clone();
        self.message = format!("正在查找主题「{id_owned}」…");
        std::thread::spawn(move || {
            let catalog_url = theme::theme_store_url();
            match theme::fetch_store_catalog(&catalog_url) {
                Ok(catalog) => {
                    let _ = tx.send(Msg::StoreLoaded(Ok(catalog
                        .themes
                        .into_iter()
                        .map(|store_theme| {
                            let cache_dir = theme::theme_store_cache_dir();
                            let preview =
                                theme::cache_store_preview(&catalog_url, &store_theme, &cache_dir)
                                    .ok()
                                    .flatten();
                            StoreRow {
                                theme: store_theme,
                                preview,
                            }
                        })
                        .collect())));
                    let _ = tx.send(Msg::OpenUrl(format!("doubao-skin://apply/{id_owned}")));
                }
                Err(error) => {
                    let _ = tx.send(Msg::StoreLoaded(Err(error)));
                }
            }
        });
        cx.notify();
    }

    fn reload_themes(&mut self, preferred_id: Option<&str>) {
        let selected_id = preferred_id.map(ToOwned::to_owned).or_else(|| {
            self.themes
                .get(self.selected)
                .map(|row| row.theme.id.clone())
        });
        self.themes = theme::list_installed()
            .into_iter()
            .map(|theme| ThemeRow {
                preview: theme.preview_style_for(self.selected_target),
                theme,
            })
            .collect();
        self.selected = selected_id
            .as_deref()
            .and_then(|id| self.themes.iter().position(|row| row.theme.id == id))
            .unwrap_or(0);
        self.surface_opacity = self
            .themes
            .get(self.selected)
            .map(|row| row.preview.surface_opacity)
            .unwrap_or(1.0);
    }

    fn refresh_target_previews(&mut self) {
        for row in &mut self.themes {
            row.preview = row.theme.preview_style_for(self.selected_target);
        }
    }

    fn switch_source(&mut self, source: SourceView, cx: &mut Context<Self>) {
        if self.source_view == source {
            return;
        }
        self.source_view = source;
        self.query.clear();
        self.search_active = false;
        if source == SourceView::Store && self.store_rows.is_empty() && !self.store_loading {
            self.load_store(cx);
        }
        cx.notify();
    }

    fn load_store(&mut self, cx: &mut Context<Self>) {
        if self.store_loading {
            return;
        }
        self.store_loading = true;
        self.store_error = None;
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let catalog_url = theme::theme_store_url();
            let result = theme::fetch_store_catalog(&catalog_url).map(|catalog| {
                let cache_dir = theme::theme_store_cache_dir();
                catalog
                    .themes
                    .into_iter()
                    .map(|store_theme| {
                        let preview =
                            theme::cache_store_preview(&catalog_url, &store_theme, &cache_dir)
                                .ok()
                                .flatten();
                        StoreRow {
                            theme: store_theme,
                            preview,
                        }
                    })
                    .collect()
            });
            let _ = tx.send(Msg::StoreLoaded(result));
        });
        cx.notify();
    }

    fn choose_package(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.installing_package {
            return;
        }
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("安装主题".into()),
        });
        let tx = self.tx.clone();
        window
            .spawn(cx, async move |_cx| {
                let paths = match receiver.await {
                    Ok(Ok(Some(paths))) => paths,
                    _ => return Ok::<(), anyhow::Error>(()),
                };
                let _ = tx.send(Msg::InstallStarted);
                std::thread::spawn(move || install_paths(paths, true, tx));
                Ok::<(), anyhow::Error>(())
            })
            .detach();
    }

    fn install_dropped_paths(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        if self.installing_package || paths.is_empty() {
            return;
        }
        self.installing_package = true;
        self.message = "正在安装主题…".into();
        let tx = self.tx.clone();
        let paths = paths.to_vec();
        std::thread::spawn(move || install_paths(paths, true, tx));
        cx.notify();
    }

    fn install_store_theme(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(row) = self.store_rows.get(index) else {
            return;
        };
        if self.installing_store_theme.is_some()
            || self
                .themes
                .iter()
                .any(|theme| theme.theme.id == row.theme.id)
        {
            return;
        }
        let item = row.theme.clone();
        self.installing_store_theme = Some(item.id.clone());
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let result = theme::download_and_install_store_theme(
                &theme::theme_store_url(),
                &item,
                &theme::user_themes_dir(),
            );
            match result {
                Ok(installed) => {
                    let _ = tx.send(Msg::Installed {
                        ids: vec![installed.id],
                        error: None,
                        open_library: false,
                    });
                }
                Err(error) => {
                    let _ = tx.send(Msg::Installed {
                        ids: Vec::new(),
                        error: Some(error),
                        open_library: false,
                    });
                }
            }
        });
        cx.notify();
    }

    fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.themes.len() {
            self.selected = index;
            self.surface_opacity = self.themes[index].preview.surface_opacity;
            self.search_active = false;
            self.restart_confirmation_target = None;
            self.message.clear();
            cx.notify();
        }
    }

    fn switch_target(&mut self, target: live::TargetApp, cx: &mut Context<Self>) {
        if target == self.selected_target {
            return;
        }
        if !target.is_installed() {
            self.message = format!("尚未安装{}", target.display_name());
            cx.notify();
            return;
        }
        if let Some(stop) = self.live_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        self.generation += 1;
        self.applying = false;
        self.restart_confirmation_target = None;
        let old_target = self.active_target.take();
        self.active_theme = None;
        self.active_surface_opacity = None;
        self.selected_target = target;
        self.refresh_target_previews();
        self.ensure_selected_match();
        self.ensure_store_selected_match();
        save_target_preference(target);
        self.message.clear();
        let previous_thread = self.live_thread.take();
        if previous_thread.is_some() || old_target.is_some() {
            let tx = self.tx.clone();
            self.live_thread = Some(std::thread::spawn(move || {
                if let Some(thread) = previous_thread {
                    let _ = thread.join();
                }
                if let Some(old_target) = old_target {
                    let tx_log = tx.clone();
                    if let Err(error) = live::restore(old_target, move |line| {
                        let _ = tx_log.send(Msg::Log(line));
                    }) {
                        let _ = tx.send(Msg::Log(format!("restore failed: {error}")));
                    }
                }
            }));
        }
        cx.notify();
    }

    fn apply_selected(&mut self, cx: &mut Context<Self>) {
        if self.applying {
            return;
        }
        let Some(row) = self.themes.get(self.selected) else {
            return;
        };
        let target = self.selected_target;
        if !row.theme.supports_target(target) {
            self.restart_confirmation_target = None;
            self.message = format!("这个主题不支持{}", target.display_name());
            cx.notify();
            return;
        }
        if !target.is_installed() {
            self.restart_confirmation_target = None;
            self.message = format!("请先安装{}", target.display_name());
            cx.notify();
            return;
        }
        let active = theme_is_active(
            self.active_target,
            self.active_theme.as_deref(),
            target,
            row.theme.id.as_str(),
        ) && (!row.preview.has_background
            || self
                .active_surface_opacity
                .is_some_and(|value| (value - self.surface_opacity).abs() < 0.001));
        if active {
            return;
        }
        let allow_restart = self.restart_confirmation_target == Some(target);
        match live::prepare_state(target) {
            Ok(live::PrepareState::RestartConfirmationRequired) if !allow_restart => {
                self.restart_confirmation_target = Some(target);
                self.message =
                    "WorkBuddy 正在运行。请先保存正在进行的任务，再明确重启并应用。".into();
                cx.notify();
                return;
            }
            Ok(live::PrepareState::WrongPortOwner) => {
                self.restart_confirmation_target = None;
                self.message = format!("端口 {} 已被其他程序占用", target.port());
                cx.notify();
                return;
            }
            Ok(live::PrepareState::NotInstalled) => {
                self.restart_confirmation_target = None;
                self.message = format!("请先安装{}", target.display_name());
                cx.notify();
                return;
            }
            Ok(
                live::PrepareState::Ready
                | live::PrepareState::LaunchRequired
                | live::PrepareState::RestartConfirmationRequired,
            ) => {}
            Err(error) => {
                self.restart_confirmation_target = None;
                self.message = format!("无法准备{}：{error}", target.display_name());
                cx.notify();
                return;
            }
        }
        if let Some(stop) = self.live_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        let mut theme = row.theme.clone();
        if row.preview.has_background {
            theme.surface_opacity = Some(self.surface_opacity);
        }
        self.generation += 1;
        let generation = self.generation;
        let stop = Arc::new(AtomicBool::new(false));
        self.live_stop = Some(stop.clone());
        self.active_target = Some(target);
        self.active_theme = Some(theme.id.clone());
        self.active_surface_opacity = theme.surface_opacity;
        self.applying = true;
        self.restart_confirmation_target = None;
        self.message = "正在应用…".into();
        let tx = self.tx.clone();
        let previous_thread = self.live_thread.take();
        self.live_thread = Some(std::thread::spawn(move || {
            if let Some(thread) = previous_thread {
                let _ = thread.join();
            }
            let tx_log = tx.clone();
            let mut reported = false;
            let result = live::run_with_restart_permission(
                &theme,
                target,
                false,
                stop,
                allow_restart,
                move |line| {
                    if !reported && line.trim_start().starts_with("injected:") {
                        reported = true;
                        let _ = tx_log.send(Msg::Applied(generation));
                    }
                    let _ = tx_log.send(Msg::Log(line));
                },
            );
            let _ = tx.send(Msg::Done {
                generation: Some(generation),
                ok: result.is_ok(),
                restoring: false,
            });
        }));
        cx.notify();
    }

    fn restore_default(&mut self, cx: &mut Context<Self>) {
        if let Some(stop) = self.live_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        self.generation += 1;
        let generation = self.generation;
        let target = self.selected_target;
        self.restart_confirmation_target = None;
        self.active_target = None;
        self.active_theme = None;
        self.active_surface_opacity = None;
        self.applying = true;
        self.message = "正在恢复…".into();
        let tx = self.tx.clone();
        let previous_thread = self.live_thread.take();
        self.live_thread = Some(std::thread::spawn(move || {
            if let Some(thread) = previous_thread {
                let _ = thread.join();
            }
            let tx_log = tx.clone();
            let result = live::restore(target, move |line| {
                let _ = tx_log.send(Msg::Log(line));
            });
            let _ = tx.send(Msg::Done {
                generation: Some(generation),
                ok: result.is_ok(),
                restoring: true,
            });
        }));
        cx.notify();
    }

    fn set_surface_opacity(&mut self, value: f32, cx: &mut Context<Self>) {
        let next = (value.clamp(MIN_SURFACE_OPACITY, 1.0) * 100.0).round() / 100.0;
        if (next - self.surface_opacity).abs() < 0.001 {
            return;
        }
        self.surface_opacity = next;
        self.restart_confirmation_target = None;
        self.message.clear();
        cx.notify();
    }

    fn selected_settings_are_active(&self, row: &ThemeRow) -> bool {
        theme_is_active(
            self.active_target,
            self.active_theme.as_deref(),
            self.selected_target,
            row.theme.id.as_str(),
        ) && (!row.preview.has_background
            || self
                .active_surface_opacity
                .is_some_and(|value| (value - self.surface_opacity).abs() < 0.001))
    }

    fn render_target_switch(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let mut segments = div()
            .w(px(336.))
            .h(px(36.))
            .p(px(2.))
            .rounded(px(8.))
            .border_1()
            .border_color(rgb(colors.border))
            .bg(rgb(colors.segmented_track))
            .flex();
        for (index, target) in live::TargetApp::ALL.into_iter().enumerate() {
            let installed = target.is_installed();
            let selected = self.selected_target == target;
            let shortcut = target_shortcut(target);
            let label = if installed {
                target.display_name().to_string()
            } else {
                format!("{} · 未安装", target.display_name())
            };
            let aria = format!(
                "{}，{}，{shortcut}",
                target.display_name(),
                if !installed {
                    "未安装"
                } else if selected {
                    "已选中"
                } else {
                    "可选"
                }
            );
            segments = segments.child(
                div()
                    .id(("target-app", index))
                    .role(Role::Button)
                    .aria_label(aria)
                    .flex_1()
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgb(if selected {
                        colors.segmented_selected
                    } else {
                        colors.segmented_track
                    }))
                    .text_size(if installed { px(12.) } else { px(11.) })
                    .font_weight(if selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(colors.text))
                    .opacity(if installed { 1.0 } else { 0.46 })
                    .child(label)
                    .when(installed, |button| {
                        button
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(colors.control).opacity(0.72)))
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                this.switch_target(target, cx)
                            }))
                    }),
            );
        }
        segments.into_any_element()
    }

    fn render_source_switch(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
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
                    .aria_label("我的主题")
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
                    .child("我的主题")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.switch_source(SourceView::Library, cx)
                    })),
            )
            .child(
                div()
                    .id("source-store")
                    .role(Role::Button)
                    .aria_label("主题商店")
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
                    .child("主题商店")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.switch_source(SourceView::Store, cx)
                    })),
            )
            .into_any_element()
    }

    fn render_opacity_control(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let percent = (self.surface_opacity * 100.0).round() as u32;
        let progress =
            ((self.surface_opacity - MIN_SURFACE_OPACITY) / SURFACE_OPACITY_RANGE).clamp(0.0, 1.0);
        div()
            .w(px(OPACITY_TRACK_WIDTH))
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_xs()
                    .text_color(rgb(colors.muted))
                    .child("界面不透明度")
                    .child(format!("{percent}%")),
            )
            .child(
                div()
                    .id("opacity-slider")
                    .role(Role::Slider)
                    .aria_label(format!("界面不透明度 {percent}%"))
                    .relative()
                    .w_full()
                    .h(px(24.))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                            this.opacity_drag_start =
                                Some((event.position.x, this.surface_opacity));
                            cx.stop_propagation();
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                        let Some((start_x, start_opacity)) = this.opacity_drag_start else {
                            return;
                        };
                        if !event.dragging() {
                            this.opacity_drag_start = None;
                            return;
                        }
                        let progress_delta = (event.position.x - start_x) / px(OPACITY_TRACK_WIDTH);
                        this.set_surface_opacity(
                            start_opacity + SURFACE_OPACITY_RANGE * progress_delta,
                            cx,
                        );
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, _cx| {
                            this.opacity_drag_start = None;
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, _cx| {
                            this.opacity_drag_start = None;
                        }),
                    )
                    .on_click(|_event, _window, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .relative()
                            .w_full()
                            .h(px(4.))
                            .rounded_full()
                            .bg(rgb(colors.border))
                            .child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top_0()
                                    .h_full()
                                    .w(px(OPACITY_TRACK_WIDTH * progress))
                                    .rounded_full()
                                    .bg(rgb(colors.slider_accent)),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .top(px(-4.))
                                    .left(px((OPACITY_TRACK_WIDTH * progress - 6.0)
                                        .clamp(0.0, OPACITY_TRACK_WIDTH - 12.0)))
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(rgb(colors.control))
                                    .border_1()
                                    .border_color(rgb(colors.slider_accent)),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_theme_thumbnail(&self, row: &ThemeRow, size: f32) -> gpui::AnyElement {
        let accent = row.preview.colors.accent;
        if let Some(path) = row.theme.preview_image_for(self.selected_target) {
            return img(SharedString::from(path.to_string_lossy().into_owned()))
                .size(px(size))
                .rounded(px(8.))
                .object_fit(ObjectFit::Cover)
                .border_1()
                .border_color(preview_rgba(accent, 0.24))
                .into_any_element();
        }
        if let Some(path) = row.preview.background.as_ref() {
            return img(SharedString::from(path.to_string_lossy().into_owned()))
                .size(px(size))
                .rounded(px(8.))
                .object_fit(ObjectFit::Cover)
                .border_1()
                .border_color(preview_rgba(accent, 0.24))
                .into_any_element();
        }
        if row.preview.icons.main.is_some() {
            return div()
                .size(px(size))
                .rounded(px(8.))
                .bg(preview_rgba(row.preview.colors.main, 1.0))
                .border_1()
                .border_color(preview_rgba(accent, 0.28))
                .flex()
                .items_center()
                .justify_center()
                .child(preview_main_icon(
                    row.preview.icons.main.as_ref(),
                    size * 0.58,
                    accent,
                    row.preview.text,
                ))
                .into_any_element();
        }
        div()
            .size(px(size))
            .rounded(px(8.))
            .bg(preview_rgba(row.preview.colors.main, 1.0))
            .border_1()
            .border_color(preview_rgba(accent, 0.35))
            .child(
                div()
                    .m(px(size * 0.34))
                    .size(px(size * 0.32))
                    .rounded_full()
                    .bg(preview_rgba(accent, 1.0)),
            )
            .into_any_element()
    }

    fn render_drop_target(&self, compact: bool, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        div()
            .id(if compact {
                "drop-compact"
            } else {
                "drop-sidebar"
            })
            .role(Role::Button)
            .aria_label("拖入主题包即可安装，或选择文件")
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
                                "正在安装主题…"
                            } else {
                                "拖入主题包即可安装"
                            }),
                    )
                    .when(!compact, |view| {
                        view.child(
                            div()
                                .id("choose-package-sidebar")
                                .role(Role::Button)
                                .aria_label("选择主题包")
                                .text_xs()
                                .text_color(rgb(colors.link))
                                .cursor_pointer()
                                .hover(|style| style.opacity(0.72))
                                .child("选择文件…")
                                .on_click(cx.listener(|this, _event, window, cx| {
                                    this.choose_package(window, cx)
                                })),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_theme_item(
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

    fn render_theme_list(&self, compact: bool, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let mut list = div()
            .id("themes")
            .role(Role::List)
            .aria_label("全部主题")
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
                    .child("没有匹配的主题"),
            );
        }
        for index in indices {
            list = list.child(self.render_theme_item(index, compact, cx));
        }
        list.into_any_element()
    }

    fn render_store_sidebar_item(&self, index: usize, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let row = &self.store_rows[index];
        let selected = index == self.store_selected;
        let installed = self
            .themes
            .iter()
            .any(|theme| theme.theme.id == row.theme.id);
        let accent = parse_store_accent(row.theme.accent.as_deref());
        div()
            .id(("store-sidebar", index))
            .role(Role::Button)
            .aria_label(row.theme.name.clone())
            .h(px(42.))
            .flex_shrink_0()
            .px_3()
            .flex()
            .items_center()
            .gap_3()
            .rounded(px(8.))
            .bg(if selected {
                rgb(accent).opacity(0.16)
            } else {
                rgb(colors.sidebar)
            })
            .cursor_pointer()
            .hover(|style| style.bg(rgb(colors.hover)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.store_selected = index;
                cx.notify();
            }))
            .child(
                div()
                    .size(px(28.))
                    .rounded(px(6.))
                    .bg(rgb(accent).opacity(0.24))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(div().size(px(12.)).rounded_full().bg(rgb(accent))),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_sm()
                    .font_weight(if selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(colors.text))
                    .child(row.theme.name.clone()),
            )
            .when(installed, |item| {
                item.child(
                    div()
                        .text_xs()
                        .text_color(rgb(colors.muted))
                        .child("已安装"),
                )
            })
            .into_any_element()
    }

    fn render_store_sidebar_list(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let mut list = div()
            .id("store-themes-sidebar")
            .role(Role::List)
            .aria_label("商店主题")
            .flex_1()
            .min_h(px(0.))
            .overflow_scroll()
            .flex_col()
            .px_3()
            .pb_4();
        if self.store_loading {
            return list
                .child(
                    div()
                        .px_3()
                        .py_4()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child("正在连接…"),
                )
                .into_any_element();
        }
        if let Some(error) = self.store_error.as_ref() {
            return list
                .child(
                    div()
                        .px_3()
                        .py_4()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child("暂时无法连接")
                        .child(div().text_xs().child(error.clone())),
                )
                .into_any_element();
        }
        let indices = self.filtered_store_indices();
        if indices.is_empty() {
            return list
                .child(
                    div()
                        .px_3()
                        .py_4()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child(if self.query.is_empty() {
                            "暂时没有可用主题"
                        } else {
                            "没有匹配的主题"
                        }),
                )
                .into_any_element();
        }
        for index in indices {
            list = list.child(self.render_store_sidebar_item(index, cx));
        }
        list.into_any_element()
    }

    fn render_store_preview(&self, row: &StoreRow, height: f32) -> gpui::AnyElement {
        let colors = self.colors;
        let accent = parse_store_accent(row.theme.accent.as_deref());
        if let Some(path) = row.preview.as_ref() {
            let shared = SharedString::from(path.to_string_lossy().into_owned());
            if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("svg"))
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
            return img(shared)
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

    fn render_store_card(
        &self,
        index: usize,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let colors = self.colors;
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
                                    store_category_label(&row.theme.category).to_string()
                                } else {
                                    format!(
                                        "{} · {}",
                                        store_category_label(&row.theme.category),
                                        row.theme.version
                                    )
                                },
                            ))
                            .child(
                                div()
                                    .id(("install-store-theme", index))
                                    .role(Role::Button)
                                    .aria_label(if installed {
                                        format!("{} 已安装", row.theme.name)
                                    } else {
                                        format!("安装 {}", row.theme.name)
                                    })
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
                                    .when(!installed && !installing, |button| {
                                        button.cursor_pointer().hover(|style| style.opacity(0.86))
                                    })
                                    .child(if installing {
                                        "正在安装…"
                                    } else if installed {
                                        "已安装"
                                    } else {
                                        "安装"
                                    })
                                    .on_click(cx.listener(move |this, _event, _window, cx| {
                                        this.install_store_theme(index, cx)
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_store(&self, compact: bool, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let mut body =
            div()
                .flex_1()
                .min_h(px(0.))
                .p(if compact { px(16.) } else { px(24.) })
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(20.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(colors.text))
                                .child("主题商店"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .when(!self.message.is_empty(), |view| {
                                    view.child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(if self.message.contains("失败") {
                                                colors.danger
                                            } else {
                                                colors.muted
                                            }))
                                            .child(self.message.clone()),
                                    )
                                })
                                .child(
                                    div()
                                        .id("refresh-store")
                                        .role(Role::Button)
                                        .aria_label("刷新主题商店")
                                        .h(px(32.))
                                        .px_3()
                                        .rounded(px(7.))
                                        .border_1()
                                        .border_color(rgb(colors.border))
                                        .bg(rgb(colors.control))
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_sm()
                                        .text_color(rgb(colors.text))
                                        .cursor_pointer()
                                        .hover(|style| style.bg(rgb(colors.hover)))
                                        .child(
                                            svg()
                                                .path("icons/refresh.svg")
                                                .size(px(15.))
                                                .text_color(rgb(colors.muted)),
                                        )
                                        .child("刷新")
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.load_store(cx)
                                        })),
                                ),
                        ),
                );
        if self.store_loading {
            return body
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child("正在连接主题商店…"),
                )
                .into_any_element();
        }
        if let Some(error) = self.store_error.as_ref() {
            return body
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child("暂时无法打开主题商店")
                        .child(div().text_xs().child(error.clone())),
                )
                .into_any_element();
        }
        let indices = self.filtered_store_indices();
        if indices.is_empty() {
            return body
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .text_color(rgb(colors.muted))
                        .child(if self.query.is_empty() {
                            "主题商店暂时没有可用主题"
                        } else {
                            "没有匹配的主题"
                        }),
                )
                .into_any_element();
        }
        let mut grid = div()
            .id("theme-store-grid")
            .role(Role::List)
            .aria_label("主题商店")
            .flex_1()
            .min_h(px(0.))
            .overflow_scroll()
            .flex()
            .flex_wrap()
            .content_start()
            .gap_4()
            .pb_4();
        for index in indices {
            grid = grid.child(self.render_store_card(index, compact, cx));
        }
        body = body.child(grid);
        body.into_any_element()
    }

    fn render_store_detail(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let colors = self.colors;
        let Some(row) = self.store_rows.get(self.store_selected) else {
            return div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(colors.muted))
                .child("选择一个主题以查看详情")
                .into_any_element();
        };
        let installed = self
            .themes
            .iter()
            .any(|theme| theme.theme.id == row.theme.id);
        let installing = self.installing_store_theme.as_deref() == Some(row.theme.id.as_str());
        let accent = parse_store_accent(row.theme.accent.as_deref());
        let store_selected = self.store_selected;
        div()
            .id("store-detail")
            .flex_1()
            .min_h(px(0.))
            .overflow_scroll()
            .p(px(32.))
            .flex()
            .flex_col()
            .items_center()
            .gap_6()
            .child(
                div()
                    .w(px(480.))
                    .max_w_full()
                    .overflow_hidden()
                    .rounded(px(12.))
                    .border_1()
                    .border_color(rgb(colors.border))
                    .bg(rgb(colors.control))
                    .child(self.render_store_preview(row, 280.)),
            )
            .child(
                div()
                    .w(px(480.))
                    .max_w_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(22.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(colors.text))
                            .child(row.theme.name.clone()),
                    )
                    .child(div().text_sm().text_color(rgb(colors.muted)).child(
                        if row.theme.description.is_empty() {
                            row.theme.category.clone()
                        } else {
                            row.theme.description.clone()
                        },
                    ))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .text_xs()
                            .text_color(rgb(colors.muted))
                            .when(!row.theme.category.is_empty(), |d| {
                                let label = store_category_label(&row.theme.category).to_string();
                                d.child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded(px(4.))
                                        .bg(rgb(colors.control))
                                        .child(label),
                                )
                            })
                            .when(!row.theme.version.is_empty(), |d| {
                                d.child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded(px(4.))
                                        .bg(rgb(colors.control))
                                        .child(format!("v{}", row.theme.version)),
                                )
                            })
                            .when(!row.theme.author.is_empty(), |d| {
                                d.child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded(px(4.))
                                        .bg(rgb(colors.control))
                                        .child(row.theme.author.clone()),
                                )
                            }),
                    )
                    .child(
                        div()
                            .id("install-store-detail")
                            .role(Role::Button)
                            .aria_label(if installed {
                                "已安装"
                            } else if installing {
                                "安装中…"
                            } else {
                                "安装主题"
                            })
                            .mt_2()
                            .h(px(40.))
                            .px_6()
                            .rounded(px(8.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .cursor_pointer()
                            .when(installed, |btn| {
                                btn.bg(rgb(colors.installed_control))
                                    .text_color(rgb(colors.muted))
                                    .child("已安装")
                            })
                            .when(!installed && !installing, |btn| {
                                btn.bg(rgb(accent))
                                    .text_color(rgb(0xffffff))
                                    .hover(|style| style.opacity(0.88))
                                    .child("安装主题")
                                    .on_click(cx.listener(move |this, _event, _window, cx| {
                                        this.install_store_theme(store_selected, cx)
                                    }))
                            })
                            .when(installing, |btn| {
                                btn.bg(rgb(colors.control))
                                    .text_color(rgb(colors.muted))
                                    .child("正在安装…")
                            }),
                    ),
            )
            .into_any_element()
    }

    fn render_preview(&self, row: &ThemeRow, compact: bool, short: bool) -> gpui::AnyElement {
        let colors = self.colors;
        let (app_name, greeting) = preview_identity(self.selected_target);
        let style = &row.preview;
        let accent = style.colors.accent;
        let surface = if style.has_background {
            self.surface_opacity
        } else {
            1.0
        };
        let opacity = theme::surface_opacity_profile(surface);
        let sidebar_opacity = if style.has_background {
            opacity.preview_sidebar
        } else {
            0.91
        };
        let main_opacity = if style.has_background {
            opacity.preview_page
        } else {
            0.56
        };
        let input_opacity = if style.has_background {
            opacity.input
        } else {
            0.94
        };
        let layer_opacity = if style.has_background {
            opacity.layer
        } else {
            0.34
        };
        let density_scale: f32 = match style.density.as_str() {
            "compact" => 0.88,
            "spacious" => 1.08,
            _ => 1.0,
        };
        let sidebar_width = (style.sidebar_width * if compact { 0.43 } else { 0.61 }).clamp(
            if compact { 94.0 } else { 126.0 },
            if compact { 132.0 } else { 190.0 },
        );
        let nav_row_height = (23.0 * density_scale).clamp(20.0, 27.0);
        let action_row_height = (24.0 * density_scale).clamp(21.0, 29.0);
        let nav_icon_size = (13.0 * density_scale).clamp(12.0, 15.0);
        let action_icon_size = (14.0 * density_scale).clamp(12.0, 16.0);
        let chat_margin = (style.chat_margin * if compact { 0.44 } else { 0.88 }).clamp(
            if compact { 8.0 } else { 16.0 },
            if compact { 24.0 } else { 48.0 },
        );
        let composer_scale = if compact { 0.82 } else { 1.08 };
        let composer_height = (style.composer_min_height * composer_scale).clamp(
            if compact { 42.0 } else { 52.0 },
            if compact { 64.0 } else { 88.0 },
        );
        let composer_padding = (style.composer_padding * if compact { 0.54 } else { 0.72 })
            .clamp(5.0, if compact { 10.0 } else { 14.0 });
        let composer_gap =
            (style.composer_gap * if compact { 0.62 } else { 0.76 }).clamp(3.0, 12.0);
        let composer_icon_size =
            (style.composer_icon_size * if compact { 0.62 } else { 0.72 }).clamp(11.0, 18.0);
        let composer_radius =
            (style.composer_radius * style.radius_scale * if compact { 0.82 } else { 1.0 })
                .clamp(4.0, 28.0);
        let mut canvas = div()
            .relative()
            .size_full()
            .overflow_hidden()
            .rounded(px(PREVIEW_CONTENT_RADIUS))
            .bg(preview_rgba(style.colors.main, 1.0));
        if let Some(path) = &style.background {
            let fit = match style.background_fit.as_str() {
                "contain" => ObjectFit::Contain,
                "fill" => ObjectFit::Fill,
                "none" => ObjectFit::None,
                "scale-down" => ObjectFit::ScaleDown,
                _ => ObjectFit::Cover,
            };
            canvas = canvas
                .child(
                    img(SharedString::from(path.to_string_lossy().into_owned()))
                        .absolute()
                        .inset_0()
                        .size_full()
                        .rounded(px(PREVIEW_CONTENT_RADIUS))
                        .object_fit(fit)
                        .opacity(style.background_opacity),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(px(PREVIEW_CONTENT_RADIUS))
                        .bg(rgb(style.background_base)
                            .opacity(style.background_veil * style.background_opacity)),
                );
        } else {
            canvas = canvas
                .child(
                    div()
                        .absolute()
                        .top(px(-80.))
                        .right(px(-40.))
                        .size(px(260.))
                        .rounded_full()
                        .bg(preview_rgba(accent, 0.16)),
                )
                .child(
                    div()
                        .absolute()
                        .bottom(px(-120.))
                        .left(px(80.))
                        .size(px(300.))
                        .rounded_full()
                        .bg(preview_rgba(style.colors.sidebar, 0.34)),
                );
        }
        let canvas = canvas.child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .child(
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
                                .child("豆包 工作")
                                .child(div().text_size(px(13.)).child("⌕")),
                        )
                        .child(preview_nav_item(
                            "新工作任务",
                            style.icons.new_task.as_ref(),
                            style.text,
                            false,
                            nav_row_height,
                            nav_icon_size,
                        ))
                        .child(preview_nav_item(
                            "定时任务",
                            style.icons.scheduled.as_ref(),
                            style.text,
                            false,
                            nav_row_height,
                            nav_icon_size,
                        ))
                        .child(preview_nav_item(
                            "技能 · 连接器 · 伙伴",
                            style.icons.skills.as_ref(),
                            style.text,
                            false,
                            nav_row_height,
                            nav_icon_size,
                        ))
                        .child(preview_nav_item(
                            "云盘",
                            style.icons.cloud.as_ref(),
                            style.text,
                            false,
                            nav_row_height,
                            nav_icon_size,
                        ))
                        .child(preview_nav_item(
                            "手机遥控电脑",
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
                                .child("置顶"),
                        )
                        .child(preview_nav_item(
                            "主对话",
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
                                .child("项目"),
                        )
                        .child(preview_nav_item(
                            "看看",
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
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .relative()
                        .rounded_r(px(PREVIEW_CONTENT_RADIUS))
                        .bg(preview_rgba(style.colors.main, main_opacity))
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .h(if compact { px(30.) } else { px(36.) })
                                .px_3()
                                .flex()
                                .items_center()
                                .justify_between()
                                .border_b_1()
                                .border_color(preview_rgba(style.text, 0.08))
                                .child(
                                    div()
                                        .text_size(px(9.))
                                        .text_color(preview_rgba(style.text, 0.48))
                                        .child(app_name),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(preview_icon(
                                            style.icons.read_aloud.as_ref(),
                                            nav_icon_size,
                                            style.text,
                                        ))
                                        .child(preview_icon(
                                            style.icons.copy.as_ref(),
                                            nav_icon_size,
                                            style.text,
                                        ))
                                        .child(preview_icon(
                                            style.icons.sidebar.as_ref(),
                                            nav_icon_size,
                                            style.text,
                                        )),
                                ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .px(px(chat_margin))
                                .pt(if compact { px(8.) } else { px(13.) })
                                .pb(if compact { px(64.) } else { px(86.) })
                                .flex()
                                .flex_col()
                                .items_center()
                                .child(preview_main_icon(
                                    style.icons.main.as_ref(),
                                    if compact { 38. } else { 52. },
                                    accent,
                                    style.text,
                                ))
                                .child(
                                    div()
                                        .mt_2()
                                        .text_size(if compact { px(15.) } else { px(19.) })
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(preview_rgba(style.text, 1.0))
                                        .child(greeting),
                                )
                                .child(
                                    div()
                                        .mt(if compact { px(8.) } else { px(12.) })
                                        .w(if compact { px(210.) } else { px(270.) })
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_size(px(9.))
                                                .text_color(preview_rgba(style.text, 0.5))
                                                .child("为你推荐"),
                                        )
                                        .child(preview_action_item(
                                            "处理日常工作",
                                            style.icons.daily_work.as_ref(),
                                            theme::PreviewColor::opaque(0x4f83d6),
                                            style.text,
                                            style.surface,
                                            layer_opacity,
                                            (action_row_height, action_icon_size),
                                        ))
                                        .child(preview_action_item(
                                            "内容创作",
                                            style.icons.content_creation.as_ref(),
                                            theme::PreviewColor::opaque(0x43a873),
                                            style.text,
                                            style.surface,
                                            layer_opacity,
                                            (action_row_height, action_icon_size),
                                        ))
                                        .child(preview_action_item(
                                            "完成调研分析",
                                            style.icons.research.as_ref(),
                                            theme::PreviewColor::opaque(0x9a67d8),
                                            style.text,
                                            style.surface,
                                            layer_opacity,
                                            (action_row_height, action_icon_size),
                                        ))
                                        .child(preview_action_item(
                                            "设计与创意",
                                            style.icons.design.as_ref(),
                                            theme::PreviewColor::opaque(0xdf648d),
                                            style.text,
                                            style.surface,
                                            layer_opacity,
                                            (action_row_height, action_icon_size),
                                        )),
                                ),
                        )
                        .child(
                            div()
                                .absolute()
                                .left(px(chat_margin))
                                .right(px(chat_margin))
                                .bottom(if compact { px(10.) } else { px(16.) })
                                .min_h(px(composer_height))
                                .p(px(composer_padding))
                                .rounded(px(composer_radius))
                                .bg(preview_rgba(style.input, input_opacity))
                                .border_1()
                                .border_color(preview_rgba(style.input_border, 0.72))
                                .flex()
                                .flex_col()
                                .gap(px(composer_gap))
                                .child(
                                    div()
                                        .text_size(px(10.))
                                        .text_color(preview_rgba(style.composer_placeholder, 0.78))
                                        .child("输入问题或任务，/ 选择技能"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(composer_gap))
                                        .child(preview_icon(
                                            style.icons.new_task.as_ref(),
                                            composer_icon_size,
                                            style.composer_icon,
                                        ))
                                        .child(preview_icon(
                                            style.icons.project.as_ref(),
                                            composer_icon_size,
                                            style.composer_icon,
                                        ))
                                        .child(preview_icon(
                                            style.icons.confirm.as_ref(),
                                            composer_icon_size,
                                            style.composer_icon,
                                        ))
                                        .child(preview_icon(
                                            style.icons.knowledge.as_ref(),
                                            composer_icon_size,
                                            style.composer_icon,
                                        ))
                                        .child(preview_icon(
                                            style.icons.more_skills.as_ref(),
                                            composer_icon_size,
                                            style.composer_icon,
                                        ))
                                        .child(preview_icon(
                                            style.icons.connector.as_ref(),
                                            composer_icon_size,
                                            style.composer_icon,
                                        ))
                                        .child(div().flex_1())
                                        .child(preview_icon(
                                            style.icons.voice.as_ref(),
                                            composer_icon_size + 2.0,
                                            style.composer_icon,
                                        )),
                                ),
                        ),
                ),
        );
        div()
            .w_full()
            .flex_1()
            .min_h(if short {
                px(188.)
            } else if compact {
                px(236.)
            } else {
                px(320.)
            })
            .max_h(if short {
                px(188.)
            } else if compact {
                px(360.)
            } else {
                px(520.)
            })
            .when(!compact, |frame| frame.aspect_ratio(16.0 / 9.0))
            .overflow_hidden()
            .rounded(px(PREVIEW_FRAME_RADIUS))
            .border_1()
            .border_color(rgb(colors.border))
            .bg(preview_rgba(style.colors.main, 1.0))
            .flex()
            .child(canvas)
            .into_any_element()
    }
}

impl Render for SkinApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let compact = window.viewport_size().width < px(900.);
        let short = uses_short_compact_layout(compact, window.viewport_size().height);
        let content = if let Some(row) = self.themes.get(self.selected) {
            let active = self.selected_settings_are_active(row);
            let target_installed = self.selected_target.is_installed();
            let theme_supported = row.theme.supports_target(self.selected_target);
            let restart_confirmation =
                self.restart_confirmation_target == Some(self.selected_target);
            let detail_message = if !target_installed {
                format!("请先安装{}", self.selected_target.display_name())
            } else if !theme_supported {
                format!("这个主题不支持{}", self.selected_target.display_name())
            } else if self.message.is_empty() || self.message == "已应用" {
                let prefix = if self.message == "已应用" {
                    "已应用 · "
                } else {
                    ""
                };
                format!(
                    "{}{} · {}",
                    prefix,
                    support_label(row.theme.target_support(self.selected_target)),
                    self.selected_target.display_name()
                )
            } else {
                self.message.clone()
            };
            div()
                .flex_1()
                .min_w(px(0.))
                .p(if short {
                    px(12.)
                } else if compact {
                    px(16.)
                } else {
                    px(24.)
                })
                .flex()
                .flex_col()
                .gap(if short {
                    px(8.)
                } else if compact {
                    px(12.)
                } else {
                    px(20.)
                })
                .child(self.render_preview(row, compact, short))
                .child(
                    div()
                        .min_h(if short || !compact { px(72.) } else { px(80.) })
                        .flex()
                        .when(compact && !short, |view| view.flex_col().items_start())
                        .when(!compact || short, |view| view.items_center())
                        .justify_between()
                        .gap(if compact { px(12.) } else { px(20.) })
                        .child(
                            div()
                                .min_w(px(0.))
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .h(px(24.))
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_size(px(20.))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(colors.text))
                                        .child(row.theme.name.clone())
                                        .when(active, |title| {
                                            title.child(
                                                div()
                                                    .h(px(20.))
                                                    .px_2()
                                                    .rounded(px(6.))
                                                    .border_1()
                                                    .border_color(preview_rgba(
                                                        row.preview.colors.accent,
                                                        0.4,
                                                    ))
                                                    .bg(preview_rgba(
                                                        row.preview.colors.accent,
                                                        0.14,
                                                    ))
                                                    .flex()
                                                    .items_center()
                                                    .text_size(px(10.))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(preview_rgba(
                                                        row.preview.colors.accent,
                                                        1.0,
                                                    ))
                                                    .child("已应用"),
                                            )
                                        }),
                                )
                                .child(
                                    div()
                                        .h(px(20.))
                                        .text_sm()
                                        .text_color(rgb(colors.muted))
                                        .child(row.theme.description.clone()),
                                )
                                .child(
                                    div()
                                        .h(px(16.))
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_xs()
                                        .text_color(rgb(if detail_message.contains("失败") {
                                            colors.danger
                                        } else {
                                            colors.muted
                                        }))
                                        .child(detail_message),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .when(compact && !short, |view| view.w_full().justify_end())
                                .when(row.preview.has_background, |view| {
                                    view.child(self.render_opacity_control(cx))
                                })
                                .child(
                                    div()
                                        .id("restore")
                                        .role(Role::Button)
                                        .aria_label("恢复默认")
                                        .h(px(36.))
                                        .px_4()
                                        .flex()
                                        .items_center()
                                        .rounded(px(7.))
                                        .border_1()
                                        .border_color(rgb(colors.border))
                                        .bg(rgb(colors.control))
                                        .text_sm()
                                        .text_color(rgb(colors.text))
                                        .cursor_pointer()
                                        .hover(|style| style.bg(rgb(colors.hover)))
                                        .child("恢复默认")
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.restore_default(cx)
                                        })),
                                )
                                .child(
                                    div()
                                        .id("apply")
                                        .role(Role::Button)
                                        .aria_label(if !target_installed {
                                            "尚未安装目标应用"
                                        } else if !theme_supported {
                                            "此主题不支持当前应用"
                                        } else if active {
                                            "正在使用"
                                        } else if restart_confirmation {
                                            "重启 WorkBuddy 并应用"
                                        } else {
                                            "应用主题"
                                        })
                                        .h(px(36.))
                                        .px_5()
                                        .flex()
                                        .items_center()
                                        .rounded(px(7.))
                                        .bg(preview_rgba(row.preview.colors.accent, 1.0))
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(0xffffff))
                                        .opacity(
                                            if self.applying
                                                || active
                                                || !target_installed
                                                || !theme_supported
                                            {
                                                0.72
                                            } else {
                                                1.0
                                            },
                                        )
                                        .when(
                                            !self.applying
                                                && !active
                                                && target_installed
                                                && theme_supported,
                                            |button| {
                                                button
                                                    .cursor_pointer()
                                                    .hover(|style| style.opacity(0.88))
                                                    .on_click(cx.listener(
                                                        |this, _event, _window, cx| {
                                                            this.apply_selected(cx)
                                                        },
                                                    ))
                                            },
                                        )
                                        .child(if !target_installed {
                                            "尚未安装"
                                        } else if !theme_supported {
                                            "主题不兼容"
                                        } else if self.applying {
                                            "正在应用…"
                                        } else if active {
                                            "正在使用"
                                        } else if restart_confirmation {
                                            "重启并应用"
                                        } else {
                                            "应用主题"
                                        }),
                                ),
                        ),
                )
                .into_any_element()
        } else {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(colors.muted))
                .child("还没有可用主题")
                .into_any_element()
        };

        let query = self.query.clone();
        let search_active = self.search_active;
        let search = div()
            .id("search")
            .role(Role::Button)
            .aria_label("搜索主题")
            .when(compact, |view| view.flex_1().min_w(px(0.)))
            .when(!compact, |view| view.w_full())
            .h(px(36.))
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(8.))
            .border_1()
            .border_color(rgb(if search_active {
                colors.focus_border
            } else {
                colors.border
            }))
            .bg(rgb(colors.control).opacity(0.92))
            .cursor_pointer()
            .on_click(cx.listener(|this, _event, window, cx| {
                this.search_active = true;
                this.focus_handle.focus(window, cx);
                cx.notify();
            }))
            .child(
                svg()
                    .path("icons/search.svg")
                    .size(px(15.))
                    .text_color(rgb(colors.muted)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .text_sm()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(rgb(if query.is_empty() {
                        colors.muted
                    } else {
                        colors.text
                    }))
                    .child(if query.is_empty() {
                        "搜索主题".to_string()
                    } else {
                        query
                    }),
            )
            .when(!self.query.is_empty(), |view| {
                view.child(
                    div()
                        .id("clear-search")
                        .role(Role::Button)
                        .aria_label("清除搜索")
                        .size(px(20.))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(14.))
                        .text_color(rgb(colors.muted))
                        .hover(|style| style.bg(rgb(colors.hover)))
                        .child("×")
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.query.clear();
                            if this.source_view == SourceView::Library {
                                this.ensure_selected_match();
                            }
                            cx.stop_propagation();
                            cx.notify();
                        })),
                )
            });

        let brand = div()
            .flex()
            .items_center()
            .text_size(px(17.))
            .font_weight(FontWeight::BOLD)
            .text_color(rgb(colors.text))
            .child("豆皮");
        let target_switch = self.render_target_switch(cx);

        let header = if compact {
            div()
                .h(px(HEADER_HEIGHT))
                .px_4()
                .border_b_1()
                .border_color(rgb(colors.border))
                .flex()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .child(div().pl(px(WINDOW_TITLE_X - 16.)).child(brand)),
                )
                .child(target_switch)
                .child(div().flex_1().min_w(px(0.)))
        } else {
            div()
                .h(px(HEADER_HEIGHT))
                .px_6()
                .border_b_1()
                .border_color(rgb(colors.border))
                .flex()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .child(div().pl(px(WINDOW_TITLE_X - 24.)).child(brand)),
                )
                .child(target_switch)
                .child(div().flex_1().min_w(px(0.)))
        };

        let body = if compact {
            if self.source_view == SourceView::Store {
                div()
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
                            .child(search),
                    )
                    .child(self.render_store(true, cx))
                    .into_any_element()
            } else {
                div()
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
                            .child(search),
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
                    .into_any_element()
            }
        } else {
            let detail = if self.source_view == SourceView::Store {
                self.render_store_detail(cx)
            } else {
                content
            };
            div()
                .flex_1()
                .min_h(px(0.))
                .flex()
                .child(
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
                                .child(search),
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
                                        .child("我的主题")
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
                                        .child("主题商店")
                                        .child(
                                            div()
                                                .id("refresh-store-sidebar")
                                                .role(Role::Button)
                                                .aria_label("刷新")
                                                .cursor_pointer()
                                                .hover(|style| style.opacity(0.72))
                                                .child(if self.store_loading {
                                                    "加载中…"
                                                } else {
                                                    "刷新"
                                                })
                                                .on_click(cx.listener(
                                                    |this, _event, _window, cx| this.load_store(cx),
                                                )),
                                        ),
                                )
                                .child(self.render_store_sidebar_list(cx))
                        }),
                )
                .child(detail)
                .into_any_element()
        };

        div()
            .id("theme-picker")
            .role(Role::Application)
            .aria_label("豆皮")
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::key_down))
            .drag_over::<ExternalPaths>(move |style, _, _, _| style.bg(rgb(colors.drop_hover)))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _window, cx| {
                this.install_dropped_paths(paths.paths(), cx)
            }))
            .bg(rgb(colors.shell))
            .flex()
            .flex_col()
            .child(header)
            .child(body)
    }
}

fn install_paths(paths: Vec<PathBuf>, open_library: bool, tx: mpsc::Sender<Msg>) {
    let installed_dir = theme::user_themes_dir();
    let mut ids = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match theme::install_theme_package(&path, &installed_dir) {
            Ok(installed) => ids.push(installed.id),
            Err(error) => {
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("主题包");
                errors.push(format!("{name}：{error}"));
            }
        }
    }
    let error = (!errors.is_empty()).then(|| errors.join("；"));
    let _ = tx.send(Msg::Installed {
        ids,
        error,
        open_library,
    });
}

fn parse_store_accent(value: Option<&str>) -> u32 {
    let Some(value) = value.map(str::trim) else {
        return 0xa64e24;
    };
    if let Some(hex) = value.strip_prefix('#') {
        if hex.len() == 6 {
            if let Ok(color) = u32::from_str_radix(hex, 16) {
                return color;
            }
        }
    }
    if let Some(inner) = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|inner| inner.strip_suffix(')'))
    {
        let parts = inner
            .split(',')
            .take(3)
            .filter_map(|part| part.trim().parse::<u32>().ok())
            .collect::<Vec<_>>();
        if parts.len() == 3 {
            return (parts[0].min(255) << 16) | (parts[1].min(255) << 8) | parts[2].min(255);
        }
    }
    0xa64e24
}

fn store_category_label(category: &str) -> &str {
    match category {
        "pure" => "纯色",
        "atmosphere" => "氛围背景",
        "gallery" => "热门灵感",
        "codex" => "编辑器配色",
        "brand" => "品牌灵感",
        _ => "主题",
    }
}

fn init_logger() {
    struct StderrLogger;
    impl log::Log for StderrLogger {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }
        fn log(&self, record: &log::Record) {
            eprintln!("[{} {}] {}", record.level(), record.target(), record.args());
        }
        fn flush(&self) {}
    }
    static LOGGER: StderrLogger = StderrLogger;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
fn set_development_icon() {
    use cocoa::appkit::{NSApp, NSApplication, NSImage};
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSData, NSString};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let bundle: id = msg_send![objc::class!(NSBundle), mainBundle];
        let path: id = msg_send![bundle, bundlePath];
        let ext = NSString::alloc(nil).init_str(".app");
        let is_bundle: bool = msg_send![path, hasSuffix: ext];
        if is_bundle {
            return;
        }
    }
    let bytes = include_bytes!("../../../assets/app-icon/AppIcon.icns");
    unsafe {
        let data = NSData::dataWithBytes_length_(nil, bytes.as_ptr().cast(), bytes.len() as _);
        let image = NSImage::initWithData_(NSImage::alloc(nil), data);
        assert!(image != nil, "embedded AppIcon.icns must be valid");
        NSApp().setApplicationIconImage_(image);
    }
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

fn show_about(_: &About, _: &mut App) {
    show_about_panel();
}

fn hide_application(_: &HideApplication, cx: &mut App) {
    cx.hide();
}

fn hide_others(_: &HideOthers, cx: &mut App) {
    cx.hide_other_apps();
}

fn show_all(_: &ShowAll, cx: &mut App) {
    cx.unhide_other_apps();
}

fn quit_application(_: &QuitApplication, cx: &mut App) {
    cx.quit();
}

fn main() {
    init_logger();
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
        set_development_icon();
        cx.bind_keys([
            KeyBinding::new("cmd-h", HideApplication, None),
            KeyBinding::new("cmd-alt-h", HideOthers, None),
            KeyBinding::new("cmd-q", QuitApplication, None),
        ]);
        cx.on_action(show_about);
        cx.on_action(hide_application);
        cx.on_action(hide_others);
        cx.on_action(show_all);
        cx.on_action(quit_application);
        cx.set_menus([application_menu()]);
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
                    view.message = "这个主题暂时不可用".into();
                    cx.notify();
                }
            });
        }
        cx.activate(true);
    });
}

#[cfg(test)]
mod ui_regression_tests {
    use super::*;

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
        let bounds = Bounds::new(point(px(20.), px(30.)), size(px(1120.), px(720.)));
        let options = main_window_options(bounds);
        assert_eq!(options.window_bounds, Some(WindowBounds::Windowed(bounds)));
        assert_eq!(options.window_min_size, Some(size(px(1120.), px(720.))));
        assert!(!options.is_resizable);
        assert!(options.is_movable);
        assert!(options.is_minimizable);
    }

    #[test]
    fn application_menu_contains_the_native_about_and_lifecycle_items() {
        let menu = application_menu();
        assert_eq!(menu.name.as_ref(), "豆皮");
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
            ["关于豆皮", "隐藏豆皮", "隐藏其他", "全部显示", "退出豆皮",]
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
        assert_eq!(preview_identity(live::TargetApp::Doubao).0, "豆包");
        assert_eq!(preview_identity(live::TargetApp::DoubaoWork).0, "豆包工作");
        assert_eq!(preview_identity(live::TargetApp::WorkBuddy).0, "WorkBuddy");
        assert_eq!(target_shortcut(live::TargetApp::WorkBuddy), "Command-3");
    }

    #[test]
    fn support_badges_distinguish_explicit_shared_and_legacy_compatibility() {
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
        assert_eq!(
            support_label(TargetSupport {
                level: SupportLevel::Unsupported,
                declaration: SupportDeclaration::Explicit,
            }),
            "不支持"
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
}

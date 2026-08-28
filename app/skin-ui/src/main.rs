//! GPUI front-end for the DoubaoWork skin tool (豆包工作皮肤工具).
//!
//! One window: theme cards with mini previews +「Live 应用」/「离线构建」
//! buttons, a「移除皮肤版」ghost button, a log area and a status line.
//! Actions run on background std threads and stream log lines back over an
//! mpsc channel.
//!
//! `--live <theme-id>` triggers the same handler as the theme's「Live 应用」
//! button right after startup (used for automated verification).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, ElementId, FontWeight, SharedString,
    Stateful, TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;

use skin_core::{build, live, theme};

const MAX_LOG_LINES: usize = 300;

// palette
const BG: u32 = 0x0d0b14; // window background
const CARD_BG: u32 = 0x171426; // theme card background
const CARD_BORDER: u32 = 0x28223d; // theme card border
const TEXT: u32 = 0xe8e5f2; // primary text
const MUTED: u32 = 0x8a84a3; // secondary text
const FAINT: u32 = 0x5a546e; // hints / idle status dot
const LOG_BG: u32 = 0x0a0810; // log area background
const OUTLINE_BORDER: u32 = 0x3a3355; // outline button border
const OUTLINE_HOVER: u32 = 0x221d38; // outline button hover bg
const DANGER: u32 = 0xe06c75; // ghost-danger text
const BUILDING: u32 = 0xe5c07b; // building status dot

enum Msg {
    Log(String),
    Done { live_gen: Option<u64>, status: String },
}

struct ThemeRow {
    theme: theme::Theme,
    preview: theme::PreviewColors,
}

struct SkinApp {
    tx: mpsc::Sender<Msg>,
    themes: Vec<ThemeRow>,
    logs: VecDeque<SharedString>, // newest first
    status: String,
    building: bool,
    live_theme: Option<String>,
    live_stop: Option<Arc<AtomicBool>>,
    live_gen: u64,
}

impl SkinApp {
    fn new(tx: mpsc::Sender<Msg>, rx: mpsc::Receiver<Msg>, cx: &mut Context<Self>) -> Self {
        let themes = theme::list(&theme::default_themes_dir())
            .into_iter()
            .map(|t| ThemeRow { preview: t.preview_colors(), theme: t })
            .collect();
        // Drain worker messages into UI state.
        cx.spawn(async move |this, cx| loop {
            while let Ok(msg) = rx.try_recv() {
                let alive = this
                    .update(cx, |this, cx| {
                        this.handle_msg(msg);
                        cx.notify();
                    })
                    .is_ok();
                if !alive {
                    return;
                }
            }
            cx.background_executor().timer(Duration::from_millis(120)).await;
        })
        .detach();
        SkinApp {
            tx,
            themes,
            logs: VecDeque::new(),
            status: "就绪".into(),
            building: false,
            live_theme: None,
            live_stop: None,
            live_gen: 0,
        }
    }

    fn handle_msg(&mut self, msg: Msg) {
        match msg {
            Msg::Log(line) => {
                self.logs.push_front(line.into());
                while self.logs.len() > MAX_LOG_LINES {
                    self.logs.pop_back();
                }
            }
            Msg::Done { live_gen, status } => {
                self.status = status;
                self.building = false;
                // a live watcher exiting clears the live state, unless a newer
                // watcher has already taken over
                if let Some(gen) = live_gen {
                    if gen == self.live_gen {
                        self.live_theme = None;
                        self.live_stop = None;
                    }
                }
            }
        }
    }

    fn push_log(&mut self, line: String) {
        self.logs.push_front(line.into());
    }

    fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.logs.clear();
        cx.notify();
    }

    fn start_live(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(row) = self.themes.get(index) else { return };
        let theme = row.theme.clone();
        if let Some(stop) = self.live_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        self.live_gen += 1;
        let gen = self.live_gen;
        let stop = Arc::new(AtomicBool::new(false));
        self.live_stop = Some(stop.clone());
        self.live_theme = Some(theme.id.clone());
        self.status = format!("Live 主题：{}（{}）— 监控页面中…", theme.name, theme.id);
        self.push_log(format!("→ Live 应用主题：{}", theme.id));
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let tx_log = tx.clone();
            let result = live::run(&theme, live::DEFAULT_PORT, false, stop, move |line| {
                let _ = tx_log.send(Msg::Log(line));
            });
            let status = match result {
                Ok(()) => "Live 监控已结束".to_string(),
                Err(e) => format!("Live 失败：{e}"),
            };
            let _ = tx.send(Msg::Done { live_gen: Some(gen), status });
        });
        cx.notify();
    }

    fn start_build(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.building {
            return;
        }
        let Some(row) = self.themes.get(index) else { return };
        let theme = row.theme.clone();
        self.building = true;
        self.status = format!("正在离线构建 {} …", theme.id);
        self.push_log(format!("→ 离线构建主题：{}", theme.id));
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let tx_log = tx.clone();
            let result = build::apply(&theme, move |line| {
                let _ = tx_log.send(Msg::Log(line));
            });
            let status = match result {
                Ok(path) => format!("构建完成：{}", path.display()),
                Err(e) => format!("构建失败：{e}"),
            };
            let _ = tx.send(Msg::Done { live_gen: None, status });
        });
        cx.notify();
    }

    fn remove_skin(&mut self, cx: &mut Context<Self>) {
        if self.building {
            return;
        }
        self.building = true;
        self.status = "正在移除皮肤版…".into();
        self.push_log("→ 移除 ~/Applications/DoubaoWork-Skin.app".into());
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let tx_log = tx.clone();
            let result = build::remove(move |line| {
                let _ = tx_log.send(Msg::Log(line));
            });
            let status = match result {
                Ok(()) => "已移除皮肤版".to_string(),
                Err(e) => format!("移除失败：{e}"),
            };
            let _ = tx.send(Msg::Done { live_gen: None, status });
        });
        cx.notify();
    }
}

/// Mix a 0xRRGGBB color toward white by `amount` (0..1).
fn lighten(c: u32, amount: f32) -> u32 {
    let mix = |v: u32| -> u32 { (v as f32 + (255.0 - v as f32) * amount) as u32 };
    (mix(c >> 16 & 0xff) << 16) | (mix(c >> 8 & 0xff) << 8) | mix(c & 0xff)
}

/// Accent-filled primary button (「Live 应用」).
fn primary_button(
    id: impl Into<ElementId>,
    label: &'static str,
    accent: u32,
) -> Stateful<gpui::Div> {
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_md()
        .text_xs()
        .text_color(rgb(0xffffff))
        .bg(rgb(accent))
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(lighten(accent, 0.15))))
        .child(label)
}

/// Subtle outline button (「离线构建」).
fn outline_button(
    id: impl Into<ElementId>,
    label: &'static str,
    disabled: bool,
) -> Stateful<gpui::Div> {
    let b = div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_md()
        .text_xs()
        .text_color(rgb(0xc9c4dd))
        .border_1()
        .border_color(rgb(OUTLINE_BORDER));
    if disabled {
        b.opacity(0.4).child(label)
    } else {
        b.cursor_pointer().hover(move |s| s.bg(rgb(OUTLINE_HOVER))).child(label)
    }
}

/// Mini app-layout preview: sidebar strip + main area + accent dot.
fn mini_preview(preview: theme::PreviewColors) -> impl IntoElement {
    div()
        .w(px(92.))
        .h(px(60.))
        .rounded_md()
        .overflow_hidden()
        .border_1()
        .border_color(rgb(0x000000).opacity(0.5))
        .flex()
        .child(div().w(px(22.)).h_full().bg(rgb(preview.sidebar)))
        .child(
            div()
                .flex_1()
                .h_full()
                .bg(rgb(preview.main))
                .p(px(6.))
                .child(div().size(px(8.)).rounded_full().bg(rgb(preview.accent))),
        )
}

impl SkinApp {
    fn render_theme_card(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let row = &self.themes[index];
        let t = &row.theme;
        let accent = row.preview.accent;
        let is_live = self.live_theme.as_deref() == Some(t.id.as_str());
        let busy = self.building;

        let mut card = div()
            .id(("card", index))
            .relative()
            .flex()
            .items_center()
            .gap_3()
            .px_3()
            .py(px(10.))
            .rounded_lg()
            .bg(rgb(CARD_BG))
            .border_1()
            .border_color(rgb(if is_live { accent } else { CARD_BORDER }))
            .hover(move |s| s.border_color(rgb(accent)))
            .child(mini_preview(row.preview))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(TEXT))
                                    .child(t.name.clone()),
                            )
                            .child(div().text_xs().text_color(rgb(FAINT)).child(t.id.clone())),
                    )
                    .child(
                        div().text_sm().text_color(rgb(MUTED)).child(t.description.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        primary_button(("live", index), "Live 应用", accent).on_click(
                            cx.listener(move |this, _ev, _window, cx| {
                                this.start_live(index, cx);
                            }),
                        ),
                    )
                    .child(
                        outline_button(("build", index), "离线构建", busy).on_click(
                            cx.listener(move |this, _ev, _window, cx| {
                                this.start_build(index, cx);
                            }),
                        ),
                    ),
            );
        if is_live {
            card = card.child(
                div()
                    .absolute()
                    .top(px(-7.))
                    .right_3()
                    .px_2()
                    .rounded_full()
                    .bg(rgb(accent))
                    .text_color(rgb(0xffffff))
                    .text_size(px(10.))
                    .child("应用中"),
            );
        }
        card
    }
}

impl Render for SkinApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // status dot: live theme accent > building yellow > idle gray
        let dot_color = if let Some(id) = &self.live_theme {
            self.themes
                .iter()
                .find(|r| &r.theme.id == id)
                .map(|r| r.preview.accent)
                .unwrap_or(FAINT)
        } else if self.building {
            BUILDING
        } else {
            FAINT
        };

        let header = div()
            .flex()
            .items_end()
            .justify_between()
            .px_4()
            .pb_2()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .text_size(px(17.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(TEXT))
                            .child("豆包工作 · 皮肤工具"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child("给豆包工作换个皮肤 · Live 注入 / 离线构建"),
                    ),
            )
            .child(
                div()
                    .id("remove")
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .text_xs()
                    .text_color(rgb(DANGER))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(DANGER).opacity(0.12)))
                    .child("移除皮肤版")
                    .on_click(cx.listener(|this, _ev, _window, cx| {
                        this.remove_skin(cx);
                    })),
            );

        let mut cards = div()
            .id("cards")
            .flex_1()
            .overflow_y_scroll()
            .px_4()
            .pb_2()
            .flex()
            .flex_col()
            .gap(px(10.));
        if self.themes.is_empty() {
            cards = cards.child(
                div().p_3().text_sm().text_color(rgb(MUTED)).child("未找到主题（themes/ 目录为空）"),
            );
        }
        for i in 0..self.themes.len() {
            cards = cards.child(self.render_theme_card(i, cx));
        }

        let log_text: String =
            self.logs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n");
        let log_section = div()
            .px_4()
            .pb_2()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_xs().text_color(rgb(FAINT)).child("日志"))
                    .child(
                        div()
                            .id("clear-logs")
                            .px_1()
                            .rounded_sm()
                            .text_xs()
                            .text_color(rgb(FAINT))
                            .cursor_pointer()
                            .hover(|s| s.text_color(rgb(MUTED)).bg(rgb(CARD_BORDER)))
                            .child("清空")
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                this.clear_logs(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .h(px(150.))
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(CARD_BORDER))
                    .bg(rgb(LOG_BG))
                    .p_2()
                    .id("log")
                    .overflow_y_scroll()
                    .child(
                        div()
                            .text_xs()
                            .font_family("Menlo")
                            .text_color(rgb(0xbdb7d4))
                            .child(if log_text.is_empty() {
                                "暂无日志".to_string()
                            } else {
                                log_text
                            }),
                    ),
            );

        let status_bar = div()
            .px_4()
            .py_2()
            .border_t_1()
            .border_color(rgb(CARD_BORDER))
            .flex()
            .items_center()
            .gap_2()
            .child(div().size(px(8.)).rounded_full().bg(rgb(dot_color)))
            .child(div().text_xs().text_color(rgb(MUTED)).child(self.status.clone()));

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(BG))
            .pt(px(30.)) // leave room for the traffic lights (transparent titlebar)
            .child(header)
            .child(cards)
            .child(log_section)
            .child(status_bar)
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
fn set_application_icon() {
    use cocoa::appkit::{NSApp, NSApplication, NSImage};
    use cocoa::base::nil;
    use cocoa::foundation::NSData;

    let bytes = include_bytes!("../../../assets/app-icon/AppIcon.icns");

    // SAFETY: GPUI invokes this closure on AppKit's main thread. NSData copies the
    // embedded bytes, NSImage retains its data, and NSApplication retains the image.
    unsafe {
        let data = NSData::dataWithBytes_length_(nil, bytes.as_ptr().cast(), bytes.len() as _);
        let image = NSImage::initWithData_(NSImage::alloc(nil), data);
        assert!(image != nil, "embedded AppIcon.icns must be a valid macOS icon");
        NSApp().setApplicationIconImage_(image);
    }
}

fn main() {
    init_logger();
    // parse `--live <theme-id>`
    let args: Vec<String> = std::env::args().collect();
    let mut live_arg: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--live" {
            live_arg = args.get(i + 1).cloned();
            i += 2;
        } else {
            i += 1;
        }
    }

    application().run(move |cx: &mut App| {
        #[cfg(target_os = "macos")]
        set_application_icon();

        let (tx, rx) = mpsc::channel::<Msg>();

        let bounds = Bounds::centered(None, size(px(780.), px(680.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: None,
                        appears_transparent: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                move |_, cx| cx.new(move |cx| SkinApp::new(tx, rx, cx)),
            )
            .unwrap();

        if let Some(id) = live_arg {
            let _ = window.update(cx, move |view, _window, cx| {
                if let Some(index) = view.themes.iter().position(|t| t.theme.id == id) {
                    view.start_live(index, cx);
                } else {
                    view.status = format!("未知主题：{id}");
                    cx.notify();
                }
            });
        }
        cx.activate(true);
    });
}

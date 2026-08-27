//! GPUI front-end for the DoubaoWork skin tool (豆包工作皮肤工具).
//!
//! One window: theme list with swatches +「Live 应用」/「离线构建」buttons,
//! a「移除皮肤版」button, a log area and a status line. Actions run on
//! background std threads and stream log lines back over an mpsc channel.
//!
//! `--live <theme-id>` triggers the same handler as the theme's「Live 应用」
//! button right after startup (used for automated verification).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, ElementId, SharedString, Stateful,
    TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;

use skin_core::{build, live, theme};

const MAX_LOG_LINES: usize = 300;

enum Msg {
    Log(String),
    Done { live_gen: Option<u64>, status: String },
}

struct ThemeRow {
    theme: theme::Theme,
    swatches: Vec<u32>,
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
            .map(|t| ThemeRow { swatches: t.swatches(4), theme: t })
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

fn action_button(
    id: impl Into<ElementId>,
    label: &'static str,
    bg: u32,
    hover_bg: u32,
    disabled: bool,
) -> Stateful<gpui::Div> {
    let b = div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_md()
        .text_xs()
        .text_color(rgb(0xffffff))
        .bg(rgb(bg))
        .child(label);
    if disabled {
        b.opacity(0.4)
    } else {
        b.cursor_pointer().hover(move |s| s.bg(rgb(hover_bg)))
    }
}

impl SkinApp {
    fn render_theme_row(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let row = &self.themes[index];
        let t = &row.theme;
        let busy = self.building;
        div()
            .flex()
            .items_center()
            .gap_3()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(rgb(0x2a2540))
            .child(
                // swatches from the theme's first css colors
                div().flex().gap_1().children(row.swatches.iter().map(|c| {
                    div().size_4().rounded_sm().bg(rgb(*c)).border_1().border_color(rgb(0x000000))
                })),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .items_baseline()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(t.name.clone()),
                            )
                            .child(div().text_xs().text_color(rgb(0x8a84a3)).child(t.id.clone())),
                    )
                    .child(
                        div().text_xs().text_color(rgb(0xa8a2c0)).child(t.description.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        action_button(("live", index), "Live 应用", 0x4e3594, 0x5d40b3, busy)
                            .on_click(cx.listener(move |this, _ev, _window, cx| {
                                this.start_live(index, cx);
                            })),
                    )
                    .child(
                        action_button(("build", index), "离线构建", 0x2b5d50, 0x37705f, busy)
                            .on_click(cx.listener(move |this, _ev, _window, cx| {
                                this.start_build(index, cx);
                            })),
                    ),
            )
    }
}

impl Render for SkinApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x16131f))
            .text_color(rgb(0xe6e2f2))
            .child(
                // header
                div()
                    .flex()
                    .items_center()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(0x2a2540))
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("豆包工作 · 皮肤工具"),
                    )
                    .child(
                        action_button("remove", "移除皮肤版", 0x7a3030, 0x944040, self.building)
                            .on_click(cx.listener(|this, _ev, _window, cx| {
                                this.remove_skin(cx);
                            })),
                    ),
            );

        // theme rows
        let mut list = div().id("themes").flex_1().overflow_y_scroll().flex().flex_col();
        if self.themes.is_empty() {
            list = list.child(div().p_3().text_sm().child("未找到主题（themes/ 目录为空）"));
        }
        for i in 0..self.themes.len() {
            list = list.child(self.render_theme_row(i, cx));
        }
        root = root.child(list);

        // log area (newest first so no auto-scroll is needed)
        let log_text: String =
            self.logs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n");
        root = root.child(
            div()
                .h(px(190.))
                .border_t_1()
                .border_color(rgb(0x2a2540))
                .bg(rgb(0x100d18))
                .p_2()
                .id("log")
                .overflow_y_scroll()
                .child(
                    div().text_xs().font_family("Menlo").text_color(rgb(0xbdb7d4)).child(
                        if log_text.is_empty() { "日志输出…".to_string() } else { log_text },
                    ),
                ),
        );

        // status line
        root.child(
            div()
                .px_3()
                .py_1()
                .border_t_1()
                .border_color(rgb(0x2a2540))
                .bg(rgb(0x1c1829))
                .text_xs()
                .text_color(rgb(0x9d97b8))
                .child(self.status.clone()),
        )
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

fn main() {
    init_logger();
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
        let (tx, rx) = mpsc::channel::<Msg>();

        let bounds = Bounds::centered(None, size(px(760.), px(660.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("豆包工作 · 皮肤工具".into()),
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

//! One-shot live injection for manual debugging:
//!   cargo run -p skin-core --example live_once -- violet-night
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn main() {
    let id = std::env::args().nth(1).unwrap_or_else(|| "violet-night".into());
    let themes_dir = skin_core::theme::default_themes_dir();
    let theme = skin_core::theme::load(&themes_dir, &id).expect("load theme");
    let stop = Arc::new(AtomicBool::new(false));
    skin_core::live::run(&theme, skin_core::live::DEFAULT_PORT, true, stop, |line| {
        eprintln!("{line}");
    })
    .expect("live run");
}

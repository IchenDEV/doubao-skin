//! One-shot live injection for manual debugging:
//!   cargo run -p skin-core --example live_once -- violet-night doubao
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn main() {
    let id = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "violet-night".into());
    let themes_dir = skin_core::theme::default_themes_dir();
    let theme = skin_core::theme::load(&themes_dir, &id).expect("load theme");
    let target = std::env::args()
        .nth(2)
        .as_deref()
        .and_then(skin_core::live::TargetApp::from_id)
        .unwrap_or(skin_core::live::TargetApp::DoubaoWork);
    let stop = Arc::new(AtomicBool::new(false));
    skin_core::live::run(&theme, target, true, stop, |line| {
        eprintln!("{line}");
    })
    .expect("live run");
}

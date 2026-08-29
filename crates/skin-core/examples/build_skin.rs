//! Offline build for manual debugging:
//!   cargo run -p skin-core --example build_skin -- violet-night
fn main() {
    let id = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "violet-night".into());
    let themes_dir = skin_core::theme::default_themes_dir();
    let theme = skin_core::theme::load(&themes_dir, &id).expect("load theme");
    let path = skin_core::build::apply(&theme, |line| eprintln!("{line}")).expect("build");
    eprintln!("built: {}", path.display());
}

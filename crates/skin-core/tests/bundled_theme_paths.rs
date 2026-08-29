use skin_core::theme::bundled_themes_dir_for_executable;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_test_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("doubao-theme-paths-{}-{stamp}", std::process::id()))
}

#[test]
fn finds_bundled_themes_from_gui_and_resource_cli_locations() {
    let root = temporary_test_dir();
    let contents = root.join("豆包主题.app/Contents");
    let themes = contents.join("Resources/themes");
    fs::create_dir_all(contents.join("MacOS")).unwrap();
    fs::create_dir_all(contents.join("Resources/bin")).unwrap();
    fs::create_dir_all(&themes).unwrap();

    assert_eq!(
        bundled_themes_dir_for_executable(&contents.join("MacOS/豆包主题")),
        Some(themes.clone())
    );
    assert_eq!(
        bundled_themes_dir_for_executable(&contents.join("Resources/bin/doubao-theme")),
        Some(themes)
    );
    fs::remove_dir_all(root).unwrap();
}

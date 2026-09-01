use skin_core::theme::bundled_themes_dir_for_executable;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_test_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("doubao-skin-paths-{}-{stamp}", std::process::id()))
}

#[test]
fn finds_bundled_themes_from_macos_gui_location() {
    let root = temporary_test_dir();
    let contents = root.join("豆皮.app/Contents");
    let themes = contents.join("Resources/themes");
    fs::create_dir_all(contents.join("MacOS")).unwrap();
    fs::create_dir_all(&themes).unwrap();

    assert_eq!(
        bundled_themes_dir_for_executable(&contents.join("MacOS/豆皮")),
        Some(themes)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn finds_bundled_themes_beside_windows_gui_executable() {
    let root = temporary_test_dir();
    let package = root.join("Doubao-Skin-Windows-x64");
    let themes = package.join("themes");
    fs::create_dir_all(&themes).unwrap();

    assert_eq!(
        bundled_themes_dir_for_executable(&package.join("doubao-skin.exe")),
        Some(themes)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn finds_bundled_themes_from_windows_helper_directory() {
    let root = temporary_test_dir();
    let package = root.join("Doubao-Skin-Windows-arm64");
    let themes = package.join("themes");
    fs::create_dir_all(package.join("helpers")).unwrap();
    fs::create_dir_all(&themes).unwrap();

    assert_eq!(
        bundled_themes_dir_for_executable(&package.join("helpers/doubao-skin-agent.exe")),
        Some(themes)
    );
    fs::remove_dir_all(root).unwrap();
}

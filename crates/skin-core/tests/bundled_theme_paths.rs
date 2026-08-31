use skin_core::authoring;
use skin_core::theme::bundled_themes_dir_for_executable;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_test_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("doubao-skin-paths-{}-{stamp}", std::process::id()))
}

fn collect_files(root: &Path, current: &Path, files: &mut BTreeSet<PathBuf>) {
    for entry in fs::read_dir(current).expect("theme tree should be readable") {
        let entry = entry.expect("theme entry should be readable");
        if entry
            .file_type()
            .expect("entry type should be readable")
            .is_dir()
        {
            collect_files(root, &entry.path(), files);
        } else {
            files.insert(
                entry
                    .path()
                    .strip_prefix(root)
                    .expect("theme file should stay under its root")
                    .to_path_buf(),
            );
        }
    }
}

#[test]
fn bundled_theme_sources_only_contain_packaged_contract_files() {
    let themes = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes");
    for entry in fs::read_dir(&themes).expect("bundled themes should be readable") {
        let entry = entry.expect("bundled theme entry should be readable");
        if !entry
            .file_type()
            .expect("entry type should be readable")
            .is_dir()
        {
            continue;
        }
        let root = entry.path();
        let report = authoring::check(&root).expect("bundled theme should validate");
        let expected = report.files.into_iter().collect::<BTreeSet<_>>();
        let mut actual = BTreeSet::new();
        collect_files(&root, &root, &mut actual);
        assert_eq!(
            actual,
            expected,
            "{} contains files that are not part of its installable contract",
            root.display()
        );
    }
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

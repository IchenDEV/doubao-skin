use skin_core::authoring::{self, Appearance, CreateOptions};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::ZipArchive;

fn temporary_test_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "doubao-skin-authoring-{label}-{}-{stamp}",
        std::process::id()
    ))
}

fn sample_options() -> CreateOptions {
    CreateOptions {
        name: "晨雾蓝".into(),
        description: "安静清透的浅蓝主题".into(),
        author: "测试作者".into(),
        accent: "#3f7de8".into(),
        appearance: Appearance::Both,
    }
}

#[test]
fn create_generates_a_strict_v2_theme_and_preview() {
    let root = temporary_test_dir("create");
    let theme_dir = root.join("morning-mist");

    let report = authoring::create(&theme_dir, &sample_options()).unwrap();

    assert_eq!(report.id, "morning-mist");
    assert!(theme_dir.join("theme.json").is_file());
    assert!(theme_dir.join("theme.css").is_file());
    let preview = image::open(theme_dir.join("preview.jpg")).unwrap();
    assert_eq!((preview.width(), preview.height()), (1200, 675));
    authoring::check(&theme_dir).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn check_rejects_id_that_does_not_match_directory() {
    let root = temporary_test_dir("mismatch");
    let theme_dir = root.join("wrong-directory");
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(
        theme_dir.join("theme.json"),
        r##"{
          "schemaVersion": 2,
          "id": "other-id",
          "name": "测试主题",
          "description": "用于严格校验的测试主题",
          "version": "1.0.0",
          "author": "测试作者",
          "preview": {"image":"preview.jpg","aspectRatio":"16:9","appearance":"dark","accent":"#8257e5"},
          "store": {"category":"pure","tags":["深色"],"sortOrder":900},
          "appearance": "dark-only"
        }"##,
    )
    .unwrap();
    fs::write(
        theme_dir.join("theme.css"),
        "html[data-skin=\"other-id\"], html[data-skin=\"other-id\"] body { --dbx-bg-body-web:#111; --s-color-bg-body:#111; --semi-color-primary:#8257e5; --semi-color-primary-hover:#936df0; --semi-color-primary-active:#7042d0; --semi-color-primary-disabled:#4c3f63; }",
    )
    .unwrap();
    image::RgbImage::new(1200, 675)
        .save(theme_dir.join("preview.jpg"))
        .unwrap();

    let error = authoring::check(&theme_dir).unwrap_err();

    assert!(error.contains("目录名"), "{error}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pack_contains_only_contract_files_and_referenced_assets() {
    let root = temporary_test_dir("pack");
    let theme_dir = root.join("morning-mist");
    authoring::create(&theme_dir, &sample_options()).unwrap();
    fs::write(theme_dir.join("notes.txt"), "not part of the package").unwrap();
    fs::write(theme_dir.join("LICENSE"), "Test license").unwrap();
    let package = root.join("morning-mist.zip");

    authoring::pack(&theme_dir, &package).unwrap();

    let source = fs::File::open(&package).unwrap();
    let mut archive = ZipArchive::new(source).unwrap();
    let mut names = (0..archive.len())
        .map(|index| archive.by_index(index).unwrap().name().to_string())
        .collect::<Vec<_>>();
    names.sort();
    assert_eq!(
        names,
        vec![
            "morning-mist/LICENSE",
            "morning-mist/preview.jpg",
            "morning-mist/theme.css",
            "morning-mist/theme.json",
        ]
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn create_and_pack_do_not_overwrite_existing_content() {
    let root = temporary_test_dir("overwrite");
    let theme_dir = root.join("occupied-theme");
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(theme_dir.join("keep.txt"), "keep me").unwrap();
    let create_error = authoring::create(&theme_dir, &sample_options()).unwrap_err();
    assert!(create_error.contains("不会覆盖"), "{create_error}");
    assert_eq!(
        fs::read_to_string(theme_dir.join("keep.txt")).unwrap(),
        "keep me"
    );

    let valid_dir = root.join("morning-mist");
    authoring::create(&valid_dir, &sample_options()).unwrap();
    let package = root.join("existing.doubao-skin.zip");
    fs::write(&package, "keep package").unwrap();
    let pack_error = authoring::pack(&valid_dir, &package).unwrap_err();
    assert!(pack_error.contains("已经存在"), "{pack_error}");
    assert_eq!(fs::read_to_string(&package).unwrap(), "keep package");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preview_is_regenerated_from_the_current_theme_style() {
    let root = temporary_test_dir("preview");
    let theme_dir = root.join("morning-mist");
    authoring::create(&theme_dir, &sample_options()).unwrap();
    let css_path = theme_dir.join("theme.css");
    let css = fs::read_to_string(&css_path).unwrap().replace(
        "--semi-color-primary: #3f7de8",
        "--semi-color-primary: #cc3344",
    );
    fs::write(css_path, css).unwrap();

    authoring::preview(&theme_dir).unwrap();

    let preview = image::open(theme_dir.join("preview.jpg"))
        .unwrap()
        .to_rgb8();
    let accent = preview.get_pixel(1040, 510).0;
    assert!(
        accent[0] > 190 && accent[1] < 80 && accent[2] < 100,
        "{accent:?}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn check_rejects_symlinks_anywhere_in_the_theme_tree() {
    use std::os::unix::fs::symlink;

    let root = temporary_test_dir("symlink");
    let theme_dir = root.join("morning-mist");
    authoring::create(&theme_dir, &sample_options()).unwrap();
    symlink(theme_dir.join("theme.css"), theme_dir.join("linked.css")).unwrap();

    let error = authoring::check(&theme_dir).unwrap_err();

    assert!(error.contains("符号链接"), "{error}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn check_reports_missing_assets_bad_scope_and_incomplete_both_variants() {
    let root = temporary_test_dir("strict");

    let missing_dir = root.join("missing-asset");
    authoring::create(&missing_dir, &sample_options()).unwrap();
    fs::remove_file(missing_dir.join("preview.jpg")).unwrap();
    let missing = authoring::check(&missing_dir).unwrap_err();
    assert!(missing.contains("不存在"), "{missing}");

    let scope_dir = root.join("bad-scope");
    authoring::create(&scope_dir, &sample_options()).unwrap();
    let css_path = scope_dir.join("theme.css");
    let css = fs::read_to_string(&css_path)
        .unwrap()
        .replace("html[data-skin=\"bad-scope\"] body", "body");
    fs::write(css_path, css).unwrap();
    let scope = authoring::check(&scope_dir).unwrap_err();
    assert!(scope.contains("html 和 body"), "{scope}");

    let variants_dir = root.join("bad-variants");
    authoring::create(&variants_dir, &sample_options()).unwrap();
    let manifest_path = variants_dir.join("theme.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["variants"].as_object_mut().unwrap().remove("dark");
    fs::write(
        &manifest_path,
        format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    )
    .unwrap();
    let variants = authoring::check(&variants_dir).unwrap_err();
    assert!(
        variants.contains("variants.light 和 variants.dark"),
        "{variants}"
    );

    fs::remove_dir_all(root).unwrap();
}

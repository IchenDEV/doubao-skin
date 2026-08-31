use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use skin_core::theme_package::{
    validate_theme_package, SupportDeclaration, SupportLevel, ThemePackageAppearance,
    ThemePackageErrorCategory, ThemeTarget,
};
use skin_core::{live, theme};

struct TempRoot(PathBuf);

impl TempRoot {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "doubao-skin-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temp root should be creatable");
        Self(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture(name: &str) -> Value {
    let path = repo_root()
        .join("design/theme-standard/fixtures/v3")
        .join(name);
    serde_json::from_slice(&fs::read(&path).expect("fixture should be readable"))
        .expect("fixture should be JSON")
}

fn write_file(root: &Path, relative: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture file should have a parent"))
        .expect("fixture parent should be creatable");
    fs::write(path, bytes).expect("fixture file should be writable");
}

fn write_test_image(root: &Path, relative: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture file should have a parent"))
        .expect("fixture parent should be creatable");
    image::RgbImage::from_pixel(2, 2, image::Rgb([48, 96, 144]))
        .save(path)
        .expect("fixture image should be writable");
}

fn write_manifest_package(temp: &TempRoot, manifest: &Value) -> PathBuf {
    let id = manifest["id"].as_str().expect("fixture needs an id");
    let root = temp.0.join(id);
    fs::create_dir_all(&root).expect("theme root should be creatable");
    write_file(
        &root,
        "theme.json",
        serde_json::to_vec_pretty(manifest).expect("fixture should serialize"),
    );
    write_test_image(&root, "preview.jpg");
    root
}

#[test]
fn explicit_target_keys_are_the_only_support_source() {
    for (fixture_name, supported) in [
        ("valid-workbuddy.json", vec![ThemeTarget::WorkBuddy]),
        (
            "valid-doubao-family.json",
            vec![ThemeTarget::Doubao, ThemeTarget::DoubaoWork],
        ),
        ("valid-all-targets.json", ThemeTarget::ALL.to_vec()),
    ] {
        let temp = TempRoot::new("support");
        let root = write_manifest_package(&temp, &fixture(fixture_name));
        let package = validate_theme_package(&root).expect("minimal package should validate");
        for target in ThemeTarget::ALL {
            let support = package.support(target);
            assert_eq!(support.declaration, SupportDeclaration::Explicit);
            assert_eq!(
                support.level,
                if supported.contains(&target) {
                    SupportLevel::Shared
                } else {
                    SupportLevel::Unsupported
                },
                "support mismatch for {fixture_name}/{target}"
            );
        }
    }
}

#[test]
fn structured_merge_order_and_null_deletion_are_resolved_per_target_and_appearance() {
    let temp = TempRoot::new("merge");
    let mut manifest = fixture("valid-all-targets.json");
    manifest["shared"]["icons"] = serde_json::json!({
        "main": "icons/main.svg",
        "send": "icons/send.svg"
    });
    manifest["shared"]["variants"] = serde_json::json!({
        "light": { "composer": { "background": "#fefefe" } },
        "dark": { "composer": { "background": "#111111" } }
    });
    manifest["targets"]["workbuddy"] = serde_json::json!({
        "composer": { "border": "1px solid #abcdef" },
        "icons": { "main": null },
        "variants": {
            "dark": { "composer": { "background": "#090909" } }
        }
    });
    let root = write_manifest_package(&temp, &manifest);
    write_file(
        &root,
        "icons/main.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    );
    write_file(
        &root,
        "icons/send.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    );

    let package = validate_theme_package(&root).expect("merged package should validate");
    assert_eq!(
        package.support(ThemeTarget::WorkBuddy).level,
        SupportLevel::Tailored
    );
    let light = package
        .resolve(ThemeTarget::WorkBuddy, ThemePackageAppearance::Light)
        .expect("light should resolve");
    assert_eq!(light.visual()["composer"]["background"], "#fefefe");
    assert_eq!(light.visual()["composer"]["border"], "1px solid #abcdef");
    assert!(light.visual()["icons"].get("main").is_none());
    assert_eq!(light.visual()["icons"]["send"], "icons/send.svg");

    let dark = package
        .resolve(ThemeTarget::WorkBuddy, ThemePackageAppearance::Dark)
        .expect("dark should resolve");
    assert_eq!(dark.visual()["composer"]["background"], "#090909");
}

#[test]
fn future_schema_fails_closed_before_legacy_fallback() {
    let temp = TempRoot::new("future-schema");
    let mut manifest = fixture("valid-all-targets.json");
    manifest["schemaVersion"] = Value::from(4);
    let root = write_manifest_package(&temp, &manifest);
    let error = validate_theme_package(&root).expect_err("future schema must fail");
    assert_eq!(error.category, ThemePackageErrorCategory::UnsupportedSchema);
    assert_eq!(error.pointer.as_deref(), Some("/schemaVersion"));
}

fn write_full_css_package(temp: &TempRoot) -> PathBuf {
    let mut manifest = fixture("valid-full.json");
    manifest["targets"]["workbuddy"]["preview"]["appearance"] = Value::String("dark".into());
    manifest["shared"]["variants"]["light"]["css"] = serde_json::json!(["styles/shared-light.css"]);
    manifest["targets"]["doubao"]["variants"] = serde_json::json!({
        "light": { "css": ["styles/doubao-light.css"] }
    });
    manifest["targets"]["workbuddy"]["variants"] = serde_json::json!({
        "light": { "css": ["styles/workbuddy-light.css"] }
    });
    let root = write_manifest_package(temp, &manifest);
    for resource in [
        "preview-workbuddy.jpg",
        "assets/bg.jpg",
        "assets/icons/main.png",
    ] {
        write_test_image(&root, resource);
    }
    write_file(
        &root,
        "styles/shared.css",
        r#"/* SHARED_BASE */
html[data-skin="gallery-whale-maid"] {
  --whale-soft: rgba(122, 78, 41, 0.24);
  color: #352970;
}"#,
    );
    write_file(
        &root,
        "styles/shared-light.css",
        r#"/* SHARED_LIGHT */
html[data-skin="gallery-whale-maid"] { color: #413573; }"#,
    );
    write_file(
        &root,
        "styles/doubao-family.css",
        r#"/* DOUBAO_FAMILY_ONLY */
html[data-skin="gallery-whale-maid"][data-skin-target="doubao"],
html[data-skin="gallery-whale-maid"][data-skin-target="doubao-work"] {
  --whale-accent: #7a4e29;
  border-color: var(--whale-soft);
}"#,
    );
    write_file(
        &root,
        "styles/workbuddy.css",
        r#"/* WORKBUDDY_ONLY */
@media (hover: hover) {
  html[data-skin="gallery-whale-maid"][data-skin-target="workbuddy"] .workbench-part {
    border-color: var(--whale-soft);
    box-shadow: 0 8px 24px rgba(65, 43, 50, 0.10);
  }
}"#,
    );
    write_file(
        &root,
        "styles/doubao-light.css",
        r#"/* DOUBAO_LIGHT */
html[data-skin="gallery-whale-maid"][data-skin-target="doubao"] { color: #30286b; }"#,
    );
    write_file(
        &root,
        "styles/workbuddy-light.css",
        r#"/* WORKBUDDY_LIGHT */
html[data-skin="gallery-whale-maid"][data-skin-target="workbuddy"] { color: #352970; }"#,
    );
    root
}

#[test]
fn css_ast_accepts_scoped_shared_and_target_subset_files() {
    let temp = TempRoot::new("css-valid");
    let root = write_full_css_package(&temp);
    let package = validate_theme_package(&root).expect("full CSS fixture should validate");
    let report = package.report().expect("validated package should report");
    assert_eq!(
        report.targets["workbuddy"].support_level,
        SupportLevel::Tailored
    );
    assert_eq!(
        report.targets["doubao"].css[&ThemePackageAppearance::Light],
        vec![
            "styles/shared.css",
            "styles/shared-light.css",
            "styles/doubao-family.css",
            "styles/doubao-light.css"
        ]
    );
}

#[test]
fn runtime_loads_only_the_selected_target_chain_and_switches_appearance_payloads() {
    let temp = TempRoot::new("runtime-v3");
    let root = write_full_css_package(&temp);
    write_file(&root, "theme.css", "/* UNREFERENCED_ROOT_THEME_CSS */");

    let theme = theme::load(&temp.0, root.to_str().expect("temp path should be UTF-8"))
        .expect("v3 runtime theme should load");
    assert_eq!(theme.schema_version, 3);
    for target in live::TargetApp::ALL {
        assert!(theme.supports_target(target));
    }
    assert!(theme
        .preview_image_for(live::TargetApp::Doubao)
        .is_some_and(|path| path.ends_with("preview.jpg")));
    assert!(theme
        .preview_image_for(live::TargetApp::WorkBuddy)
        .is_some_and(|path| path.ends_with("preview-workbuddy.jpg")));
    assert_eq!(
        theme.preview_style_for(live::TargetApp::Doubao).text.rgb,
        0x352970
    );
    assert_eq!(
        theme.preview_style_for(live::TargetApp::WorkBuddy).text.rgb,
        0xf7f8fa
    );

    let workbuddy = theme.live_js_for(live::TargetApp::WorkBuddy);
    assert!(workbuddy.contains("__doubaoSkinByAppearance"));
    assert!(workbuddy.contains("MODE=\"auto\""));
    assert!(workbuddy.contains("WORKBUDDY_ONLY"));
    assert!(workbuddy.contains("SHARED_LIGHT"));
    assert!(workbuddy.contains("WORKBUDDY_LIGHT"));
    assert!(workbuddy.contains("color-scheme:light"));
    assert!(workbuddy.contains("color-scheme:dark"));
    assert!(!workbuddy.contains("DOUBAO_FAMILY_ONLY"));
    assert!(!workbuddy.contains("UNREFERENCED_ROOT_THEME_CSS"));

    let doubao = theme.live_js_for(live::TargetApp::Doubao);
    assert!(doubao.contains("DOUBAO_FAMILY_ONLY"));
    assert!(!doubao.contains("WORKBUDDY_ONLY"));
    assert!(!doubao.contains("UNREFERENCED_ROOT_THEME_CSS"));

    let shared_variant = doubao.find("SHARED_LIGHT").unwrap();
    let engine = doubao[..shared_variant]
        .rfind("--s-color-text-primary")
        .unwrap();
    let shared = doubao[..shared_variant].rfind("SHARED_BASE").unwrap();
    let target = shared_variant + doubao[shared_variant..].find("DOUBAO_FAMILY_ONLY").unwrap();
    let target_variant = target + doubao[target..].find("DOUBAO_LIGHT").unwrap();
    let runtime = target_variant
        + doubao[target_variant..]
            .find("prefers-reduced-motion")
            .unwrap();
    assert!(engine < shared);
    assert!(shared < shared_variant);
    assert!(shared_variant < target);
    assert!(target < target_variant);
    assert!(target_variant < runtime);
}

#[test]
fn legacy_support_and_workbuddy_css_compatibility_are_fixed() {
    for (schema_version, workbuddy_supported) in [(1, false), (2, true)] {
        let temp = TempRoot::new("legacy");
        let id = format!("legacy-v{schema_version}");
        let root = temp.0.join(&id);
        fs::create_dir_all(&root).unwrap();
        write_file(&root, "theme.css", "html { color: red; }");
        write_file(
            &root,
            "theme.json",
            serde_json::to_vec(&serde_json::json!({
                "schemaVersion": schema_version,
                "id": id,
                "name": "Legacy",
                "appearance": "both"
            }))
            .unwrap(),
        );
        let package = validate_theme_package(&root).unwrap();
        assert_eq!(
            package.support(ThemeTarget::WorkBuddy).is_supported(),
            workbuddy_supported
        );
        assert_eq!(
            package.support(ThemeTarget::WorkBuddy).declaration,
            SupportDeclaration::LegacyInferred
        );
        if workbuddy_supported {
            assert!(package
                .resolve(ThemeTarget::WorkBuddy, ThemePackageAppearance::Light)
                .unwrap()
                .css_files
                .is_empty());
        }
        assert_eq!(
            package
                .resolve(ThemeTarget::Doubao, ThemePackageAppearance::Dark)
                .unwrap()
                .css_files
                .len(),
            1
        );
    }
}

#[test]
fn target_appearance_can_narrow_or_expand_shared_appearance() {
    let temp = TempRoot::new("appearance");
    let mut manifest = fixture("valid-all-targets.json");
    manifest["shared"]["appearance"] = Value::String("light-only".into());
    manifest["targets"]["doubao"]["appearance"] = Value::String("dark-only".into());
    manifest["targets"]["workbuddy"]["appearance"] = Value::String("both".into());
    let root = write_manifest_package(&temp, &manifest);
    let package = validate_theme_package(&root).unwrap();
    assert_eq!(
        package.appearances(ThemeTarget::Doubao),
        vec![ThemePackageAppearance::Dark]
    );
    assert_eq!(
        package.appearances(ThemeTarget::DoubaoWork),
        vec![ThemePackageAppearance::Light]
    );
    assert_eq!(
        package.appearances(ThemeTarget::WorkBuddy),
        vec![ThemePackageAppearance::Light, ThemePackageAppearance::Dark]
    );
}

#[test]
fn fake_images_and_unsafe_svg_resources_fail_closed() {
    let temp = TempRoot::new("fake-image");
    let root = write_manifest_package(&temp, &fixture("valid-all-targets.json"));
    write_file(&root, "preview.jpg", b"not a jpeg");
    let error = validate_theme_package(&root).unwrap_err();
    assert_eq!(error.category, ThemePackageErrorCategory::Resource);

    let temp = TempRoot::new("unsafe-svg");
    let mut manifest = fixture("valid-all-targets.json");
    manifest["shared"]["icons"] = serde_json::json!({ "main": "icons/main.svg" });
    let root = write_manifest_package(&temp, &manifest);
    write_file(
        &root,
        "icons/main.svg",
        r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.com/x.png"/></svg>"#,
    );
    let error = validate_theme_package(&root).unwrap_err();
    assert_eq!(error.category, ThemePackageErrorCategory::Resource);
}

#[test]
fn exact_case_symlink_and_duplicate_effective_css_are_rejected() {
    let temp = TempRoot::new("case");
    let mut manifest = fixture("valid-all-targets.json");
    manifest["shared"]["icons"] = serde_json::json!({ "main": "icons/Main.svg" });
    let root = write_manifest_package(&temp, &manifest);
    write_file(
        &root,
        "icons/main.svg",
        r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
    );
    assert_eq!(
        validate_theme_package(&root).unwrap_err().category,
        ThemePackageErrorCategory::MissingResource
    );

    let temp = TempRoot::new("duplicate-css");
    let mut manifest = fixture("valid-all-targets.json");
    manifest["shared"]["css"] = serde_json::json!(["styles/shared.css"]);
    manifest["targets"]["doubao"]["css"] = serde_json::json!(["styles/shared.css"]);
    let root = write_manifest_package(&temp, &manifest);
    write_file(
        &root,
        "styles/shared.css",
        r#"html[data-skin="fixture-all-targets"] { color: red; }"#,
    );
    assert_eq!(
        validate_theme_package(&root).unwrap_err().category,
        ThemePackageErrorCategory::Css
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let temp = TempRoot::new("symlink");
        let mut manifest = fixture("valid-all-targets.json");
        manifest["shared"]["icons"] = serde_json::json!({ "main": "icons/main.svg" });
        let root = write_manifest_package(&temp, &manifest);
        write_file(
            &root,
            "outside.svg",
            r#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
        );
        fs::create_dir_all(root.join("icons")).unwrap();
        symlink(root.join("outside.svg"), root.join("icons/main.svg")).unwrap();
        assert_eq!(
            validate_theme_package(&root).unwrap_err().category,
            ThemePackageErrorCategory::Path
        );
    }
}

#[test]
fn css_ast_rejects_scope_property_url_reserved_and_at_rule_bypasses() {
    let cases = [
        (
            "unscoped selector",
            ".workbench-part { color: red; }",
        ),
        (
            "wrong target",
            "html[data-skin=\"gallery-whale-maid\"][data-skin-target=\"doubao\"] { color: red; }",
        ),
        (
            "escaped layout property",
            "html[data-skin=\"gallery-whale-maid\"][data-skin-target=\"workbuddy\"] { d\\69 splay: block; }",
        ),
        (
            "escaped url function",
            "html[data-skin=\"gallery-whale-maid\"][data-skin-target=\"workbuddy\"] { background: u\\72l(https://example.com/x.png); }",
        ),
        (
            "escaped reserved variable",
            "html[data-skin=\"gallery-whale-maid\"][data-skin-target=\"workbuddy\"] { --doubao-skin-runtime-\\63olor: red; }",
        ),
        (
            "import",
            "@import 'https://example.com/theme.css';",
        ),
        (
            "keyframes",
            "@keyframes pulse { from { opacity: 0; } to { opacity: 1; } }",
        ),
    ];

    for (name, css) in cases {
        let temp = TempRoot::new("css-invalid");
        let root = write_full_css_package(&temp);
        write_file(&root, "styles/workbuddy.css", css);
        let error = validate_theme_package(&root)
            .unwrap_err_or_else(|| panic!("{name} should fail CSS validation"));
        assert_eq!(error.category, ThemePackageErrorCategory::Css, "{name}");
    }
}

trait ResultExt<T, E> {
    fn unwrap_err_or_else(self, f: impl FnOnce() -> E) -> E;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn unwrap_err_or_else(self, f: impl FnOnce() -> E) -> E {
        match self {
            Ok(_) => f(),
            Err(error) => error,
        }
    }
}

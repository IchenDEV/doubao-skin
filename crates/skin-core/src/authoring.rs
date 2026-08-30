//! Theme authoring helpers used by the `doubao-skin` command-line tool.

use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use image::{Rgb, RgbImage};
use serde_json::{json, Value};
use zip::write::SimpleFileOptions;

use crate::theme;

const MAX_PACKAGE_ENTRIES: usize = 2_048;
const MAX_PACKAGE_CONTENT_BYTES: u64 = 512 * 1024 * 1024;
const REQUIRED_CSS_VARIABLES: [&str; 6] = [
    "--dbx-bg-body-web",
    "--s-color-bg-body",
    "--semi-color-primary",
    "--semi-color-primary-hover",
    "--semi-color-primary-active",
    "--semi-color-primary-disabled",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
    Both,
}

impl Appearance {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "light" | "light-only" => Ok(Self::Light),
            "dark" | "dark-only" => Ok(Self::Dark),
            "both" => Ok(Self::Both),
            _ => Err("外观必须是 light、dark 或 both".into()),
        }
    }

    fn manifest_value(self) -> &'static str {
        match self {
            Self::Light => "light-only",
            Self::Dark => "dark-only",
            Self::Both => "both",
        }
    }

    fn preview_value(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light | Self::Both => "light",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateOptions {
    pub name: String,
    pub description: String,
    pub author: String,
    pub accent: String,
    pub appearance: Appearance,
}

#[derive(Debug, Clone)]
pub struct CheckReport {
    pub id: String,
    /// Package paths relative to the theme directory, sorted for deterministic output.
    pub files: Vec<PathBuf>,
}

pub fn create(theme_dir: &Path, options: &CreateOptions) -> Result<CheckReport, String> {
    if theme_dir.exists() {
        if !theme_dir.is_dir() {
            return Err("目标位置不是文件夹".into());
        }
        let mut entries =
            fs::read_dir(theme_dir).map_err(|e| format!("无法读取目标文件夹：{e}"))?;
        if entries.next().is_some() {
            return Err("目标文件夹不是空的，不会覆盖现有内容".into());
        }
    }
    let id = theme_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "请使用主题 ID 作为文件夹名称".to_string())?;
    validate_kebab_id(id)?;
    validate_text("主题名称", &options.name, 2, 16)?;
    validate_text("主题描述", &options.description, 1, 80)?;
    validate_text("作者", &options.author, 1, 40)?;
    let accent = parse_hex_color(&options.accent)?;

    let created_dir = !theme_dir.exists();
    fs::create_dir_all(theme_dir).map_err(|e| format!("无法创建主题文件夹：{e}"))?;
    let result = (|| {
        let variants = match options.appearance {
            Appearance::Both => Some(json!({"light": {}, "dark": {}})),
            Appearance::Light | Appearance::Dark => None,
        };
        let mut manifest = json!({
            "$schema": "../../design/theme-standard/theme-v2.schema.json",
            "schemaVersion": 2,
            "id": id,
            "name": options.name.trim(),
            "description": options.description.trim(),
            "version": "1.0.0",
            "author": options.author.trim(),
            "preview": {
                "image": "preview.jpg",
                "aspectRatio": "16:9",
                "appearance": options.appearance.preview_value(),
                "accent": normalize_hex(&options.accent),
            },
            "store": {
                "category": "pure",
                "tags": [if options.appearance == Appearance::Dark { "深色" } else { "浅色" }],
                "sortOrder": 900,
            },
            "appearance": options.appearance.manifest_value(),
            "typography": {
                "ui": "-apple-system, BlinkMacSystemFont, \"PingFang SC\", sans-serif",
                "body": "-apple-system, BlinkMacSystemFont, \"PingFang SC\", sans-serif",
                "code": "\"SFMono-Regular\", Menlo, monospace",
                "scale": 1,
                "lineHeight": 1.6,
            },
            "layout": {
                "density": "comfortable",
                "sidebarWidth": 252,
                "chatMaxWidth": 920,
                "composerMaxWidth": 760,
                "selfMessageMaxWidth": 420,
                "chatMargin": 28,
            },
            "effects": {
                "radiusScale": 1,
                "motion": "gentle",
                "transitionMs": 180,
            }
        });
        if let Some(variants) = variants {
            manifest["variants"] = variants;
        }
        let manifest_text = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("无法生成 theme.json：{e}"))?;
        fs::write(theme_dir.join("theme.json"), format!("{manifest_text}\n"))
            .map_err(|e| format!("无法写入 theme.json：{e}"))?;
        fs::write(theme_dir.join("theme.css"), generated_css(id, accent))
            .map_err(|e| format!("无法写入 theme.css：{e}"))?;
        render_preview(
            theme_dir.join("preview.jpg").as_path(),
            accent,
            options.appearance,
        )?;
        check(theme_dir)?;
        preview(theme_dir)?;
        check(theme_dir)
    })();
    if result.is_err() && created_dir {
        let _ = fs::remove_dir_all(theme_dir);
    }
    result
}

pub fn check(theme_dir: &Path) -> Result<CheckReport, String> {
    if !theme_dir.is_dir() {
        return Err("主题文件夹不存在".into());
    }
    scan_tree_safety(theme_dir)?;
    let manifest_path = theme_dir.join("theme.json");
    let manifest_text =
        fs::read_to_string(&manifest_path).map_err(|e| format!("无法读取 theme.json：{e}"))?;
    let manifest: Value =
        serde_json::from_str(&manifest_text).map_err(|e| format!("theme.json 格式错误：{e}"))?;
    let object = manifest
        .as_object()
        .ok_or_else(|| "theme.json 顶层必须是对象".to_string())?;
    if object.get("schemaVersion").and_then(Value::as_u64) != Some(2) {
        return Err("新主题必须使用 schemaVersion 2".into());
    }
    let id = required_string(object.get("id"), "id")?;
    validate_kebab_id(id)?;
    let directory_id = theme_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "无法读取主题目录名".to_string())?;
    if id != directory_id {
        return Err(format!("主题 ID 必须与目录名一致：应为 {directory_id}"));
    }
    validate_text(
        "主题名称",
        required_string(object.get("name"), "name")?,
        2,
        16,
    )?;
    validate_text(
        "主题描述",
        required_string(object.get("description"), "description")?,
        1,
        80,
    )?;
    let version = required_string(object.get("version"), "version")?;
    if !valid_semver(version) {
        return Err("主题版本必须使用 1.0.0 这样的语义化版本".into());
    }
    validate_text(
        "作者",
        required_string(object.get("author"), "author")?,
        1,
        40,
    )?;

    validate_discovery_fields(&manifest)?;
    validate_appearance(&manifest)?;

    let css_path = theme_dir.join("theme.css");
    let css = fs::read_to_string(&css_path).map_err(|e| format!("无法读取 theme.css：{e}"))?;
    let scope = format!("html[data-skin=\"{id}\"]");
    if !css.contains(&scope) || !css.contains(&format!("{scope} body")) {
        return Err("theme.css 必须同时限制在主题 html 和 body 作用域内".into());
    }
    for variable in REQUIRED_CSS_VARIABLES {
        if !css.contains(variable) {
            return Err(format!("theme.css 缺少必需变量 {variable}"));
        }
    }

    let mut files = BTreeSet::from([PathBuf::from("theme.json"), PathBuf::from("theme.css")]);
    collect_manifest_assets(&manifest, &mut files)?;
    for optional in ["icon.icns", "LICENSE", "LICENSE.md", "LICENSE.txt"] {
        if theme_dir.join(optional).exists() {
            files.insert(validate_asset(theme_dir, optional, optional)?);
        }
    }
    for relative in &files {
        validate_asset(theme_dir, relative.to_string_lossy().as_ref(), "主题资源")?;
    }
    let preview = required_string(manifest.pointer("/preview/image"), "preview.image")?;
    let preview_path = theme_dir.join(validate_relative_path(preview, "预览图")?);
    let dimensions = image::image_dimensions(&preview_path)
        .map_err(|e| format!("无法读取预览图 {}：{e}", preview_path.display()))?;
    if dimensions != (1200, 675) {
        return Err(format!(
            "预览图必须是 1200 × 675，当前为 {} × {}",
            dimensions.0, dimensions.1
        ));
    }

    theme::load(theme_dir, theme_dir.to_string_lossy().as_ref())
        .map_err(|e| format!("主题无法加载：{e}"))?;
    Ok(CheckReport {
        id: id.to_string(),
        files: files.into_iter().collect(),
    })
}

pub fn preview(theme_dir: &Path) -> Result<PathBuf, String> {
    let report = check(theme_dir)?;
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(theme_dir.join("theme.json"))
            .map_err(|e| format!("无法读取 theme.json：{e}"))?,
    )
    .map_err(|e| format!("theme.json 格式错误：{e}"))?;
    let relative = required_string(manifest.pointer("/preview/image"), "preview.image")?;
    let path = theme_dir.join(validate_relative_path(relative, "预览图")?);
    if !report.files.iter().any(|file| file == Path::new(relative)) {
        return Err("预览图没有包含在主题包资源中".into());
    }
    let loaded = theme::load(theme_dir, theme_dir.to_string_lossy().as_ref())
        .map_err(|e| format!("主题无法加载：{e}"))?;
    render_theme_preview(&path, &loaded.preview_style())?;
    Ok(path)
}

pub fn pack(theme_dir: &Path, output: &Path) -> Result<CheckReport, String> {
    if output.exists() {
        return Err(format!("输出文件已经存在，不会覆盖：{}", output.display()));
    }
    let report = check(theme_dir)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("无法创建输出文件夹：{e}"))?;
    }
    let destination = fs::File::create(output).map_err(|e| format!("无法创建主题包：{e}"))?;
    let mut archive = zip::ZipWriter::new(destination);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    let mut total = 0u64;
    let result = (|| {
        for relative in &report.files {
            let source_path = theme_dir.join(relative);
            let mut source = fs::File::open(&source_path)
                .map_err(|e| format!("无法读取 {}：{e}", relative.display()))?;
            let size = source
                .metadata()
                .map_err(|e| format!("无法读取主题资源信息：{e}"))?
                .len();
            total = total.saturating_add(size);
            if total > MAX_PACKAGE_CONTENT_BYTES {
                return Err("主题包内容过大".into());
            }
            let archive_name = format!("{}/{}", report.id, path_for_zip(relative)?);
            archive
                .start_file(archive_name, options)
                .map_err(|e| format!("无法写入主题包：{e}"))?;
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let count = source
                    .read(&mut buffer)
                    .map_err(|e| format!("无法读取主题资源：{e}"))?;
                if count == 0 {
                    break;
                }
                archive
                    .write_all(&buffer[..count])
                    .map_err(|e| format!("无法写入主题包：{e}"))?;
            }
        }
        let destination = archive
            .finish()
            .map_err(|e| format!("无法完成主题包：{e}"))?;
        let package_size = destination
            .metadata()
            .map_err(|e| format!("无法读取已生成主题包：{e}"))?
            .len();
        validate_package_size(package_size)?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(output);
        return Err(error);
    }
    Ok(report)
}

fn validate_package_size(package_size: u64) -> Result<(), String> {
    if package_size > theme::MAX_PACKAGE_BYTES {
        return Err("主题包过大，压缩后最多支持 200 MiB".into());
    }
    Ok(())
}

fn validate_discovery_fields(manifest: &Value) -> Result<(), String> {
    let preview = manifest
        .get("preview")
        .and_then(Value::as_object)
        .ok_or_else(|| "theme.json 缺少 preview".to_string())?;
    required_string(preview.get("image"), "preview.image")?;
    if preview.get("aspectRatio").and_then(Value::as_str) != Some("16:9") {
        return Err("preview.aspectRatio 必须是 16:9".into());
    }
    if !matches!(
        preview.get("appearance").and_then(Value::as_str),
        Some("light" | "dark")
    ) {
        return Err("preview.appearance 必须是 light 或 dark".into());
    }
    parse_hex_color(required_string(preview.get("accent"), "preview.accent")?)?;

    let store = manifest
        .get("store")
        .and_then(Value::as_object)
        .ok_or_else(|| "theme.json 缺少 store".to_string())?;
    if !matches!(
        store.get("category").and_then(Value::as_str),
        Some("pure" | "atmosphere" | "gallery" | "codex" | "brand" | "misc")
    ) {
        return Err("store.category 无效".into());
    }
    let tags = store
        .get("tags")
        .and_then(Value::as_array)
        .ok_or_else(|| "store.tags 必须是数组".to_string())?;
    if tags.is_empty() || tags.len() > 8 {
        return Err("store.tags 需要 1 到 8 个标签".into());
    }
    let mut unique = BTreeSet::new();
    for tag in tags {
        let tag = tag
            .as_str()
            .ok_or_else(|| "主题标签必须是文字".to_string())?;
        validate_text("主题标签", tag, 1, 16)?;
        if !unique.insert(tag) {
            return Err("store.tags 不能包含重复标签".into());
        }
    }
    if !matches!(
        store.get("sortOrder").and_then(Value::as_u64),
        Some(0..=9_999)
    ) {
        return Err("store.sortOrder 必须是 0 到 9999 的整数".into());
    }
    Ok(())
}

fn validate_appearance(manifest: &Value) -> Result<(), String> {
    let appearance = required_string(manifest.get("appearance"), "appearance")?;
    if !matches!(appearance, "light-only" | "dark-only" | "both") {
        return Err("appearance 必须是 light-only、dark-only 或 both".into());
    }
    if appearance == "both" {
        let variants = manifest
            .get("variants")
            .and_then(Value::as_object)
            .ok_or_else(|| "appearance 为 both 时必须提供 variants".to_string())?;
        if !variants.get("light").is_some_and(Value::is_object)
            || !variants.get("dark").is_some_and(Value::is_object)
        {
            return Err("appearance 为 both 时必须提供 variants.light 和 variants.dark".into());
        }
    }
    Ok(())
}

fn collect_manifest_assets(manifest: &Value, files: &mut BTreeSet<PathBuf>) -> Result<(), String> {
    insert_pointer_asset(manifest, "/preview/image", "预览图", files)?;
    if let Some(background) = manifest.get("background") {
        if let Some(path) = background.as_str() {
            files.insert(validate_relative_path(path, "背景图")?);
        } else if background.is_object() {
            insert_pointer_asset(manifest, "/background/src", "背景资源", files)?;
            insert_pointer_asset(manifest, "/background/poster", "背景封面", files)?;
        }
    }
    if let Some(assets) = manifest
        .pointer("/typography/assets")
        .and_then(Value::as_array)
    {
        for (index, asset) in assets.iter().enumerate() {
            if let Some(path) = asset.get("src").and_then(Value::as_str) {
                files.insert(validate_relative_path(path, &format!("字体资源 {index}"))?);
            }
        }
    }
    collect_icon_assets(manifest.get("icons"), "图标", files)?;
    collect_icon_assets(manifest.pointer("/variants/light/icons"), "浅色图标", files)?;
    collect_icon_assets(manifest.pointer("/variants/dark/icons"), "深色图标", files)?;
    Ok(())
}

fn insert_pointer_asset(
    manifest: &Value,
    pointer: &str,
    label: &str,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    if let Some(value) = manifest.pointer(pointer) {
        let path = value
            .as_str()
            .ok_or_else(|| format!("{label}路径必须是文字"))?;
        files.insert(validate_relative_path(path, label)?);
    }
    Ok(())
}

fn collect_icon_assets(
    value: Option<&Value>,
    label: &str,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    let Some(icons) = value.and_then(Value::as_object) else {
        return Ok(());
    };
    for (slot, value) in icons {
        let path = value
            .as_str()
            .ok_or_else(|| format!("{label} {slot} 路径必须是文字"))?;
        files.insert(validate_relative_path(path, &format!("{label} {slot}"))?);
    }
    Ok(())
}

fn validate_asset(theme_dir: &Path, relative: &str, label: &str) -> Result<PathBuf, String> {
    let relative = validate_relative_path(relative, label)?;
    let path = theme_dir.join(&relative);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|_| format!("{label}不存在：{}", relative.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("{label}不能是符号链接：{}", relative.display()));
    }
    if !metadata.is_file() {
        return Err(format!("{label}必须是文件：{}", relative.display()));
    }
    let root = theme_dir
        .canonicalize()
        .map_err(|e| format!("无法读取主题文件夹：{e}"))?;
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("无法读取{label}：{e}"))?;
    if !canonical.starts_with(&root) {
        return Err(format!("{label}超出主题文件夹"));
    }
    Ok(relative)
}

fn validate_relative_path(value: &str, label: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    let valid = !value.is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if valid {
        Ok(path.to_path_buf())
    } else {
        Err(format!("{label}包含不安全的路径"))
    }
}

fn scan_tree_safety(theme_dir: &Path) -> Result<(), String> {
    fn visit(path: &Path, count: &mut usize) -> Result<(), String> {
        for entry in fs::read_dir(path).map_err(|e| format!("无法读取主题文件夹：{e}"))? {
            let entry = entry.map_err(|e| format!("无法读取主题文件：{e}"))?;
            *count += 1;
            if *count > MAX_PACKAGE_ENTRIES {
                return Err("主题包含的文件过多".into());
            }
            let kind = entry
                .file_type()
                .map_err(|e| format!("无法读取主题文件：{e}"))?;
            if kind.is_symlink() {
                return Err(format!("主题不能包含符号链接：{}", entry.path().display()));
            }
            if kind.is_dir() {
                visit(&entry.path(), count)?;
            }
        }
        Ok(())
    }
    let mut count = 0;
    visit(theme_dir, &mut count)
}

fn required_string<'a>(value: Option<&'a Value>, field: &str) -> Result<&'a str, String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("theme.json 缺少 {field}"))
}

fn validate_text(label: &str, value: &str, minimum: usize, maximum: usize) -> Result<(), String> {
    let length = value.trim().chars().count();
    if (minimum..=maximum).contains(&length) {
        Ok(())
    } else {
        Err(format!(
            "{label}长度需要在 {minimum} 到 {maximum} 个字符之间"
        ))
    }
}

fn validate_kebab_id(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        });
    if valid {
        Ok(())
    } else {
        Err("主题 ID 必须使用小写英文、数字和短横线（kebab-case）".into())
    }
}

fn valid_semver(value: &str) -> bool {
    let core_end = value.find(['-', '+']).unwrap_or(value.len());
    let (core, suffix) = value.split_at(core_end);
    let parts = core.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
        && (suffix.is_empty()
            || (suffix.len() > 1
                && matches!(suffix.as_bytes()[0], b'-' | b'+')
                && suffix[1..]
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-'))))
}

fn parse_hex_color(value: &str) -> Result<[u8; 3], String> {
    let hex = value
        .trim()
        .strip_prefix('#')
        .ok_or_else(|| "强调色必须是 #RRGGBB".to_string())?;
    if hex.len() != 6 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("强调色必须是 #RRGGBB".into());
    }
    Ok([
        u8::from_str_radix(&hex[0..2], 16).map_err(|_| "强调色无效".to_string())?,
        u8::from_str_radix(&hex[2..4], 16).map_err(|_| "强调色无效".to_string())?,
        u8::from_str_radix(&hex[4..6], 16).map_err(|_| "强调色无效".to_string())?,
    ])
}

fn normalize_hex(value: &str) -> String {
    format!(
        "#{}",
        value.trim().trim_start_matches('#').to_ascii_lowercase()
    )
}

fn mix(color: [u8; 3], target: [u8; 3], amount: f32) -> [u8; 3] {
    std::array::from_fn(|index| {
        (color[index] as f32 * (1.0 - amount) + target[index] as f32 * amount).round() as u8
    })
}

fn hex(color: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2])
}

fn generated_css(id: &str, accent: [u8; 3]) -> String {
    let hover = hex(mix(accent, [255, 255, 255], 0.14));
    let active = hex(mix(accent, [0, 0, 0], 0.16));
    let disabled = hex(mix(accent, [128, 128, 128], 0.55));
    format!(
        "html[data-skin=\"{id}\"],\nhtml[data-skin=\"{id}\"] body {{\n  --dbx-bg-body-web: #f5f7fb;\n  --s-color-bg-body: #ffffff;\n  --semi-color-primary: {};\n  --semi-color-primary-hover: {hover};\n  --semi-color-primary-active: {active};\n  --semi-color-primary-disabled: {disabled};\n}}\n\nhtml[data-skin=\"{id}\"][data-theme=\"dark\"],\nhtml[data-skin=\"{id}\"][data-theme=\"dark\"] body {{\n  --dbx-bg-body-web: #17191f;\n  --s-color-bg-body: #111318;\n}}\n",
        hex(accent)
    )
}

fn render_preview(path: &Path, accent: [u8; 3], appearance: Appearance) -> Result<(), String> {
    let dark = appearance == Appearance::Dark;
    let page = if dark { [17, 19, 24] } else { [250, 251, 253] };
    let sidebar = if dark { [27, 30, 37] } else { [240, 243, 248] };
    let surface = if dark { [34, 37, 45] } else { [255, 255, 255] };
    let muted = if dark { [75, 80, 94] } else { [218, 223, 232] };
    let text = if dark { [229, 232, 238] } else { [58, 63, 73] };
    let mut image = RgbImage::from_pixel(1200, 675, Rgb(page));
    fill_rect(&mut image, 0, 0, 252, 675, sidebar);
    fill_rect(&mut image, 284, 54, 850, 54, surface);
    fill_rect(&mut image, 284, 132, 850, 426, surface);
    for row in 0..5 {
        let y = 86 + row * 54;
        fill_rect(
            &mut image,
            30,
            y,
            154,
            12,
            if row == 1 { accent } else { muted },
        );
    }
    for row in 0..5 {
        let y = 176 + row * 66;
        fill_rect(&mut image, 326, y, 420 + (row % 2) * 160, 12, text);
        fill_rect(&mut image, 326, y + 25, 300 + (row % 3) * 95, 9, muted);
    }
    fill_rect(
        &mut image,
        326,
        498,
        760,
        42,
        if dark { [45, 49, 59] } else { [245, 247, 251] },
    );
    fill_rect(&mut image, 1038, 505, 34, 28, accent);
    image.save(path).map_err(|e| format!("无法生成预览图：{e}"))
}

fn render_theme_preview(path: &Path, style: &theme::PreviewStyle) -> Result<(), String> {
    let mut image = if let Some(background) = style.background.as_ref() {
        image::open(background)
            .map_err(|e| format!("无法读取预览背景 {}：{e}", background.display()))?
            .resize_to_fill(1200, 675, image::imageops::FilterType::Lanczos3)
            .to_rgb8()
    } else {
        RgbImage::from_pixel(1200, 675, Rgb(rgb_components(style.background_base)))
    };
    blend_rect(&mut image, 0, 0, 1200, 675, style.colors.main);
    let sidebar_width = style.sidebar_width.round().clamp(180.0, 360.0) as u32;
    blend_rect(&mut image, 0, 0, sidebar_width, 675, style.colors.sidebar);
    blend_rect(&mut image, 284, 54, 850, 54, style.surface);
    blend_rect(&mut image, 284, 132, 850, 426, style.surface);
    let muted = theme::PreviewColor {
        rgb: rgb_value(mix(
            rgb_components(style.text.rgb),
            rgb_components(style.colors.main.rgb),
            0.68,
        )),
        alpha: style.text.alpha,
    };
    for row in 0..5 {
        let y = 86 + row * 54;
        blend_rect(
            &mut image,
            30,
            y,
            154,
            12,
            if row == 1 { style.colors.accent } else { muted },
        );
    }
    for row in 0..5 {
        let y = 176 + row * 66;
        blend_rect(&mut image, 326, y, 420 + (row % 2) * 160, 12, style.text);
        blend_rect(&mut image, 326, y + 25, 300 + (row % 3) * 95, 9, muted);
    }
    blend_rect(&mut image, 326, 498, 760, 42, style.input);
    blend_rect(&mut image, 1038, 505, 34, 28, style.colors.accent);
    image.save(path).map_err(|e| format!("无法生成预览图：{e}"))
}

fn fill_rect(image: &mut RgbImage, x: u32, y: u32, width: u32, height: u32, color: [u8; 3]) {
    let max_x = (x + width).min(image.width());
    let max_y = (y + height).min(image.height());
    for row in y..max_y {
        for column in x..max_x {
            image.put_pixel(column, row, Rgb(color));
        }
    }
}

fn blend_rect(
    image: &mut RgbImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: theme::PreviewColor,
) {
    let source = rgb_components(color.rgb);
    let alpha = color.alpha.clamp(0.0, 1.0);
    let max_x = (x + width).min(image.width());
    let max_y = (y + height).min(image.height());
    for row in y..max_y {
        for column in x..max_x {
            let destination = image.get_pixel(column, row).0;
            let blended = std::array::from_fn(|index| {
                (source[index] as f32 * alpha + destination[index] as f32 * (1.0 - alpha)).round()
                    as u8
            });
            image.put_pixel(column, row, Rgb(blended));
        }
    }
}

fn rgb_components(rgb: u32) -> [u8; 3] {
    [
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    ]
}

fn rgb_value(components: [u8; 3]) -> u32 {
    ((components[0] as u32) << 16) | ((components[1] as u32) << 8) | components[2] as u32
}

fn path_for_zip(path: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    for component in path.components() {
        let Component::Normal(value) = component else {
            return Err("主题包包含不安全的路径".into());
        };
        parts.push(
            value
                .to_str()
                .ok_or_else(|| "主题资源路径必须使用 UTF-8".to_string())?,
        );
    }
    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_archive_must_fit_the_installer_limit() {
        assert!(validate_package_size(theme::MAX_PACKAGE_BYTES).is_ok());
        assert!(validate_package_size(theme::MAX_PACKAGE_BYTES + 1).is_err());
    }
}

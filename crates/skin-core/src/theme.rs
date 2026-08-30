//! Theme loading and injection-snippet generation.
//!
//! A theme is a directory containing:
//!   theme.json  {"id": "violet-night", "name": "...", "description": "..."}
//!   theme.css   CSS rules injected into every embedded page, scoped to
//!               html[data-skin][data-theme=dark] (see the bundled themes).
//!   icon.icns   (optional) replaces the app icon of the skin build

use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::live;

/// Default location of the bundled themes: `<repo>/themes`, resolved at
/// compile time from this crate (`crates/skin-core` -> repo root).
/// Override with the `DOUBAO_SKIN_THEMES_DIR` environment variable.
pub fn default_themes_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("DOUBAO_SKIN_THEMES_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(bundled) = bundled_themes_dir_for_executable(&executable) {
            return bundled;
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../themes")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes"))
}

/// Finds themes bundled beside a Windows executable or under macOS Resources.
pub fn bundled_themes_dir_for_executable(executable: &Path) -> Option<PathBuf> {
    let parent = executable.parent()?;
    let container = parent.parent()?;
    [
        parent.join("themes"),
        container.join("Resources/themes"),
        container.join("themes"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_dir())
}

pub const DEFAULT_THEME_STORE_URL: &str = "https://doubao-skin.idevlab.dev/themes/catalog.json";

const MAX_CATALOG_BYTES: u64 = 5 * 1024 * 1024;
pub(crate) const MAX_PACKAGE_BYTES: u64 = 200 * 1024 * 1024;
const MAX_PACKAGE_CONTENT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_PACKAGE_ENTRIES: usize = 2_048;

fn product_directory(base: Option<PathBuf>) -> PathBuf {
    base.unwrap_or_else(std::env::temp_dir).join("Doubao Skin")
}

pub fn app_data_dir() -> PathBuf {
    std::env::var_os("DOUBAO_SKIN_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| product_directory(dirs::data_local_dir()))
}

/// User-installed themes live outside the signed app bundle so updates do not
/// remove them. Tests and local builds can override this path explicitly.
pub fn user_themes_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("DOUBAO_SKIN_USER_THEMES_DIR") {
        return PathBuf::from(dir);
    }
    app_data_dir().join("themes")
}

pub fn theme_store_cache_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("DOUBAO_SKIN_THEME_STORE_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    product_directory(dirs::cache_dir()).join("theme-store")
}

pub fn theme_store_url() -> String {
    std::env::var("DOUBAO_SKIN_THEME_STORE_URL")
        .unwrap_or_else(|_| DEFAULT_THEME_STORE_URL.to_string())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreCatalog {
    pub schema_version: u32,
    pub themes: Vec<StoreTheme>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreTheme {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub package_url: String,
    pub sha256: String,
    #[serde(default)]
    pub package_size: Option<u64>,
    #[serde(default)]
    pub preview_url: Option<String>,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
}

/// Marker bytes used to detect pages that already carry an injection.
pub const MARKER: &[u8] = b"data-skin";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    Auto,
}

impl ThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeAppearance {
    LightOnly,
    #[default]
    DarkOnly,
    Both,
}

impl ThemeAppearance {
    fn from_legacy_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::LightOnly,
            ThemeMode::Dark => Self::DarkOnly,
            ThemeMode::Auto => Self::Both,
        }
    }

    fn mode(self) -> ThemeMode {
        match self {
            Self::LightOnly => ThemeMode::Light,
            Self::DarkOnly => ThemeMode::Dark,
            Self::Both => ThemeMode::Auto,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Typography {
    pub ui: Option<String>,
    pub body: Option<String>,
    pub code: Option<String>,
    pub scale: Option<f32>,
    pub line_height: Option<f32>,
    pub assets: Vec<FontAsset>,
}

#[derive(Debug, Clone)]
pub struct FontAsset {
    pub family: String,
    pub path: PathBuf,
    pub weight: String,
    pub style: String,
}

#[derive(Debug, Clone, Default)]
pub struct ThemeLayout {
    pub density: Option<String>,
    pub sidebar_width: Option<f32>,
    pub chat_max_width: Option<f32>,
    pub composer_max_width: Option<f32>,
    pub self_message_max_width: Option<f32>,
    pub chat_margin: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct ComposerStyle {
    pub background: Option<String>,
    pub border: Option<String>,
    pub text_color: Option<String>,
    pub placeholder_color: Option<String>,
    pub caret_color: Option<String>,
    pub icon_color: Option<String>,
    pub send_button_background: Option<String>,
    pub send_button_icon_color: Option<String>,
    pub radius: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub padding: Option<f32>,
    pub gap: Option<f32>,
    pub icon_size: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct ContentStyle {
    pub chat_background: Option<String>,
    pub user_message_background: Option<String>,
    pub user_message_text: Option<String>,
    pub assistant_message_background: Option<String>,
    pub assistant_message_text: Option<String>,
    pub code_background: Option<String>,
    pub code_header_background: Option<String>,
    pub selection_color: Option<String>,
    pub scrollbar_color: Option<String>,
    pub scrollbar_hover_color: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ThemeIcons {
    pub main: Option<PathBuf>,
    pub new_task: Option<PathBuf>,
    pub scheduled: Option<PathBuf>,
    pub skills: Option<PathBuf>,
    pub cloud: Option<PathBuf>,
    pub remote: Option<PathBuf>,
    pub conversation: Option<PathBuf>,
    pub project: Option<PathBuf>,
    pub confirm: Option<PathBuf>,
    pub connector: Option<PathBuf>,
    pub send: Option<PathBuf>,
    pub stop: Option<PathBuf>,
    pub attach: Option<PathBuf>,
    pub voice: Option<PathBuf>,
    pub tools: Option<PathBuf>,
    pub knowledge: Option<PathBuf>,
    pub more_skills: Option<PathBuf>,
    pub daily_work: Option<PathBuf>,
    pub content_creation: Option<PathBuf>,
    pub research: Option<PathBuf>,
    pub design: Option<PathBuf>,
    pub read_aloud: Option<PathBuf>,
    pub copy: Option<PathBuf>,
    pub sidebar: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct AppearanceVariant {
    pub composer: ComposerStyle,
    pub content: ContentStyle,
    pub icons: ThemeIcons,
}

#[derive(Debug, Clone, Default)]
pub struct ThemeVariants {
    pub light: Option<AppearanceVariant>,
    pub dark: Option<AppearanceVariant>,
}

#[derive(Debug, Clone, Default)]
pub struct ThemeEffects {
    pub radius_scale: Option<f32>,
    pub shadow: Option<String>,
    pub blur: Option<f32>,
    pub motion: Option<String>,
    pub transition_ms: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundKind {
    Image,
    Video,
    Gradient,
}

#[derive(Debug, Clone)]
pub struct BackgroundSpec {
    pub kind: BackgroundKind,
    pub source: Option<PathBuf>,
    pub poster: Option<PathBuf>,
    pub gradient: Option<String>,
    pub fit: String,
    pub position: String,
    pub opacity: f32,
    pub veil: f32,
    pub blur: f32,
    pub animation: String,
    pub duration_seconds: f32,
    pub legacy: bool,
}

/// Colors for the UI's mini theme preview, parsed from theme.css by variable
/// name (not by position).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewColor {
    pub rgb: u32,
    pub alpha: f32,
}

impl PreviewColor {
    pub const fn opaque(rgb: u32) -> Self {
        Self { rgb, alpha: 1.0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PreviewColors {
    /// sidebar strip: --dbx-bg-body-web, fallback --N50
    pub sidebar: PreviewColor,
    /// main content: --s-color-bg-body, fallback --N00
    pub main: PreviewColor,
    /// accent dot/button: --semi-color-primary, fallback --B500
    pub accent: PreviewColor,
}

#[derive(Debug, Clone)]
pub struct PreviewStyle {
    pub colors: PreviewColors,
    pub surface: PreviewColor,
    pub text: PreviewColor,
    pub input: PreviewColor,
    pub input_border: PreviewColor,
    pub composer_text: PreviewColor,
    pub composer_placeholder: PreviewColor,
    pub composer_icon: PreviewColor,
    pub composer_radius: f32,
    pub composer_min_height: f32,
    pub composer_padding: f32,
    pub composer_gap: f32,
    pub composer_icon_size: f32,
    pub font_family: String,
    pub density: String,
    pub sidebar_width: f32,
    pub chat_margin: f32,
    pub radius_scale: f32,
    pub icons: ThemeIcons,
    pub background: Option<PathBuf>,
    pub background_opacity: f32,
    pub background_veil: f32,
    pub background_base: u32,
    pub background_fit: String,
    pub animated: bool,
    pub has_background: bool,
    pub surface_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceOpacityProfile {
    pub surface: f32,
    pub page: f32,
    pub sidebar: f32,
    pub layer: f32,
    pub input: f32,
    pub preview_page: f32,
    pub preview_sidebar: f32,
}

pub fn composite_alpha(bottom: f32, top: f32) -> f32 {
    1.0 - (1.0 - bottom.clamp(0.0, 1.0)) * (1.0 - top.clamp(0.0, 1.0))
}

pub fn surface_opacity_profile(surface: f32) -> SurfaceOpacityProfile {
    let surface = surface.clamp(0.35, 1.0);
    let page = (surface * 0.55).clamp(0.18, 0.72);
    let sidebar = (surface * 0.75).clamp(0.24, 0.86);
    let layer = (surface * 0.65).clamp(0.18, 0.78);
    let input = (surface + 0.08).clamp(0.43, 1.0);
    SurfaceOpacityProfile {
        surface,
        page,
        sidebar,
        layer,
        input,
        preview_page: composite_alpha(page, page),
        preview_sidebar: composite_alpha(page, sidebar),
    }
}

impl Theme {
    /// Preview colors for the UI. Themes without a color ramp (pure-dark)
    /// fall back to neutral dark grays + a blue accent (#3370eb).
    pub fn preview_colors(&self) -> PreviewColors {
        let variant = self.preview_variant();
        PreviewColors {
            sidebar: self
                .preview_css_color("--dbx-bg-body-web")
                .or_else(|| self.preview_css_color("--N50"))
                .unwrap_or_else(|| PreviewColor::opaque(0x17161e)),
            main: variant
                .and_then(|value| value.content.chat_background.as_deref())
                .or(self.content.chat_background.as_deref())
                .and_then(parse_preview_color)
                .or_else(|| self.preview_css_color("--s-color-bg-body"))
                .or_else(|| self.preview_css_color("--N00"))
                .unwrap_or_else(|| {
                    PreviewColor::opaque(match self.preview_mode {
                        ThemeMode::Light => 0xf8f9fb,
                        ThemeMode::Dark | ThemeMode::Auto => 0x121017,
                    })
                }),
            accent: self
                .preview_css_color("--semi-color-primary")
                .or_else(|| self.preview_css_color("--B500"))
                .or_else(|| self.preview_accent.map(PreviewColor::opaque))
                .unwrap_or_else(|| PreviewColor::opaque(0x3370eb)),
        }
    }

    pub fn preview_style(&self) -> PreviewStyle {
        let colors = self.preview_colors();
        let variant = self.preview_variant();
        let composer = variant.map(|value| &value.composer);
        let content = variant.map(|value| &value.content);
        let background = self.background_spec.as_ref();
        let text = content
            .and_then(|value| value.assistant_message_text.as_deref())
            .or(self.content.assistant_message_text.as_deref())
            .and_then(parse_preview_color)
            .unwrap_or_else(|| {
                PreviewColor::opaque(match self.preview_mode {
                    ThemeMode::Light => 0x000000,
                    ThemeMode::Dark | ThemeMode::Auto => 0xffffff,
                })
            });
        PreviewStyle {
            colors,
            surface: content
                .and_then(|value| value.assistant_message_background.as_deref())
                .or(self.content.assistant_message_background.as_deref())
                .and_then(parse_preview_color)
                .or_else(|| self.preview_css_color("--s-color-bg-float"))
                .unwrap_or(colors.main),
            text,
            input: composer
                .and_then(|value| value.background.as_deref())
                .or(self.composer.background.as_deref())
                .and_then(parse_preview_color)
                .or_else(|| self.preview_css_color("--s-color-bg-float"))
                .unwrap_or(colors.main),
            input_border: composer
                .and_then(|value| value.border.as_deref())
                .or(self.composer.border.as_deref())
                .and_then(parse_preview_color)
                .or_else(|| self.preview_css_color("--s-color-border-secondary"))
                .unwrap_or(colors.accent),
            composer_text: composer
                .and_then(|value| value.text_color.as_deref())
                .or(self.composer.text_color.as_deref())
                .and_then(parse_preview_color)
                .unwrap_or(text),
            composer_placeholder: composer
                .and_then(|value| value.placeholder_color.as_deref())
                .or(self.composer.placeholder_color.as_deref())
                .and_then(parse_preview_color)
                .unwrap_or(text),
            composer_icon: composer
                .and_then(|value| value.icon_color.as_deref())
                .or(self.composer.icon_color.as_deref())
                .and_then(parse_preview_color)
                .unwrap_or(text),
            composer_radius: composer
                .and_then(|value| value.radius)
                .or(self.composer.radius)
                .unwrap_or(20.0)
                .clamp(0.0, 40.0),
            composer_min_height: composer
                .and_then(|value| value.min_height)
                .or(self.composer.min_height)
                .unwrap_or(52.0)
                .clamp(36.0, 120.0),
            composer_padding: composer
                .and_then(|value| value.padding)
                .or(self.composer.padding)
                .unwrap_or(14.0)
                .clamp(6.0, 32.0),
            composer_gap: composer
                .and_then(|value| value.gap)
                .or(self.composer.gap)
                .unwrap_or(10.0)
                .clamp(2.0, 24.0),
            composer_icon_size: composer
                .and_then(|value| value.icon_size)
                .or(self.composer.icon_size)
                .unwrap_or(20.0)
                .clamp(12.0, 32.0),
            font_family: self
                .typography
                .body
                .clone()
                .or_else(|| self.typography.ui.clone())
                .unwrap_or_else(|| "系统字体".into()),
            density: self
                .layout
                .density
                .clone()
                .unwrap_or_else(|| "comfortable".into()),
            sidebar_width: self
                .layout
                .sidebar_width
                .unwrap_or(252.0)
                .clamp(180.0, 360.0),
            chat_margin: self.layout.chat_margin.unwrap_or(32.0).clamp(12.0, 72.0),
            radius_scale: self.effects.radius_scale.unwrap_or(1.0).clamp(0.5, 1.6),
            icons: self.preview_icons(),
            background: self.background.clone(),
            background_opacity: background.map_or(1.0, |value| value.opacity),
            background_veil: background.map_or(0.0, |value| value.veil),
            background_base: {
                let (r, g, b) = self.base_color();
                ((r as u32) << 16) | ((g as u32) << 8) | b as u32
            },
            background_fit: background
                .map(|value| value.fit.clone())
                .unwrap_or_else(|| "cover".into()),
            animated: self
                .background_spec
                .as_ref()
                .is_some_and(|b| b.kind == BackgroundKind::Video || b.animation != "none"),
            has_background: self.background_spec.is_some(),
            surface_opacity: self.surface_opacity.unwrap_or(1.0),
        }
    }

    fn preview_variant(&self) -> Option<&AppearanceVariant> {
        match self.preview_mode {
            ThemeMode::Light => self.variants.light.as_ref(),
            ThemeMode::Dark => self.variants.dark.as_ref(),
            ThemeMode::Auto => None,
        }
    }

    fn preview_icons(&self) -> ThemeIcons {
        let variant = self.preview_variant().map(|value| &value.icons);
        ThemeIcons {
            main: variant
                .and_then(|icons| icons.main.clone())
                .or_else(|| self.icons.main.clone()),
            new_task: variant
                .and_then(|icons| icons.new_task.clone())
                .or_else(|| self.icons.new_task.clone()),
            scheduled: variant
                .and_then(|icons| icons.scheduled.clone())
                .or_else(|| self.icons.scheduled.clone()),
            skills: variant
                .and_then(|icons| icons.skills.clone())
                .or_else(|| self.icons.skills.clone()),
            cloud: variant
                .and_then(|icons| icons.cloud.clone())
                .or_else(|| self.icons.cloud.clone()),
            remote: variant
                .and_then(|icons| icons.remote.clone())
                .or_else(|| self.icons.remote.clone()),
            conversation: variant
                .and_then(|icons| icons.conversation.clone())
                .or_else(|| self.icons.conversation.clone()),
            project: variant
                .and_then(|icons| icons.project.clone())
                .or_else(|| self.icons.project.clone()),
            confirm: variant
                .and_then(|icons| icons.confirm.clone())
                .or_else(|| self.icons.confirm.clone()),
            connector: variant
                .and_then(|icons| icons.connector.clone())
                .or_else(|| self.icons.connector.clone()),
            send: variant
                .and_then(|icons| icons.send.clone())
                .or_else(|| self.icons.send.clone()),
            stop: variant
                .and_then(|icons| icons.stop.clone())
                .or_else(|| self.icons.stop.clone()),
            attach: variant
                .and_then(|icons| icons.attach.clone())
                .or_else(|| self.icons.attach.clone()),
            voice: variant
                .and_then(|icons| icons.voice.clone())
                .or_else(|| self.icons.voice.clone()),
            tools: variant
                .and_then(|icons| icons.tools.clone())
                .or_else(|| self.icons.tools.clone()),
            knowledge: variant
                .and_then(|icons| icons.knowledge.clone())
                .or_else(|| self.icons.knowledge.clone()),
            more_skills: variant
                .and_then(|icons| icons.more_skills.clone())
                .or_else(|| self.icons.more_skills.clone()),
            daily_work: variant
                .and_then(|icons| icons.daily_work.clone())
                .or_else(|| self.icons.daily_work.clone()),
            content_creation: variant
                .and_then(|icons| icons.content_creation.clone())
                .or_else(|| self.icons.content_creation.clone()),
            research: variant
                .and_then(|icons| icons.research.clone())
                .or_else(|| self.icons.research.clone()),
            design: variant
                .and_then(|icons| icons.design.clone())
                .or_else(|| self.icons.design.clone()),
            read_aloud: variant
                .and_then(|icons| icons.read_aloud.clone())
                .or_else(|| self.icons.read_aloud.clone()),
            copy: variant
                .and_then(|icons| icons.copy.clone())
                .or_else(|| self.icons.copy.clone()),
            sidebar: variant
                .and_then(|icons| icons.sidebar.clone())
                .or_else(|| self.icons.sidebar.clone()),
        }
    }

    /// Resolve a custom property from the preview's declared appearance.
    /// Unscoped values are fallbacks; declarations scoped only to the
    /// opposite appearance must not leak into the preview.
    fn preview_css_color(&self, var: &str) -> Option<PreviewColor> {
        let desired = match self.preview_mode {
            ThemeMode::Light => "data-theme=light",
            ThemeMode::Dark => "data-theme=dark",
            ThemeMode::Auto => return self.css_preview_color(var),
        };
        let opposite = match self.preview_mode {
            ThemeMode::Light => "data-theme=dark",
            ThemeMode::Dark => "data-theme=light",
            ThemeMode::Auto => unreachable!(),
        };
        let mut offset = 0;
        let mut scoped = None;
        let mut unscoped = None;
        while let Some(relative) = self.css[offset..].find(var) {
            let start = offset + relative;
            let after_name = &self.css[start + var.len()..];
            let trimmed = after_name.trim_start();
            if let Some(value) = trimmed.strip_prefix(':').and_then(|value| {
                let end = value.find([';', '}']).unwrap_or(value.len());
                parse_preview_color(value[..end].trim())
            }) {
                let before = &self.css[..start];
                if let Some(open) = before.rfind('{') {
                    let selector_start = before[..open].rfind('}').map_or(0, |end| end + 1);
                    let selector = &before[selector_start..open];
                    if selector.contains(desired) {
                        scoped = Some(value);
                    } else if !selector.contains(opposite) {
                        unscoped = Some(value);
                    }
                }
            }
            offset = start + var.len();
        }
        scoped.or(unscoped)
    }

    /// Value of a css custom property as 0xRRGGBB; understands `#rrggbb`
    /// and `rgb[a](r, g, b[, a])`.
    fn css_color(&self, var: &str) -> Option<u32> {
        self.css_preview_color(var).map(|color| color.rgb)
    }

    fn css_preview_color(&self, var: &str) -> Option<PreviewColor> {
        let start = self.css.find(var)?;
        let after = &self.css[start + var.len()..];
        let colon = after.find(':')?;
        if colon > 4 {
            return None; // the match was a prefix of a longer variable name
        }
        let value = after[colon + 1..].split(';').next()?.trim();
        parse_preview_color(value)
    }
}

/// Blend every pixel toward `base` by `k`: out = img*(1-k) + base*k.
fn bake_veil(img: &mut image::RgbImage, base: (u8, u8, u8), k: f32) {
    let (br, bg, bb) = (base.0 as f32, base.1 as f32, base.2 as f32);
    let keep = 1.0 - k;
    for px in img.pixels_mut() {
        px.0[0] = (px.0[0] as f32 * keep + br * k) as u8;
        px.0[1] = (px.0[1] as f32 * keep + bg * k) as u8;
        px.0[2] = (px.0[2] as f32 * keep + bb * k) as u8;
    }
}

fn parse_preview_color(value: &str) -> Option<PreviewColor> {
    let value = value.trim();
    if !value.starts_with('#') && !value.starts_with("rgb") {
        let start = [value.find('#'), value.find("rgba("), value.find("rgb(")]
            .into_iter()
            .flatten()
            .min()?;
        return parse_preview_color(&value[start..]);
    }
    if let Some(hex) = value.strip_prefix('#') {
        let hex: String = hex
            .chars()
            .take_while(|character| character.is_ascii_hexdigit())
            .collect();
        let expand = |value: u8| (value << 4) | value;
        return match hex.len() {
            3 | 4 => {
                let mut digits = hex.chars().filter_map(|digit| digit.to_digit(16));
                let r = expand(digits.next()? as u8) as u32;
                let g = expand(digits.next()? as u8) as u32;
                let b = expand(digits.next()? as u8) as u32;
                let alpha = digits
                    .next()
                    .map(|value| expand(value as u8) as f32 / 255.0)
                    .unwrap_or(1.0);
                Some(PreviewColor {
                    rgb: (r << 16) | (g << 8) | b,
                    alpha,
                })
            }
            6 | 8 => {
                let rgb = u32::from_str_radix(&hex[..6], 16).ok()?;
                let alpha = if hex.len() == 8 {
                    u8::from_str_radix(&hex[6..], 16).ok()? as f32 / 255.0
                } else {
                    1.0
                };
                Some(PreviewColor { rgb, alpha })
            }
            _ => None,
        };
    }
    let (inner, has_alpha) = if let Some(inner) = value.strip_prefix("rgba(") {
        (inner, true)
    } else {
        (value.strip_prefix("rgb(")?, false)
    };
    let inner = inner.split_once(')')?.0;
    let mut parts = inner.split(',').map(|p| p.trim());
    let r: u32 = parts.next()?.parse().ok()?;
    let g: u32 = parts.next()?.parse().ok()?;
    let b: u32 = parts.next()?.parse().ok()?;
    if r > 255 || g > 255 || b > 255 {
        return None;
    }
    let alpha = if has_alpha {
        parts.next()?.parse::<f32>().ok()?.clamp(0.0, 1.0)
    } else {
        1.0
    };
    alpha.is_finite().then_some(PreviewColor {
        rgb: (r << 16) | (g << 8) | b,
        alpha,
    })
}

fn parse_color_value(value: &str) -> Option<u32> {
    parse_preview_color(value).map(|color| color.rgb)
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    /// A package-owned 16:9 preview used by the local library and store.
    pub preview_image: Option<PathBuf>,
    /// Appearance represented by the package preview and the native mockup.
    pub preview_mode: ThemeMode,
    pub preview_accent: Option<u32>,
    pub store_category: Option<String>,
    pub store_tags: Vec<String>,
    pub store_sort_order: Option<u32>,
    pub css: String,
    pub icon: Option<PathBuf>,
    /// Optional atmosphere background image (theme.json "background"),
    /// resolved relative to the theme directory.
    pub background: Option<PathBuf>,
    /// How strongly the background image is blended toward the theme's base
    /// color before encoding ("veil", 0..1, default 0.45). Baking the veil
    /// into the image keeps surface-token alphas independent of readability:
    /// image visibility is no longer (container alpha x css veil).
    pub veil: f32,
    /// Opacity of the main foreground surfaces over an atmosphere background.
    /// Text, icons, menus and solid accent buttons are deliberately unaffected.
    pub surface_opacity: Option<f32>,
    pub appearance: ThemeAppearance,
    pub mode: ThemeMode,
    pub background_spec: Option<BackgroundSpec>,
    pub typography: Typography,
    pub layout: ThemeLayout,
    pub composer: ComposerStyle,
    pub content: ContentStyle,
    pub icons: ThemeIcons,
    pub variants: ThemeVariants,
    pub effects: ThemeEffects,
    pub path: PathBuf,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThemeMeta {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    preview: PreviewMeta,
    #[serde(default)]
    store: StoreMeta,
    #[serde(default)]
    background: Option<BackgroundMeta>,
    #[serde(default)]
    veil: Option<f32>,
    #[serde(default)]
    surface_opacity: Option<f32>,
    #[serde(default)]
    mode: ThemeMode,
    #[serde(default)]
    appearance: Option<ThemeAppearance>,
    #[serde(default)]
    typography: TypographyMeta,
    #[serde(default)]
    layout: LayoutMeta,
    #[serde(default)]
    composer: ComposerMeta,
    #[serde(default)]
    content: ContentMeta,
    #[serde(default)]
    icons: IconsMeta,
    #[serde(default)]
    variants: VariantsMeta,
    #[serde(default)]
    effects: EffectsMeta,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PreviewMeta {
    image: Option<String>,
    appearance: Option<ThemeMode>,
    accent: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct StoreMeta {
    category: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    sort_order: Option<u32>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BackgroundMeta {
    Path(String),
    Options(BackgroundOptionsMeta),
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct BackgroundOptionsMeta {
    #[serde(rename = "type")]
    kind: Option<String>,
    src: Option<String>,
    poster: Option<String>,
    gradient: Option<String>,
    fit: Option<String>,
    position: Option<String>,
    opacity: Option<f32>,
    veil: Option<f32>,
    blur: Option<f32>,
    animation: Option<String>,
    duration_seconds: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TypographyMeta {
    ui: Option<String>,
    body: Option<String>,
    code: Option<String>,
    scale: Option<f32>,
    line_height: Option<f32>,
    #[serde(default)]
    assets: Vec<FontAssetMeta>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FontAssetMeta {
    family: String,
    src: String,
    #[serde(default = "default_normal")]
    weight: String,
    #[serde(default = "default_normal")]
    style: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LayoutMeta {
    density: Option<String>,
    sidebar_width: Option<f32>,
    chat_max_width: Option<f32>,
    composer_max_width: Option<f32>,
    self_message_max_width: Option<f32>,
    chat_margin: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ComposerMeta {
    background: Option<String>,
    border: Option<String>,
    text_color: Option<String>,
    placeholder_color: Option<String>,
    caret_color: Option<String>,
    icon_color: Option<String>,
    send_button_background: Option<String>,
    send_button_icon_color: Option<String>,
    radius: Option<f32>,
    min_height: Option<f32>,
    max_height: Option<f32>,
    padding: Option<f32>,
    gap: Option<f32>,
    icon_size: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ContentMeta {
    chat_background: Option<String>,
    user_message_background: Option<String>,
    user_message_text: Option<String>,
    assistant_message_background: Option<String>,
    assistant_message_text: Option<String>,
    code_background: Option<String>,
    code_header_background: Option<String>,
    selection_color: Option<String>,
    scrollbar_color: Option<String>,
    scrollbar_hover_color: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct IconsMeta {
    main: Option<String>,
    new_task: Option<String>,
    scheduled: Option<String>,
    skills: Option<String>,
    cloud: Option<String>,
    remote: Option<String>,
    conversation: Option<String>,
    project: Option<String>,
    confirm: Option<String>,
    connector: Option<String>,
    send: Option<String>,
    stop: Option<String>,
    attach: Option<String>,
    voice: Option<String>,
    tools: Option<String>,
    knowledge: Option<String>,
    more_skills: Option<String>,
    daily_work: Option<String>,
    content_creation: Option<String>,
    research: Option<String>,
    design: Option<String>,
    read_aloud: Option<String>,
    copy: Option<String>,
    sidebar: Option<String>,
}

#[derive(Deserialize, Default)]
struct VariantsMeta {
    light: Option<AppearanceVariantMeta>,
    dark: Option<AppearanceVariantMeta>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AppearanceVariantMeta {
    #[serde(default)]
    composer: ComposerMeta,
    #[serde(default)]
    content: ContentMeta,
    #[serde(default)]
    icons: IconsMeta,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct EffectsMeta {
    radius_scale: Option<f32>,
    shadow: Option<String>,
    blur: Option<f32>,
    motion: Option<String>,
    transition_ms: Option<u32>,
}

fn default_schema_version() -> u32 {
    1
}

fn default_normal() -> String {
    "normal".into()
}

fn default_veil() -> f32 {
    0.45
}

impl Theme {
    /// Snippet injected into HTML pages (offline build). Live mode calls the
    /// same bootstrap so both application paths support the same theme fields.
    pub fn snippet(&self) -> Vec<u8> {
        let css = self.injected_css();
        let script = self.bootstrap_js(None, None);
        format!(
            "<script nonce=\"argus-csp-token\">{script}</script><style nonce=\"argus-csp-token\">{css}</style>"
        )
        .into_bytes()
    }

    /// JS string evaluated in live (CDP) mode.
    pub fn live_js(&self) -> String {
        live::theme_js(self, live::TargetApp::DoubaoWork)
    }

    pub fn live_js_for(&self, target: live::TargetApp) -> String {
        live::theme_js(self, target)
    }

    /// Theme CSS, with the `--skin-bg-image` variable rule prepended when the
    /// theme has a background image. theme.css can then reference
    /// `var(--skin-bg-image)` (e.g. in a fixed body::before layer).
    pub fn effective_css(&self) -> String {
        let mut out = String::new();
        if let Some(uri) = self.background_image_data_uri() {
            out.push_str(&format!(
                "html[data-skin], html[data-skin] body {{ --skin-bg-image: url(\"{uri}\"); }}\n"
            ));
        }
        out.push_str(&self.css);
        out.push_str(&self.semantic_css());
        out.push_str(&self.backdrop_css());
        out.push_str(&self.surface_opacity_css());
        out.push_str("\nhtml[data-skin][data-skin-target=doubao] #chat-route-main{background-color:transparent!important;}");
        out
    }

    pub(crate) fn injected_css(&self) -> String {
        let color_scheme = match self.mode {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
            ThemeMode::Auto => "light dark",
        };
        format!(
            "html{{color-scheme:{color_scheme}}}{}",
            self.effective_css()
        )
    }

    /// Legacy image themes keep their baked veil. In v2 the veil is a
    /// separate layer, allowing images, gradients and video to behave alike.
    fn background_image_data_uri(&self) -> Option<String> {
        let spec = self.background_spec.as_ref()?;
        if spec.kind != BackgroundKind::Image {
            return None;
        }
        let path = spec.source.as_ref()?;
        let img = image::open(path).ok()?;
        let img = if img.width() > 1920 {
            let height = img.height() * 1920 / img.width();
            img.resize(1920, height, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };
        let mut rgb = img.to_rgb8();
        if spec.legacy {
            bake_veil(&mut rgb, self.base_color(), self.veil.clamp(0.0, 1.0));
        }
        let mut jpeg = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 75);
        encoder.encode_image(&rgb).ok()?;
        Some(format!(
            "data:image/jpeg;base64,{}",
            crate::ws::base64_encode(&jpeg)
        ))
    }

    /// Theme base color for the veil: --s-color-bg-body, else #121317.
    fn base_color(&self) -> (u8, u8, u8) {
        let c = self.css_color("--s-color-bg-body").unwrap_or(0x121317);
        ((c >> 16) as u8, (c >> 8) as u8, c as u8)
    }

    /// First `n` distinct `#rrggbb` colors found in the theme CSS (for UI
    /// swatches), as 0xRRGGBB values.
    pub fn swatches(&self, n: usize) -> Vec<u32> {
        let bytes = self.css.as_bytes();
        let mut colors: Vec<u32> = Vec::new();
        let mut i = 0;
        while i + 7 <= bytes.len() && colors.len() < n {
            if bytes[i] == b'#' {
                if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 7]) {
                    if let Ok(v) = u32::from_str_radix(hex, 16) {
                        if !colors.contains(&v) {
                            colors.push(v);
                        }
                        i += 7;
                        continue;
                    }
                }
            }
            i += 1;
        }
        colors
    }

    fn background_runtime_json(&self) -> serde_json::Value {
        let Some(spec) = self.background_spec.as_ref().filter(|b| !b.legacy) else {
            return serde_json::Value::Null;
        };
        let source = match spec.kind {
            BackgroundKind::Image => self.background_image_data_uri(),
            BackgroundKind::Video => spec.source.as_deref().and_then(asset_data_uri),
            BackgroundKind::Gradient => None,
        };
        serde_json::json!({
            "kind": match spec.kind {
                BackgroundKind::Image => "image",
                BackgroundKind::Video => "video",
                BackgroundKind::Gradient => "gradient",
            },
            "source": source,
            "poster": spec.poster.as_deref().and_then(asset_data_uri),
            "gradient": spec.gradient,
        })
    }

    fn semantic_css(&self) -> String {
        let mut css = String::new();
        for font in &self.typography.assets {
            if let Some(uri) = asset_data_uri(&font.path) {
                css.push_str(&format!(
                    "\n@font-face{{font-family:{};src:url(\"{}\");font-weight:{};font-style:{};font-display:swap;}}",
                    css_string(&font.family),
                    uri,
                    css_atom(&font.weight, "normal"),
                    css_atom(&font.style, "normal")
                ));
            }
        }

        let mut vars: Vec<(&str, String)> = Vec::new();
        let (red, green, blue) = self.base_color();
        let linear = |channel: u8| {
            let channel = channel as f32 / 255.0;
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        };
        let luminance = 0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue);
        let dark_surface = luminance < 0.179;
        let (text_rgb, text_hex, tertiary_alpha, quaternary_alpha, accent_text) = if dark_surface {
            ("255,255,255", "#ffffff", "0.62", "0.55", "#77b0ff")
        } else {
            ("0,0,0", "#000000", "0.72", "0.66", "#16356f")
        };
        for (name, value) in [
            ("--s-color-text-primary", text_hex.to_string()),
            ("--s-color-text-primary-raw", text_rgb.to_string()),
            ("--s-color-text-secondary", format!("rgba({text_rgb},0.85)")),
            (
                "--s-color-text-tertiary",
                format!("rgba({text_rgb},{tertiary_alpha})"),
            ),
            (
                "--s-color-text-quaternary",
                format!("rgba({text_rgb},{quaternary_alpha})"),
            ),
            ("--s-color-text-disable", format!("rgba({text_rgb},0.20)")),
            ("--s-color-brand-primary-default", accent_text.to_string()),
            ("--dbx-text-primary", format!("rgba({text_rgb},0.90)")),
            ("--dbx-text-secondary", format!("rgba({text_rgb},0.70)")),
            (
                "--dbx-text-tertiary",
                format!("rgba({text_rgb},{tertiary_alpha})"),
            ),
            ("--dbx-text-disable", format!("rgba({text_rgb},0.30)")),
            ("--dbx-text-highlight", accent_text.to_string()),
            ("--dbx-text-markdown", format!("rgba({text_rgb},0.90)")),
        ] {
            vars.push((name, value));
        }
        push_css_value(&mut vars, "--skin-font-ui", self.typography.ui.as_deref());
        push_css_value(
            &mut vars,
            "--skin-font-body",
            self.typography
                .body
                .as_deref()
                .or(self.typography.ui.as_deref()),
        );
        push_css_value(
            &mut vars,
            "--skin-font-code",
            self.typography.code.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--font-dbx-sans-serif",
            self.typography
                .body
                .as_deref()
                .or(self.typography.ui.as_deref()),
        );
        push_css_value(
            &mut vars,
            "--input-guidance-input-editor-font",
            self.typography
                .body
                .as_deref()
                .or(self.typography.ui.as_deref()),
        );
        push_css_value(
            &mut vars,
            "--code-font-family-mono",
            self.typography.code.as_deref(),
        );
        push_number(
            &mut vars,
            "--skin-type-scale",
            self.typography.scale,
            0.8,
            1.4,
            "",
        );
        push_number(
            &mut vars,
            "--skin-line-height",
            self.typography.line_height,
            1.1,
            2.0,
            "",
        );
        push_number(
            &mut vars,
            "--sidebar-width",
            self.layout.sidebar_width,
            180.0,
            420.0,
            "px",
        );
        push_number(
            &mut vars,
            "--chat-area-max-width",
            self.layout.chat_max_width,
            520.0,
            1400.0,
            "px",
        );
        push_number(
            &mut vars,
            "--composebox-max-width",
            self.layout.composer_max_width,
            420.0,
            1200.0,
            "px",
        );
        push_number(
            &mut vars,
            "--self-message-box-max-width",
            self.layout.self_message_max_width,
            180.0,
            900.0,
            "px",
        );
        push_number(
            &mut vars,
            "--chat-area-margin",
            self.layout.chat_margin,
            0.0,
            96.0,
            "px",
        );
        let density = match self.layout.density.as_deref() {
            Some("compact") => (18.0, 10.0, 6.0),
            Some("spacious") => (36.0, 18.0, 12.0),
            _ => (28.0, 14.0, 10.0),
        };
        if self.layout.density.is_some() && self.layout.chat_margin.is_none() {
            push_number(
                &mut vars,
                "--chat-area-margin",
                Some(density.0),
                0.0,
                96.0,
                "px",
            );
        }
        if self.layout.density.is_some() && self.composer.padding.is_none() {
            push_number(
                &mut vars,
                "--input-guidance-input-container-padding",
                Some(density.1),
                4.0,
                40.0,
                "px",
            );
        }
        if self.layout.density.is_some() && self.composer.gap.is_none() {
            push_number(
                &mut vars,
                "--input-guidance-input-container-gap",
                Some(density.2),
                0.0,
                32.0,
                "px",
            );
        }

        push_css_value(
            &mut vars,
            "--input-guidance-input-container-background",
            self.composer.background.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--color-composebox-background",
            self.composer.background.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--cr-composebox-background-color",
            self.composer.background.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--input-guidance-input-container-border",
            self.composer.border.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--input-guidance-input-editor-color",
            self.composer.text_color.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--color-composebox-font",
            self.composer.text_color.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--input-guidance-input-editor-placeholder-color",
            self.composer.placeholder_color.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--cr-composebox-input-placeholder-color",
            self.composer.placeholder_color.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--input-guidance-input-editor-caret-color",
            self.composer.caret_color.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--semi-color-focus-border",
            self.composer.caret_color.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--color-composebox-input-icon",
            self.composer.icon_color.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--color-composebox-submit-button-background",
            self.composer.send_button_background.as_deref(),
        );
        push_css_value(
            &mut vars,
            "--color-composebox-submit-button-icon",
            self.composer.send_button_icon_color.as_deref(),
        );
        push_number(
            &mut vars,
            "--input-guidance-input-container-radius",
            self.composer.radius,
            0.0,
            48.0,
            "px",
        );
        push_number(
            &mut vars,
            "--composebox-border-radius",
            self.composer.radius,
            0.0,
            48.0,
            "px",
        );
        push_number(
            &mut vars,
            "--cr-composebox-expanded-border-radius",
            self.composer.radius,
            0.0,
            48.0,
            "px",
        );
        push_number(
            &mut vars,
            "--input-guidance-input-container-min-height",
            self.composer.min_height,
            32.0,
            160.0,
            "px",
        );
        push_number(
            &mut vars,
            "--max-composebox-height",
            self.composer.max_height,
            80.0,
            520.0,
            "px",
        );
        push_number(
            &mut vars,
            "--cr-composebox-max-height",
            self.composer.max_height,
            80.0,
            520.0,
            "px",
        );
        push_number(
            &mut vars,
            "--input-guidance-input-container-max-height",
            self.composer.max_height,
            80.0,
            520.0,
            "px",
        );
        push_number(
            &mut vars,
            "--input-guidance-input-container-padding",
            self.composer.padding,
            4.0,
            40.0,
            "px",
        );
        push_number(
            &mut vars,
            "--input-guidance-input-container-gap",
            self.composer.gap,
            0.0,
            32.0,
            "px",
        );
        push_number(
            &mut vars,
            "--chat-input-icon-size",
            self.composer.icon_size,
            12.0,
            40.0,
            "px",
        );
        push_number(
            &mut vars,
            "--chat-input-icon-size-full",
            self.composer.icon_size,
            12.0,
            40.0,
            "px",
        );

        for (name, value) in [
            ("--chat-bg-color", self.content.chat_background.as_deref()),
            (
                "--chatarea-bg-color",
                self.content.chat_background.as_deref(),
            ),
            (
                "--g-send-msg-bubble-bg",
                self.content.user_message_background.as_deref(),
            ),
            (
                "--g-send-msg-bubble-text",
                self.content.user_message_text.as_deref(),
            ),
            (
                "--color-g-send-msg-bubble-text",
                self.content.user_message_text.as_deref(),
            ),
            (
                "--g-msg-bubble-bg",
                self.content.assistant_message_background.as_deref(),
            ),
            (
                "--g-msg-bubble-text",
                self.content.assistant_message_text.as_deref(),
            ),
            (
                "--chat-md-codeblock-bg-color",
                self.content.code_background.as_deref(),
            ),
            (
                "--chat-md-codeblock-header-bg-color",
                self.content.code_header_background.as_deref(),
            ),
            (
                "--skin-selection-color",
                self.content.selection_color.as_deref(),
            ),
            (
                "--skin-scrollbar-color",
                self.content.scrollbar_color.as_deref(),
            ),
            (
                "--skin-scrollbar-hover-color",
                self.content.scrollbar_hover_color.as_deref(),
            ),
        ] {
            push_css_value(&mut vars, name, value);
        }

        push_number(
            &mut vars,
            "--skin-radius-scale",
            self.effects.radius_scale,
            0.5,
            2.0,
            "",
        );
        push_css_value(&mut vars, "--skin-shadow", self.effects.shadow.as_deref());
        push_number(
            &mut vars,
            "--skin-surface-blur",
            self.effects.blur,
            0.0,
            40.0,
            "px",
        );
        push_number(
            &mut vars,
            "--dbx-bg-blur-md",
            self.effects.blur,
            0.0,
            100.0,
            "px",
        );
        if let Some(ms) = self.effects.transition_ms {
            vars.push(("--skin-transition", format!("{}ms", ms.clamp(0, 2000))));
        }
        for (name, path) in [
            ("--skin-icon-main", self.icons.main.as_deref()),
            ("--skin-icon-new-task", self.icons.new_task.as_deref()),
            ("--skin-icon-scheduled", self.icons.scheduled.as_deref()),
            ("--skin-icon-skills", self.icons.skills.as_deref()),
            ("--skin-icon-cloud", self.icons.cloud.as_deref()),
            ("--skin-icon-remote", self.icons.remote.as_deref()),
            (
                "--skin-icon-conversation",
                self.icons.conversation.as_deref(),
            ),
            ("--skin-icon-project", self.icons.project.as_deref()),
            ("--skin-icon-confirm", self.icons.confirm.as_deref()),
            ("--skin-icon-connector", self.icons.connector.as_deref()),
            ("--skin-icon-send", self.icons.send.as_deref()),
            ("--skin-icon-stop", self.icons.stop.as_deref()),
            ("--skin-icon-attach", self.icons.attach.as_deref()),
            ("--skin-icon-voice", self.icons.voice.as_deref()),
            ("--skin-icon-tools", self.icons.tools.as_deref()),
            ("--skin-icon-knowledge", self.icons.knowledge.as_deref()),
            ("--skin-icon-more-skills", self.icons.more_skills.as_deref()),
            ("--skin-icon-daily-work", self.icons.daily_work.as_deref()),
            (
                "--skin-icon-content-creation",
                self.icons.content_creation.as_deref(),
            ),
            ("--skin-icon-research", self.icons.research.as_deref()),
            ("--skin-icon-design", self.icons.design.as_deref()),
            ("--skin-icon-read-aloud", self.icons.read_aloud.as_deref()),
            ("--skin-icon-copy", self.icons.copy.as_deref()),
            ("--skin-icon-sidebar", self.icons.sidebar.as_deref()),
        ] {
            if let Some(uri) = path.and_then(asset_data_uri) {
                vars.push((name, format!("url(\"{uri}\")")));
            }
        }

        if !vars.is_empty() {
            css.push_str("\nhtml[data-skin],html[data-skin] body{");
            for (name, value) in vars {
                css.push_str(name);
                css.push(':');
                css.push_str(&value);
                css.push_str("!important;");
            }
            css.push('}');
        }
        if let Some(variant) = &self.variants.light {
            css.push_str(&appearance_variant_css("light", variant));
        }
        if let Some(variant) = &self.variants.dark {
            css.push_str(&appearance_variant_css("dark", variant));
        }
        css.push_str("\nhtml[data-skin] .md-box-root{--md-box-color-fg:var(--dbx-text-markdown)!important;--md-box-color-fg-muted:var(--dbx-text-secondary)!important;}html[data-skin] .bg-g-send-msg-bubble-bg .md-box-root{--md-box-color-fg:var(--color-g-send-msg-bubble-text)!important;--md-box-color-fg-muted:var(--color-g-send-msg-bubble-text)!important;}");
        if self.typography.ui.is_some()
            || self.typography.body.is_some()
            || !self.typography.assets.is_empty()
        {
            css.push_str("\nhtml[data-skin] body,html[data-skin] button,html[data-skin] input,html[data-skin] textarea{font-family:var(--skin-font-body,var(--skin-font-ui,var(--font-dbx-sans-serif,sans-serif)))!important;}");
        }
        if self.typography.code.is_some() {
            css.push_str("\nhtml[data-skin] code,html[data-skin] pre{font-family:var(--skin-font-code,var(--code-font-family-mono,monospace))!important;}");
        }
        if self.typography.scale.is_some() {
            css.push_str("\nhtml[data-skin] body{font-size:calc(14px * var(--skin-type-scale,1))!important;}");
        }
        if self.typography.line_height.is_some() {
            css.push_str("\nhtml[data-skin] body{line-height:var(--skin-line-height,1.5);}");
        }
        if self.content.selection_color.is_some() {
            css.push_str(
                "\nhtml[data-skin] ::selection{background:var(--skin-selection-color)!important;}",
            );
        }
        if self.content.scrollbar_color.is_some() {
            css.push_str("\nhtml[data-skin] ::-webkit-scrollbar-thumb{background:var(--skin-scrollbar-color)!important;}html[data-skin] ::-webkit-scrollbar-thumb:hover{background:var(--skin-scrollbar-hover-color,var(--skin-scrollbar-color))!important;}");
        }
        if self.composer.border.is_some()
            || self.composer.caret_color.is_some()
            || self.variants.light.as_ref().is_some_and(|variant| {
                variant.composer.border.is_some() || variant.composer.caret_color.is_some()
            })
            || self.variants.dark.as_ref().is_some_and(|variant| {
                variant.composer.border.is_some() || variant.composer.caret_color.is_some()
            })
        {
            css.push_str("\nhtml[data-skin] [data-doubao-theme-composer]{border:var(--input-guidance-input-container-border)!important;}html[data-skin] [data-doubao-theme-composer]:focus-within{border-color:var(--semi-color-focus-border)!important;box-shadow:0 0 0 1px color-mix(in srgb,var(--semi-color-focus-border) 28%,transparent)!important;outline:none!important;}");
        }
        if self.effects.shadow.is_some() {
            css.push_str("\nhtml[data-skin] body{--dbx-drop-shadow-md:var(--skin-shadow)!important;--s-shadow-level1:var(--skin-shadow)!important;}");
        }
        if self.effects.radius_scale.is_some() {
            css.push_str("\nhtml[data-skin] body{--semi-border-radius-small:calc(4px * var(--skin-radius-scale))!important;--semi-border-radius-medium:calc(8px * var(--skin-radius-scale))!important;--semi-border-radius-large:calc(12px * var(--skin-radius-scale))!important;}");
        }
        if self.effects.transition_ms.is_some() {
            css.push_str("\nhtml[data-skin] button,html[data-skin] input,html[data-skin] textarea{transition-duration:var(--skin-transition)!important;}");
        }
        if self.icons.any()
            || self.variants.light.as_ref().is_some_and(|v| v.icons.any())
            || self.variants.dark.as_ref().is_some_and(|v| v.icons.any())
        {
            css.push_str("\nhtml[data-skin] svg[data-doubao-theme-icon]{background-color:currentColor!important;mask:var(--doubao-theme-icon) center/contain no-repeat!important;-webkit-mask:var(--doubao-theme-icon) center/contain no-repeat!important;}html[data-skin] svg[data-doubao-theme-icon] *{opacity:0!important;}html[data-skin] img[data-doubao-theme-icon]{content:var(--doubao-theme-icon)!important;object-fit:contain!important;}html[data-skin] img[data-doubao-theme-icon=main]{content:var(--skin-icon-main)!important;}html[data-skin] svg[data-doubao-theme-icon=main]{background:var(--skin-icon-main) center/contain no-repeat!important;mask:none!important;-webkit-mask:none!important;}html[data-skin] [data-doubao-theme-icon=main]{--doubao-theme-icon:var(--skin-icon-main);}html[data-skin] [data-doubao-theme-icon=new-task]{--doubao-theme-icon:var(--skin-icon-new-task);}html[data-skin] [data-doubao-theme-icon=scheduled]{--doubao-theme-icon:var(--skin-icon-scheduled);}html[data-skin] [data-doubao-theme-icon=skills]{--doubao-theme-icon:var(--skin-icon-skills);}html[data-skin] [data-doubao-theme-icon=cloud]{--doubao-theme-icon:var(--skin-icon-cloud);}html[data-skin] [data-doubao-theme-icon=remote]{--doubao-theme-icon:var(--skin-icon-remote);}html[data-skin] [data-doubao-theme-icon=conversation]{--doubao-theme-icon:var(--skin-icon-conversation);}html[data-skin] [data-doubao-theme-icon=project]{--doubao-theme-icon:var(--skin-icon-project);}html[data-skin] [data-doubao-theme-icon=confirm]{--doubao-theme-icon:var(--skin-icon-confirm);}html[data-skin] [data-doubao-theme-icon=connector]{--doubao-theme-icon:var(--skin-icon-connector);}html[data-skin] [data-doubao-theme-icon=send]{--doubao-theme-icon:var(--skin-icon-send);}html[data-skin] [data-doubao-theme-icon=stop]{--doubao-theme-icon:var(--skin-icon-stop);}html[data-skin] [data-doubao-theme-icon=attach]{--doubao-theme-icon:var(--skin-icon-attach);}html[data-skin] [data-doubao-theme-icon=voice]{--doubao-theme-icon:var(--skin-icon-voice);}html[data-skin] [data-doubao-theme-icon=tools]{--doubao-theme-icon:var(--skin-icon-tools);}html[data-skin] [data-doubao-theme-icon=knowledge]{--doubao-theme-icon:var(--skin-icon-knowledge);}html[data-skin] [data-doubao-theme-icon=more-skills]{--doubao-theme-icon:var(--skin-icon-more-skills);}html[data-skin] [data-doubao-theme-icon=daily-work]{--doubao-theme-icon:var(--skin-icon-daily-work);}html[data-skin] [data-doubao-theme-icon=content-creation]{--doubao-theme-icon:var(--skin-icon-content-creation);}html[data-skin] [data-doubao-theme-icon=research]{--doubao-theme-icon:var(--skin-icon-research);}html[data-skin] [data-doubao-theme-icon=design]{--doubao-theme-icon:var(--skin-icon-design);}html[data-skin] [data-doubao-theme-icon=read-aloud]{--doubao-theme-icon:var(--skin-icon-read-aloud);}html[data-skin] [data-doubao-theme-icon=copy]{--doubao-theme-icon:var(--skin-icon-copy);}html[data-skin] [data-doubao-theme-icon=sidebar]{--doubao-theme-icon:var(--skin-icon-sidebar);}");
        }
        if self.effects.motion.as_deref() == Some("none") {
            css.push_str("\nhtml[data-skin] *,html[data-skin] *::before,html[data-skin] *::after{animation-duration:0.001ms!important;animation-iteration-count:1!important;transition-duration:0.001ms!important;}");
        }
        css
    }

    fn backdrop_css(&self) -> String {
        let Some(spec) = self.background_spec.as_ref().filter(|b| !b.legacy) else {
            return String::new();
        };
        let (r, g, b) = self.base_color();
        let animation = match spec.animation.as_str() {
            "ken-burns" => "doubao-skin-ken-burns",
            "drift" => "doubao-skin-drift",
            "pulse" => "doubao-skin-pulse",
            _ => "none",
        };
        let object_fit = css_atom(&spec.fit, "cover");
        let background_fit = match object_fit {
            "contain" => "contain",
            "fill" => "100% 100%",
            "none" | "scale-down" => "auto",
            _ => "cover",
        };
        "\nhtml[data-skin] body{isolation:isolate;background-color:transparent!important;}#doubao-skin-backdrop{position:fixed;inset:-3%;z-index:-1;pointer-events:none;overflow:hidden;background-size:%BACKGROUND_FIT%;background-position:%POSITION%;background-repeat:no-repeat;opacity:%OPACITY%;filter:blur(%BLUR%px);animation:%ANIMATION% %DURATION%s ease-in-out infinite alternate;}#doubao-skin-backdrop video{width:100%;height:100%;object-fit:%OBJECT_FIT%;object-position:%POSITION%;}#doubao-skin-backdrop::after{content:\"\";position:absolute;inset:0;background:rgba(%R%,%G%,%B%,%VEIL%);}@keyframes doubao-skin-ken-burns{from{transform:scale(1.02)}to{transform:scale(1.10) translate3d(-1.5%,1%,0)}}@keyframes doubao-skin-drift{from{transform:translate3d(-1%,0,0)}to{transform:translate3d(1%,0.8%,0)}}@keyframes doubao-skin-pulse{from{opacity:%PULSE_FROM%}to{opacity:%OPACITY%}}@media(prefers-reduced-motion:reduce){#doubao-skin-backdrop{animation:none!important;}}"
            .replace("%BACKGROUND_FIT%", background_fit)
            .replace("%OBJECT_FIT%", object_fit)
            .replace("%POSITION%", css_atom(&spec.position, "center"))
            .replace("%OPACITY%", &spec.opacity.to_string())
            .replace("%BLUR%", &spec.blur.to_string())
            .replace("%ANIMATION%", animation)
            .replace("%DURATION%", &spec.duration_seconds.to_string())
            .replace("%R%", &r.to_string())
            .replace("%G%", &g.to_string())
            .replace("%B%", &b.to_string())
            .replace("%VEIL%", &spec.veil.to_string())
            .replace(
                "%PULSE_FROM%",
                &(spec.opacity * 0.86).clamp(0.0, 1.0).to_string(),
            )
    }

    fn surface_opacity_css(&self) -> String {
        let Some(surface) = self.surface_opacity else {
            return String::new();
        };
        let opacity = surface_opacity_profile(surface);
        let surface = opacity.surface;
        let page = opacity.page;
        let sidebar = opacity.sidebar;
        let layer = opacity.layer;
        let input = opacity.input;
        let legacy_background = if self
            .background_spec
            .as_ref()
            .is_some_and(|background| background.legacy)
        {
            "html:root[data-skin] body{background-image:var(--skin-bg-image)!important;}"
        } else {
            ""
        };
        format!(
            "\nhtml:root[data-skin],html:root[data-skin] body{{--skin-surface-opacity:{surface};--N00:rgba(var(--N00-raw),{page})!important;--N50:rgba(var(--N50-raw),{sidebar})!important;--N100:rgba(var(--N100-raw),{layer})!important;--N200:rgba(var(--N200-raw),{layer})!important;--s-color-bg-body:rgba(var(--s-color-bg-body-raw),{page})!important;--s-color-bg-secondary:rgba(var(--s-color-bg-secondary-raw),{layer})!important;--s-color-bg-base:rgba(var(--s-color-bg-base-raw),{layer})!important;--s-color-bg-tertiary:rgba(var(--s-color-bg-tertiary-raw),{layer})!important;--s-color-bg-quaternary:rgba(var(--s-color-bg-quaternary-raw),{layer})!important;--s-color-bg-primary:rgba(var(--s-color-bg-primary-raw),{layer})!important;--s-color-bg-content-base:rgba(var(--s-color-bg-content-base-raw),{page})!important;--dbx-bg-base-web:rgba(var(--dbx-bg-base-web-raw),{layer})!important;--dbx-bg-base-2:rgba(var(--dbx-bg-base-2-raw),{layer})!important;--dbx-bg-base-5:rgba(var(--dbx-bg-base-5-raw),{layer})!important;--dbx-bg-body-web:rgba(var(--dbx-bg-body-web-raw),{sidebar})!important;--dbx-bg-body-white:rgba(var(--dbx-bg-body-white-raw),{sidebar})!important;--dbx-bg-body-mac:rgba(var(--dbx-bg-body-web-raw),{sidebar})!important;--chat-bg-color:rgba(var(--s-color-bg-body-raw),{page})!important;--chatarea-bg-color:rgba(var(--s-color-bg-body-raw),{page})!important;--g-msg-bubble-bg:rgba(var(--s-color-bg-float-raw),{surface})!important;--input-guidance-input-container-background:rgba(var(--s-color-bg-float-raw),{input})!important;--color-composebox-background:rgba(var(--s-color-bg-float-raw),{input})!important;--cr-composebox-background-color:rgba(var(--s-color-bg-float-raw),{input})!important;}}html[data-skin][data-skin-target=doubao-work] [class*=\"greeting-text-\"]{{overflow:clip!important;}}{legacy_background}"
        )
    }

    pub(crate) fn bootstrap_js(&self, css: Option<&str>, target: Option<&str>) -> String {
        let skin = serde_json::to_string(&self.id).unwrap_or_else(|_| "\"theme\"".into());
        let mode = serde_json::to_string(self.mode.as_str()).unwrap();
        let target = target
            .map(serde_json::to_string)
            .transpose()
            .unwrap_or(None)
            .unwrap_or_else(|| "null".into());
        let css = css
            .map(serde_json::to_string)
            .transpose()
            .unwrap_or(None)
            .unwrap_or_else(|| "null".into());
        let background = self.background_runtime_json().to_string();
        let light = self.variants.light.as_ref().map(|variant| &variant.icons);
        let dark = self.variants.dark.as_ref().map(|variant| &variant.icons);
        let icons = serde_json::json!({
            "main": self.icons.main.is_some() || light.is_some_and(|icons| icons.main.is_some()) || dark.is_some_and(|icons| icons.main.is_some()),
            "new-task": self.icons.new_task.is_some() || light.is_some_and(|icons| icons.new_task.is_some()) || dark.is_some_and(|icons| icons.new_task.is_some()),
            "scheduled": self.icons.scheduled.is_some() || light.is_some_and(|icons| icons.scheduled.is_some()) || dark.is_some_and(|icons| icons.scheduled.is_some()),
            "skills": self.icons.skills.is_some() || light.is_some_and(|icons| icons.skills.is_some()) || dark.is_some_and(|icons| icons.skills.is_some()),
            "cloud": self.icons.cloud.is_some() || light.is_some_and(|icons| icons.cloud.is_some()) || dark.is_some_and(|icons| icons.cloud.is_some()),
            "remote": self.icons.remote.is_some() || light.is_some_and(|icons| icons.remote.is_some()) || dark.is_some_and(|icons| icons.remote.is_some()),
            "conversation": self.icons.conversation.is_some() || light.is_some_and(|icons| icons.conversation.is_some()) || dark.is_some_and(|icons| icons.conversation.is_some()),
            "project": self.icons.project.is_some() || light.is_some_and(|icons| icons.project.is_some()) || dark.is_some_and(|icons| icons.project.is_some()),
            "confirm": self.icons.confirm.is_some() || light.is_some_and(|icons| icons.confirm.is_some()) || dark.is_some_and(|icons| icons.confirm.is_some()),
            "connector": self.icons.connector.is_some() || light.is_some_and(|icons| icons.connector.is_some()) || dark.is_some_and(|icons| icons.connector.is_some()),
            "send": self.icons.send.is_some() || light.is_some_and(|icons| icons.send.is_some()) || dark.is_some_and(|icons| icons.send.is_some()),
            "stop": self.icons.stop.is_some() || light.is_some_and(|icons| icons.stop.is_some()) || dark.is_some_and(|icons| icons.stop.is_some()),
            "attach": self.icons.attach.is_some() || light.is_some_and(|icons| icons.attach.is_some()) || dark.is_some_and(|icons| icons.attach.is_some()),
            "voice": self.icons.voice.is_some() || light.is_some_and(|icons| icons.voice.is_some()) || dark.is_some_and(|icons| icons.voice.is_some()),
            "tools": self.icons.tools.is_some() || light.is_some_and(|icons| icons.tools.is_some()) || dark.is_some_and(|icons| icons.tools.is_some()),
            "knowledge": self.icons.knowledge.is_some() || light.is_some_and(|icons| icons.knowledge.is_some()) || dark.is_some_and(|icons| icons.knowledge.is_some()),
            "more-skills": self.icons.more_skills.is_some() || light.is_some_and(|icons| icons.more_skills.is_some()) || dark.is_some_and(|icons| icons.more_skills.is_some()),
            "daily-work": self.icons.daily_work.is_some() || light.is_some_and(|icons| icons.daily_work.is_some()) || dark.is_some_and(|icons| icons.daily_work.is_some()),
            "content-creation": self.icons.content_creation.is_some() || light.is_some_and(|icons| icons.content_creation.is_some()) || dark.is_some_and(|icons| icons.content_creation.is_some()),
            "research": self.icons.research.is_some() || light.is_some_and(|icons| icons.research.is_some()) || dark.is_some_and(|icons| icons.research.is_some()),
            "design": self.icons.design.is_some() || light.is_some_and(|icons| icons.design.is_some()) || dark.is_some_and(|icons| icons.design.is_some()),
            "read-aloud": self.icons.read_aloud.is_some() || light.is_some_and(|icons| icons.read_aloud.is_some()) || dark.is_some_and(|icons| icons.read_aloud.is_some()),
            "copy": self.icons.copy.is_some() || light.is_some_and(|icons| icons.copy.is_some()) || dark.is_some_and(|icons| icons.copy.is_some()),
            "sidebar": self.icons.sidebar.is_some() || light.is_some_and(|icons| icons.sidebar.is_some()) || dark.is_some_and(|icons| icons.sidebar.is_some()),
        })
        .to_string();
        JS_BOOTSTRAP
            .replace("%SKIN%", &skin)
            .replace("%MODE%", &mode)
            .replace("%TARGET%", &target)
            .replace("%CSS%", &css)
            .replace("%BACKGROUND%", &background)
            .replace("%ICONS%", &icons)
    }
}

const JS_BOOTSTRAP: &str = r#"(function(){
if(window.__doubaoSkinRuntime&&typeof window.__doubaoSkinRuntime.destroy==='function')window.__doubaoSkinRuntime.destroy();
var SKIN=%SKIN%,MODE=%MODE%,TARGET=%TARGET%,CSS=%CSS%,BG=%BACKGROUND%,ICONS=%ICONS%;
var media=window.matchMedia&&window.matchMedia('(prefers-color-scheme:dark)');
var observer=null,timer=null,pending=null,original={root:null,body:null};
function attrState(el,name){return {had:el.hasAttribute(name),value:el.getAttribute(name)};}
function restoreAttr(el,name,state){if(!el||!state)return;if(state.had)el.setAttribute(name,state.value);else el.removeAttribute(name);}
function rememberOriginal(){
  var e=document.documentElement,b=document.body;
  if(e&&!original.root)original.root={dataTheme:attrState(e,'data-theme'),dataSkin:attrState(e,'data-skin'),dataSkinTarget:attrState(e,'data-skin-target')};
  if(b&&!original.body)original.body={themeMode:attrState(b,'theme-mode')};
}
function appMode(){
  var e=document.documentElement,b=document.body,values=[e&&e.getAttribute('data-theme'),e&&e.getAttribute('theme-mode'),b&&b.getAttribute('theme-mode'),b&&b.getAttribute('data-theme')];
  for(var i=0;i<values.length;i++){if(values[i]==='dark'||values[i]==='light')return values[i];}
  return media&&media.matches?'dark':'light';
}
function chosenMode(){return MODE==='auto'?appMode():MODE;}
function iconTarget(el){if(el.matches&&el.matches('svg,img'))return el;return el.querySelector&&el.querySelector('svg,img');}
function markIcons(){
  var rules={'new-task':/new work|new task|新工作任务|新建任务/i,scheduled:/scheduled|定时任务/i,'more-skills':/更多技能|more skills?/i,knowledge:/企业知识|enterprise knowledge/i,skills:/技能\s*[·•]\s*连接器|伙伴|skills?/i,connector:/^\s*(connector|连接器)\s*$/i,cloud:/cloud|云盘/i,remote:/remote|手机遥控/i,conversation:/conversation|主对话/i,project:/project|项目|看看/i,confirm:/confirm|按需确认/i,'daily-work':/处理日常工作|^工作任务$|daily work|work task/i,'content-creation':/内容创作|content creation/i,research:/完成调研分析|调研分析|research/i,design:/设计与创意|design and creative/i,'read-aloud':/自动播报|朗读|read aloud|speaker|静音|mute/i,sidebar:/打开侧栏|关闭侧栏|sidebar/i,copy:/复制|copy/i,send:/send|发送|提交/i,stop:/stop|停止|取消生成/i,attach:/attach|附件|上传|文件/i,voice:/voice|语音|麦克风|Auto\s*(高|低)/i,tools:/^\s*(tool|tools|工具|更多)\s*$/i};
  function markNearbyText(slot){
    if(!ICONS[slot])return;
    Array.from(document.querySelectorAll('span')).forEach(function(label){
      var text=(label.textContent||'').trim();if(!text||text.length>32||!rules[slot].test(text))return;
      var node=label.parentElement;
      for(var depth=0;node&&depth<5;depth++,node=node.parentElement){var target=Array.from(node.children||[]).find(function(child){return child.matches&&child.matches('svg,img');});if(target){target.setAttribute('data-doubao-theme-icon',slot);return;}}
    });
  }
  document.querySelectorAll('[data-doubao-theme-icon]').forEach(function(el){el.removeAttribute('data-doubao-theme-icon');});
  document.querySelectorAll('button,[role=button],a,[role=link],[draggable=true]').forEach(function(el){
    var text=[el.getAttribute('aria-label'),el.getAttribute('title'),el.getAttribute('description'),el.textContent].filter(Boolean).join(' ');
    Object.keys(rules).some(function(slot){if(ICONS[slot]&&rules[slot].test(text)){var target=iconTarget(el);if(target)target.setAttribute('data-doubao-theme-icon',slot);return true;}return false;});
  });
  markNearbyText('new-task');markNearbyText('conversation');markNearbyText('daily-work');markNearbyText('content-creation');markNearbyText('research');markNearbyText('design');
  if(ICONS.copy){var sideIcon=document.querySelector('[data-doubao-theme-icon=sidebar]'),side=sideIcon&&(sideIcon.closest('button,[role=button]')||sideIcon);if(side&&side.parentElement){var siblings=Array.from(side.parentElement.children).filter(function(el){return el.matches('button,[role=button]');}),sideIndex=siblings.indexOf(side);if(sideIndex>0){var candidate=siblings[sideIndex-1],label=[candidate.getAttribute('aria-label'),candidate.getAttribute('title'),candidate.textContent].filter(Boolean).join(' ').trim(),target=iconTarget(candidate);if(!label&&target)target.setAttribute('data-doubao-theme-icon','copy');}}}
  if(ICONS.main){var marked=false;document.querySelectorAll('img,svg,[role=img]').forEach(function(el){var identity=[el.getAttribute('alt'),el.getAttribute('aria-label'),el.getAttribute('title'),el.getAttribute('data-testid'),el.id,el.getAttribute('class'),el.getAttribute('src')].filter(Boolean).join(' ');if(/doubao[-_ ]?(logo|icon)|豆包(?:工作)?(?:图标|logo)|(?:app|product|brand)[-_ ]?logo/i.test(identity)){el.setAttribute('data-doubao-theme-icon','main');marked=true;}});if(!marked){var candidates=Array.from(document.querySelectorAll('main img,main svg,[role=main] img,[role=main] svg,img,svg')).filter(function(el){var r=el.getBoundingClientRect(),s=getComputedStyle(el);return r.width>=52&&r.width<=144&&r.height>=52&&r.height<=144&&r.width/r.height>.72&&r.width/r.height<1.38&&r.left>Math.max(180,innerWidth*.18)&&r.top>40&&r.bottom<innerHeight*.72&&s.visibility!=='hidden'&&s.display!=='none';}).sort(function(a,b){var ar=a.getBoundingClientRect(),br=b.getBoundingClientRect();return Math.abs(ar.left+ar.width/2-innerWidth*.52)+Math.abs(ar.top+ar.height/2-innerHeight*.39)-Math.abs(br.left+br.width/2-innerWidth*.52)-Math.abs(br.top+br.height/2-innerHeight*.39);});if(candidates[0])candidates[0].setAttribute('data-doubao-theme-icon','main');}}
  document.querySelectorAll('[data-doubao-theme-composer]').forEach(function(el){el.removeAttribute('data-doubao-theme-composer');});
  document.querySelectorAll('textarea,[contenteditable=true],[role=textbox]').forEach(function(editor){
    var r=editor.getBoundingClientRect(),s=getComputedStyle(editor);if(r.width<120||r.height<12||s.display==='none'||s.visibility==='hidden')return;
    var node=editor.parentElement,best=null,bestWidth=0;
    for(var depth=0;node&&depth<12;depth++,node=node.parentElement){var box=node.getBoundingClientRect(),style=getComputedStyle(node),isComposerSize=box.width>=Math.min(320,innerWidth*.28)&&box.height>=44&&box.height<=innerHeight*.96;if(!isComposerSize)continue;if(parseFloat(style.borderRadius)>=12&&box.width>bestWidth){best=node;bestWidth=box.width;}}
    if(best)best.setAttribute('data-doubao-theme-composer','');
  });
}
function ensureBackdrop(){
  var old=document.getElementById('doubao-skin-backdrop');
  if(!BG){if(old)old.remove();return;}
  var layer=old;
  if(!layer){layer=document.createElement('div');layer.id='doubao-skin-backdrop';layer.setAttribute('aria-hidden','true');}
  if(BG.kind==='image')layer.style.backgroundImage='url("'+BG.source+'")';
  if(BG.kind==='gradient')layer.style.backgroundImage=BG.gradient||'none';
  if(BG.kind==='video'){
    var video=layer.querySelector('video');
    if(!video){video=document.createElement('video');video.muted=true;video.loop=true;video.autoplay=true;video.playsInline=true;layer.appendChild(video);}
    if(video.src!==BG.source)video.src=BG.source||'';
    if(BG.poster)video.poster=BG.poster;
    video.play().catch(function(){});
  }
  if(document.body&&!layer.parentNode)document.body.prepend(layer);
}
function apply(){
  var selected=chosenMode(),e=document.documentElement,b=document.body;
  if(!e)return;
  rememberOriginal();
  if(MODE!=='auto'&&e.getAttribute('data-theme')!==selected)e.setAttribute('data-theme',selected);
  if(e.getAttribute('data-skin')!==SKIN)e.setAttribute('data-skin',SKIN);
  if(TARGET&&e.getAttribute('data-skin-target')!==TARGET)e.setAttribute('data-skin-target',TARGET);
  if(MODE!=='auto'&&b&&b.getAttribute('theme-mode')!==selected)b.setAttribute('theme-mode',selected);
  if(CSS!==null&&document.head){var s=document.getElementById('doubao-skin-style');if(!s){s=document.createElement('style');s.id='doubao-skin-style';s.setAttribute('nonce','argus-csp-token');document.head.appendChild(s);}if(s.textContent!==CSS)s.textContent=CSS;}
  ensureBackdrop();markIcons();
}
function schedule(){if(pending!==null)return;pending=setTimeout(function(){pending=null;apply();},0);}
function start(){if(observer||!document.documentElement)return;apply();observer=new MutationObserver(schedule);observer.observe(document.documentElement,{attributes:true,childList:true,subtree:true,attributeFilter:['data-theme','data-skin','data-skin-target','theme-mode','aria-label','title']});}
function destroy(){
  if(observer)observer.disconnect();
  document.removeEventListener('DOMContentLoaded',start);
  if(media&&media.removeEventListener)media.removeEventListener('change',schedule);
  if(pending!==null)clearTimeout(pending);
  if(timer)clearInterval(timer);
  if(window.__doubaoSkinTimer===timer)window.__doubaoSkinTimer=null;
  var e=document.documentElement,b=document.body,style=document.getElementById('doubao-skin-style'),backdrop=document.getElementById('doubao-skin-backdrop');
  if(style)style.remove();if(backdrop)backdrop.remove();
  if(original.root){restoreAttr(e,'data-skin',original.root.dataSkin);restoreAttr(e,'data-skin-target',original.root.dataSkinTarget);if(MODE!=='auto')restoreAttr(e,'data-theme',original.root.dataTheme);}
  if(original.body&&MODE!=='auto')restoreAttr(b,'theme-mode',original.body.themeMode);
  document.querySelectorAll('[data-doubao-theme-icon]').forEach(function(el){el.removeAttribute('data-doubao-theme-icon');});
  document.querySelectorAll('[data-doubao-theme-composer]').forEach(function(el){el.removeAttribute('data-doubao-theme-composer');});
  if(window.__doubaoSkinRuntime&&window.__doubaoSkinRuntime.destroy===destroy)delete window.__doubaoSkinRuntime;
}
start();
document.addEventListener('DOMContentLoaded',start);
if(media&&media.addEventListener)media.addEventListener('change',schedule);
if(window.__doubaoSkinTimer)clearInterval(window.__doubaoSkinTimer);timer=setInterval(schedule,2000);window.__doubaoSkinTimer=timer;
window.__doubaoSkinRuntime={skin:SKIN,target:TARGET,destroy:destroy};
})();"#;

fn appearance_variant_css(mode: &str, variant: &AppearanceVariant) -> String {
    let mut vars: Vec<(&str, String)> = Vec::new();
    let (text_rgb, text_hex, tertiary_alpha, quaternary_alpha, accent_text) = if mode == "dark" {
        ("255,255,255", "#ffffff", "0.62", "0.55", "#77b0ff")
    } else {
        ("0,0,0", "#000000", "0.72", "0.66", "#16356f")
    };
    for (name, value) in [
        ("--s-color-text-primary", text_hex.to_string()),
        ("--s-color-text-primary-raw", text_rgb.to_string()),
        ("--s-color-text-secondary", format!("rgba({text_rgb},0.85)")),
        (
            "--s-color-text-tertiary",
            format!("rgba({text_rgb},{tertiary_alpha})"),
        ),
        (
            "--s-color-text-quaternary",
            format!("rgba({text_rgb},{quaternary_alpha})"),
        ),
        ("--s-color-text-disable", format!("rgba({text_rgb},0.20)")),
        ("--s-color-brand-primary-default", accent_text.to_string()),
        ("--dbx-text-primary", format!("rgba({text_rgb},0.90)")),
        ("--dbx-text-secondary", format!("rgba({text_rgb},0.70)")),
        (
            "--dbx-text-tertiary",
            format!("rgba({text_rgb},{tertiary_alpha})"),
        ),
        ("--dbx-text-disable", format!("rgba({text_rgb},0.30)")),
        ("--dbx-text-highlight", accent_text.to_string()),
        ("--dbx-text-markdown", format!("rgba({text_rgb},0.90)")),
    ] {
        vars.push((name, value));
    }
    for (name, value) in [
        (
            "--input-guidance-input-container-background",
            variant.composer.background.as_deref(),
        ),
        (
            "--color-composebox-background",
            variant.composer.background.as_deref(),
        ),
        (
            "--cr-composebox-background-color",
            variant.composer.background.as_deref(),
        ),
        (
            "--input-guidance-input-container-border",
            variant.composer.border.as_deref(),
        ),
        (
            "--input-guidance-input-editor-color",
            variant.composer.text_color.as_deref(),
        ),
        (
            "--color-composebox-font",
            variant.composer.text_color.as_deref(),
        ),
        (
            "--input-guidance-input-editor-placeholder-color",
            variant.composer.placeholder_color.as_deref(),
        ),
        (
            "--cr-composebox-input-placeholder-color",
            variant.composer.placeholder_color.as_deref(),
        ),
        (
            "--input-guidance-input-editor-caret-color",
            variant.composer.caret_color.as_deref(),
        ),
        (
            "--semi-color-focus-border",
            variant.composer.caret_color.as_deref(),
        ),
        (
            "--color-composebox-input-icon",
            variant.composer.icon_color.as_deref(),
        ),
        (
            "--color-composebox-submit-button-background",
            variant.composer.send_button_background.as_deref(),
        ),
        (
            "--color-composebox-submit-button-icon",
            variant.composer.send_button_icon_color.as_deref(),
        ),
        (
            "--chat-bg-color",
            variant.content.chat_background.as_deref(),
        ),
        (
            "--chatarea-bg-color",
            variant.content.chat_background.as_deref(),
        ),
        (
            "--g-send-msg-bubble-bg",
            variant.content.user_message_background.as_deref(),
        ),
        (
            "--g-send-msg-bubble-text",
            variant.content.user_message_text.as_deref(),
        ),
        (
            "--color-g-send-msg-bubble-text",
            variant.content.user_message_text.as_deref(),
        ),
        (
            "--g-msg-bubble-bg",
            variant.content.assistant_message_background.as_deref(),
        ),
        (
            "--g-msg-bubble-text",
            variant.content.assistant_message_text.as_deref(),
        ),
        (
            "--chat-md-codeblock-bg-color",
            variant.content.code_background.as_deref(),
        ),
        (
            "--chat-md-codeblock-header-bg-color",
            variant.content.code_header_background.as_deref(),
        ),
        (
            "--skin-selection-color",
            variant.content.selection_color.as_deref(),
        ),
        (
            "--skin-scrollbar-color",
            variant.content.scrollbar_color.as_deref(),
        ),
        (
            "--skin-scrollbar-hover-color",
            variant.content.scrollbar_hover_color.as_deref(),
        ),
    ] {
        push_css_value(&mut vars, name, value);
    }
    for (name, value, min, max, unit) in [
        (
            "--input-guidance-input-container-radius",
            variant.composer.radius,
            0.0,
            48.0,
            "px",
        ),
        (
            "--composebox-border-radius",
            variant.composer.radius,
            0.0,
            48.0,
            "px",
        ),
        (
            "--cr-composebox-expanded-border-radius",
            variant.composer.radius,
            0.0,
            48.0,
            "px",
        ),
        (
            "--input-guidance-input-container-min-height",
            variant.composer.min_height,
            32.0,
            160.0,
            "px",
        ),
        (
            "--input-guidance-input-container-max-height",
            variant.composer.max_height,
            80.0,
            520.0,
            "px",
        ),
        (
            "--max-composebox-height",
            variant.composer.max_height,
            80.0,
            520.0,
            "px",
        ),
        (
            "--cr-composebox-max-height",
            variant.composer.max_height,
            80.0,
            520.0,
            "px",
        ),
        (
            "--input-guidance-input-container-padding",
            variant.composer.padding,
            4.0,
            40.0,
            "px",
        ),
        (
            "--input-guidance-input-container-gap",
            variant.composer.gap,
            0.0,
            32.0,
            "px",
        ),
        (
            "--chat-input-icon-size",
            variant.composer.icon_size,
            12.0,
            40.0,
            "px",
        ),
        (
            "--chat-input-icon-size-full",
            variant.composer.icon_size,
            12.0,
            40.0,
            "px",
        ),
    ] {
        push_number(&mut vars, name, value, min, max, unit);
    }
    for (name, path) in [
        ("--skin-icon-main", variant.icons.main.as_deref()),
        ("--skin-icon-new-task", variant.icons.new_task.as_deref()),
        ("--skin-icon-scheduled", variant.icons.scheduled.as_deref()),
        ("--skin-icon-skills", variant.icons.skills.as_deref()),
        ("--skin-icon-cloud", variant.icons.cloud.as_deref()),
        ("--skin-icon-remote", variant.icons.remote.as_deref()),
        (
            "--skin-icon-conversation",
            variant.icons.conversation.as_deref(),
        ),
        ("--skin-icon-project", variant.icons.project.as_deref()),
        ("--skin-icon-confirm", variant.icons.confirm.as_deref()),
        ("--skin-icon-connector", variant.icons.connector.as_deref()),
        ("--skin-icon-send", variant.icons.send.as_deref()),
        ("--skin-icon-stop", variant.icons.stop.as_deref()),
        ("--skin-icon-attach", variant.icons.attach.as_deref()),
        ("--skin-icon-voice", variant.icons.voice.as_deref()),
        ("--skin-icon-tools", variant.icons.tools.as_deref()),
        ("--skin-icon-knowledge", variant.icons.knowledge.as_deref()),
        (
            "--skin-icon-more-skills",
            variant.icons.more_skills.as_deref(),
        ),
        (
            "--skin-icon-daily-work",
            variant.icons.daily_work.as_deref(),
        ),
        (
            "--skin-icon-content-creation",
            variant.icons.content_creation.as_deref(),
        ),
        ("--skin-icon-research", variant.icons.research.as_deref()),
        ("--skin-icon-design", variant.icons.design.as_deref()),
        (
            "--skin-icon-read-aloud",
            variant.icons.read_aloud.as_deref(),
        ),
        ("--skin-icon-copy", variant.icons.copy.as_deref()),
        ("--skin-icon-sidebar", variant.icons.sidebar.as_deref()),
    ] {
        if let Some(uri) = path.and_then(asset_data_uri) {
            vars.push((name, format!("url(\"{uri}\")")));
        }
    }
    let mut css =
        format!("\nhtml[data-skin][data-theme={mode}],html[data-skin][data-theme={mode}] body{{");
    for (name, value) in vars {
        css.push_str(name);
        css.push(':');
        css.push_str(&value);
        css.push_str("!important;");
    }
    css.push('}');
    css
}

impl ThemeIcons {
    fn any(&self) -> bool {
        self.main.is_some()
            || self.new_task.is_some()
            || self.scheduled.is_some()
            || self.skills.is_some()
            || self.cloud.is_some()
            || self.remote.is_some()
            || self.conversation.is_some()
            || self.project.is_some()
            || self.confirm.is_some()
            || self.connector.is_some()
            || self.send.is_some()
            || self.stop.is_some()
            || self.attach.is_some()
            || self.voice.is_some()
            || self.tools.is_some()
            || self.knowledge.is_some()
            || self.more_skills.is_some()
            || self.daily_work.is_some()
            || self.content_creation.is_some()
            || self.research.is_some()
            || self.design.is_some()
            || self.read_aloud.is_some()
            || self.copy.is_some()
            || self.sidebar.is_some()
    }
}

fn css_atom<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty()
        || value
            .chars()
            .any(|c| matches!(c, ';' | '{' | '}' | '\n' | '\r'))
    {
        fallback
    } else {
        value
    }
}

fn css_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn push_css_value(vars: &mut Vec<(&'static str, String)>, name: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        let clean = css_atom(value, "");
        if !clean.is_empty() {
            vars.push((name, clean.to_string()));
        }
    }
}

fn push_number(
    vars: &mut Vec<(&'static str, String)>,
    name: &'static str,
    value: Option<f32>,
    min: f32,
    max: f32,
    unit: &str,
) {
    if let Some(value) = value.filter(|value| value.is_finite()) {
        vars.push((name, format!("{}{unit}", value.clamp(min, max))));
    }
}

fn asset_data_uri(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let mime = match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => "application/octet-stream",
    };
    Some(format!(
        "data:{mime};base64,{}",
        crate::ws::base64_encode(&bytes)
    ))
}

fn safe_asset(root: &Path, relative: &str, label: &str) -> Result<PathBuf, String> {
    let candidate = Path::new(relative);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("{label} must stay inside the theme directory"));
    }
    let path = root.join(candidate);
    if !path.is_file() {
        return Err(format!("{label} not found: {}", path.display()));
    }
    Ok(path)
}

fn composer_from_meta(meta: ComposerMeta) -> ComposerStyle {
    ComposerStyle {
        background: meta.background,
        border: meta.border,
        text_color: meta.text_color,
        placeholder_color: meta.placeholder_color,
        caret_color: meta.caret_color,
        icon_color: meta.icon_color,
        send_button_background: meta.send_button_background,
        send_button_icon_color: meta.send_button_icon_color,
        radius: meta.radius,
        min_height: meta.min_height,
        max_height: meta.max_height,
        padding: meta.padding,
        gap: meta.gap,
        icon_size: meta.icon_size,
    }
}

fn content_from_meta(meta: ContentMeta) -> ContentStyle {
    ContentStyle {
        chat_background: meta.chat_background,
        user_message_background: meta.user_message_background,
        user_message_text: meta.user_message_text,
        assistant_message_background: meta.assistant_message_background,
        assistant_message_text: meta.assistant_message_text,
        code_background: meta.code_background,
        code_header_background: meta.code_header_background,
        selection_color: meta.selection_color,
        scrollbar_color: meta.scrollbar_color,
        scrollbar_hover_color: meta.scrollbar_hover_color,
    }
}

fn icons_from_meta(root: &Path, meta: IconsMeta, label: &str) -> Result<ThemeIcons, String> {
    let resolve = |relative: Option<String>, slot: &str| {
        relative
            .as_deref()
            .map(|value| safe_asset(root, value, &format!("{label} {slot} icon")))
            .transpose()
    };
    Ok(ThemeIcons {
        main: resolve(meta.main, "main")?,
        new_task: resolve(meta.new_task, "newTask")?,
        scheduled: resolve(meta.scheduled, "scheduled")?,
        skills: resolve(meta.skills, "skills")?,
        cloud: resolve(meta.cloud, "cloud")?,
        remote: resolve(meta.remote, "remote")?,
        conversation: resolve(meta.conversation, "conversation")?,
        project: resolve(meta.project, "project")?,
        confirm: resolve(meta.confirm, "confirm")?,
        connector: resolve(meta.connector, "connector")?,
        send: resolve(meta.send, "send")?,
        stop: resolve(meta.stop, "stop")?,
        attach: resolve(meta.attach, "attach")?,
        voice: resolve(meta.voice, "voice")?,
        tools: resolve(meta.tools, "tools")?,
        knowledge: resolve(meta.knowledge, "knowledge")?,
        more_skills: resolve(meta.more_skills, "moreSkills")?,
        daily_work: resolve(meta.daily_work, "dailyWork")?,
        content_creation: resolve(meta.content_creation, "contentCreation")?,
        research: resolve(meta.research, "research")?,
        design: resolve(meta.design, "design")?,
        read_aloud: resolve(meta.read_aloud, "readAloud")?,
        copy: resolve(meta.copy, "copy")?,
        sidebar: resolve(meta.sidebar, "sidebar")?,
    })
}

fn appearance_variant_from_meta(
    root: &Path,
    meta: AppearanceVariantMeta,
    label: &str,
) -> Result<AppearanceVariant, String> {
    Ok(AppearanceVariant {
        composer: composer_from_meta(meta.composer),
        content: content_from_meta(meta.content),
        icons: icons_from_meta(root, meta.icons, label)?,
    })
}

/// Load a theme by id (looked up in `themes_dir`) or by directory path.
pub fn load(themes_dir: &Path, id_or_path: &str) -> Result<Theme, String> {
    let direct = Path::new(id_or_path);
    let path = if direct.is_dir() {
        direct.to_path_buf()
    } else {
        themes_dir.join(id_or_path)
    };
    let meta_text = fs::read_to_string(path.join("theme.json"))
        .map_err(|e| format!("cannot read {}: {e}", path.join("theme.json").display()))?;
    let meta: ThemeMeta =
        serde_json::from_str(&meta_text).map_err(|e| format!("bad theme.json: {e}"))?;
    let appearance_is_explicit = meta.appearance.is_some();
    let appearance = meta
        .appearance
        .unwrap_or_else(|| ThemeAppearance::from_legacy_mode(meta.mode));
    let mode = appearance.mode();
    let css = fs::read_to_string(path.join("theme.css"))
        .map_err(|e| format!("cannot read theme.css: {e}"))?;
    let icon = path.join("icon.icns");
    let schema_version = meta.schema_version;
    let preview_image = meta
        .preview
        .image
        .as_deref()
        .map(|relative| safe_asset(&path, relative, "preview image"))
        .transpose()?;
    let preview_mode = meta.preview.appearance.unwrap_or(match appearance {
        ThemeAppearance::LightOnly => ThemeMode::Light,
        ThemeAppearance::DarkOnly | ThemeAppearance::Both => ThemeMode::Dark,
    });
    if preview_mode == ThemeMode::Auto {
        return Err("主题预览外观必须是 light 或 dark".into());
    }
    let preview_accent = meta
        .preview
        .accent
        .as_deref()
        .map(|value| parse_color_value(value).ok_or_else(|| "主题预览强调色无效".to_string()))
        .transpose()?;
    let version = meta.version.unwrap_or_else(|| "1.0.0".into());
    if !valid_package_version(&version) {
        return Err("主题包版本必须使用 1.0.0 这样的格式".into());
    }
    let author = meta.author.unwrap_or_default().trim().to_string();
    if author.chars().count() > 40 {
        return Err("主题作者名称过长".into());
    }
    let store_category = meta.store.category;
    if store_category.as_deref().is_some_and(|category| {
        !matches!(
            category,
            "pure" | "atmosphere" | "gallery" | "codex" | "brand" | "misc"
        )
    }) {
        return Err("主题商店分类无效".into());
    }
    let store_tags = meta
        .store
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .collect::<Vec<_>>();
    if store_tags.len() > 8
        || store_tags
            .iter()
            .any(|tag| tag.is_empty() || tag.chars().count() > 16)
    {
        return Err("主题商店标签无效".into());
    }
    let store_sort_order = meta.store.sort_order;
    let veil = meta.veil.unwrap_or_else(default_veil).clamp(0.0, 1.0);
    let (background, background_spec) = match meta.background {
        Some(BackgroundMeta::Path(relative)) => {
            let resolved = safe_asset(&path, &relative, "background").ok();
            let spec = resolved.as_ref().map(|source| BackgroundSpec {
                kind: BackgroundKind::Image,
                source: Some(source.clone()),
                poster: None,
                gradient: None,
                fit: "cover".into(),
                position: "center".into(),
                opacity: 1.0,
                veil,
                blur: 0.0,
                animation: "none".into(),
                duration_seconds: 18.0,
                legacy: true,
            });
            (resolved, spec)
        }
        Some(BackgroundMeta::Options(options)) => {
            let kind = match options.kind.as_deref() {
                Some("image") => BackgroundKind::Image,
                Some("video") => BackgroundKind::Video,
                Some("gradient") => BackgroundKind::Gradient,
                Some(other) => return Err(format!("unsupported background type: {other}")),
                None if options.gradient.is_some() => BackgroundKind::Gradient,
                None => BackgroundKind::Image,
            };
            let source = match (kind, options.src.as_deref()) {
                (BackgroundKind::Gradient, _) => None,
                (_, Some(relative)) => Some(safe_asset(&path, relative, "background source")?),
                _ => return Err("background source is required for image and video".into()),
            };
            let poster = options
                .poster
                .as_deref()
                .map(|relative| safe_asset(&path, relative, "background poster"))
                .transpose()?;
            if kind == BackgroundKind::Gradient && options.gradient.is_none() {
                return Err("background gradient is required for gradient type".into());
            }
            let preview = if kind == BackgroundKind::Video {
                poster.clone()
            } else {
                source.clone()
            };
            let fit = match options.fit.as_deref() {
                Some("contain") => "contain",
                Some("fill") => "fill",
                Some("none") => "none",
                Some("scale-down") => "scale-down",
                _ => "cover",
            };
            let animation = match options.animation.as_deref() {
                Some("ken-burns") => "ken-burns",
                Some("drift") => "drift",
                Some("pulse") => "pulse",
                _ => "none",
            };
            let spec = BackgroundSpec {
                kind,
                source,
                poster,
                gradient: options.gradient,
                fit: fit.into(),
                position: css_atom(options.position.as_deref().unwrap_or("center"), "center")
                    .into(),
                opacity: options.opacity.unwrap_or(1.0).clamp(0.0, 1.0),
                veil: options.veil.unwrap_or(veil).clamp(0.0, 1.0),
                blur: options.blur.unwrap_or(0.0).clamp(0.0, 40.0),
                animation: animation.into(),
                duration_seconds: options.duration_seconds.unwrap_or(18.0).clamp(2.0, 120.0),
                legacy: false,
            };
            (preview, Some(spec))
        }
        None => (None, None),
    };
    let surface_opacity = meta
        .surface_opacity
        .map(|value| value.clamp(0.35, 1.0))
        .or_else(|| (schema_version >= 2 && background_spec.is_some()).then_some(0.68));

    let typography = Typography {
        ui: meta.typography.ui,
        body: meta.typography.body,
        code: meta.typography.code,
        scale: meta.typography.scale,
        line_height: meta.typography.line_height,
        assets: meta
            .typography
            .assets
            .into_iter()
            .map(|font| {
                Ok(FontAsset {
                    family: font.family,
                    path: safe_asset(&path, &font.src, "font")?,
                    weight: font.weight,
                    style: font.style,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
    };
    let layout = ThemeLayout {
        density: meta.layout.density,
        sidebar_width: meta.layout.sidebar_width,
        chat_max_width: meta.layout.chat_max_width,
        composer_max_width: meta.layout.composer_max_width,
        self_message_max_width: meta.layout.self_message_max_width,
        chat_margin: meta.layout.chat_margin,
    };
    let composer = composer_from_meta(meta.composer);
    let content = content_from_meta(meta.content);
    let icons = icons_from_meta(&path, meta.icons, "theme")?;
    let variants = ThemeVariants {
        light: meta
            .variants
            .light
            .map(|variant| appearance_variant_from_meta(&path, variant, "light variant"))
            .transpose()?,
        dark: meta
            .variants
            .dark
            .map(|variant| appearance_variant_from_meta(&path, variant, "dark variant"))
            .transpose()?,
    };
    if appearance_is_explicit
        && appearance == ThemeAppearance::Both
        && (variants.light.is_none() || variants.dark.is_none())
    {
        return Err("appearance \"both\" requires both variants.light and variants.dark".into());
    }
    let effects = ThemeEffects {
        radius_scale: meta.effects.radius_scale,
        shadow: meta.effects.shadow,
        blur: meta.effects.blur,
        motion: meta.effects.motion,
        transition_ms: meta.effects.transition_ms,
    };
    Ok(Theme {
        schema_version: meta.schema_version,
        name: meta.name.unwrap_or_else(|| meta.id.clone()),
        id: meta.id,
        description: meta.description.unwrap_or_default(),
        version,
        author,
        preview_image,
        preview_mode,
        preview_accent,
        store_category,
        store_tags,
        store_sort_order,
        css,
        icon: icon.exists().then_some(icon),
        background,
        veil,
        surface_opacity,
        appearance,
        mode,
        background_spec,
        typography,
        layout,
        composer,
        content,
        icons,
        variants,
        effects,
        path,
    })
}

/// List all themes in `themes_dir` (directories containing theme.json).
pub fn list(themes_dir: &Path) -> Vec<Theme> {
    let mut dirs: Vec<PathBuf> = match fs::read_dir(themes_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir() && p.join("theme.json").exists())
            .collect(),
        Err(_) => return Vec::new(),
    };
    dirs.sort();
    let mut themes: Vec<Theme> = dirs
        .iter()
        .filter_map(|p| {
            let id = p.file_name()?.to_str()?;
            load(themes_dir, id).ok()
        })
        .collect();
    themes.sort_by(|left, right| {
        left.store_sort_order
            .unwrap_or(u32::MAX)
            .cmp(&right.store_sort_order.unwrap_or(u32::MAX))
            .then_with(|| left.name.cmp(&right.name))
    });
    themes
}

/// Bundled themes followed by user-installed themes. A user-installed theme
/// with the same id replaces its bundled copy, which also makes package
/// updates visible without changing the app bundle.
pub fn list_available(bundled_dir: &Path, installed_dir: &Path) -> Vec<Theme> {
    let mut themes = list(bundled_dir);
    for installed in list(installed_dir) {
        if let Some(index) = themes.iter().position(|theme| theme.id == installed.id) {
            themes[index] = installed;
        } else {
            themes.push(installed);
        }
    }
    themes
}

pub fn list_installed() -> Vec<Theme> {
    list_available(&default_themes_dir(), &user_themes_dir())
}

pub fn fetch_store_catalog(url: &str) -> Result<StoreCatalog, String> {
    validate_store_url(url)?;
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(20)))
        .build()
        .into();
    let mut response = agent
        .get(url)
        .call()
        .map_err(|_| "暂时无法连接主题商店".to_string())?;
    let mut reader = response.body_mut().as_reader().take(MAX_CATALOG_BYTES + 1);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|_| "读取主题商店失败".to_string())?;
    if bytes.len() as u64 > MAX_CATALOG_BYTES {
        return Err("主题商店返回的数据过大".into());
    }
    let catalog: StoreCatalog =
        serde_json::from_slice(&bytes).map_err(|_| "主题商店数据格式不正确".to_string())?;
    validate_store_catalog(&catalog)?;
    Ok(catalog)
}

pub fn cache_store_preview(
    catalog_url: &str,
    item: &StoreTheme,
    cache_dir: &Path,
) -> Result<Option<PathBuf>, String> {
    let Some(asset) = item
        .thumbnail_url
        .as_deref()
        .or(item.preview_url.as_deref())
        .or(item.icon_url.as_deref())
    else {
        return Ok(None);
    };
    let url = resolve_store_url(catalog_url, asset)?;
    let extension = asset
        .split('?')
        .next()
        .and_then(|path| Path::new(path).extension())
        .and_then(|value| value.to_str())
        .filter(|value| matches!(*value, "jpg" | "jpeg" | "png" | "webp" | "svg"))
        .unwrap_or("jpg");
    let destination = cache_dir.join(format!("{}.{}", item.id, extension));
    if destination.is_file()
        && destination
            .metadata()
            .map(|meta| meta.len() > 0)
            .unwrap_or(false)
    {
        return Ok(Some(destination));
    }
    fs::create_dir_all(cache_dir).map_err(|_| "无法创建主题商店缓存".to_string())?;
    download_file(&url, &destination, 20 * 1024 * 1024, None)?;
    Ok(Some(destination))
}

pub fn download_and_install_store_theme(
    catalog_url: &str,
    item: &StoreTheme,
    installed_dir: &Path,
) -> Result<Theme, String> {
    validate_theme_id(&item.id)?;
    if item.sha256.len() != 64 || !item.sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("主题包缺少有效校验信息".into());
    }
    let url = resolve_store_url(catalog_url, &item.package_url)?;
    let temporary = std::env::temp_dir().join(format!(
        "doubao-skin-store-{}-{}",
        std::process::id(),
        unique_stamp()
    ));
    fs::create_dir_all(&temporary).map_err(|_| "无法准备主题下载".to_string())?;
    let package = temporary.join(format!("{}.zip", item.id));
    let result = (|| {
        download_file(
            &url,
            &package,
            MAX_PACKAGE_BYTES,
            Some(item.sha256.as_str()),
        )?;
        install_theme_package(&package, installed_dir)
    })();
    let _ = fs::remove_dir_all(&temporary);
    result
}

pub fn install_theme_package(package: &Path, installed_dir: &Path) -> Result<Theme, String> {
    if !package.is_file()
        || package
            .extension()
            .and_then(|value| value.to_str())
            .is_none_or(|value| !value.eq_ignore_ascii_case("zip"))
    {
        return Err("请选择 .zip 主题包".into());
    }
    let package_size = package
        .metadata()
        .map_err(|_| "无法读取主题包".to_string())?
        .len();
    if package_size > MAX_PACKAGE_BYTES {
        return Err("主题包过大，最多支持 200 MB".into());
    }

    fs::create_dir_all(installed_dir).map_err(|_| "无法创建主题安装目录".to_string())?;
    let staging = installed_dir.join(format!(
        ".install-{}-{}",
        std::process::id(),
        unique_stamp()
    ));
    fs::create_dir(&staging).map_err(|_| "无法准备主题安装".to_string())?;

    let result = (|| {
        let source = fs::File::open(package).map_err(|_| "无法打开主题包".to_string())?;
        let mut archive =
            zip::ZipArchive::new(source).map_err(|_| "主题包已损坏，无法解压".to_string())?;
        if archive.len() > MAX_PACKAGE_ENTRIES {
            return Err("主题包包含的文件过多".into());
        }
        let mut extracted_bytes = 0u64;
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|_| "主题包已损坏，无法解压".to_string())?;
            let relative = entry
                .enclosed_name()
                .ok_or_else(|| "主题包包含不安全的文件路径".to_string())?
                .to_path_buf();
            if entry
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
            {
                return Err("主题包不能包含符号链接".into());
            }
            extracted_bytes = extracted_bytes.saturating_add(entry.size());
            if extracted_bytes > MAX_PACKAGE_CONTENT_BYTES {
                return Err("主题包解压后的内容过大".into());
            }
            let destination = staging.join(relative);
            if entry.is_dir() {
                fs::create_dir_all(&destination).map_err(|_| "无法创建主题文件夹".to_string())?;
                continue;
            }
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|_| "无法创建主题文件夹".to_string())?;
            }
            let mut output =
                fs::File::create(&destination).map_err(|_| "无法写入主题文件".to_string())?;
            std::io::copy(&mut entry, &mut output).map_err(|_| "无法写入主题文件".to_string())?;
        }

        let root = find_packaged_theme_root(&staging)?;
        let theme = load(installed_dir, &root.to_string_lossy())?;
        validate_theme_id(&theme.id)?;
        let destination = installed_dir.join(&theme.id);
        let backup = installed_dir.join(format!(".backup-{}-{}", theme.id, unique_stamp()));
        if destination.exists() {
            if !destination.is_dir() {
                return Err("同名安装位置不是主题文件夹".into());
            }
            fs::rename(&destination, &backup).map_err(|_| "无法更新已安装主题".to_string())?;
        }
        if let Err(error) = fs::rename(&root, &destination) {
            if backup.exists() {
                let _ = fs::rename(&backup, &destination);
            }
            return Err(format!("无法安装主题：{error}"));
        }
        if backup.exists() {
            let _ = fs::remove_dir_all(&backup);
        }
        load(installed_dir, &theme.id)
    })();
    if staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn validate_store_catalog(catalog: &StoreCatalog) -> Result<(), String> {
    if catalog.schema_version != 1 {
        return Err("暂不支持这个主题商店版本".into());
    }
    if catalog.themes.len() > 500 {
        return Err("主题商店返回的主题过多".into());
    }
    for item in &catalog.themes {
        validate_theme_id(&item.id)?;
        if item.name.trim().is_empty() {
            return Err("主题商店包含未命名主题".into());
        }
        if item.package_url.trim().is_empty() {
            return Err("主题商店包含无效主题包".into());
        }
        if !item.version.is_empty() && !valid_package_version(&item.version) {
            return Err("主题商店包含无效版本号".into());
        }
        if item.tags.len() > 8
            || item
                .tags
                .iter()
                .any(|tag| tag.trim().is_empty() || tag.chars().count() > 16)
        {
            return Err("主题商店包含无效标签".into());
        }
    }
    Ok(())
}

fn validate_store_url(url: &str) -> Result<(), String> {
    if url.starts_with("https://")
        || url.starts_with("http://127.0.0.1")
        || url.starts_with("http://localhost")
    {
        Ok(())
    } else {
        Err("主题商店地址必须使用 HTTPS".into())
    }
}

fn resolve_store_url(catalog_url: &str, value: &str) -> Result<String, String> {
    if value.starts_with("https://") {
        return Ok(value.to_string());
    }
    if value.starts_with("http://") {
        validate_store_url(value)?;
        return Ok(value.to_string());
    }
    if value.contains("..") || value.contains('\0') {
        return Err("主题商店包含不安全的下载地址".into());
    }
    if value.starts_with('/') {
        let scheme_end = catalog_url
            .find("://")
            .ok_or_else(|| "主题商店地址无效".to_string())?;
        let authority_end = catalog_url[scheme_end + 3..]
            .find('/')
            .map(|index| scheme_end + 3 + index)
            .unwrap_or(catalog_url.len());
        return Ok(format!("{}{}", &catalog_url[..authority_end], value));
    }
    let base = catalog_url
        .rsplit_once('/')
        .map(|(base, _)| base)
        .ok_or_else(|| "主题商店地址无效".to_string())?;
    Ok(format!("{base}/{value}"))
}

fn download_file(
    url: &str,
    destination: &Path,
    max_bytes: u64,
    expected_sha256: Option<&str>,
) -> Result<(), String> {
    validate_store_url(url)?;
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();
    let response = agent
        .get(url)
        .call()
        .map_err(|_| "下载主题失败，请稍后再试".to_string())?;
    let mut reader = response.into_body().into_reader();
    let mut output = fs::File::create(destination).map_err(|_| "无法保存主题下载".to_string())?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|_| "下载主题失败，请稍后再试".to_string())?;
        if read == 0 {
            break;
        }
        total = total.saturating_add(read as u64);
        if total > max_bytes {
            let _ = fs::remove_file(destination);
            return Err("下载的主题文件过大".into());
        }
        output
            .write_all(&buffer[..read])
            .map_err(|_| "无法保存主题下载".to_string())?;
        hasher.update(&buffer[..read]);
    }
    if total == 0 {
        let _ = fs::remove_file(destination);
        return Err("下载的主题文件为空".into());
    }
    if let Some(expected) = expected_sha256 {
        let actual = hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if !actual.eq_ignore_ascii_case(expected) {
            let _ = fs::remove_file(destination);
            return Err("主题包校验失败，请刷新商店后重试".into());
        }
    }
    Ok(())
}

fn find_packaged_theme_root(staging: &Path) -> Result<PathBuf, String> {
    if staging.join("theme.json").is_file() && staging.join("theme.css").is_file() {
        return Ok(staging.to_path_buf());
    }
    let mut roots = fs::read_dir(staging)
        .map_err(|_| "无法读取解压后的主题包".to_string())?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.is_dir() && path.join("theme.json").is_file() && path.join("theme.css").is_file()
        })
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err("主题包必须包含一个 theme.json 和 theme.css".into());
    }
    Ok(roots.remove(0))
}

fn validate_theme_id(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        && id
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric());
    if valid {
        Ok(())
    } else {
        Err("主题 ID 只能使用字母、数字、短横线、下划线和点".into())
    }
}

fn valid_package_version(value: &str) -> bool {
    let core = value.split(['-', '+']).next().unwrap_or_default();
    let parts = core.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
}

fn unique_stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use zip::write::SimpleFileOptions;

    static THEME_STORE_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn platform_directories_keep_product_data_below_the_system_base() {
        assert_eq!(
            product_directory(Some(PathBuf::from("/platform/data"))),
            PathBuf::from("/platform/data/Doubao Skin")
        );
        assert_eq!(
            product_directory(Some(PathBuf::from(r"C:\Users\tester\AppData\Local"))),
            PathBuf::from(r"C:\Users\tester\AppData\Local").join("Doubao Skin")
        );
    }

    #[test]
    fn theme_store_uses_public_default_and_env_override() {
        assert_eq!(
            DEFAULT_THEME_STORE_URL,
            "https://doubao-skin.idevlab.dev/themes/catalog.json"
        );

        let _lock = THEME_STORE_ENV_LOCK.lock().unwrap();
        let key = "DOUBAO_SKIN_THEME_STORE_URL";
        let previous = std::env::var_os(key);
        let expected_override = "https://preview.example/themes/catalog.json";
        std::env::set_var(key, expected_override);
        let actual = theme_store_url();
        if let Some(previous) = previous {
            std::env::set_var(key, previous);
        } else {
            std::env::remove_var(key);
        }
        assert_eq!(actual, expected_override);
    }

    fn temporary_test_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "doubao-skin-{label}-{}-{}",
            std::process::id(),
            unique_stamp()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_theme_package(path: &Path, entries: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        for (name, contents) in entries {
            archive
                .start_file(*name, SimpleFileOptions::default())
                .unwrap();
            archive.write_all(contents).unwrap();
        }
        archive.finish().unwrap();
    }

    #[test]
    fn installs_a_theme_package_and_updates_the_same_id() {
        let root = temporary_test_dir("install");
        let installed = root.join("installed");
        let package = root.join("custom.zip");
        write_theme_package(
            &package,
            &[
                (
                    "custom-theme/theme.json",
                    r#"{"id":"custom-theme","name":"第一次安装"}"#.as_bytes(),
                ),
                ("custom-theme/theme.css", b":root{--B500:#123456;}"),
            ],
        );
        let first = install_theme_package(&package, &installed).unwrap();
        assert_eq!(first.id, "custom-theme");
        assert_eq!(first.name, "第一次安装");

        write_theme_package(
            &package,
            &[
                (
                    "custom-theme/theme.json",
                    r#"{"id":"custom-theme","name":"更新后"}"#.as_bytes(),
                ),
                ("custom-theme/theme.css", b":root{--B500:#654321;}"),
            ],
        );
        let updated = install_theme_package(&package, &installed).unwrap();
        assert_eq!(updated.name, "更新后");
        assert_eq!(list(&installed).len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_zip_path_traversal() {
        let root = temporary_test_dir("traversal");
        let installed = root.join("installed");
        let package = root.join("unsafe.zip");
        write_theme_package(
            &package,
            &[
                ("../escaped.txt", b"no"),
                ("theme.json", br#"{"id":"unsafe"}"#),
                ("theme.css", b":root{}"),
            ],
        );
        let error = install_theme_package(&package, &installed).unwrap_err();
        assert!(error.contains("不安全"), "unexpected error: {error}");
        assert!(!root.join("escaped.txt").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn validates_and_resolves_store_catalog_entries() {
        let catalog: StoreCatalog = serde_json::from_str(
            r#"{
              "schemaVersion": 1,
              "themes": [{
                "id": "custom-theme",
                "name": "自定义主题",
                "description": "测试",
                "packageUrl": "/themes/packages/custom-theme.zip",
                "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "thumbnailUrl": "custom-theme.card.jpg"
              }]
            }"#,
        )
        .unwrap();
        validate_store_catalog(&catalog).unwrap();
        assert_eq!(
            resolve_store_url(
                "https://example.com/themes/catalog.json",
                &catalog.themes[0].package_url
            )
            .unwrap(),
            "https://example.com/themes/packages/custom-theme.zip"
        );
        assert_eq!(
            resolve_store_url(
                "https://example.com/themes/catalog.json",
                catalog.themes[0].thumbnail_url.as_deref().unwrap()
            )
            .unwrap(),
            "https://example.com/themes/custom-theme.card.jpg"
        );
    }

    #[test]
    fn loads_bundled_themes() {
        let themes = list(&default_themes_dir());
        assert!(themes.len() >= 4, "expected bundled themes, got {themes:?}");
        assert!(
            themes.iter().all(|theme| theme.schema_version == 2),
            "all bundled themes should use the v2 manifest"
        );
        for theme in themes.iter().filter(|theme| theme.id != "pure-dark") {
            assert!(
                theme.typography.body.is_some()
                    && theme.layout.chat_max_width.is_some()
                    && theme.composer.background.is_some()
                    && theme.content.assistant_message_text.is_some(),
                "{} is missing a v2 theme section",
                theme.id
            );
        }
        for theme in &themes {
            assert_eq!(
                theme.version, "1.0.0",
                "{} needs a package version",
                theme.id
            );
            assert_eq!(theme.author, "豆皮", "{} needs an author", theme.id);
            assert!(
                theme
                    .preview_image
                    .as_ref()
                    .is_some_and(|path| path.is_file()),
                "{} needs a bundled preview image",
                theme.id
            );
            assert!(
                theme.preview_accent.is_some(),
                "{} needs a preview accent",
                theme.id
            );
            assert!(
                theme.store_category.is_some(),
                "{} needs a store category",
                theme.id
            );
            assert!(
                !theme.store_tags.is_empty(),
                "{} needs store tags",
                theme.id
            );
            assert!(
                theme.store_sort_order.is_some(),
                "{} needs store ordering",
                theme.id
            );
            assert_eq!(
                theme.appearance,
                ThemeAppearance::Both,
                "{} must declare both appearances",
                theme.id
            );
            assert_eq!(
                theme.mode,
                ThemeMode::Auto,
                "{} must follow Doubao appearance",
                theme.id
            );
            assert!(
                theme.variants.light.is_some(),
                "{} needs a light variant",
                theme.id
            );
            assert!(
                theme.variants.dark.is_some(),
                "{} needs a dark variant",
                theme.id
            );
            let effective = theme.effective_css();
            assert!(
                effective.contains("[data-theme=light]"),
                "{} needs light appearance CSS",
                theme.id
            );
            assert!(
                effective.contains("[data-theme=dark]"),
                "{} needs dark appearance CSS",
                theme.id
            );
            let snippet = String::from_utf8(theme.snippet()).unwrap();
            assert!(snippet.contains("MODE=\"auto\""));
            assert!(snippet.contains("html{color-scheme:light dark}"));
            if theme.background_spec.is_some() {
                assert!(
                    theme.surface_opacity.is_some(),
                    "{} needs a foreground surface opacity",
                    theme.id
                );
                assert!(
                    effective.contains("--skin-surface-opacity:"),
                    "{} needs surface opacity CSS",
                    theme.id
                );
            }
        }
        let violet = themes
            .iter()
            .find(|t| t.id == "violet-night")
            .expect("violet-night");
        assert_eq!(violet.name, "暗夜紫");
        let snippet = String::from_utf8(violet.snippet()).unwrap();
        assert!(snippet.contains("nonce=\"argus-csp-token\""));
        assert!(snippet.contains("var SKIN=\"violet-night\",MODE=\"auto\""));
        assert!(snippet.contains("html{color-scheme:light dark}"));
        assert!(snippet.contains("--s-color-bg-body:#16131f"));
        assert!(!violet.swatches(4).is_empty());
        assert_eq!(violet.swatches(1)[0], 0x0d0b16);
        let pv = violet.preview_colors();
        assert_eq!(pv.sidebar.rgb, 0x1f1a2c);
        assert_eq!(pv.main.rgb, 0x16131f);
        assert_eq!(pv.accent.rgb, 0x9d7bea);
        let pure = themes
            .iter()
            .find(|t| t.id == "pure-dark")
            .expect("pure-dark");
        let pure_preview = pure.preview_style();
        assert_eq!(pure.preview_mode, ThemeMode::Dark);
        assert_eq!(pure_preview.colors.sidebar.rgb, 0x17161e);
        assert_eq!(pure_preview.colors.main.rgb, 0x121419);
        assert_eq!(pure_preview.colors.accent.rgb, 0x3370eb);
        assert_eq!(pure_preview.text.rgb, 0xf7f8fa);
        assert_eq!(pure_preview.input.rgb, 0x1d1f25);
        assert_eq!(pure_preview.input_border.rgb, 0x6694f0);

        let cyber = themes
            .iter()
            .find(|t| t.id == "cyber-neon")
            .expect("cyber-neon");
        let cyber_css = cyber.effective_css();
        assert!(cyber_css.contains("--s-color-text-secondary:rgba(255,255,255,0.85)!important"));
        assert!(cyber_css.contains("--dbx-text-tertiary:rgba(255,255,255,0.62)!important"));
        assert!(cyber_css.contains("--s-color-brand-primary-default:#77b0ff!important"));
        assert!(
            cyber_css.contains(".md-box-root{--md-box-color-fg:var(--dbx-text-markdown)!important")
        );
        assert!(cyber_css.contains(
            ".bg-g-send-msg-bubble-bg .md-box-root{--md-box-color-fg:var(--color-g-send-msg-bubble-text)!important"
        ));

        let qq = themes
            .iter()
            .find(|t| t.id == "qq-light-blue")
            .expect("qq-light-blue");
        let qq_css = qq.effective_css();
        assert!(qq_css.contains("--s-color-text-secondary:rgba(0,0,0,0.85)!important"));
        assert!(qq_css.contains("--s-color-text-tertiary:rgba(0,0,0,0.72)!important"));
        assert!(qq_css.contains("--dbx-text-highlight:#16356f!important"));
        assert_eq!(qq.preview_mode, ThemeMode::Light);
        assert_eq!(
            qq.preview_style().text.rgb,
            0x35475a,
            "the QQ light preview must use its runtime light appearance text color"
        );

        let whale = themes
            .iter()
            .find(|t| t.id == "gallery-whale-maid")
            .expect("gallery-whale-maid");
        assert!(whale
            .effective_css()
            .contains("--s-color-text-quaternary:rgba(0,0,0,0.66)!important"));

        let snack = themes
            .iter()
            .find(|t| t.id == "doubao-snack-giggle")
            .expect("doubao-snack-giggle");
        assert_eq!(snack.surface_opacity, Some(0.52));
        let snack_preview = snack.preview_style();
        assert!((snack_preview.background_veil - 0.04).abs() < 0.0001);
        assert_eq!(snack_preview.background_base, 0xfff8ed);
        assert!(snack_preview.icons.main.is_some());
        assert!(snack_preview.icons.daily_work.is_some());
        assert!(snack_preview.icons.read_aloud.is_some());
        assert!(snack
            .effective_css()
            .contains("background-image:var(--skin-bg-image)!important"));

        let dessert = themes
            .iter()
            .find(|t| t.id == "doubao-dessert-giggle")
            .expect("doubao-dessert-giggle");
        let dessert_preview = dessert.preview_style();
        assert!(dessert.icons.main.is_none());
        assert!(dessert_preview.icons.main.is_some());
        assert!(dessert_preview.icons.new_task.is_some());
        assert!(dessert_preview.icons.voice.is_some());
    }

    #[test]
    fn preview_colors_preserve_theme_alpha() {
        let rgba = parse_preview_color("rgba(189,153,153,0.16)").unwrap();
        assert_eq!(rgba.rgb, 0xbd9999);
        assert!((rgba.alpha - 0.16).abs() < 0.0001);

        let border = parse_preview_color("1px solid rgba(122,78,41,.28)").unwrap();
        assert_eq!(border.rgb, 0x7a4e29);
        assert!((border.alpha - 0.28).abs() < 0.0001);

        let opaque = parse_preview_color("rgb(53,41,112)").unwrap();
        assert_eq!(opaque.rgb, 0x352970);
        assert_eq!(opaque.alpha, 1.0);

        let hex_alpha = parse_preview_color("#1f1a2cd9!important").unwrap();
        assert_eq!(hex_alpha.rgb, 0x1f1a2c);
        assert!((hex_alpha.alpha - 0xd9 as f32 / 255.0).abs() < 0.0001);

        let whale = load(&default_themes_dir(), "gallery-whale-maid").unwrap();
        let preview = whale.preview_style();
        assert_eq!(preview.colors.main.rgb, 0xbd9999);
        assert!((preview.colors.main.alpha - 0.16).abs() < 0.0001);
        assert_eq!(preview.input.rgb, 0xffffff);
        assert!((preview.input.alpha - 0.96).abs() < 0.0001);
        assert_eq!(preview.input_border.rgb, 0x7a4e29);
        assert!((preview.input_border.alpha - 0.28).abs() < 0.0001);
        assert_eq!(preview.composer_placeholder.rgb, 0x352970);
        assert!((preview.composer_placeholder.alpha - 0.60).abs() < 0.0001);
    }

    #[test]
    fn every_bundled_theme_preview_has_valid_translucent_colors() {
        let themes = list(&default_themes_dir());
        assert_eq!(
            themes.len(),
            30,
            "audit must cover the complete bundled catalog"
        );

        let mut translucent_themes = Vec::new();
        for theme in &themes {
            let preview = theme.preview_style();
            let visible_colors = [
                preview.colors.sidebar,
                preview.colors.main,
                preview.colors.accent,
                preview.surface,
                preview.text,
                preview.input,
                preview.input_border,
                preview.composer_text,
                preview.composer_placeholder,
                preview.composer_icon,
            ];
            for color in visible_colors {
                assert!(
                    color.rgb <= 0x00ff_ffff,
                    "{} has invalid preview RGB",
                    theme.id
                );
                assert!(
                    color.alpha.is_finite() && (0.0..=1.0).contains(&color.alpha),
                    "{} has invalid preview alpha {}",
                    theme.id,
                    color.alpha
                );
            }
            if visible_colors.iter().any(|color| color.alpha < 0.9999) {
                translucent_themes.push(theme.id.as_str());
            }
        }

        assert_eq!(
            translucent_themes.len(),
            29,
            "29 bundled themes currently rely on at least one translucent preview color"
        );
    }

    #[test]
    fn runtime_surface_opacity_outranks_variant_and_legacy_background_rules() {
        let mut theme =
            load(&default_themes_dir(), "doubao-snack-giggle").expect("doubao-snack-giggle");
        theme.surface_opacity = Some(0.40);
        let css = theme.effective_css();

        assert!(
            css.contains(
                "html:root[data-skin],html:root[data-skin] body{--skin-surface-opacity:0.4;"
            ),
            "the runtime profile needs equal-or-higher specificity than appearance variants"
        );
        assert!(
            css.contains(
                "html:root[data-skin] body{background-image:var(--skin-bg-image)!important;}"
            ),
            "legacy background themes must not retain a second fixed dark gradient"
        );
        for expected in [
            "--s-color-bg-body:rgba(var(--s-color-bg-body-raw),0.22000001)!important",
            "--dbx-bg-body-web:rgba(var(--dbx-bg-body-web-raw),0.3)!important",
            "--s-color-bg-base:rgba(var(--s-color-bg-base-raw),0.26)!important",
            "--color-composebox-background:rgba(var(--s-color-bg-float-raw),0.48000002)!important",
        ] {
            assert!(css.contains(expected), "missing runtime value {expected}");
        }
    }

    #[test]
    fn doubao_work_greeting_animation_mask_is_clipped() {
        let mut theme =
            load(&default_themes_dir(), "doubao-snack-giggle").expect("doubao-snack-giggle");
        theme.surface_opacity = Some(0.40);
        let css = theme.effective_css();

        assert!(
            css.contains(
                r#"html[data-skin][data-skin-target=doubao-work] [class*="greeting-text-"]{overflow:clip!important;}"#
            ),
            "the DoubaoWork greeting must clip its animated mask after it leaves the text bounds"
        );
        assert!(
            css.contains("--chat-bg-color:rgba(var(--s-color-bg-body-raw),0.22000001)!important"),
            "the fix must preserve the transparent chat surface"
        );
        assert!(
            !css.contains("Q0pGud"),
            "the fix must not depend on a build-specific CSS module hash"
        );
    }

    #[test]
    fn bundled_theme_css_does_not_claim_runtime_surface_priority() {
        let controlled = [
            "--s-color-bg-body:",
            "--s-color-bg-content-base:",
            "--dbx-bg-body-web:",
            "--dbx-bg-body-white:",
            "--dbx-bg-body-mac:",
            "--chat-bg-color:",
            "--chatarea-bg-color:",
        ];
        let mut offenders = Vec::new();
        for theme in list(&default_themes_dir()) {
            for (index, line) in theme.css.lines().enumerate() {
                if line.contains("!important")
                    && controlled.iter().any(|token| line.contains(token))
                {
                    offenders.push(format!("{}:{}", theme.id, index + 1));
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "theme defaults must not override the runtime slider: {}",
            offenders.join(", ")
        );
    }

    #[test]
    fn snack_theme_owns_its_icon_palette() {
        let theme =
            load(&default_themes_dir(), "doubao-snack-giggle").expect("doubao-snack-giggle");
        let css = theme.effective_css();

        for token in [
            "--snack-icon-cocoa",
            "--snack-icon-peach",
            "--snack-icon-drool",
            "--snack-icon-mint",
            "--snack-icon-grape",
            "--snack-icon-berry",
        ] {
            assert!(css.contains(token), "missing {token}");
        }
        assert!(css.contains("svg[data-doubao-theme-icon=new-task]"));
        assert!(css.contains("svg[data-doubao-theme-icon=conversation]"));
        assert!(css.contains("img[data-doubao-theme-icon=conversation]"));
        assert!(css.contains("img[data-doubao-theme-icon=daily-work]"));
        assert!(css.contains("img[data-doubao-theme-icon=content-creation]"));
        assert!(css.contains("img[data-doubao-theme-icon=research]"));
        assert!(css.contains("img[data-doubao-theme-icon=design]"));
        assert!(
            !css.contains("[data-doubao-theme-icon] svg:first-of-type"),
            "icon colors must target only the marked glyph, not its button or trailing chevron"
        );
    }

    #[test]
    fn live_theme_runtime_replaces_previous_observer() {
        let theme = load(&default_themes_dir(), "violet-night").expect("violet-night");
        let js = theme.live_js();
        assert!(js.contains("window.__doubaoSkinRuntime.destroy()"));
        assert!(js.contains("observer.disconnect()"));
        assert!(js.contains("removeEventListener('DOMContentLoaded',start)"));
        assert!(js.contains("media.removeEventListener('change',schedule)"));
        assert!(js.contains("MODE!=='auto'&&e.getAttribute('data-theme')"));
        assert!(js.contains("restoreAttr(e,'data-theme'"));
        assert!(js.contains("doubao-skin-style"));
        assert!(js.contains("doubao-skin-backdrop"));
        assert!(js.contains("[data-doubao-theme-icon]"));
        assert!(js.contains("[data-doubao-theme-composer]"));
        assert!(js.contains("new MutationObserver(schedule)"));
        assert!(
            js.contains("if(!e)return;"),
            "new-document injection must wait until documentElement exists"
        );
    }

    #[test]
    fn appearance_capability_selects_runtime_mode() {
        let root = std::env::temp_dir().join(format!("appearance-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let cases = [
            ("light-only", ThemeAppearance::LightOnly, ThemeMode::Light),
            ("dark-only", ThemeAppearance::DarkOnly, ThemeMode::Dark),
            ("both", ThemeAppearance::Both, ThemeMode::Auto),
        ];
        for (id, appearance, mode) in cases {
            let path = root.join(id);
            std::fs::create_dir_all(&path).unwrap();
            let variants = if appearance == ThemeAppearance::Both {
                serde_json::json!({"light": {}, "dark": {}})
            } else {
                serde_json::json!({})
            };
            std::fs::write(
                path.join("theme.json"),
                serde_json::json!({
                    "id": id,
                    "appearance": id,
                    "variants": variants,
                })
                .to_string(),
            )
            .unwrap();
            std::fs::write(path.join("theme.css"), "html{}").unwrap();

            let theme = load(&root, id).unwrap();
            assert_eq!(theme.appearance, appearance);
            assert_eq!(theme.mode, mode);
            let snippet = String::from_utf8(theme.snippet()).unwrap();
            assert!(snippet.contains(&format!("MODE=\"{}\"", mode.as_str())));
        }

        let invalid = root.join("invalid-both");
        std::fs::create_dir_all(&invalid).unwrap();
        std::fs::write(
            invalid.join("theme.json"),
            r#"{"id":"invalid-both","appearance":"both","variants":{"light":{}}}"#,
        )
        .unwrap();
        std::fs::write(invalid.join("theme.css"), "html{}").unwrap();
        assert!(load(&root, "invalid-both")
            .unwrap_err()
            .contains("requires both variants.light and variants.dark"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn v2_theme_maps_semantic_fields_and_runtime_assets() {
        let dir = std::env::temp_dir().join(format!("v2-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("icons")).unwrap();
        std::fs::write(
            dir.join("theme.json"),
            r##"{
              "schemaVersion":2,"id":"v2-test","mode":"auto",
              "background":{"type":"video","src":"loop.mp4","poster":"poster.png","veil":0.2,"animation":"pulse"},
              "surfaceOpacity":0.52,
              "typography":{"body":"Example Sans, sans-serif","code":"Example Mono","scale":1.1,"assets":[{"family":"Example Sans","src":"font.woff2","weight":"500"}]},
              "layout":{"sidebarWidth":260,"chatMaxWidth":960,"composerMaxWidth":780},
              "composer":{"background":"#fffaf8","border":"1px solid #e0b0b8","textColor":"#40251f","placeholderColor":"#765b54","caretColor":"#d85f76","iconColor":"#8f3f55","radius":24,"minHeight":56,"padding":15,"gap":11,"iconSize":22},
              "content":{"userMessageBackground":"#d85f76","codeBackground":"#f7e8eb","selectionColor":"rgba(216,95,118,.2)"},
              "icons":{"main":"icons/root-main.svg","send":"icons/send.svg","knowledge":"icons/knowledge.svg","readAloud":"icons/read-aloud.svg"},
              "variants":{"light":{"composer":{"background":"#ffffff","border":"1px solid #dddddd","placeholderColor":"#665544","iconColor":"#aa3344"},"icons":{"main":"icons/main.svg"}},"dark":{"composer":{"background":"#202124","border":"1px solid #555555"},"icons":{"main":"icons/main.svg"}}},
              "effects":{"radiusScale":1.2,"shadow":"0 8px 24px rgba(0,0,0,.12)","transitionMs":180}
            }"##,
        )
        .unwrap();
        std::fs::write(
            dir.join("theme.css"),
            "html[data-skin]{--s-color-bg-body:#fffaf8;}",
        )
        .unwrap();
        std::fs::write(dir.join("loop.mp4"), b"video").unwrap();
        image::RgbImage::from_pixel(32, 18, image::Rgb([255u8, 210, 190]))
            .save(dir.join("poster.png"))
            .unwrap();
        std::fs::write(dir.join("font.woff2"), b"font").unwrap();
        std::fs::write(
            dir.join("icons/send.svg"),
            br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("icons/main.svg"),
            br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("icons/root-main.svg"),
            br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("icons/knowledge.svg"),
            br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("icons/read-aloud.svg"),
            br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#,
        )
        .unwrap();

        let theme = load(&dir, dir.to_str().unwrap()).unwrap();
        assert_eq!(theme.schema_version, 2);
        assert_eq!(theme.mode, ThemeMode::Auto);
        assert_eq!(theme.surface_opacity, Some(0.52));
        assert_eq!(theme.background, Some(dir.join("poster.png")));
        let preview = theme.preview_style();
        assert!((preview.background_opacity - 1.0).abs() < 0.0001);
        assert!((preview.background_veil - 0.2).abs() < 0.0001);
        assert_eq!(preview.background_base, 0xfffaf8);
        assert_eq!(preview.background_fit, "cover");
        assert_eq!(preview.icons.main, Some(dir.join("icons/main.svg")));
        assert_eq!(preview.icons.send, Some(dir.join("icons/send.svg")));
        assert_eq!(preview.composer_text.rgb, 0x40251f);
        assert_eq!(preview.composer_placeholder.rgb, 0x765b54);
        assert_eq!(preview.composer_icon.rgb, 0x8f3f55);
        assert_eq!(preview.composer_min_height, 56.0);
        assert_eq!(preview.composer_padding, 15.0);
        assert_eq!(preview.composer_gap, 11.0);
        assert_eq!(preview.composer_icon_size, 22.0);
        assert_eq!(preview.sidebar_width, 260.0);
        assert_eq!(preview.chat_margin, 32.0);
        assert_eq!(preview.radius_scale, 1.2);
        let css = theme.effective_css();
        assert!(css.contains("@font-face"));
        assert!(css.contains("--sidebar-width:260px!important"));
        assert!(css.contains("--input-guidance-input-container-background:#fffaf8!important"));
        assert!(css.contains("--g-send-msg-bubble-bg:#d85f76!important"));
        assert!(css.contains("--skin-icon-send:url(\"data:image/svg+xml;base64,"));
        assert!(css.contains("--skin-icon-knowledge:url(\"data:image/svg+xml;base64,"));
        assert!(css.contains("--skin-icon-read-aloud:url(\"data:image/svg+xml;base64,"));
        assert!(
            !css.contains("[data-doubao-theme-icon] svg:first-of-type"),
            "button-level icon selectors also replace trailing chevrons"
        );
        assert!(css.contains("html[data-skin][data-theme=light]"));
        assert!(css.contains("--input-guidance-input-container-border:1px solid #dddddd!important"));
        assert!(css.contains("--input-guidance-input-container-border:1px solid #555555!important"));
        assert!(css.contains("--semi-color-focus-border:"));
        assert!(css.contains("[data-doubao-theme-composer]:focus-within"));
        assert!(css.contains("--skin-icon-main:url(\"data:image/svg+xml;base64,"));
        assert!(css.contains("#doubao-skin-backdrop"));
        assert!(css.contains("--skin-surface-opacity:0.52"));
        assert!(css.contains("--s-color-bg-body:rgba(var(--s-color-bg-body-raw),"));
        assert!(css.contains("#doubao-skin-backdrop{position:fixed;inset:-3%;z-index:-1;"));
        assert!(!css.contains("body>*:not(#doubao-skin-backdrop)"));
        assert!(!css.contains("--skin-bg-image"));
        let js = theme.live_js();
        assert!(js.contains("MODE=\"auto\""));
        assert!(js.contains("data:video/mp4;base64,"));
        assert!(js.contains("data:image/png;base64,"));
        assert!(js.contains("data-doubao-theme-icon"));
        assert!(js.contains("function iconTarget"));
        assert!(js.contains("function markNearbyText"));
        assert!(js.contains("markNearbyText('new-task')"));
        assert!(js.contains("markNearbyText('conversation')"));
        assert!(js.contains("markNearbyText('daily-work')"));
        assert!(js.contains("markNearbyText('content-creation')"));
        assert!(js.contains("markNearbyText('research')"));
        assert!(js.contains("markNearbyText('design')"));
        assert!(js.contains("^工作任务$"));
        assert!(js.contains("removeAttribute('data-doubao-theme-icon')"));
        assert!(js.contains("data-doubao-theme-composer"));
        assert!(js.contains("dataSkin:attrState(e,'data-skin')"));
        assert!(js.contains("dataSkinTarget:attrState(e,'data-skin-target')"));
        assert!(js.contains("restoreAttr(e,'data-skin',original.root.dataSkin)"));
        assert!(js.contains("restoreAttr(e,'data-skin-target',original.root.dataSkinTarget)"));
        assert!(
            js.contains("box.height<=innerHeight*.96"),
            "expanded skill pickers must keep their rounded outer composer shell eligible"
        );
        assert!(
            !js.contains("best=best||fallback"),
            "a square bordered inner layer must not become the composer fallback"
        );
        assert!(js.contains("\"main\":true"));
        assert!(js.contains("\"knowledge\":true"));
        assert!(js.contains("\"read-aloud\":true"));
        assert!(js.contains("[draggable=true]"));
        assert!(js.contains("Auto\\s*(高|低)"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn background_image_becomes_data_uri() {
        // build a synthetic theme with a background image in a temp dir
        let dir = std::env::temp_dir().join(format!("bg-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("theme.json"),
            r#"{"id":"bg-test","background":"bg.jpg"}"#,
        )
        .unwrap();
        std::fs::write(dir.join("theme.css"), "html{}").unwrap();
        let img = image::RgbImage::from_pixel(2560, 1600, image::Rgb([10u8, 20, 30]));
        img.save(dir.join("bg.jpg")).unwrap();

        let t = load(&dir, dir.to_str().unwrap()).unwrap();
        assert_eq!(t.appearance, ThemeAppearance::DarkOnly);
        assert_eq!(t.surface_opacity, None);
        assert_eq!(t.background.as_deref(), Some(dir.join("bg.jpg").as_path()));
        let css = t.effective_css();
        assert!(css.starts_with("html[data-skin], html[data-skin] body { --skin-bg-image: url(\"data:image/jpeg;base64,"));
        assert!(css.contains("html{}"));
        assert!(css.contains("--s-color-text-secondary:rgba(255,255,255,0.85)!important"));

        // decode the data URI and check the resize to <=1920 wide
        let b64 = css
            .split("data:image/jpeg;base64,")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let jpeg = crate::ws::base64_decode(b64);
        let decoded = image::load_from_memory(&jpeg).unwrap();
        assert_eq!(decoded.width(), 1920);
        assert_eq!(decoded.height(), 1200);
        // veil baked in: solid (10,20,30) blended 45% toward default base
        // #121317 => (13.6, 19.6, 26.9); allow JPEG noise
        let px = decoded.to_rgb8().get_pixel(960, 600).0;
        assert!((px[0] as i32 - 14).abs() <= 2, "r={}", px[0]);
        assert!((px[1] as i32 - 20).abs() <= 2, "g={}", px[1]);
        assert!((px[2] as i32 - 27).abs() <= 2, "b={}", px[2]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn veil_field_and_base_color_parsing() {
        let dir = std::env::temp_dir().join(format!("veil-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("theme.json"),
            r#"{"id":"veil-test","background":"bg.jpg","veil":0.5}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("theme.css"),
            "html[data-skin]{--s-color-bg-body:rgba(40,20,60,0.40);}",
        )
        .unwrap();
        image::RgbImage::from_pixel(64, 64, image::Rgb([200u8, 200, 200]))
            .save(dir.join("bg.jpg"))
            .unwrap();
        let t = load(&dir, dir.to_str().unwrap()).unwrap();
        assert_eq!(t.veil, 0.5);
        assert_eq!(t.base_color(), (40, 20, 60));
        // baked: 200*0.5 + base*0.5 => (120, 110, 130)
        let css = t.effective_css();
        let b64 = css
            .split("base64,")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let decoded = image::load_from_memory(&crate::ws::base64_decode(b64)).unwrap();
        let px = decoded.to_rgb8().get_pixel(10, 10).0;
        assert!((px[0] as i32 - 120).abs() <= 3, "r={}", px[0]);
        assert!((px[1] as i32 - 110).abs() <= 3, "g={}", px[1]);
        assert!((px[2] as i32 - 130).abs() <= 3, "b={}", px[2]);
        std::fs::remove_dir_all(&dir).ok();
    }
}

//! Theme loading and injection-snippet generation (port of `theme.py`).
//!
//! A theme is a directory containing:
//!   theme.json  {"id": "violet-night", "name": "...", "description": "..."}
//!   theme.css   CSS rules injected into every embedded page, scoped to
//!               html[data-skin][data-theme=dark] (see the bundled themes).
//!   icon.icns   (optional) replaces the app icon of the skin build

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::live;

/// Default location of the bundled themes: `<repo>/themes`, resolved at
/// compile time from this crate (`app/skin-core` -> repo root).
/// Override with the `DOUBAO_SKIN_THEMES_DIR` environment variable.
pub fn default_themes_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("DOUBAO_SKIN_THEMES_DIR") {
        return PathBuf::from(dir);
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../themes")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes"))
}

/// Forces dark theme + marks the page with the theme id. The MutationObserver
/// re-applies both whenever the app's own theme manager rewrites them.
fn offline_script(theme_id: &str) -> String {
    // `%s` stands in for Python's `%r` (single-quoted repr; theme ids are
    // simple ASCII slugs, so no escaping is needed).
    const SCRIPT: &str = concat!(
        "<script nonce=\"argus-csp-token\">(function(){",
        "function f(){var e=document.documentElement;",
        "if(e.getAttribute('data-theme')!=='dark')e.setAttribute('data-theme','dark');",
        "var s=%s;",
        "if(e.getAttribute('data-skin')!==s)e.setAttribute('data-skin',s);",
        "var b=document.body;",
        "if(b&&b.getAttribute('theme-mode')!=='dark')b.setAttribute('theme-mode','dark');}",
        "f();new MutationObserver(f).observe(document.documentElement,",
        "{attributes:true,attributeFilter:['data-theme','data-skin']});",
        "document.addEventListener('DOMContentLoaded',function(){f();",
        "new MutationObserver(f).observe(document.body,",
        "{attributes:true,attributeFilter:['theme-mode']});});",
        "})();</script>"
    );
    SCRIPT.replace("%s", &format!("'{theme_id}'"))
}

/// Marker bytes used to detect pages that already carry an injection.
pub const MARKER: &[u8] = b"data-skin";

/// Colors for the UI's mini theme preview, parsed from theme.css by variable
/// name (not by position).
#[derive(Debug, Clone, Copy)]
pub struct PreviewColors {
    /// sidebar strip: --dbx-bg-body-web, fallback --N50
    pub sidebar: u32,
    /// main content: --s-color-bg-body, fallback --N00
    pub main: u32,
    /// accent dot/button: --semi-color-primary, fallback --B500
    pub accent: u32,
}

impl Theme {
    /// Preview colors for the UI. Themes without a color ramp (pure-dark)
    /// fall back to neutral dark grays + a blue accent (#3370eb).
    pub fn preview_colors(&self) -> PreviewColors {
        PreviewColors {
            sidebar: self.css_color("--dbx-bg-body-web")
                .or_else(|| self.css_color("--N50"))
                .unwrap_or(0x17161e),
            main: self.css_color("--s-color-bg-body")
                .or_else(|| self.css_color("--N00"))
                .unwrap_or(0x121017),
            accent: self.css_color("--semi-color-primary")
                .or_else(|| self.css_color("--B500"))
                .unwrap_or(0x3370eb),
        }
    }

    /// Value of a css custom property as 0xRRGGBB; understands `#rrggbb`
    /// and `rgb[a](r, g, b[, a])`.
    fn css_color(&self, var: &str) -> Option<u32> {
        let start = self.css.find(var)?;
        let after = &self.css[start + var.len()..];
        let colon = after.find(':')?;
        if colon > 4 {
            return None; // the match was a prefix of a longer variable name
        }
        let value = after[colon + 1..].split(';').next()?.trim();
        parse_color_value(value)
    }
}

fn parse_color_value(value: &str) -> Option<u32> {
    if let Some(hex) = value.strip_prefix('#') {
        let hex = hex.trim();
        if hex.len() == 6 {
            return u32::from_str_radix(hex, 16).ok();
        }
        return None;
    }
    let inner = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let mut parts = inner.split(',').map(|p| p.trim());
    let r: u32 = parts.next()?.parse().ok()?;
    let g: u32 = parts.next()?.parse().ok()?;
    let b: u32 = parts.next()?.parse().ok()?;
    Some((r << 16) | (g << 8) | b)
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub description: String,
    pub css: String,
    pub icon: Option<PathBuf>,
    pub path: PathBuf,
}

#[derive(Deserialize)]
struct ThemeMeta {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

impl Theme {
    /// Snippet injected into HTML pages (offline build): a <script> forcing
    /// the dark theme attributes, plus a <style> with the theme CSS.
    pub fn snippet(&self) -> Vec<u8> {
        let mut out = offline_script(&self.id);
        out.push_str("<style nonce=\"argus-csp-token\">html{color-scheme:dark}");
        out.push_str(&self.css);
        out.push_str("</style>");
        out.into_bytes()
    }

    /// JS string evaluated in live (CDP) mode.
    pub fn live_js(&self) -> String {
        live::theme_js(self)
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
    let css = fs::read_to_string(path.join("theme.css"))
        .map_err(|e| format!("cannot read theme.css: {e}"))?;
    let icon = path.join("icon.icns");
    Ok(Theme {
        name: meta.name.unwrap_or_else(|| meta.id.clone()),
        id: meta.id,
        description: meta.description.unwrap_or_default(),
        css,
        icon: icon.exists().then_some(icon),
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
    dirs.iter().filter_map(|p| {
        let id = p.file_name()?.to_str()?;
        load(themes_dir, id).ok()
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_bundled_themes() {
        let themes = list(&default_themes_dir());
        assert!(themes.len() >= 4, "expected bundled themes, got {themes:?}");
        let violet = themes.iter().find(|t| t.id == "violet-night").expect("violet-night");
        assert_eq!(violet.name, "暗夜紫");
        let snippet = String::from_utf8(violet.snippet()).unwrap();
        assert!(snippet.contains("nonce=\"argus-csp-token\""));
        assert!(snippet.contains("var s='violet-night';"));
        assert!(snippet.contains("html{color-scheme:dark}"));
        assert!(snippet.contains("--s-color-bg-body:#16131f"));
        assert!(!violet.swatches(4).is_empty());
        assert_eq!(violet.swatches(1)[0], 0x0d0b16);
        let pv = violet.preview_colors();
        assert_eq!(pv.sidebar, 0x1f1a2c);
        assert_eq!(pv.main, 0x16131f);
        assert_eq!(pv.accent, 0x9d7bea);
        let pure = themes.iter().find(|t| t.id == "pure-dark").expect("pure-dark");
        assert_eq!(pure.preview_colors().accent, 0x3370eb);
    }
}

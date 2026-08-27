//! Offline build pipeline (port of `build.py`):
//! clone DoubaoWork.app -> inject theme -> re-sign.
//!
//! Why a clone? The original app in /Applications is protected by macOS App
//! Management (MACL) and cannot be modified even by its owner. APFS clonefile
//! makes the copy instant and free of extra disk space.
//!
//! Why resources.pak patching? The main chat UI is not loose files — its
//! pages are baked into Chromium's resources.pak as gzip-compressed entries.
//!
//! Signing: modifying sealed resources breaks the code signature, and an
//! unsigned/broken-seal app is refused by Gatekeeper ("damaged"). We re-sign
//! ad-hoc, BEFORE first launch (once LaunchServices registers the app, MACL
//! locks the bundle).

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::write::GzEncoder;
use flate2::{Compression, GzBuilder};

use crate::pak;
use crate::theme::{Theme, MARKER};

pub const SOURCE_APP: &str = "/Applications/DoubaoWork.app";

pub fn skin_app() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
        .join("Applications/DoubaoWork-Skin.app")
}

const BROWSER_RESOURCES: &str =
    "Contents/Helpers/DoubaoWork Browser.app/Contents/Frameworks/DoubaoWork Browser Framework.framework";

/// HTML entry points that live on disk (side panel / office docs / etc.)
const DISK_ENTRIES: &[&str] = &[
    "local_webcontents/apps/doubao-office/index.html",
    "local_webcontents/extensions/ai-views/side_panel.html",
    "local_webcontents/extensions/ai-views/popup.html",
    "local_webcontents/extensions/ai-views/options.html",
];

/// The framework's Resources dir: Versions contains a "Current" symlink plus
/// exactly one real version directory.
fn framework_dir(app: &Path) -> Result<PathBuf, String> {
    let base = app.join(BROWSER_RESOURCES).join("Versions");
    let mut versions: Vec<PathBuf> = fs::read_dir(&base)
        .map_err(|e| format!("cannot list {}: {e}", base.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().and_then(|n| n.to_str()) != Some("Current")
                && p.is_dir()
                && p.symlink_metadata().map(|m| !m.file_type().is_symlink()).unwrap_or(false)
        })
        .collect();
    versions.sort();
    if versions.len() != 1 {
        return Err(format!("unexpected framework versions: {versions:?}"));
    }
    Ok(versions.remove(0))
}

/// Insert snippet right after <head>. Backs up the original as .orig and
/// always restores from it first, so re-runs replace the old injection.
fn inject_into_html(path: &Path, snippet: &[u8]) -> Result<bool, String> {
    let backup = path.with_extension(format!(
        "{}.orig",
        path.extension().and_then(|e| e.to_str()).unwrap_or("")
    ));
    let text = if backup.exists() {
        fs::read(&backup)
    } else {
        let orig = fs::read(path);
        if let Ok(orig) = &orig {
            fs::write(&backup, orig)
                .map_err(|e| format!("cannot write {}: {e}", backup.display()))?;
        }
        orig
    }
    .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    if text.windows(MARKER.len()).any(|w| w == MARKER) {
        return Ok(false); // pristine backup already contains an injection? bail
    }
    let Some(pos) = find_subslice(&text, b"<head>") else {
        return Ok(false);
    };
    let pos = pos + b"<head>".len();
    let mut out = Vec::with_capacity(text.len() + snippet.len());
    out.extend_from_slice(&text[..pos]);
    out.extend_from_slice(snippet);
    out.extend_from_slice(&text[pos..]);
    fs::write(path, &out).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(true)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn gunzip(blob: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read as _;
    let mut dec = flate2::read::GzDecoder::new(blob);
    let mut out = Vec::new();
    dec.read_to_end(&mut out).ok()?;
    Some(out)
}

fn gzip_mtime0(raw: &[u8]) -> Vec<u8> {
    let mut enc: GzEncoder<Vec<u8>> =
        GzBuilder::new().mtime(0).write(Vec::new(), Compression::new(6));
    enc.write_all(raw).expect("gzip write");
    enc.finish().expect("gzip finish")
}

/// Inject the snippet into every Doubao HTML page baked into the pak.
/// Returns the resource ids that were patched.
fn patch_resources_pak(pak_path: &Path, snippet: &[u8]) -> Result<Vec<u16>, String> {
    let parsed = pak::parse(pak_path)?;
    let mut blobs: Vec<(u16, Vec<u8>)> = Vec::new();
    let mut patched = Vec::new();
    for (rid, blob) in parsed.blobs() {
        let mut out = blob.to_vec();
        if blob.starts_with(b"\x1f\x8b") {
            if let Some(mut raw) = gunzip(blob) {
                let head = &raw[..raw.len().min(4000)];
                let already = raw.windows(MARKER.len()).any(|w| w == MARKER);
                let is_doctype = {
                    let t: Vec<u8> = raw
                        .iter()
                        .skip_while(|b| b.is_ascii_whitespace())
                        .map(|b| b.to_ascii_lowercase())
                        .collect();
                    t.starts_with(b"<!doctype")
                };
                if !already
                    && is_doctype
                    && find_subslice(head, b"og:url").is_some()
                    && find_subslice(head, b"doubao").is_some()
                {
                    if let Some(pos) = find_subslice(&raw, b"<head>") {
                        let pos = pos + b"<head>".len();
                        let mut merged = Vec::with_capacity(raw.len() + snippet.len());
                        merged.extend_from_slice(&raw[..pos]);
                        merged.extend_from_slice(snippet);
                        merged.extend_from_slice(&raw[pos..]);
                        raw = merged;
                        out = gzip_mtime0(&raw);
                        patched.push(rid);
                    }
                }
            }
        }
        blobs.push((rid, out));
    }
    let rebuilt = pak::build(&blobs, &parsed.aliases, 1);
    fs::write(pak_path, &rebuilt)
        .map_err(|e| format!("cannot write {}: {e}", pak_path.display()))?;
    Ok(patched)
}

/// Clone the original app, inject `theme`, re-sign. Returns the skin app path.
pub fn apply<F: FnMut(String)>(theme: &Theme, mut log: F) -> Result<PathBuf, String> {
    let source = Path::new(SOURCE_APP);
    if !source.exists() {
        return Err(format!("original app not found: {}", source.display()));
    }
    let skin = skin_app();
    if skin.exists() {
        log(format!("removing previous build: {}", skin.display()));
        fs::remove_dir_all(&skin).map_err(|e| format!("cannot remove {}: {e}", skin.display()))?;
    }
    log(format!("cloning {} (APFS clonefile)…", source.display()));
    run(Command::new("cp").arg("-Rc").arg(source).arg(&skin))?;

    let snippet = theme.snippet();
    let resources = framework_dir(&skin)?.join("Resources");

    for rel in DISK_ENTRIES {
        let entry = resources.join(rel);
        if entry.exists() && inject_into_html(&entry, &snippet)? {
            log(format!("  injected: {rel}"));
        }
    }

    let patched = patch_resources_pak(&resources.join("resources.pak"), &snippet)?;
    log(format!("  injected into {} pak-embedded pages", patched.len()));

    if let Some(icon) = &theme.icon {
        log("  applying theme icon".into());
        fs::copy(icon, skin.join("Contents/Resources/app.icns"))
            .map_err(|e| format!("icon copy failed: {e}"))?;
        fs::copy(
            icon,
            skin.join("Contents/Helpers/DoubaoWork Browser.app/Contents/Resources/app.icns"),
        )
        .map_err(|e| format!("icon copy failed: {e}"))?;
    }

    // Re-sign BEFORE first launch: once LaunchServices registers the app,
    // MACL locks the bundle. Ad-hoc is enough for local use.
    log("re-signing (ad-hoc)…".into());
    run(Command::new("codesign")
        .args(["--force", "--deep", "--sign", "-"])
        .arg(&skin))?;
    let _ = Command::new("xattr")
        .args(["-dr", "com.apple.quarantine"])
        .arg(&skin)
        .output();
    let _ = Command::new("touch").arg(skin.join("Contents")).output();
    log(format!("done: {}", skin.display()));
    log("first launch will ask for keychain access to 'DoubaoWork Safe Storage' \
         — enter your Mac password and choose Always Allow."
        .into());
    Ok(skin)
}

pub fn remove<F: FnMut(String)>(mut log: F) -> Result<(), String> {
    let skin = skin_app();
    if skin.exists() {
        fs::remove_dir_all(&skin).map_err(|e| format!("cannot remove {}: {e}", skin.display()))?;
        log(format!("removed: {}", skin.display()));
    } else {
        log("nothing to remove".into());
    }
    Ok(())
}

fn run(cmd: &mut Command) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("cannot run {cmd:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd:?} failed with {status}"))
    }
}

"""Build pipeline: clone DoubaoWork.app -> inject theme -> re-sign.

Why a clone? The original app in /Applications is protected by macOS App
Management (MACL) and cannot be modified even by its owner. APFS clonefile
makes the copy instant and free of extra disk space.

Why resources.pak patching? The main chat UI is not loose files — its pages
are baked into Chromium's resources.pak as gzip-compressed entries.

Signing: modifying sealed resources breaks the code signature, and an
unsigned/broken-seal app is refused by Gatekeeper ("damaged"). We re-sign
ad-hoc. Side effect: on first launch macOS asks for keychain access
("DoubaoWork Safe Storage") once per (re)build.
"""
import gzip
import shutil
import subprocess
from pathlib import Path

from . import pak
from .theme import Theme

SOURCE_APP = Path("/Applications/DoubaoWork.app")
SKIN_APP = Path.home() / "Applications/DoubaoWork-Skin.app"

_BROWSER_RESOURCES = (
    "Contents/Helpers/DoubaoWork Browser.app/Contents/Frameworks/"
    "DoubaoWork Browser Framework.framework"
)

# HTML entry points that live on disk (side panel / office docs / etc.)
_DISK_ENTRIES = (
    "local_webcontents/apps/doubao-office/index.html",
    "local_webcontents/extensions/ai-views/side_panel.html",
    "local_webcontents/extensions/ai-views/popup.html",
    "local_webcontents/extensions/ai-views/options.html",
)


def _framework_dir(app: Path) -> Path:
    base = app / _BROWSER_RESOURCES / "Versions"
    versions = [p for p in base.iterdir() if p.is_dir() and p.name != "Current"]
    if len(versions) != 1:
        raise RuntimeError(f"unexpected framework versions: {versions}")
    return versions[0]


def _inject_into_html(path: Path, snippet: bytes) -> bool:
    """Insert snippet right after <head>. Backs up the original as .orig and
    always restores from it first, so re-runs replace the old injection."""
    backup = path.with_suffix(path.suffix + ".orig")
    if backup.exists():
        text = backup.read_bytes()
    else:
        text = path.read_bytes()
        backup.write_bytes(text)
    from .theme import MARKER
    if MARKER in text:  # pristine backup already contains an injection? bail
        return False
    pos = text.find(b"<head>")
    if pos == -1:
        return False
    pos += len(b"<head>")
    path.write_bytes(text[:pos] + snippet + text[pos:])
    return True


def _patch_resources_pak(pak_path: Path, snippet: bytes) -> list[int]:
    """Inject the snippet into every Doubao HTML page baked into the pak."""
    from .theme import MARKER
    data, entries, aliases = pak.parse(pak_path)
    blobs, patched = [], []
    for rid, blob in pak.iter_blobs(data, entries):
        out = blob
        if blob[:2] == b"\x1f\x8b":
            try:
                raw = gzip.decompress(blob)
            except OSError:
                raw = None
            if (
                raw is not None
                and MARKER not in raw
                and raw.lstrip().lower().startswith(b"<!doctype")
                and b"og:url" in raw[:4000]
                and b"doubao" in raw[:4000]
            ):
                pos = raw.find(b"<head>")
                if pos != -1:
                    pos += len(b"<head>")
                    raw = raw[:pos] + snippet + raw[pos:]
                    out = gzip.compress(raw, compresslevel=6, mtime=0)
                    patched.append(rid)
        blobs.append((rid, out))
    pak_path.write_bytes(pak.build(blobs, aliases))
    return patched


def apply(theme: Theme, log=print) -> Path:
    if not SOURCE_APP.exists():
        raise SystemExit(f"original app not found: {SOURCE_APP}")
    if SKIN_APP.exists():
        log(f"removing previous build: {SKIN_APP}")
        shutil.rmtree(SKIN_APP)
    log(f"cloning {SOURCE_APP} (APFS clonefile)…")
    subprocess.run(["cp", "-Rc", str(SOURCE_APP), str(SKIN_APP)], check=True)

    snippet = theme.snippet()
    framework = _framework_dir(SKIN_APP)
    resources = framework / "Resources"

    for rel in _DISK_ENTRIES:
        entry = resources / rel
        if entry.exists() and _inject_into_html(entry, snippet):
            log(f"  injected: {rel}")

    patched = _patch_resources_pak(resources / "resources.pak", snippet)
    log(f"  injected into {len(patched)} pak-embedded pages")

    if theme.icon:
        log("  applying theme icon")
        shutil.copyfile(theme.icon, SKIN_APP / "Contents/Resources/app.icns")
        shutil.copyfile(
            theme.icon,
            SKIN_APP / "Contents/Helpers/DoubaoWork Browser.app/Contents/Resources/app.icns",
        )

    # Re-sign BEFORE first launch: once LaunchServices registers the app,
    # MACL locks the bundle. Ad-hoc is enough for local use.
    log("re-signing (ad-hoc)…")
    subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(SKIN_APP)], check=True)
    subprocess.run(["xattr", "-dr", "com.apple.quarantine", str(SKIN_APP)],
                   check=False, capture_output=True)
    (SKIN_APP / "Contents").touch()
    log(f"done: {SKIN_APP}")
    log("first launch will ask for keychain access to 'DoubaoWork Safe Storage'"
        " — enter your Mac password and choose Always Allow.")
    return SKIN_APP


def remove(log=print) -> None:
    if SKIN_APP.exists():
        shutil.rmtree(SKIN_APP)
        log(f"removed: {SKIN_APP}")
    else:
        log("nothing to remove")

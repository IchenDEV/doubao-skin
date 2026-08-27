"""Theme loading and injection-snippet generation.

A theme is a directory containing:
    theme.json  {"id": "violet-night", "name": "...", "description": "...",
                 "background": "bg.jpg"  (optional)}
    theme.css   CSS rules injected into every embedded page. Use selectors
                scoped to html[data-skin="<id>"] so they win over app styles,
                and target both html and body (the app defines some tokens
                on `html, body` directly, which beats html-level inheritance).
    icon.icns   (optional) replaces the app icon of the skin build
    background image (optional, named in theme.json): exposed to theme.css as
                var(--skin-bg-image). NOTE: unlike the Rust engine
                (app/skin-core), this stdlib-only CLI embeds the image as-is
                (no resize/re-encode) — keep the file reasonably small.
"""
import base64
import json
from dataclasses import dataclass
from pathlib import Path

THEMES_DIR = Path(__file__).resolve().parent.parent / "themes"

# Forces dark theme + marks the page with the theme id. The MutationObserver
# re-applies both whenever the app's own theme manager rewrites them.
_SCRIPT = (
    '<script nonce="argus-csp-token">(function(){'
    'function f(){var e=document.documentElement;'
    "if(e.getAttribute('data-theme')!=='dark')e.setAttribute('data-theme','dark');"
    "var s=%r;"
    "if(e.getAttribute('data-skin')!==s)e.setAttribute('data-skin',s);"
    "var b=document.body;"
    "if(b&&b.getAttribute('theme-mode')!=='dark')b.setAttribute('theme-mode','dark');}"
    "f();new MutationObserver(f).observe(document.documentElement,"
    "{attributes:true,attributeFilter:['data-theme','data-skin']});"
    "document.addEventListener('DOMContentLoaded',function(){f();"
    "new MutationObserver(f).observe(document.body,"
    "{attributes:true,attributeFilter:['theme-mode']});});"
    '})();</script>'
)

MARKER = b"data-skin"


@dataclass
class Theme:
    id: str
    name: str
    description: str
    css: str
    icon: Path | None
    path: Path
    background: Path | None = None

    def effective_css(self) -> str:
        """Theme CSS with the --skin-bg-image variable prepended when the
        theme has a background image (embedded as-is, no resize)."""
        if not self.background:
            return self.css
        mime = "image/png" if self.background.suffix.lower() == ".png" else "image/jpeg"
        uri = f"data:{mime};base64," + base64.b64encode(self.background.read_bytes()).decode()
        return (f'html[data-skin], html[data-skin] body {{ --skin-bg-image: url("{uri}"); }}\n'
                + self.css)

    def snippet(self) -> bytes:
        script = _SCRIPT % self.id
        style = '<style nonce="argus-csp-token">html{color-scheme:dark}' + self.effective_css() + "</style>"
        return (script + style).encode("utf-8")


def load(theme_id_or_path: str) -> Theme:
    path = Path(theme_id_or_path)
    if not path.is_dir():
        path = THEMES_DIR / theme_id_or_path
    meta = json.loads((path / "theme.json").read_text(encoding="utf-8"))
    css = (path / "theme.css").read_text(encoding="utf-8")
    icon = path / "icon.icns"
    background = path / meta["background"] if meta.get("background") else None
    return Theme(
        id=meta["id"],
        name=meta.get("name", meta["id"]),
        description=meta.get("description", ""),
        css=css,
        icon=icon if icon.exists() else None,
        path=path,
        background=background if background and background.exists() else None,
    )


def list_themes() -> list[Theme]:
    return [load(p.name) for p in sorted(THEMES_DIR.iterdir())
            if p.is_dir() and (p / "theme.json").exists()]

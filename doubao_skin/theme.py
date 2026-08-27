"""Theme loading and injection-snippet generation.

A theme is a directory containing:
    theme.json  {"id": "violet-night", "name": "...", "description": "..."}
    theme.css   CSS rules injected into every embedded page. Use selectors
                scoped to html[data-skin="<id>"] so they win over app styles,
                and target both html and body (the app defines some tokens
                on `html, body` directly, which beats html-level inheritance).
    icon.icns   (optional) replaces the app icon of the skin build
"""
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

    def snippet(self) -> bytes:
        script = _SCRIPT % self.id
        style = '<style nonce="argus-csp-token">html{color-scheme:dark}' + self.css + "</style>"
        return (script + style).encode("utf-8")


def load(theme_id_or_path: str) -> Theme:
    path = Path(theme_id_or_path)
    if not path.is_dir():
        path = THEMES_DIR / theme_id_or_path
    meta = json.loads((path / "theme.json").read_text(encoding="utf-8"))
    css = (path / "theme.css").read_text(encoding="utf-8")
    icon = path / "icon.icns"
    return Theme(
        id=meta["id"],
        name=meta.get("name", meta["id"]),
        description=meta.get("description", ""),
        css=css,
        icon=icon if icon.exists() else None,
        path=path,
    )


def list_themes() -> list[Theme]:
    return [load(p.name) for p in sorted(THEMES_DIR.iterdir())
            if p.is_dir() and (p / "theme.json").exists()]

"""Live mode: reskin the ORIGINAL app at runtime via CDP — no file changes.

How it works:
1. (re)launch DoubaoWork with --remote-debugging-port
2. poll the debugger's /json target list; for every embedded app page
   (doubaowork:// scheme, side-panel extension pages), connect over CDP and
   evaluate a small JS that forces dark theme + installs the theme CSS
3. also register the same JS via Page.addScriptToEvaluateOnNewDocument so
   navigations keep the skin; keep watching for new pages

Trade-offs vs the offline `apply` build: no re-signing, no keychain prompt,
no Gatekeeper issue and themes can be hot-swapped — but a debug port is open
on localhost while this runs, and the theme is gone when the app quits.
"""
import json
import subprocess
import time
import urllib.request
from pathlib import Path

from .theme import Theme
from .ws import CDP

APP_BINARY = Path("/Applications/DoubaoWork.app/Contents/MacOS/DoubaoWork")
DEFAULT_PORT = 9222

# inject only into the app's own embedded pages, never into web tabs
URL_PATTERNS = ("doubaowork://", "chrome://doubaowork",
                "side_panel.html", "popup.html", "options.html")

_JS = """(function(){
  var SKIN = %s, CSS = %s;
  function f(){
    var e=document.documentElement;
    if(e.getAttribute('data-theme')!=='dark')e.setAttribute('data-theme','dark');
    if(e.getAttribute('data-skin')!==SKIN)e.setAttribute('data-skin',SKIN);
    var b=document.body;
    if(b&&b.getAttribute('theme-mode')!=='dark')b.setAttribute('theme-mode','dark');
    var s=document.getElementById('doubao-skin-style');
    if(document.head){
      if(!s){s=document.createElement('style');s.id='doubao-skin-style';
        s.setAttribute('nonce','argus-csp-token');document.head.appendChild(s);}
      if(s.textContent!==CSS)s.textContent=CSS;
    }
  }
  f();
  new MutationObserver(f).observe(document.documentElement,
    {attributes:true,attributeFilter:['data-theme','data-skin']});
  document.addEventListener('DOMContentLoaded',f);
  if(window.__doubaoSkinTimer)clearInterval(window.__doubaoSkinTimer);
  window.__doubaoSkinTimer=setInterval(f,2000);
})();"""


def theme_js(theme: Theme) -> str:
    css = "html{color-scheme:dark}" + theme.css
    return _JS % (json.dumps(theme.id), json.dumps(css))


def _port_up(port: int) -> bool:
    try:
        urllib.request.urlopen(f"http://localhost:{port}/json/version", timeout=1)
        return True
    except OSError:
        return False


def _targets(port: int) -> list[dict]:
    with urllib.request.urlopen(f"http://localhost:{port}/json", timeout=3) as r:
        return json.load(r)


def _ensure_running(port: int, log) -> None:
    if _port_up(port):
        log("debug port already up — reusing the running instance")
        return
    if not APP_BINARY.exists():
        raise SystemExit(f"app not found: {APP_BINARY}")
    # a running instance without the debug flag must be restarted first
    subprocess.run(["osascript", "-e", 'tell application "DoubaoWork" to quit'],
                   check=False, capture_output=True)
    for _ in range(20):
        if subprocess.run(["pgrep", "-f", "DoubaoWork.app/Contents/MacOS"],
                          capture_output=True).returncode != 0:
            break
        time.sleep(0.5)
    log(f"launching {APP_BINARY} --remote-debugging-port={port}")
    subprocess.Popen(
        [str(APP_BINARY), f"--remote-debugging-port={port}"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    for _ in range(60):
        if _port_up(port):
            return
        time.sleep(0.5)
    raise SystemExit("debug port did not come up")


def run(theme: Theme, port: int = DEFAULT_PORT, once: bool = False, log=print) -> None:
    _ensure_running(port, log)
    js = theme_js(theme)
    injected: set[str] = set()
    log(f"live theme: {theme.name} ({theme.id}) — watching pages…")
    while True:
        try:
            targets = _targets(port)
        except OSError:
            log("debug port went away (app quit?) — exiting")
            return
        for t in targets:
            if t.get("type") != "page":
                continue
            url = t.get("url", "")
            tid = t.get("id", "")
            if not tid or tid in injected or not any(p in url for p in URL_PATTERNS):
                continue
            try:
                cdp = CDP(t["webSocketDebuggerUrl"])
                cdp.call("Page.enable")
                cdp.call("Page.addScriptToEvaluateOnNewDocument", {"source": js})
                cdp.evaluate(js)
                cdp.close()
                injected.add(tid)
                log(f"  injected: {url[:70]}")
            except Exception as e:  # page may be mid-navigation; retry next round
                log(f"  retry later: {url[:60]} ({e})")
        if once:
            log(f"done — {len(injected)} page(s) themed")
            return
        time.sleep(2)

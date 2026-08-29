---
name: apply-doubao-theme
description: Safely list, validate, install, apply, restore, and manage offline builds of themes for the DoubaoWork desktop app.
---

# Apply Doubao Theme

Use the existing Rust CLI for every operation. Keep read-only discovery separate from actions that change installed themes or the running app.

## Find the CLI

Resolve one executable in this order:

1. Use the exact path in `DOUBAO_THEME_CLI` when it is set.
2. Use `doubao-theme` from `PATH` when available.
3. Use `/Applications/豆包主题.app/Contents/Resources/bin/doubao-theme` when it exists.
4. In this repository, use `cargo run -p skin-core --bin doubao-theme --`.

If none is available, suggest installing the standalone CLI:

```bash
curl -fsSL https://raw.githubusercontent.com/IchenDEV/doubao-skin/main/scripts/install-cli.sh | sh
```

Do not reimplement installation or application with shell scripts.

## Read-only actions

- `list --json` may run immediately to show available themes.
- `check <theme-dir> --json` may run immediately to validate an authoring directory.
- Resolve a requested theme by exact ID or explicit directory. If several names are plausible, show the matches and ask the user to choose.

## Approval gate for side effects

Before each command below, state the exact target and effect, then wait for the user's explicit approval in the current conversation. Earlier approval for a different target or action does not carry over.

- `install <package>`: installs or updates that package under the user's Doubao Skin theme directory.
- `apply <theme>`: may start or restart the selected app with its local debugging port; tell the user to save active work first. Default target is `doubao-work`. Do not add `--watch` unless the user asks for ongoing developer mode.
- `restore`: removes the live theme from responsive pages without deleting installed packages. Warn that an existing `--watch` process can apply it again.
- `build <theme>`: replaces the existing `~/Applications/DoubaoWork-Skin.app` clone, never `/Applications/DoubaoWork.app`.
- `remove-build`: deletes only `~/Applications/DoubaoWork-Skin.app`.

If approval is absent, stop after the read-only checks. Never interpret automatic Skill discovery as authorization.

## Execute and verify

1. Prefer `--json` and parse the single result envelope. Keep CLI stderr as diagnostic evidence, not user-facing product copy.
2. For `apply`, require a successful result with at least one responsive page. A listening port alone is not success.
3. For `restore`, require at least one cleaned responsive page. Zero pages is a failure that needs an open app or a stopped watcher.
4. For real-window validation, use an isolated window or conversation with no private content. Verify the selected theme marker and visible result; after restore, verify the marker and theme-owned layers are gone.
5. Never read page text, cookies, request headers, account data, attachments, tools, or workspace content.

## Result

Report the exact theme, target app, completed action, verified page count, and any remaining action. Say “主题已应用” or “已恢复默认” only after the corresponding verification succeeds.

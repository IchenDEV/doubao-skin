---
name: apply-doubao-theme
description: Safely list, validate, install, apply, restore, and manage themes for Doubao, DoubaoWork, and WorkBuddy.
---

# Apply Doubao Theme

Use the existing Rust CLI for every operation. Keep read-only discovery separate from actions that change installed themes or the running app.

## Find the CLI

Resolve one executable in this order:

1. Use the exact path in `DOUBAO_SKIN_CLI` when it is set.
2. Use `doubao-skin` from `PATH` when available.
3. In this repository, use `cargo run -p skin-core --bin doubao-skin --`.

If none is available, explain the exact target and ask before installing the standalone CLI. Use Scoop on Windows:

```powershell
scoop install https://github.com/IchenDEV/doubao-skin/releases/latest/download/doubao-skin.json
```

On macOS or Linux, use the platform-detecting installer:

```bash
curl -fsSL https://raw.githubusercontent.com/IchenDEV/doubao-skin/main/scripts/install-cli.sh | sh
```

Installing the CLI must not install, launch, or modify the desktop app. When this repository is already present, prefer `cargo run` instead of installing another copy.

The portable commands are `list`, `check`, `install`, and the authoring commands. `apply` and `restore` require macOS or Windows with the corresponding official client installed. `build` and `remove-build` are macOS-only. On Linux, do not attempt live application or an offline app clone.

Do not reimplement installation or application with shell scripts.

## Read-only actions

- `list --json` may run immediately to show available themes.
- `check <theme-dir> --json` may run immediately to validate an authoring directory.
- Read the check/list target report rather than inferring support from schema version. If the requested target is unsupported, stop and explain that the theme cannot be partially applied.
- Resolve a requested theme by exact ID or explicit directory. If several names are plausible, show the matches and ask the user to choose.

## Approval gate for side effects

Before each command below, state the exact target and effect, then wait for the user's explicit approval in the current conversation. Earlier approval for a different target or action does not carry over.

- `install <package>`: installs or updates that package under the user's Doubao Skin theme directory.
- `apply <theme> --target <doubao|doubao-work|workbuddy>` (macOS/Windows; WorkBuddy is macOS-only): may start or restart that exact app with its local debugging port; tell the user to save active work first. Use `doubao-work` only when the user gave no target and no current app context resolves it. Do not add `--watch` unless the user asks for ongoing developer mode.
- `restore` (macOS/Windows; WorkBuddy is macOS-only): removes the live theme from responsive pages without deleting installed packages. Warn that an existing `--watch` process can apply it again.
- `build <theme>` (macOS only): replaces the existing `~/Applications/DoubaoWork-Skin.app` clone, never `/Applications/DoubaoWork.app`.
- `remove-build` (macOS only): deletes only `~/Applications/DoubaoWork-Skin.app`.

If approval is absent, stop after the read-only checks. Never interpret automatic Skill discovery as authorization.

## Execute and verify

1. Prefer `--json` and parse the single result envelope. Keep CLI stderr as diagnostic evidence, not user-facing product copy.
2. For `apply`, require a successful result with at least one responsive page. A listening port alone is not success.
3. For `restore`, require at least one cleaned responsive page. Zero pages is a failure that needs an open app or a stopped watcher.
4. For real-window validation, use an isolated window or conversation with no private content. Verify the selected target and current light/dark appearance, the theme marker, and the visible result; after restore, verify the marker and theme-owned layers are gone.
5. Never read page text, cookies, request headers, account data, attachments, tools, or workspace content.

## Result

Report the exact theme, target app, completed action, verified page count, and any remaining action. Say “主题已应用” or “已恢复默认” only after the corresponding verification succeeds.

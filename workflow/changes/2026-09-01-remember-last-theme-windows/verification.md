---
id: "2026-09-01-remember-last-theme-windows"
stage: verification
status: pending
owner: "codex"
created: "2026-09-01"
based_on: plan.md
commit: "f4cbb216932e4c4066f27713bb00ca2f88d1dd95"
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: remember last theme windows

## Automated checks

- Red proof: after adding a Windows-sized login-session regression, `cargo test -p skin-core auto_theme` failed at compile time because `consume_login_open_from` still required `u32`; after generalizing to `u64` and adding the legacy alias, all 13 auto-theme tests passed.
- `cargo test -p skin-core --test bundled_theme_paths`: 3/3 passed, including `helpers/doubao-skin-agent.exe -> <package>/themes`.
- A new Windows runtime red test, `registration_rolls_back_a_helper_that_exits_during_startup`, failed before registration waited for the spawned helper and passed after the fix. The implementation now observes the helper for 750 ms, captures an early stderr failure, and removes the just-written Run value instead of reporting a false success.
- `cargo test -p doubao-skin-desktop`: 27/27 passed after the startup-confirmation fix. Coverage includes quoted Unicode paths, the 260 UTF-16 boundary, exact/stale/missing registration status, idempotent registration, spawn/read/startup-confirmation rollback, orphan cleanup, macOS helper regression, Windows LUID composition and package path derivation.
- `sh -n scripts/package/windows.sh scripts/package/verify-windows-exe.sh`: passed.
- `./scripts/check.sh rust`: passed before and after rebasing the branch onto `origin/main` `d023678`; desktop, core, authoring, bundled-path and CLI suites passed, followed by warning-as-error checks and the desktop build.
- `./scripts/check.sh all`: passed at tested code commit `f4cbb21` after the rebase, including workflow, Rust, Web tests, TypeScript, Next.js static build and audit; `git diff --check` also passed.
- `cargo check -p doubao-skin-desktop --bin doubao-skin-agent --target x86_64-pc-windows-msvc` on macOS stopped in the pre-existing `ring` C build because the host has no Windows MSVC SDK header `assert.h`; it did not reach product code and is not Windows evidence.
- Draft PR #16 CI run `33487861016` passed all six jobs after the rebase: development workflow, Web, Rust workspace, Windows x64, Windows x86 and Windows ARM64. The native Windows jobs passed their test, PE-subsystem and helper-package-layout checks.
- Downloaded the rebased run's `Windows-native-arm64` artifact. Both sidecar checks passed; the GUI ZIP SHA-256 is `f9d7451ad0eb1ff34fa17bbbf3eab6a8e8c7568ae67afc19850278d8efdb751d`. Direct ZIP inspection confirmed the expected top-level GUI, helper, licenses and bundled themes.
- Served only that checked artifact over the VMware host-only network. The guest GET returned HTTP 200 and the package was extracted to `C:\Users\idevlab\Downloads\REBASED\Doubao-Skin-Windows-arm64` before launch.

## Behavioral evidence

- Pure backend tests prove the app-owned Run value is only reported enabled when the helper exists and the quoted command matches exactly. Spawn or read-confirmation failure attempts exact rollback; rollback failure remains visibly recoverable instead of claiming success.
- `auto-theme-session.json` accepts the old `audit_session_id` field, rewrites `login_session_id`, supports values above `u32::MAX`, rejects zero and preserves once-per-login consumption.
- Shared supervisor tests preserve main-app priority, target-exit hold, next manual-launch recovery and one login-start action. Windows helper path and AuthenticationId conversion are independently covered.
- In the Windows 11 ARM VM, an older `3b31f16` package reproduced the reported regression: during a 30-second no-input observation, a blank Windows Terminal titled with `C:\WINDOWS\system32\taskli...` appeared without user action. The old source invoked `tasklist` directly.
- Rebasing onto `d023678` incorporated `b0cc90f` (`fix(windows): suppress background process consoles`), which routes background `tasklist` and `taskkill` through `CREATE_NO_WINDOW`. This is the source-level cause/fix pair for the reproduced popup.
- The rebased ARM64 package showed WorkBuddy selected, “纯暗” saved, parent “自动保持上次主题” on, child “登录时打开豆包” off, and “豆皮后台服务已注册”. Manual apply succeeded with “正在使用” and “已应用 · 专属适配 · WorkBuddy”.
- Closing the WorkBuddy window produced its native “仍在后台运行” notification. Reopening it from the green tray icon left the same theme session active: 豆皮 still showed “正在使用”. WorkBuddy's logged-out landing page itself remains white because the theme adapter targets the signed-in workspace; this VM therefore proves watcher continuity, not signed-in page styling.
- With the parent on and the child off, five passive VNC observation blocks covered 69.5 seconds and 83 samples. The first three 10.5-second blocks had maximum sampled-pixel deltas of 7, 0 and 21; the later 18- and 20-second blocks were entirely unchanged. No terminal, console, helper window, taskbar item or tray item appeared during any block.
- Full process-exit/helper takeover, a fresh Windows login and signed-in WorkBuddy page styling remain pending. The current evidence closes the reported recurring-terminal regression and the same-process tray reopen path only.

## Visual evidence

- Existing layout regression proves the control group still contains exactly two dependent switches and preserves the native Windows titlebar/resource behavior.
- VMware Fusion's Windows 11 ARM guest was observed and operated over localhost-only VNC. The rebased packaged desktop UI showed the two dependent Windows switches, WorkBuddy target support, registered parent state, disabled child state and active theme state.
- The old popup and the rebased no-popup behavior were both observed without opening a guest terminal. VNC screenshots establish visible window state only; they are not treated as registry or process-list evidence.

## Security and privacy evidence

- Registry code is confined to `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\\DoubaoSkinAutoTheme`; it never enumerates or deletes sibling values and does not read/write undocumented `StartupApproved` data.
- The Run command contains one quoted absolute helper path, rejects embedded quote/NUL and overlong UTF-16 commands, and never invokes a shell.
- Helper uses a session-local mutex and token `AuthenticationId`; Win32 token/mutex handles use owned guards. It has no listener, tray, window or administrator path.
- No official Doubao, DoubaoWork or WorkBuddy files were modified. The VM's current-user `DoubaoSkinAutoTheme` Run value intentionally remains registered to the checked rebased package so the requested parent-on/child-off state can be inspected; cleanup is still required before final verification is closed.

## Deviations and residual risk

- The Windows VM is visually controllable through VNC, but sustained text input is not reliable enough for command-output evidence. Native CI supplies compilation/PE/package proof, while the VNC run supplies the popup and visible state evidence recorded above.
- Public HKCU Run registration cannot observe Windows Settings' undocumented external disable database. UI therefore reports registration, never silently recreates a deleted value, as approved in Spec.
- A portable package move leaves an old absolute Run path until the user explicitly closes/reopens the parent switch. This is intentional to avoid overriding an external disable choice.
- The rebased branch compiled and packaged on native Windows x64/x86/ARM64 runners and its ARM64 artifact passed the targeted VNC popup check. Do not mark the whole change complete until full process-exit/helper handoff, fresh-login behavior, signed-in target styling, precise startup-value cleanup and a fresh-context or human verdict are recorded.

## Verdict

Pending. The reported recurring Windows Terminal is reproduced on the old package and absent from the rebased native ARM64 artifact across 69.5 seconds of passive observation. Local/full CI gates pass and the parent-on/child-off tray-reopen path retains an active theme session. Full helper handoff/login coverage, cleanup and the required fresh-context or human verdict remain open.

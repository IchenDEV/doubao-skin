---
id: "2026-09-01-remember-last-theme-windows"
stage: verification
status: pending
owner: "codex"
created: "2026-09-01"
based_on: plan.md
commit: "6353108f3d7a46f360f2c521e86e952123229153"
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
- `./scripts/check.sh rust`: passed at `6353108`; desktop 27, core 53, authoring 7, bundled paths 3 and CLI 4 passed, followed by warning-as-error checks and desktop build.
- `./scripts/check.sh all`: passed at `6353108`, including workflow, Rust, Web tests, TypeScript, Next.js static build and audit; `git diff --check` also passed.
- `cargo check -p doubao-skin-desktop --bin doubao-skin-agent --target x86_64-pc-windows-msvc` on macOS stopped in the pre-existing `ring` C build because the host has no Windows MSVC SDK header `assert.h`; it did not reach product code and is not Windows evidence.
- Draft PR CI run `33441260595` passed all six jobs at implementation commit `a640a79`: development workflow, Web, Rust workspace, Windows x64, Windows x86 and Windows ARM64. The x64 job ran the native core and desktop regression suites before building the desktop/CLI packages; all three Windows jobs passed the final PE and helper-package-layout checks.
- Downloaded the `Windows-native-arm64` artifact from run `33441260595`. `shasum -a 256 -c Doubao-Skin-Windows-arm64.zip.sha256` passed, and direct ZIP inspection confirmed one top-level `doubao-skin.exe`, `helpers/doubao-skin-agent.exe`, licenses and bundled themes.
- PR #15 was merged externally at `82426e6`, before the startup-confirmation commit. Pushing `6353108` to its former head branch did not create a pull-request event, so no native Windows artifact yet contains the latest fix.

## Behavioral evidence

- Pure backend tests prove the app-owned Run value is only reported enabled when the helper exists and the quoted command matches exactly. Spawn or read-confirmation failure attempts exact rollback; rollback failure remains visibly recoverable instead of claiming success.
- `auto-theme-session.json` accepts the old `audit_session_id` field, rewrites `login_session_id`, supports values above `u32::MAX`, rejects zero and preserves once-per-login consumption.
- Shared supervisor tests preserve main-app priority, target-exit hold, next manual-launch recovery and one login-start action. Windows helper path and AuthenticationId conversion are independently covered.
- In the Windows 11 ARM VM, the `82426e6` package enabled the parent switch and wrote the exact current-user Run value, but the helper was not present in the observed process list and the target reopened with its default white appearance. This reproduces the false-success behavior that motivated `6353108`; it is evidence about the old artifact, not acceptance of the fix.
- Agent single-instance count on `6353108`, main/helper watcher handoff, manual target launch, 30-second exit hold and new-login behavior remain pending a native artifact containing the fix.

## Visual evidence

- Existing layout regression proves the control group still contains exactly two dependent switches and preserves the native Windows titlebar/resource behavior.
- VMware Fusion's Windows 11 ARM guest was observed and operated over localhost-only VNC. The packaged desktop UI showed the two Windows switches and allowed the parent registration action; the target application's white post-launch state was also visible.
- VNC keyboard focus is unreliable for long command input, so command text that leaked into the browser is not treated as process or registry evidence. Normal/narrow UI, no-console helper, restored target theme and login behavior screenshots for `6353108` remain pending.

## Security and privacy evidence

- Registry code is confined to `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\\DoubaoSkinAutoTheme`; it never enumerates or deletes sibling values and does not read/write undocumented `StartupApproved` data.
- The Run command contains one quoted absolute helper path, rejects embedded quote/NUL and overlong UTF-16 commands, and never invokes a shell.
- Helper uses a session-local mutex and token `AuthenticationId`; Win32 token/mutex handles use owned guards. It has no listener, tray, window or administrator path.
- No official Doubao/DoubaoWork files were modified. The VM's current-user `DoubaoSkinAutoTheme` Run value was intentionally created by the old package during acceptance and remains registered; disabling it requires a fresh user confirmation immediately before that system-setting change.

## Deviations and residual risk

- The Windows VM is visually controllable through VNC, but sustained text input is not reliable enough for command-output evidence. Native CI supplies compilation/PE/package proof for the merged artifact, but cannot replace runtime acceptance of `6353108`.
- Public HKCU Run registration cannot observe Windows Settings' undocumented external disable database. UI therefore reports registration, never silently recreates a deleted value, as approved in Spec.
- A portable package move leaves an old absolute Run path until the user explicitly closes/reopens the parent switch. This is intentional to avoid overriding an external disable choice.
- Windows-specific product code through `82426e6` compiled and packaged on native Windows x64/x86/ARM64 runners. The latest `6353108` startup-confirmation fix has only local full-gate evidence until a new native Windows CI artifact is authorized and produced. Do not mark complete until that artifact passes VM evidence or the requirement is explicitly waived by a human verifier.

## Verdict

Pending. Local implementation and full gates pass at `6353108`; the old Windows artifact reproduces the false-success failure. A native Windows artifact containing `6353108`, real runtime/visual acceptance, cleanup, and the required fresh-context or human verdict remain open.

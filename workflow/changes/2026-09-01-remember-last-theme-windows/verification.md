---
id: "2026-09-01-remember-last-theme-windows"
stage: verification
status: pending
owner: "codex"
created: "2026-09-01"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: remember last theme windows

## Automated checks

- Red proof: after adding a Windows-sized login-session regression, `cargo test -p skin-core auto_theme` failed at compile time because `consume_login_open_from` still required `u32`; after generalizing to `u64` and adding the legacy alias, all 13 auto-theme tests passed.
- `cargo test -p skin-core --test bundled_theme_paths`: 3/3 passed, including `helpers/doubao-skin-agent.exe -> <package>/themes`.
- `cargo test -p doubao-skin-desktop`: 25/25 passed after the final compensation fix. Coverage includes quoted Unicode paths, the 260 UTF-16 boundary, exact/stale/missing registration status, idempotent registration, spawn/read-confirmation rollback, orphan cleanup, macOS helper regression, Windows LUID composition and package path derivation.
- `sh -n scripts/package/windows.sh scripts/package/verify-windows-exe.sh`: passed.
- `./scripts/check.sh rust`: passed after the final implementation; desktop 25, core 53, authoring 7, bundled paths 3 and CLI 4 passed, followed by warning-as-error checks and desktop build.
- `./scripts/check.sh all`: passed after the final read-confirmation compensation patch, including workflow, Rust, Web tests, TypeScript, Next.js static build and audit; `git diff --check` also passed.
- `cargo check -p doubao-skin-desktop --bin doubao-skin-agent --target x86_64-pc-windows-msvc` on macOS stopped in the pre-existing `ring` C build because the host has no Windows MSVC SDK header `assert.h`; it did not reach product code and is not Windows evidence.
- Draft PR CI run `33440186944` proved the Windows product builds and helper package layout on x86 and ARM64, plus the x64 product code compiled before tests. The x64 job then exposed one platform-specific test-fixture defect: a POSIX `/Test/...` path was not absolute under Windows. The fixture now constructs native absolute paths and the Win32 instance guard warning is removed; rerun is pending.

## Behavioral evidence

- Pure backend tests prove the app-owned Run value is only reported enabled when the helper exists and the quoted command matches exactly. Spawn or read-confirmation failure attempts exact rollback; rollback failure remains visibly recoverable instead of claiming success.
- `auto-theme-session.json` accepts the old `audit_session_id` field, rewrites `login_session_id`, supports values above `u32::MAX`, rejects zero and preserves once-per-login consumption.
- Shared supervisor tests preserve main-app priority, target-exit hold, next manual-launch recovery and one login-start action. Windows helper path and AuthenticationId conversion are independently covered.
- Actual HKCU Run registration, agent single-instance process count, main/helper watcher handoff, manual target launch, 30-second exit hold and new-login behavior: pending Windows VM evidence.

## Visual evidence

- Existing layout regression proves the control group still contains exactly two dependent switches and preserves the native Windows titlebar/resource behavior.
- The available VMware Fusion Windows 11 ARM VM is running, but the UI capture layer currently returns a black guest display and rejects input. No screenshot or visual acceptance is claimed from that state.
- Normal/narrow Windows window, no-console helper, restored target theme and login behavior screenshots: pending a controllable Windows session.

## Security and privacy evidence

- Registry code is confined to `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\\DoubaoSkinAutoTheme`; it never enumerates or deletes sibling values and does not read/write undocumented `StartupApproved` data.
- The Run command contains one quoted absolute helper path, rejects embedded quote/NUL and overlong UTF-16 commands, and never invokes a shell.
- Helper uses a session-local mutex and token `AuthenticationId`; Win32 token/mutex handles use owned guards. It has no listener, tray, window or administrator path.
- No official Doubao/DoubaoWork files or startup settings were changed during local work. Actual Windows cleanup and before/after official-file evidence remain pending VM acceptance.

## Deviations and residual risk

- The planned local Windows VM path is temporarily unavailable for trustworthy input/visual capture; native CI supplies compilation/PE/package proof, but cannot replace login/visual acceptance.
- Public HKCU Run registration cannot observe Windows Settings' undocumented external disable database. UI therefore reports registration, never silently recreates a deleted value, as approved in Spec.
- A portable package move leaves an old absolute Run path until the user explicitly closes/reopens the parent switch. This is intentional to avoid overriding an external disable choice.
- Windows-specific product code compiled on native Windows x64/x86/ARM64 runners in run `33440186944`; the first x64 test run failed only on the non-native absolute-path fixture described above. Do not mark complete until the corrected native CI rerun passes and VM evidence is obtained or explicitly waived by a human verifier.

## Verdict

Pending. Local implementation and gates pass, but native Windows CI and real Windows runtime/visual acceptance remain open.

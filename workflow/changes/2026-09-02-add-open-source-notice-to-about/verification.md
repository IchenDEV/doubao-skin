---
id: "2026-09-02-add-open-source-notice-to-about"
stage: verification
status: passed
owner: "codex"
created: "2026-09-02"
based_on: plan.md
commit: "7a41ea5365a6526cff807cb63e61b96f617e2bbf"
verification_mode: "human"
verified_by: "idevlab"
verified_at: "2026-09-02"
---

# Verification: add open source notice to about

## Automated checks

- Test-first failure: `cargo test -p doubao-skin-desktop ui_regression_tests --locked` failed before implementation with unresolved imports for `OFFICIAL_REPOSITORY_URL`, `OPEN_SOURCE_NOTICE`, `shows_about_entry`, `about_version`, `about_key_action`, and `AboutKeyAction`.
- After implementation, `cargo test -p doubao-skin-desktop ui_regression_tests --locked` passed 22 UI regression tests, including the three new About content, platform visibility, and modal-key handling tests.
- After the first real-window review found the notice too dense, the content regression test was changed first to require three short paragraphs. It failed against the one-line notice and passed after the shared notice gained blank lines, so the same spacing applies to macOS and Windows.
- After the follow-up request to center the content, macOS applies a native centered paragraph style to the complete credits string, while the Windows modal centers its heading, version, notice, and repository group without moving the close action.
- `cargo test -p doubao-skin-desktop --locked` passed 40 tests across the main app and helper binaries.
- `cargo check -p doubao-skin-desktop --locked` passed without project warnings. The existing future-incompatibility notice for dependency `block v0.1.6` remains unrelated.
- `./scripts/check.sh workflow` passed, including artifact validation, workflow policy cases, and portability boundaries. `git diff --check` passed.
- `./scripts/check.sh all` completed with explicit `CHECK_ALL_PASSED`; Rust tests/checks, supply-chain policy, Web tests, TypeScript, Next.js production build, and vulnerability scan passed.
- `./scripts/package.sh desktop-macos` built `dist/豆皮.app`, `Doubao-Skin-macOS-arm64.zip`, and `Doubao-Skin-macOS-arm64.dmg`. The app plist passed lint and `codesign --verify --deep --strict --verbose=2` passed. The local package uses the expected ad-hoc development signature; no certificate identity was claimed.
- Packaged plist readback: `CFBundleDisplayName=豆皮`, `CFBundleShortVersionString=0.5.0`, `CFBundleVersion=1`, and `NSHumanReadableCopyright=Copyright © 2026 豆皮贡献者`.
- Windows source/build probes remain incomplete on this host. The native packaging script correctly refused macOS with `Windows desktop packages must be built on a Windows MSVC host`. Plain cross-check stopped in `ring` because the Windows C sysroot was unavailable; `cargo xwin check` stopped in existing `psm`/`ring` assembly and `/imsvc` toolchain issues before checking the application crate.

## Behavioral evidence

- Computer Use launched the exact packaged path `/Users/chenli/.codex/worktrees/5014/doubao-work-skin/dist/豆皮.app`. A separate older `/Applications/豆皮.app` process was already running, but binary inspection showed only the `dist` executable contains the new notice; the inspected About Panel displayed that new notice.
- The real macOS application menu kept `关于豆皮` first, followed by Services, Hide, Hide Others, Show All, and Quit.
- The real AppKit About Panel exposed `豆皮 版本0.5.0 (1)`, the complete approved notice, a separate accessibility link for `https://github.com/IchenDEV/doubao-skin`, and the existing copyright.
- Selecting About again while the panel was open returned the same single dialog in the accessibility tree; no duplicate panel appeared.
- Clicking the link opened the default browser at the exact address `github.com/IchenDEV/doubao-skin`. Closing the panel returned to the same usable main window.
- Windows behavior is covered by deterministic tests for Windows-only entry visibility, shared content/version, and modal key consumption, but the packaged Windows titlebar, focus, link, close button, and state-preservation behavior still require the approved real Windows run.

## Visual evidence

- macOS dark appearance, real packaged About Panel: `/Users/chenli/.codex/visualizations/2026/09/02/doubao-skin-about-macos.png` (SHA-256 `278b92ed4f6eeec72687d9c10249713eafdef3c45c57b40994cdf044f8fa2530`).
- macOS dark appearance after the spacing correction: `/Users/chenli/.codex/visualizations/2026/09/02/doubao-skin-about-macos-spaced.png` (SHA-256 `8436742f53c9a276977012bbe857bd89c88eaeba53aa1d186c5607c08a2350b4`).
- macOS dark appearance after the centering correction: `/Users/chenli/.codex/visualizations/2026/09/02/doubao-skin-about-macos-centered.png` (SHA-256 `d41614e73a175b9a316f282fdb12c4dc72ba6ae0cc475fbc674cd413051cba78`).
- The final screenshot shows three distinct centered notice paragraphs, the centered full GitHub URL, packaged icon, app name, version/build, and copyright without clipping.
- macOS light appearance was not changed during validation because that would alter the user's system appearance. The panel uses the AppKit standard adaptive surface; a real light-appearance screenshot remains pending if required.
- Windows normal/narrow and light/dark screenshots remain pending because the encrypted Windows 11 ARM VM could not be started without the user's password.

## Security and privacy evidence

- `OFFICIAL_REPOSITORY_URL` and `OPEN_SOURCE_NOTICE` are compile-time constants. Neither is read from a theme, environment variable, remote catalogue, or user input.
- The Windows link path calls GPUI `cx.open_url` only inside the explicit link click handler. About rendering itself has no HTTP client or background task.
- macOS attaches a fixed `NSURL` only to the fixed URL range of the standard About credits string. The live click opened only the approved repository.
- The saved screenshot contains only public app identity and project information; it contains no account, conversation, theme path, or credential data.

## Deviations and residual risk

- Windows real-window validation depends on the available Windows VM and was not replaced by cross-compilation evidence. VMware Fusion found `/Users/chenli/Virtual Machines.localized/Windows 11 64-bit Arm.vmwarevm/Windows 11 64-bit Arm.vmx`, but `vmrun start` returned `A password is required for this operation`; the agent did not request or handle the password.
- The approved plan described a single boolean About state; implementation also adds one GPUI `FocusHandle` so the Windows dialog can receive focus and return focus to the main application. This is a minimal accessibility requirement, not a new state-management layer.
- The repository's fixed 1120×720 production window prevents resizing the packaged macOS application to a separate narrow size. Windows narrow-layout acceptance remains part of the pending real Windows run/probe.
- The local macOS artifacts are ad-hoc signed rather than verified against a persistent certificate fingerprint. This is sufficient for local interaction proof, not release-signature or notarization proof.

## Verdict

Passed with human-accepted residual risk. Implementation self-check passed for code, tests, full local gate, macOS packaging, real macOS About interaction, accessibility readback, single-instance behavior, and exact browser destination. On 2026-09-02, human reviewer `idevlab` explicitly accepted the recorded evidence and the remaining Windows real-window, narrow-layout, and light/dark validation gaps as sufficient for this change; those gaps remain documented above and are not represented as completed behavior.

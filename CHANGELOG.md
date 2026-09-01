# Changelog

All notable changes are recorded here. This project follows Semantic Versioning once the first stable release is published.

## 0.5.0 - 2026-09-01

### Added

- Theme package v3 contract with explicit Doubao, DoubaoWork, and WorkBuddy scope, shared/target layers, strict CSS/resource validation, migration tooling, and target-aware desktop/Web catalog UI.
- Experimental WorkBuddy 5.3.14 live theming for v2 themes, with a dedicated loopback CDP port, strict main-renderer matching, explicit restart confirmation, and no automatic relaunch after user quit.
- Standalone CLI distribution published as separate platform-specific release assets.
- One-line CLI installer: `curl -fsSL .../install-cli.sh | sh`.
- `doubao-skin --version` flag.
- Windows desktop packages built on native Windows runners for x64, x86, and ARM64, each with one `doubao-skin.exe` entry point.
- CLI binary standardized as `doubao-skin` for consistency with the product name.
- CLI-only release assets for macOS universal, Linux x64/ARM64, and Windows x64/x86/ARM64, kept separate from every desktop package.
- Scoop manifest for architecture-aware Windows CLI installation and a platform-detecting macOS/Linux installer.
- Browser-local desktop download recommendation with manual architecture selection.
- Background theme persistence with separate controls for remembering the last theme and opening the target app at login.

### Changed

- macOS universal CLI archives are signed after `lipo` with the same stable community identity as the desktop app.

### Fixed

- Windows background process checks no longer open recurring Terminal windows.
- Windows login-start registration can be removed reliably, and helper startup is verified before the UI reports success.
- Reopening WorkBuddy from the system tray keeps the active theme watcher available without forcing WorkBuddy to launch at login.

## 0.1.0 - 2026-08-29

### Added

- Native Rust theme engine and GPUI desktop application.
- Theme gallery and installable theme packages.
- Experimental native-main-chat OpenAI-compatible protocol bridge.
- CI, universal macOS packaging, checksums, and GitHub Release automation.

### Changed

- Upgraded the web application to the latest stable Next.js, React, TypeScript, and pnpm releases.
- Upgraded the Rust toolchain, GPUI revision, HTTP, archive, hashing, and supporting dependencies.
- Replaced the website's npm workflow and lockfile with pnpm.

# Changelog

All notable changes are recorded here. This project follows Semantic Versioning once the first stable release is published.

## Unreleased

### Added

- Theme package v3 contract with explicit Doubao, DoubaoWork, and WorkBuddy scope, shared/target layers, strict CSS/resource validation, migration tooling, and target-aware desktop/Web catalog UI.
- Experimental WorkBuddy 5.3.14 live theming for v2 themes, with a dedicated loopback CDP port, strict main-renderer matching, explicit restart confirmation, and no automatic relaunch after user quit.
- Standalone CLI distribution: `doubao-theme-macOS-universal.tar.gz` published as a separate release asset.
- One-line CLI installer: `curl -fsSL .../install-cli.sh | sh`.
- `doubao-theme --version` flag.

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

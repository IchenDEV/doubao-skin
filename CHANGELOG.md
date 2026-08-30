# Changelog

All notable changes are recorded here. This project follows Semantic Versioning once the first stable release is published.

## 0.4.0 - 2026-08-31

### Added

- `doubao-skin --version` flag.
- Windows desktop packages built on native Windows runners for x64, x86, and ARM64, each with one `doubao-skin.exe` entry point.
- CLI binary standardized as `doubao-skin` for consistency with the product name.
- CLI-only release assets for macOS universal, Linux x64/ARM64, and Windows x64/x86/ARM64, kept separate from every desktop package.
- Scoop manifest for architecture-aware Windows CLI installation and a platform-detecting macOS/Linux installer.
- Browser-local desktop download recommendation with manual architecture selection.

### Changed

- macOS universal CLI archives are signed after `lipo` with the same stable community identity as the desktop app.

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

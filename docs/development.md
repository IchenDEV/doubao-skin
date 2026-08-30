# Development

## Prerequisites

- macOS 13 or later
- Stable Rust 1.97.1 or later
- Node.js 24.19.0+ and pnpm 12.0.0
- `zip`, `codesign`, `ditto`, and Xcode command-line tools for packaging
- Xcode 26 or later with Icon Composer to regenerate the adaptive app icon; packaging falls back to the checked-in compiled icon resources when full Xcode is unavailable

## Change workflow

Read [development-workflow.md](development-workflow.md) before changing product code. The shortest path is:

```bash
./scripts/devflow new concise-change-name user medium
./scripts/devflow validate
./scripts/check.sh all
```

The artifact chain preserves the product intent, accepted design, implementation plan, verification evidence, and incident feedback loop. `AGENTS.md` contains the compact repository contract read by coding agents; `REVIEW.md` contains the independent review passes.

## Rust workspace

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p doubao-skin-desktop
cargo run -p skin-core --bin doubao-skin -- --help
```

The desktop app loads `themes/` from the repository by default. Override it during development with `DOUBAO_SKIN_THEMES_DIR=/absolute/path/to/themes`.

## Theme gallery

```bash
corepack pnpm --dir apps/web install --frozen-lockfile
corepack pnpm --dir apps/web dev
corepack pnpm --dir apps/web check
```

When a theme or its metadata changes, regenerate the website database, previews, catalog, and installable packages:

```bash
corepack pnpm --dir apps/web sync
```

Commit generated changes under `apps/web/data` and `apps/web/public/themes` with their source theme change.

## Local macOS package

```bash
./scripts/package.sh desktop-macos
./scripts/package.sh desktop-macos --universal
```

The script builds with the lockfile, creates an app bundle, copies themes and license notices, and signs the bundle. It then writes ZIP and DMG packages plus an independent SHA-256 checksum for each under `dist/`. The DMG contains the same signed app and an `Applications` symlink.

CLI packaging is a separate build path and never writes into the desktop bundle:

```bash
./scripts/package.sh cli --universal-macos
./scripts/package.sh cli x86_64-pc-windows-msvc
./scripts/package.sh cli --host
```

The first command creates the macOS universal CLI tarball, the second creates a Windows CLI-only ZIP, and `--host` creates a native macOS or Linux tarball.

Windows desktop and CLI assets are built by GitHub Actions on `windows-2025` runners. The Windows package commands fail on non-Windows hosts, so a macOS cross-build can never be mistaken for Windows CI evidence.

The packaged CLI keeps portable authoring and package-management commands on every platform. Live `apply`/`restore` is limited to macOS and Windows, where the official clients exist; offline `build`/`remove-build` is isolated to macOS and fails before resolving paths or invoking tools elsewhere.

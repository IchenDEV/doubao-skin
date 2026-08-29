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
cargo run -p skin-core --bin doubao-theme -- --help
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
./scripts/build-macos.sh
./scripts/build-macos.sh --universal
```

The script builds with the lockfile, creates an app bundle, copies themes, the `doubao-theme` CLI, `create-doubao-theme` and `apply-doubao-theme` Skills, and license notices, and signs the bundle. It then writes ZIP and DMG packages plus an independent SHA-256 checksum for each under `dist/`. The DMG contains the same signed app and an `Applications` symlink.

The script also produces a standalone CLI tarball (`doubao-theme-macOS-{arch}.tar.gz`) containing just the `doubao-theme` binary and license, with its own SHA-256 checksum. This allows users to install only the CLI without the desktop app.

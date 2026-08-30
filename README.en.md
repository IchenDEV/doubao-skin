<div align="center">

[简体中文](README.md) · [English](README.en.md)

# Doubao Skin

**A native theme manager for the macOS and Windows versions of Doubao and Doubao Work.**

[Theme gallery](https://doubao-skin.idevlab.dev) · [Guide & downloads](https://doubao-skin.idevlab.dev/guide#download) · [Create & contribute](https://doubao-skin.idevlab.dev/contribute)

[![CI](https://github.com/IchenDEV/doubao-skin/actions/workflows/ci.yml/badge.svg)](https://github.com/IchenDEV/doubao-skin/actions/workflows/ci.yml)
[![Website](https://img.shields.io/badge/website-doubao--skin.idevlab.dev-5b7ee5)](https://doubao-skin.idevlab.dev)
[![License: MIT](https://img.shields.io/badge/license-MIT-2f81f7)](LICENSE)

</div>

![Doubao Skin desktop app](docs/images/app.png)

![Doubao Skin online gallery](docs/images/gallery.png)

> macOS and Windows are supported. This is an independent project and does not modify the official Doubao or Doubao Work apps.

## Features

- Native macOS and Windows app for browsing, previewing, installing, applying, and restoring themes.
- 30 built-in themes across solid colors, atmospheric backgrounds, editor palettes, and brand-inspired styles.
- Online theme store with verifiable `.doubao-skin.zip` packages.
- Universal macOS ZIP/DMG plus separate Windows x64, x86, and ARM64 ZIPs.
- One Rust toolchain shared by the desktop app, CLI, Codex plugin, and Claude Code plugin.
- Live injection and offline-clone modes while the official app bundle remains untouched.
- Responsive website with compound filters, dark mode, theme details, guides, and contribution documentation.

## Download and use

Download the latest build from [GitHub Releases](https://github.com/IchenDEV/doubao-skin/releases/latest):

- `Doubao-Skin-macOS-universal.dmg`: recommended; open it and drag the app to Applications.
- `Doubao-Skin-macOS-universal.zip`: unzip and run directly.
- `Doubao-Skin-Windows-x64.zip`: for most Windows PCs.
- `Doubao-Skin-Windows-arm64.zip`: for Snapdragon and other ARM Windows PCs.
- `Doubao-Skin-Windows-x86.zip`: for 32-bit Windows only.
- `.sha256`: SHA-256 checksums for each package.

If macOS blocks the first launch, go to **System Settings → Privacy & Security**, scroll down to “Security”, click **Open Anyway**, and enter your admin password.

Release packages use one continuous community self-signed certificate. They are not Apple-notarized; future versions retain this signing identity unless an announced security rotation is required.

1. Open Doubao Skin.
2. Choose Doubao or Doubao Work.
3. Pick a theme and review the preview.
4. Select **Apply Theme**. Use **Restore Default** to undo it.

The in-app store installs remote themes directly. You can also drag a local package into the window or import one with **Install Theme…**. Installed themes use the platform data directory: `~/Library/Application Support/Doubao Skin/themes/` on macOS, `%LOCALAPPDATA%\Doubao Skin\themes\` on Windows, and `$XDG_DATA_HOME/Doubao Skin/themes/` on Linux (falling back to `~/.local/share`).

## Themes

Browse and filter every theme at [doubao-skin.idevlab.dev](https://doubao-skin.idevlab.dev). The current set contains 30 themes:

- **Solid and editor palettes:** Violet Night, Ocean Cyan, Forest, Pure Dark, Peach Sunset, Huaxia Blue, plus Catppuccin, Dracula, Nord, Gruvbox, Solarized, and One Half adaptations.
- **Atmospheric backgrounds:** Gothic Void, Sakura Night, Cyber Neon, Mist Forest, Cozy Room, Neon Koi, Moon Pine, Crimson Rain, and Machine Overseer.
- **Bright and brand-inspired:** QQ Light Blue, Whale Maid, Tea Party, Flower Club, Starry Room, Snack Giggle, Dessert Giggle, GitHub Repository, and Claude Warm.

Asset provenance and licenses are recorded in each `theme.json`, the [theme research notes](design/theme-standard/codex-theme-research.md), and [third-party notices](THIRD_PARTY_NOTICES.md).

## Agent plugins

The repository is a Marketplace for both Codex and Claude Code. Installing it provides `$create-doubao-theme` and `$apply-doubao-theme`.

Codex:

```bash
codex plugin marketplace add IchenDEV/doubao-skin
codex plugin add doubao-skin@doubao-skin
```

Claude Code:

```text
/plugin marketplace add IchenDEV/doubao-skin
/plugin install doubao-skin@doubao-skin
```

- `$create-doubao-theme`: create, validate, preview, and package a theme from natural language.
- `$apply-doubao-theme`: list, install, apply, restore, and manage themes.

Plugin sources are under [`plugins/doubao-skin`](plugins/doubao-skin). The website also publishes an [Agent Skills Discovery Draft 0.2.0 index](https://doubao-skin.idevlab.dev/.well-known/agent-skills/index.json).

## Developer CLI

`doubao-skin` supports theme development and plugin automation. It uses separate Release assets and is never bundled into the desktop package.

### Install

On Windows, Scoop selects x64, x86, or ARM64 automatically:

```powershell
scoop install https://github.com/IchenDEV/doubao-skin/releases/latest/download/doubao-skin.json
```

On macOS or Linux, the installer detects the current platform and verifies the checksum:

```bash
curl -fsSL https://raw.githubusercontent.com/IchenDEV/doubao-skin/main/scripts/install-cli.sh | sh
```

### Usage

```bash
doubao-skin list
doubao-skin create themes/my-theme \
  --name "My Theme" --description "A calm dark theme" \
  --accent "#5b7ee5" --appearance both --author "Local user"
doubao-skin check themes/my-theme
doubao-skin preview themes/my-theme
doubao-skin pack themes/my-theme dist/my-theme.doubao-skin.zip
```

`list`, `create`, `check`, `preview`, `pack`, and `install` are portable. `apply` and `restore` require the official macOS or Windows client; the offline-clone commands `build` and `remove-build` are macOS-only.

Run `doubao-skin --help` for the complete interface.

## Build from source

Requirements: Rust 1.97.1+, Node.js 24.19+, and pnpm 12. Desktop packages must be built on their target operating system.

```bash
# Test and validate the whole project
./scripts/check.sh all

# Run the desktop app
cargo run -p doubao-skin-desktop

# Build for the host architecture
./scripts/package.sh desktop-macos

# Build a universal Apple Silicon + Intel package
./scripts/package.sh desktop-macos --universal

# Build a Windows x64 package from Windows Git Bash
./scripts/package.sh desktop-windows x86_64-pc-windows-msvc
```

Website:

```bash
corepack pnpm --dir apps/web install --frozen-lockfile
corepack pnpm --dir apps/web sync
corepack pnpm --dir apps/web dev
```

`sync` generates the SQLite catalog, normalized previews, store JSON, and installable theme packages from `themes/`. Do not edit `apps/web/data` or `apps/web/public/themes` by hand.

## Repository layout

```text
apps/desktop        Native GPUI desktop app
apps/web            Next.js theme gallery
crates/skin-core    Theme engine, CLI, live/offline builds, and protocol bridge
themes              Built-in themes and assets
plugins             Codex and Claude Code plugins
design              Theme schema and design rules
docs                Architecture, development, contribution, release, and deployment docs
workflow            Artifact-driven delivery and verification records
```

## Create and contribute themes

Use `doubao-skin create` or `$create-doubao-theme` to generate a `schemaVersion: 2` theme. See the [theme standard](design/theme-standard/README.md) and [JSON schema](design/theme-standard/theme-v2.schema.json).

Before opening a pull request:

```bash
cargo run -p skin-core --bin doubao-skin -- check themes/my-theme
corepack pnpm --dir apps/web sync
./scripts/check.sh all
```

Follow [Submitting a theme](docs/submitting-themes.md) and the [contribution guide](CONTRIBUTING.md). The website does not accept direct uploads; themes are submitted through GitHub pull requests.

## Website deployment

`apps/web` runs on Vercel at [doubao-skin.idevlab.dev](https://doubao-skin.idevlab.dev). Once the GitHub repository is connected to the existing Vercel project, pushes to `main` deploy to Production while other branches and pull requests receive Preview deployments. See [Website deployment](docs/website-deployment.md) for configuration and acceptance checks.

## Documentation

- [Documentation index](docs/README.md)
- [Architecture](docs/architecture.md)
- [Local development](docs/development.md)
- [Development workflow](docs/development-workflow.md)
- [macOS releases](docs/releasing.md)
- [Security policy](SECURITY.md)

## License

The Rust core, website, project documentation, and original theme definitions are available under the [MIT License](LICENSE). Third-party assets and dependencies remain subject to [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and the matching files under `LICENSES/`.

---
id: "2026-08-30-windows-build-and-cli-rename"
stage: verification
status: pending
owner: "Codex implementation agent"
created: "2026-08-30"
based_on: plan.md
commit: ""
risk: "critical"
verification_mode: "human"
verified_by: ""
verified_at: ""
---

# Verification: Windows native build and CLI rename

## Automated checks

- `./scripts/check.sh all` passed from the repository root.
  - Workflow validation and approval-policy tests passed for 13 artifact sets.
  - Rust formatting, workspace tests, and Clippy with warnings denied passed.
  - Tests passed: 11 desktop, 33 core library, 7 authoring, 2 bundled-theme
    path, and 4 CLI integration tests.
  - Twelve Web tests, TypeScript, the 38-page Next.js production build, and the
    high-severity dependency audit passed; the audit found no known
    vulnerabilities.
- Earlier diagnostic packages were built for `x86_64-pc-windows-msvc`,
  `i686-pc-windows-msvc`, and `aarch64-pc-windows-msvc` on macOS. They remain
  useful for the real-Windows visual regressions below, but are not accepted as
  native Windows CI evidence.
- All three simplified Windows archive checks passed with `shasum -a 256 -c`:
  - `Doubao-Skin-Windows-x64.zip`:
    `e68f4e70d911a1bb9c1430f305d98363fa2a253362a64d1b58467852815913d9`
  - `Doubao-Skin-Windows-x86.zip`:
    `af2c3bda12d5fa08fde6df68592c0a01d212d72dd2a6cefcb75bfa4a2453451d`
  - `Doubao-Skin-Windows-arm64.zip`:
    `7810a0bff584bf4b83298c092284c3a4b21931a1cb3ee65ac44cc3d3f826ee85`
- `file` identified the desktop executables as Windows GUI PE files for
  x86-64, Intel 80386, and AArch64. `xcrun llvm-objdump -p` confirmed the
  Windows GUI subsystem plus `DYNAMIC_BASE` and `NX_COMPAT` on all three;
  64-bit targets also retain `HIGH_ENTROPY_VA`.
- The earlier Windows EXE verifier passed for all three diagnostic desktop
  executables.
  `llvm-readobj --coff-resources` confirms both `ICON` (type 3) and
  `GROUP_ICON` (type 14), and packaging now fails before archive creation if
  either resource or the GUI subsystem is absent.
- The pre-change archive regression found two top-level executables in every
  complete Windows package plus three standalone CLI archives. After the fix,
  archive listing checks find exactly one top-level `doubao-skin.exe`, five
  bundled themes, and license notices in every package; no embedded CLI binary
  or packaged Skill directory remains.
- `unzip -tq` and checksum verification passed for all three rebuilt archives.
- `./scripts/package.sh desktop-macos` produced a host arm64 ZIP and DMG without
  `Contents/Resources/bin` or `Contents/Resources/skills`. Both checksum checks
  passed; the ZIP hash is
  `ea949ea021d24fafaced9f508a268295801252c5ea9c1cd3a614b84bef97c5da`
  and the DMG hash is
  `2e8025ddc7caf36f199200ef4d3a22e4ce69540209c151fcc59f0130593c1306`.
- `git diff --check`, workflow YAML parsing, shell syntax checks, and the
  generated Skill discovery check passed. Product-code and documentation
  searches found no remaining `doubao-theme` CLI reference; matches are limited
  to stable DOM attributes and Skill identifiers.
- Before restoring the independent CLI chain, the regression preflight failed
  because `scripts/install-cli.sh` and the Windows CLI archive were absent and
  `/guide` still rendered `Windows · Coming Soon`. This recorded the clarified
  requirement as a red baseline before implementation.
- The earlier CLI packaging pass produced and checksum-verified four host-buildable
  CLI-only artifacts:
  - `doubao-skin-cli-macOS-universal.tar.gz`:
    `cb232ce4f7c7faa91560fcae9e79259198dee2b82d6748142dd6ba55da820c33`
  - `doubao-skin-cli-Windows-x64.zip`:
    `378a720be16519887134f58d72ef142fe7d9ed3ad6ad9133fb6e8f686fcf86d6`
  - `doubao-skin-cli-Windows-x86.zip`:
    `34f877b484462f6943ef386560dd545d3e484283010585adc33a8102dfc90ea9`
  - `doubao-skin-cli-Windows-arm64.zip`:
    `b4c37e7b030a902dd1eb0df27a13692f2a4499f940dfb3b3fab5ad4b523f0760`
- Each Windows CLI ZIP contains exactly `doubao-skin.exe` and `LICENSE`; the
  macOS CLI tarball contains exactly `doubao-skin` and `LICENSE`. Conversely,
  all three Desktop ZIPs contain one GUI executable and no `bin/` or `skills/`
  entries, and the macOS app contains neither `Contents/Resources/bin` nor
  `Contents/Resources/skills`.
- The Scoop generator produced a valid
  `doubao-skin.json` whose x64, x86, and ARM64 URLs and hashes match those
  CLI-only archives.
- The Web platform tests passed for macOS/iPad distinction, Windows x64/x86/
  ARM64 selection, and unsupported/mobile fallback. The production Web check
  passed 12 tests, generated Skill discovery validation, TypeScript, and the
  38-route Next.js build.
- The ARM64 live-apply regression first reproduced an endless “正在应用” state.
  `Test-NetConnection localhost -Port 9223` succeeded and `/json` returned the
  three expected `doubao://` pages, while `Test-Path /dev/urandom` returned
  false. The new tests cover host-platform WebSocket randomness and the 30
  second initial-injection error boundary; `cargo test -p skin-core --lib`
  passed all 33 tests, and `./scripts/check.sh rust` passed the full Rust gate.
- The repaired ARM64 archive was transferred into the Windows 11 ARM VM and
  independently hashed inside the guest. Its SHA-256 exactly matched
  `e0bfc867d65744dc970aac2af872b82a3a5fbb4e928952412ee63626189747d1`.
- The Windows launcher regression creates both
  `Application/Doubao.exe` and `Application/app/Doubao.exe` and confirms that
  discovery selects the official outer launcher. A second regression confirms
  that Windows starts the resolved executable with its parent directory as the
  working directory. `cargo test -p skin-core --lib windows_` passed all four
  focused tests.
- The repository-wide platform audit first added four regression tests that
  failed to compile because the portable directory and capability-boundary
  helpers did not exist. After implementation, all 33 core tests pass. The
  user theme and cache roots now use the operating system's standard data and
  cache APIs instead of `HOME` plus macOS-only suffixes; tests cover both
  POSIX and Windows-style bases.
- Live mode now rejects platforms other than macOS and Windows before resolving
  target paths or launching commands. Offline cloning similarly rejects every
  non-macOS platform before accessing `/Applications`, the home directory, or
  destructive filesystem operations; an unavailable home directory can no
  longer fall back to a relative deletion target.
- `cargo check -p skin-core --example read_skin` passed after replacing its
  runtime `curl` process with the existing native HTTP client and a bounded
  timeout. A follow-up product-source scan found no remaining hard-coded
  `HOME`, `/dev/urandom`, or external `curl` invocation.
- macOS application-menu shortcuts and native menu construction are now
  registered only in macOS builds. Earlier x64, x86, and ARM64 diagnostic
  builds passed without the cross-platform dead-code warnings exposed by the
  first implementation.
- `scripts/checks/portability.sh` was first run against the unfixed source and
  rejected direct Windows data-directory environment access, relative
  persistence fallback, hard-coded `/Applications` paths, and system commands
  outside adapters. After the refactor it passes across all Rust source under
  `crates/` and `apps/`, and is part of `./scripts/check.sh workflow`.
- `./scripts/check.sh workflow`, `./scripts/check.sh rust`, and
  `./scripts/check.sh web` pass after the script move and platform-adapter
  refactor. This includes 34 core tests, 11 desktop tests, 13 integration tests,
  Clippy with warnings denied, 12 Web tests, the 38-route production build, and
  the high-severity dependency audit.
- `./scripts/package.sh cli --host` produced and ran the macOS ARM64 CLI-only
  package. `./scripts/package.sh desktop-macos` produced and verified the host
  ZIP and DMG. Invoking `desktop-windows` on macOS now fails before Cargo with
  the required Windows-host error.
- Workflow YAML parsing, shell/Node syntax checks, `git diff --check`, and the
  case-insensitive path collision scan pass with the organized script tree.

## Behavioral evidence

- Desktop sources are grouped under `app/`, `ui/`, `preview/`, and `store/`.
  The source root now keeps only the entry point, internationalization module,
  regression tests, and those four module directories.
- The developer CLI remains `doubao-skin` and is again a separately packaged
  Release asset. It is never embedded in the desktop packages. The desktop
  Cargo binary remains internally named `doubao-skin-app` to avoid a build-
  output collision, while Windows desktop packages expose their sole GUI entry
  as `doubao-skin.exe`.
- The bundled-theme resolver now checks a sibling `themes/` directory before
  the macOS bundle layouts, with regression coverage for both flat Windows
  executables.
- Release publication now gates independent Desktop and CLI matrices. Desktop
  uploads remain CLI-free; CLI uploads remain theme/GUI-free. Windows Desktop
  CI also asserts one top-level EXE and its exact `doubao-skin.exe` name.
- The CLI matrix covers macOS universal, native Linux x64/ARM64, and Windows
  x64/x86/ARM64. A downstream job generates the Scoop manifest only after all
  CLI architectures pass; publication requires Desktop, CLI, and Scoop jobs.
- `scripts/install-cli.sh` detects macOS or Linux architecture, downloads the
  CLI-only archive and sidecar checksum, verifies it, and installs only the
  `doubao-skin` executable. Windows instructions use the generated Scoop
  manifest and do not invoke the desktop installer.
- `/guide` recommends a Desktop download from browser-local platform evidence,
  never auto-downloads, and keeps all macOS/Windows architecture alternatives
  visible. Its CLI section is separate and presents Scoop for Windows plus the
  CLI-only installer for macOS/Linux.
- Windows uses a native, non-transparent titlebar so GPUI leaves the system
  close/minimize controls visible. The title value is empty to avoid repeating
  the product name already shown in the content header; macOS retains its
  transparent custom header.
- Theme thumbnails, preview backgrounds, and raster icons are now passed to
  GPUI as filesystem resources. The regression reproduces a `C:\...` path and
  confirms it is not classified as a URI.
- Installed-app discovery now covers the default per-user layouts for both
  products, read-only Windows Apps/ARP registry entries, Program Files, and
  explicit `DOUBAO_SKIN_DOUBAO_PATH` / `DOUBAO_SKIN_DOUBAO_WORK_PATH`
  overrides. Windows launch/quit uses the resolved executable and native
  process commands instead of macOS `open`, AppleScript, and `pkill`.
- Cross-platform theme authoring, package installation, cache storage, and CLI
  examples no longer encode a macOS filesystem or shell-command assumption.
  The remaining operating-system commands are confined to explicit adapters:
  target-app discovery/start/stop for live mode, macOS offline cloning and
  signing, and platform build/package scripts.
- When an installation contains both the official outer launcher and the
  internal `Application/app/` executable, Windows now selects the outer
  launcher first and starts it from its own directory. This avoids treating an
  implementation-detail binary as the installed application's public entry.
- WebSocket handshake nonces and client-frame masks now come from the native
  operating-system random provider on every supported platform. Before the
  first successful injection, repeated CDP failures retain their concrete
  cause and return after 30 seconds so the desktop UI can leave its applying
  state and report failure.
- Release publication depends on successful tag validation, every Desktop and
  CLI matrix build, and Scoop manifest generation. A failed platform build
  cannot publish a partial release.

## Visual evidence

- The user's first real-Windows screenshots established four failures in the
  original packages: missing EXE icon, missing native window controls, blank
  bundled-theme images, and both target applications reported as uninstalled.
  The second real-Windows screenshot confirms the rebuilt x64 package displays
  the icon and window controls, loads bundled-theme images, and recognizes both
  target applications. It also revealed the redundant “豆皮” native title; the
  latest ARM64 VM pass confirms that the native caption is now title-free while
  retaining the system icon and window controls.
- The ARM64 VM reproduced the live-apply failure before the fix. The corrected
  ARM64 archive was then downloaded and launched in the same guest. Applying
  the visually distinctive built-in `甜点偷笑` theme changed the actual Doubao
  conversation window to the expected illustrated background and themed UI;
  this was observed in the target window rather than only in the desktop
  preview. The desktop state also changed to `已应用` / `正在使用`.
- A direct CDP diagnostic inspected the visible launcher page, background page,
  and chat page after applying `纯暗`. Every page contained the expected
  `data-doubao-skin="pure-dark"` marker plus the injected style and adopted
  stylesheet. The launcher remained light because the theme declares
  `appearance: "both"` and the official client reported `theme: "light"`;
  this appearance is therefore adaptive-theme behavior, not missing Windows
  injection.
- In the in-app Browser at 1440×1000 and 390×844, `/guide` rendered the macOS
  Desktop recommendation, all four manual Desktop alternatives, and a separate
  two-card CLI section without overflow. Clicking the CLI copy control changed
  its label to `已复制`. DOM inspection found one copy of each guide section and
  the browser console contained no errors or warnings.

## Security and privacy evidence

- The full vendored `patches/gpui_windows` crate, temporary two-file GPUI
  shader correction, cargo-xwin preparation script, and GPUI-specific release
  profile override were removed. Windows packaging has one implementation and
  refuses to run on a non-Windows host.
- The i686-only `/SAFESEH:NO` exception is required by psm's 32-bit assembly
  object; PE inspection confirms DEP (`NX_COMPAT`) and ASLR (`DYNAMIC_BASE`)
  remain enabled.
- No credential, official app resource, conversation content, cookie, header,
  workspace data, attachment, tool payload, or unknown protocol block was
  added to the packages or repository.
- Windows registry access is read-only and limited to installed-app discovery.
  Process termination remains scoped to the executable name resolved for the
  selected target application.

## Deviations and residual risk

- Earlier cargo-xwin diagnostics exposed GPUI shader/resource issues and an
  i686 SafeSEH incompatibility. The cross-build workaround was deleted instead
  of becoming a second supported path. The native Windows matrix must now show
  whether any target-specific flag is still required.
- Initial package names differed only by capitalization and collided on the
  macOS filesystem. An intermediate interpretation removed CLI distribution;
  the user clarified that only Desktop/CLI co-packaging was unwanted. CLI-only
  assets and installers are therefore restored under the unambiguous
  `doubao-skin-cli-*` prefix while all Desktop packages stay CLI-free.
- Real-Windows x64 screenshots now verify
  the repaired EXE icon, window controls, bundled images, installed-app
  discovery, native title controls, title-free caption, and DirectX rendering.
  The ARM64 VM now also verifies real target-window theme application. x86
  native compilation remains pending remote CI.
- The current official ARM64 Doubao installer was downloaded from the official
  site and used for an overwrite repair, but starting the official client with
  a CDP port still displays its own `安装文件缺失` dialog naming
  `mcp_helper.dll`. A filesystem check confirms that DLL is absent after the
  official reinstall. The target window and theme injection work after the
  dialog is dismissed. This repository must not redistribute or synthesize the
  proprietary DLL, so a completely clean startup remains blocked by the
  official ARM64 installation rather than by the theme package.
- These packages are unsigned ZIPs, not MSI/MSIX installers. Release
  publication was not run; native GitHub Windows CI is the remaining automated
  package gate.
- Linux x64 and ARM64 CLI jobs are configured on native GitHub-hosted runners,
  but their archives were not built on this macOS host. Their final package
  evidence therefore remains the remote release matrix rather than this local
  verification run.
- A local Linux target check was attempted twice. The macOS host has the Rust
  target installed but lacks both an `x86_64-linux-gnu-gcc` toolchain and a
  Linux C sysroot (`assert.h`), so `ring` could not be compiled here. This is a
  host-toolchain limitation, not Linux execution evidence; native Linux CI is
  still required before release.
- The ARM64 VM still visibly shows the previously verified build applying the
  illustrated theme to the real target window. The final audit rebuild has new
  hashes and passed cross-build/resource/package checks, but the available
  Computer Use channel could read the Fusion display without delivering clicks
  into the guest, and `vmrun` correctly refused guest operations without a
  password. No credential was requested or stored, so this newest hash was not
  relaunched in the guest during the audit pass.

## Verdict

The four reported compatibility defects passed a real-Windows x64 visual pass
and remain protected by regression/package checks. The live-apply hang is
traced to a Windows-incompatible WebSocket random source, repaired with native
OS randomness, protected by regression tests, and rebuilt into all three
single-entry Windows Desktop archives. The ARM64 VM now provides an additional
real target-window pass for theme application and button state. Final human
verdict remains pending only because the current official ARM64 Doubao install
still reports its own missing `mcp_helper.dll` when launched with CDP; x86,
Linux CLI, and remote publication also remain unexecuted on their native or CI
environments.

The additional repository-wide compatibility audit removed generic-code
dependencies on macOS user paths, `/dev/urandom`, external `curl`, and macOS
menus. Unavoidable platform behavior is now isolated behind capability checks
or target-specific adapters instead of leaking into portable theme and CLI
operations. Packaging is exposed through one dispatcher with grouped internal
scripts, and the obsolete macOS Windows-cross/GPUI-patch path is gone. The
remaining acceptance gate is the remote native Windows CI build followed by
one final launch smoke check of the resulting Windows artifact.

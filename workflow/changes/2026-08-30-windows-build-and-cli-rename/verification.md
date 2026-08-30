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
  - Workflow validation and approval-policy tests passed for 14 artifact sets.
  - Rust formatting, workspace tests, and Clippy with warnings denied passed.
  - Tests passed: 11 desktop, 35 core library, 7 authoring, 2 bundled-theme
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
  helpers did not exist. After implementation, all 35 core tests pass. The
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
  refactor. This includes 35 core tests, 11 desktop tests, 13 integration tests,
  Clippy with warnings denied, 12 Web tests, the 38-route production build, and
  the high-severity dependency audit.
- `./scripts/package.sh cli --host` produced and ran the macOS ARM64 CLI-only
  package. `./scripts/package.sh desktop-macos` produced and verified the host
  ZIP and DMG. Invoking `desktop-windows` on macOS now fails before Cargo with
  the required Windows-host error.
- Workflow YAML parsing, shell/Node syntax checks, `git diff --check`, and the
  case-insensitive path collision scan pass with the organized script tree.
- Native CI run `33321611462` provided two red baselines unavailable on macOS:
  x64 ran a macOS LaunchServices path test under Windows path semantics, while
  x86 and ARM64 completed GPUI release compilation but could not find
  `llvm-readobj`. The test is now macOS-compiled only; Windows packaging installs
  `llvm-tools-preview` and resolves both `llvm-readobj` and
  `llvm-readobj.exe`. The same run also exposed and prompted removal of two
  macOS-only dead-code warnings from Windows release builds.
- Native CI run `33322177480` passed the repaired implementation on
  `windows-2025` for x64, x86, and ARM64. All three jobs built both the desktop
  and independent CLI archives; the desktop verifier passed `ICON`,
  `GROUP_ICON`, and Windows GUI subsystem checks on both the Cargo output and
  packaged `doubao-skin.exe`. The x64 job additionally passed all 32
  Windows-applicable core tests.
- The same run uploaded `Windows-native-x64` (10,752,679 bytes),
  `Windows-native-x86` (10,122,277 bytes), and `Windows-native-arm64`
  (10,271,142 bytes). Each artifact contains its desktop ZIP, CLI-only ZIP,
  and both sidecar SHA-256 files. Rust workspace and Web jobs also passed.
- The development-workflow job remains red by design because this file is
  truthfully `pending`; `scripts/devflow check-pr` requires the final human or
  fresh-context verdict before the PR gate can pass.

### Independent fresh-context audit on 2026-08-31

- The audit compared `origin/main..HEAD` at
  `a4bba0ae6305bac74059688d24ac59c33568ada8` and separately reviewed the
  uncommitted `scripts/checks/portability.sh` change. `git diff --check
  origin/main..HEAD` and `git diff --check` both passed.
- During the audit, the implementation worktree added the mixed-bitness ARP
  fix and its Windows-only dependency, updated the lockfile, and corrected the
  CHANGELOG. Those follow-up diffs were independently reread before the final
  local gate below; they are not part of the audited HEAD or its CI artifacts.
- `./scripts/check.sh workflow` passed: 14 artifact sets validated, workflow
  policy cases passed, and the portability boundary check passed. With
  `PATH=/usr/bin:/bin:/usr/sbin:/sbin`, `rg` was unavailable and
  `/bin/bash scripts/checks/portability.sh` still passed, directly exercising
  the working-tree scanner fallback.
- After the follow-up fixes below, `./scripts/check.sh all` passed on the final
  worktree, including 11 desktop tests, 36 core tests, 7 authoring tests, 2
  bundled-theme path tests, 4 CLI integration tests, formatting, Clippy with
  warnings denied, 12 Web tests, generated-Skill consistency, TypeScript, the
  38-route production build, and the high-severity dependency audit.
- CI run
  [`33323010961`](https://github.com/IchenDEV/doubao-skin/actions/runs/33323010961)
  is for the exact audited HEAD. Rust workspace, Web application, and native
  Windows x64, x86, and ARM64 jobs all succeeded. Its sole failed job is
  Development workflow: artifact validation passed and `check-pr` rejected
  this verification's truthful `pending` status.
- The run's three unexpired artifacts were downloaded and independently
  checked. GitHub reports `Windows-native-x64` at 10,752,890 bytes,
  `Windows-native-x86` at 10,122,236 bytes, and `Windows-native-arm64` at
  10,271,196 bytes. All six sidecar checksums passed. The Desktop ZIP hashes
  are `e5d2ed8297bb97727e74e260eed0a39ee46c8bcae3b1896c2cc2db1a09227752`,
  `947a814a765da9e2cc2f9ac4e673086b3986a3f38d77337241169059b2257804`,
  and `c55297e47064f60f63037e351ceaae1a813169ccf51db609644419bb1b418db2`;
  the CLI ZIP hashes are
  `2dfd7eab9de9078c884e8fe7a6dd6b6f8c38b6ca692b8bf72375fc7c9ede65f8`,
  `9576bd228af1aa0a614fca2dc478eca58d8e9eb40eb09e668821c290061b3b04`,
  and `7ab2d918c8f2e140b9364897fae058c204a928d9a02faea20a299c85e9bfe84f`
  in x64, x86, ARM64 order.
- Every downloaded Desktop ZIP has exactly one top-level
  `doubao-skin.exe`, no `bin/` or `skills/` entries, and the expected sibling
  theme tree. Every CLI ZIP contains exactly `doubao-skin.exe` and `LICENSE`.
  `file` identified matching x86-64, Intel 80386, and AArch64 GUI/console
  binaries. `scripts/package/verify-windows-exe.sh` independently confirmed
  ICON, GROUP_ICON, and Windows GUI subsystem resources on all three Desktop
  executables.
- Regenerating `doubao-skin.json` from the three downloaded CLI sidecars
  produced matching x64, x86, and ARM64 URLs and hashes. On macOS, both
  `./scripts/package.sh desktop-windows x86_64-pc-windows-msvc` and
  `./scripts/package.sh cli x86_64-pc-windows-msvc` failed before Cargo with
  the required Windows-MSVC-host boundary.
- A fresh `cargo check -p skin-core --lib --target
  x86_64-pc-windows-msvc --locked` attempt could not compile `ring` on this
  macOS host because the host has no Windows C sysroot (`assert.h` was
  missing). Consequently the new Windows-only registry-view code still needs
  the repository-mandated native Windows CI rerun rather than local
  cross-target evidence.
- Native CI run
  [`33326541149`](https://github.com/IchenDEV/doubao-skin/actions/runs/33326541149)
  is for follow-up commit `0201305d427e701ae0aaca63b87ca45cfc28205a`.
  Rust workspace, Web application, and native Windows x64, x86, and ARM64 jobs
  all succeeded. The x64 runner passed all 34 Windows-applicable core tests,
  including both mixed-bitness registry-view regressions, before packaging.
- The same run built and uploaded separate Desktop and CLI archives for every
  Windows architecture. The produced files are
  `Doubao-Skin-Windows-{x64,x86,arm64}.zip` and
  `doubao-skin-cli-Windows-{x64,x86,arm64}.zip`; GitHub reports the enclosing
  artifacts at 10,760,984, 10,159,629, and 10,278,575 bytes respectively.
  Its sole failed job is still Development workflow, where `check-pr`
  correctly rejects this verification's truthful `pending` status.

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
  of becoming a second supported path. Native Windows CI confirms the scoped
  i686 SafeSEH flag is sufficient and ARM64 needs no cross-compiler workaround.
- Initial package names differed only by capitalization and collided on the
  macOS filesystem. An intermediate interpretation removed CLI distribution;
  the user clarified that only Desktop/CLI co-packaging was unwanted. CLI-only
  assets and installers are therefore restored under the unambiguous
  `doubao-skin-cli-*` prefix while all Desktop packages stay CLI-free.
- Real-Windows x64 screenshots now verify
  the repaired EXE icon, window controls, bundled images, installed-app
  discovery, native title controls, title-free caption, and DirectX rendering.
  The ARM64 VM now also verifies real target-window theme application. x86,
  x64, and ARM64 now all have native Windows compilation/package evidence.
- The current official ARM64 Doubao installer was downloaded from the official
  site and used for an overwrite repair, but starting the official client with
  a CDP port still displays its own `安装文件缺失` dialog naming
  `mcp_helper.dll`. A filesystem check confirms that DLL is absent after the
  official reinstall. The target window and theme injection work after the
  dialog is dismissed. This repository must not redistribute or synthesize the
  proprietary DLL, so a completely clean startup remains blocked by the
  official ARM64 installation rather than by the theme package.
- These packages are unsigned ZIPs, not MSI/MSIX installers. Release
  publication was not run. The native Windows package gate passed, while final
  verification status still requires the repository-mandated human or
  fresh-context verdict.
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
- The first independent pass found that mixed-bitness ARP discovery opened
  only the process-default registry view. The follow-up worktree now explicitly
  enumerates Registry64 and Registry32 with the Windows `KEY_WOW64_64KEY` and
  `KEY_WOW64_32KEY` constants, deduplicates discovered paths, and adds a
  seam through which production discovery reads each view. The regression
  injects distinct fake entries, asserts the Registry64/Registry32 visit order,
  and proves paths from both views are collected. A Windows-only test also
  checks the enum-to-`KEY_WOW64_*` mapping. The 36-test core suite passes
  locally; native Windows x64 run `33326541149` passed both Windows-only
  registry assertions and all 34 Windows-applicable core tests.
- The first independent pass also found that `CHANGELOG.md` called the packages
  cross-compiled and retained the old CLI binary name. The follow-up worktree
  now says the packages are built on native Windows runners and describes the
  CLI only as `doubao-skin`. A fresh tracked-source search finds no old CLI
  reference outside the accepted DOM attributes and Skill identifiers.
- The existing record describes real-window x64 and ARM64 checks but does not
  provide inspectable normal-and-narrow final-state evidence for the native UI
  change. The 1440x1000 and 390x844 evidence is for the Web guide, not the
  native window. This remains an evidence gap under the repository's native UI
  definition of done.
- The ARP fix, CHANGELOG correction, and `portability.sh` fallback were not
  present in the earlier audited run `33323010961`. They are committed in the
  follow-up and run `33326541149` now supplies replacement native Windows
  artifacts for x64, x86, and ARM64.

## Verdict

The four reported compatibility defects passed a real-Windows x64 visual pass
and remain protected by regression/package checks. The live-apply hang is
traced to a Windows-incompatible WebSocket random source, repaired with native
OS randomness, protected by regression tests, and rebuilt into all three
single-entry Windows Desktop archives. The ARM64 VM now provides an additional
real target-window pass for theme application and button state. Final human
verdict remains pending only because the current official ARM64 Doubao install
still reports its own missing `mcp_helper.dll` when launched with CDP. Linux
CLI and release publication remain unexecuted on their final release paths.

The additional repository-wide compatibility audit removed generic-code
dependencies on macOS user paths, `/dev/urandom`, external `curl`, and macOS
menus. Unavoidable platform behavior is now isolated behind capability checks
or target-specific adapters instead of leaking into portable theme and CLI
operations. Packaging is exposed through one dispatcher with grouped internal
scripts, and the obsolete macOS Windows-cross/GPUI-patch path is gone. All
three native Windows CI jobs now pass and retain downloadable test artifacts.
The file stays `pending` until a human or fresh-context verifier records the
final verdict; the newest native artifact also has not yet been relaunched in
the guest VM.

### Independent fresh-context verdict on 2026-08-31

`pending`. Native Windows compilation, package naming, PE resources,
Desktop/CLI archive separation, and the mixed-bitness registry regressions are
now proven for x64, x86, and ARM64 at follow-up commit `0201305`. The complete
local gate plus the no-`rg` portability check also pass. A final pass still
requires inspectable normal/narrow native-window validation of the latest
package. The current VMware control channel can read the guest display but
reports `noWindowsAvailable` for click and keypress operations, so that visual
gate cannot be completed in this audit without a working interactive guest
channel.

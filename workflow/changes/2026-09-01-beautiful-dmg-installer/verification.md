---
id: "2026-09-01-beautiful-dmg-installer"
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

# Verification: beautiful dmg installer

## Automated checks

- Baseline structural assertions failed against the previous direct-UDZO implementation because it had no DMG backgrounds, `tiffutil`, UDRW/convert step, Finder AppleScript, or `.DS_Store` validation. The same assertions passed after implementation.
- `bash -n scripts/package/macos.sh`: passed.
- `./scripts/check.sh workflow` and `./scripts/devflow validate`: passed with the accepted intent, spec, and plan chain.
- `./scripts/check.sh all`: passed. This covered workflow policy, Rust formatting/build/tests, the native desktop targets, Web tests, TypeScript, the Next.js production build, skill/catalog consistency, lockfile policy, and dependency audit.
- After moving the work onto current `origin/main` at `453bf34`, both host and universal packages were rebuilt and `./scripts/check.sh all` passed again, so the PR evidence is based on the final branch base rather than the earlier detached checkout.
- `sips -g pixelWidth -g pixelHeight` reports `660 x 400` for `assets/dmg/install-background.png` and `1320 x 800` for `assets/dmg/install-background@2x.png`. `tiffutil -cathidpicheck` combined them as 72 dpi and 144 dpi representations of the same layout.
- `shasum -a 256 -c` passed for the arm64 and universal ZIP/DMG pairs. `hdiutil verify` passed for both final DMGs.
- `git diff --check` passed. The final cleanup check found no `*.rw.dmg`, `*.udzo.dmg`, `*.tmp`, `.dmg-staging.*`, `豆皮-DMG-*` volume, or attached image belonging to this worktree.

## Behavioral evidence

- Two controlled host-package runs replaced only `osascript` through a temporary `PATH` and forced exit status 42 at the Finder-layout step. Both package commands failed closed and left no final DMG/checksum, temporary UDRW/UDZO image, staging directory, or mounted build volume. The already-produced ZIP remained intact by design.
- A real host build produced `Doubao-Skin-macOS-arm64.{zip,dmg}`. A real universal build produced `Doubao-Skin-macOS-universal.{zip,dmg}`. Both read-only DMGs contain exactly the visible `豆皮.app` and `/Applications` symlink plus hidden `.DS_Store` and `.background/install-background.tiff` resources.
- `codesign --verify --deep --strict` passed for the app in each DMG. `CFBundleShortVersionString` remained `0.5.0`; the host main app and agent are `arm64`, and the universal main app and agent are `x86_64 arm64`.
- `diff -qr` found no difference between the app copied from each ZIP and the app in its corresponding DMG. Copying the mounted app with `ditto` into an isolated temporary `Applications` directory preserved the bundle and passed strict signing verification; the real `/Applications` directory was not written.

## Visual evidence

- The candidate layout was first tested with a disposable placeholder bundle, then both final host and universal DMGs were opened as real Finder windows on macOS 27.0 with a Retina display. The final host and universal windows restored the same accepted layout.
- The Finder window is 660 x 428 with a 660 x 400 warm-white and soft-gold background, a restrained title, 120 pt app and Applications icons at the left and right, and a centered gold arrow describing the drag direction. Toolbar, sidebar, path bar, status bar, hidden background resources, and scrollbars are absent.
- The app name and Applications label remain readable and are not baked into the background. The 1x source was inspected at native size and the 2x representation was inspected through the final Retina Finder window.
- Product-only evidence: [dmg-installer-host.jpeg](evidence/dmg-installer-host.jpeg). It contains only the installer Finder window and no user files, debugging UI, logs, or other application windows.

## Security and privacy evidence

- Packaging copied the already-built app bundle; it did not modify `/Applications/DoubaoWork.app`, the real `/Applications` directory, application code, themes, user content, or network state.
- The background uses project-created geometry, typography, and color only. No third-party or official application artwork was added.
- No Apple credentials, cookies, production signing identity, notarization profile, or remote release capability was used. The local builds retained the repository's default ad-hoc signing behavior.
- Cleanup targets are limited to the exact build image/device and task-specific files beneath `dist`; the unrelated pre-existing `/Users/chenli/Downloads/Doubao-Skin-macOS-universal.dmg` mount was left attached and untouched.

## Deviations and residual risk

- The accepted plan proposed attaching the image at an arbitrary temporary POSIX mountpoint. In the Finder prototype, a window addressed that way wrote `.DS_Store` but did not restore its layout reliably. The implementation instead uses a unique temporary volume name under `/Volumes`, addresses the disk by name in Finder, then renames the volume to the final `豆皮` label before detaching. This also avoids colliding with the user's already-mounted older `豆皮` DMG while preserving the required final label.
- Current macOS 27 emits a deprecation warning for `hdiutil` and recommends the newer `diskutil image` family. The existing repository and release flow use `hdiutil`; migrating that infrastructure is outside this visual packaging change.
- Visual validation covered the native 1x asset and a real Retina Finder window on macOS 27.0. No separate non-Retina display or macOS 12 hardware was available, so older supported-system rendering still depends on the system TIFF contract and future CI/release validation.
- Finder automation passed in the local interactive session. A hosted release runner without a usable Finder session could still fail; the script intentionally fails closed instead of silently shipping an unstyled DMG.
- These local artifacts are ad-hoc signed and were not notarized or stapled. This work does not claim Gatekeeper trust, production release readiness, upload, tag creation, or publication.

## Verdict

Implementation evidence is complete and the applicable local gates pass. Final verdict remains pending for a fresh-context verifier or the human approver; this author has not self-approved the verification stage.

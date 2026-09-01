---
id: "2026-08-29-disable-window-resizing"
stage: verification
status: pending
owner: "codex"
created: "2026-08-29"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: disable window resizing

## Automated checks

- Test-first failure: `cargo test -p doubao-skin-desktop ui_regression_tests --locked` failed before implementation because `main_window_options` did not exist.
- Desktop regressions: `cargo test -p doubao-skin-desktop --locked` passed all 9 tests, including fixed bounds, non-resizable state, and preserved movable/minimizable defaults.
- Scoped lint: `cargo clippy -p doubao-skin-desktop --all-targets --locked -- -D warnings` passed.
- Workflow policy: `./scripts/check.sh workflow` passed for all 8 artifact sets.
- Packaging: `./scripts/build-macos.sh --universal` passed; `lipo -archs` reports `x86_64 arm64`; strict code-sign verification passed.
- Archive: `dist/Doubao-Skin-macOS-universal.zip`, SHA-256 `5341ee65a1cdb650314bef2ef4f45ca38d9fbb2b25aba376496c8a7ebbc7f24f`.

## Behavioral evidence

- The window options use one `1120 × 720` size for both initial bounds and minimum size, with `is_resizable: false`.
- The live accessibility tree reports the zoom button as disabled while the close and minimize buttons remain enabled.
- A bottom-right edge drag toward a smaller size left the captured window at the same `1120 × 721` outer-frame screenshot size; the one extra pixel is the captured AppKit border around the configured `1120 × 720` bounds.

## Visual evidence

- `/Users/idevlab/.codex/visualizations/2026/08/29/theme-window-fixed-1120x720.png` (SHA-256 `99ac33dd949f23c241fe358dec1d9df06affd897de1897de62907aca9e16d191`).
- The image was captured from the final universal app after the resize attempt and contains only the theme tool's static preview.

## Security and privacy evidence

- Only the theme tool's own `WindowOptions` changed. The official 豆包 and 豆包工作 windows, bundles, conversations, cookies, and account data were not touched.

## Deviations and residual risk

- No product-scope deviation.
- The full `./scripts/check.sh rust` gate is currently blocked before the scoped checks by unrelated concurrent formatting changes in `crates/skin-core/src/authoring.rs`, `crates/skin-core/src/lib.rs`, and `crates/skin-core/tests/doubao_theme_cli.rs`. Running the workspace tests independently is also blocked by the unrelated compile-time `CARGO_BIN_EXE_doubao-theme` lookup in `doubao_theme_cli.rs`. The two task-scoped packages and Clippy pass.

## Follow-up: full-titlebar dragging

### Regression proof

- Red test: `cargo test -p doubao-skin-desktop ui_regression_tests::custom_titlebar_owns_window_dragging --locked` failed before implementation because `WindowOptions.app_owns_titlebar_drag` was `false`.
- Green test: the same command passed after the window declared application-owned titlebar dragging.
- The full `./scripts/check.sh all` gate passed on 2026-09-01, including workflow validation, 32 desktop tests, 70 core tests, integration tests, checks, supply-chain policy, Web tests, TypeScript, and the production Next.js build.
- `git diff --check` passed.

### Implementation evidence

- The shared compact and regular header roots start the native window move operation on a primary mouse-down event.
- The target segmented control stops that mouse-down from bubbling to the header, preserving direct target selection.
- The change remains inside the accepted movable-but-non-resizable window behavior; themes, target injection, package formats, and official client windows are unchanged.

### Real-window evidence

- `./scripts/package.sh desktop-macos` built the current arm64 package and strict deep code-sign verification passed.
- Computer Use launched this exact `dist/豆皮.app` and exercised titlebar drags from the left blank area, right blank area, and the blank area beside the target selector.
- Clicking `WorkBuddy` changed its accessibility state to selected, proving the segmented control was not converted into a drag surface; `豆包工作` was then selected again to restore the original state.
- The window remains fixed at the accepted `1120 × 720` content size, so a narrow-window drag check is not applicable to this product configuration.
- Screenshot: `/Users/chenli/.codex/visualizations/2026/09/01/01a05cfa-0f83-7c92-a00e-a39344abbfb2/doubao-skin-full-titlebar-drag.png` (`1120 × 721` captured outer frame, SHA-256 `b045a68863fac3df09f9535cb571d5f8561013cbced846dffdda650f388a484c`).

## Verdict

Implementation self-check passed. Status remains pending until a fresh-context verifier reviews the code, commands, and screenshot.

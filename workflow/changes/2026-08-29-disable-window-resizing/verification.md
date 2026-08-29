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

## Verdict

Implementation self-check passed. Status remains pending until a fresh-context verifier reviews the code, commands, and screenshot.

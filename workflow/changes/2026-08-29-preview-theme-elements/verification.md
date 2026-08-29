---
id: "2026-08-29-preview-theme-elements"
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

# Verification: preview theme elements

## Automated checks

- Test-first failure: `cargo test -p skin-core v2_theme_maps_semantic_fields_and_runtime_assets -- --exact` failed before implementation with 14 `E0609` errors for the missing `PreviewStyle.icons` and composer preview fields.
- Target regressions: `cargo test -p skin-core loads_bundled_themes --locked` and `cargo test -p skin-core v2_theme_maps_semantic_fields_and_runtime_assets --locked` passed.
- Desktop compile: `cargo check -p doubao-skin-desktop --locked` passed.
- Full Rust suite: `cargo test --workspace --all-targets --locked` passed, including 6 desktop UI regressions and 25 core tests.
- Lints: `cargo clippy --workspace --all-targets --locked -- -D warnings` passed.
- Repository gates: `cargo fmt --all -- --check`, `./scripts/check.sh workflow`, and `./scripts/check.sh web` passed. The combined `./scripts/check.sh` also completed its workflow, Rust, Clippy, TypeScript, Next.js build, and audit stages without a reported failure.
- Packaging: `./scripts/build-macos.sh --universal` passed; `lipo -archs` reports `x86_64 arm64`; `codesign --verify --deep --strict` passed.
- Archive: `dist/Doubao-Skin-macOS-universal.zip`, SHA-256 `8042b80d46500acc91707d72d2c9479ba8b829b9a5481f42b48650b7c9d90b42`.

## Behavioral evidence

- `PreviewStyle.icons` now contains one merged icon set selected by `preview.appearance`: variant values win and each missing field falls back to root `icons`.
- The synthetic v2 theme test proves a dark-variant `main` icon and a root-level `send` icon coexist in the preview result; it also checks composer text, placeholder, icon color, height, padding, gap, icon size, sidebar width, page margin, and radius scale.
- Bundled-theme assertions prove `甜点偷笑` receives `main/newTask/voice` from `variants.light.icons`, while `馋嘴豆包` retains root `main/dailyWork/readAloud`.
- Desktop rendering no longer reads `row.theme.icons` for the large preview. Visible sidebar, topbar, main, recommendation, and composer slots all read `PreviewStyle.icons`.
- SVG assets use the preview color; raster assets use `img()` with `ObjectFit::Contain`, preserving embedded color and transparency.

## Visual evidence

- Normal window (1120 × 721), `馋嘴豆包`: `/Users/idevlab/.codex/visualizations/2026/08/29/theme-elements-preview-normal.png` (SHA-256 `6932f77c83545d300df1985be316581b92607803b7e4a755d95d286423838223`). Custom drooling-head main icon, sidebar icons, recommendation icons, composer icons, input border, radius, and spacing are visible without duplicated glyphs or placeholder blocks.
- Normal window (1120 × 721), `甜点偷笑`: `/Users/idevlab/.codex/visualizations/2026/08/29/theme-elements-preview-dessert-normal.png` (SHA-256 `7ab674f31999b5e710247b49b7fe52ea057963ee2b00ebf9d06936039f05fcbb`). Its light-variant raster main icon and custom navigation/composer SVGs appear instead of neutral placeholders.
- Minimum window (720 × 561 screenshot of the 720 × 560 layout): `/Users/idevlab/.codex/visualizations/2026/08/29/theme-elements-preview-narrow.png` (SHA-256 `36d7c41f0b059e27b835e3ee77caa792229aa19df62a3d69c2bb2b121a6d90eb`). The preview remains legible; the custom main icon keeps its aspect ratio and the composer stays inside the canvas.
- Both normal screenshots were taken from an isolated copy of the final universal archive; `馋嘴豆包` was also inspected in the 720 × 560 compact layout.
- Visual QA used isolated, ad-hoc-signed copies of the built app with temporary unique bundle identifiers so concurrently running theme-tool QA windows could not contaminate the accessibility tree or screenshot target.

## Security and privacy evidence

- The implementation only reads already-sandboxed local theme asset paths; it adds no network access or script execution.
- Visual QA used only the theme tool's static mock preview. No real 豆包/豆包工作 conversation, account, cookie, attachment, or official app bundle was opened or modified.
- Theme packages, runtime injection, theme application state, and `/Applications` contents were not changed by this work.

## Deviations and residual risk

- No product-scope deviation: only the two planned Rust files and this verification artifact were changed for the feature; theme assets and runtime CSS/JS were untouched.
- The worktree had active parallel edits and packaging. One combined format check was initially blocked by unrelated `live.rs` line wrapping; after that parallel edit settled, the full repository format check passed. No unrelated logic was reverted.
- GPUI's static mock cannot execute arbitrary theme CSS, reproduce animation, or preserve alpha separately when the existing color parser reduces `rgba()` to RGB. The preview therefore represents modeled v2 fields and uses readable preview opacity, not every DOM-state detail.
- Fresh-context verification is still required by repository policy; the implementation session does not issue the final independent verdict.

## Verdict

Implementation self-check passed. Status remains pending until a fresh-context verifier reviews the code, commands, and screenshots.

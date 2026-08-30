---
id: "2026-08-30-fix-greeting-animation-overlay"
stage: verification
status: pending
owner: "codex"
created: "2026-08-30"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: fix greeting animation overlay

## Automated checks

- Original screenshot replay (before code change): ImageMagick sampled the user-provided image and returned `upper RGB distance=0.101961`, `lower RGB distance=0.00784314`, then `RED: 640x90 greeting-mask block detected` with exit 1. This is the exact reported two-surface symptom.
- Test-first red: `cargo test -p skin-core doubao_work_greeting_animation_mask_is_clipped --locked` failed before the CSS change at `theme::tests::doubao_work_greeting_animation_mask_is_clipped` because the DoubaoWork-scoped clipping rule was absent.
- Test-first green: the identical command passed after adding one runtime CSS declaration; 1 selected test passed and 23 were filtered out.
- `cargo test -p skin-core theme::tests --locked`: passed, 16 theme tests.
- `cargo fmt --check`: passed after applying `cargo fmt` to the new assertion layout.
- `./scripts/check.sh rust`: exit 0; the full Rust gate included 24 `skin-core` tests, 9 desktop UI regression tests, CLI/integration/example builds, and the remaining workspace Rust checks.
- `./scripts/check.sh workflow`: passed (`devflow: validated 13 artifact set(s)` and approval/policy cases passed).
- `git diff --check`: passed.

## Behavioral evidence

- Pre-fix live CDP probe on the official DoubaoWork homepage found `.greeting-text-Q0pGud::after` from the official `chrome://doubaowork-chat` stylesheet. Its computed size was `320 × 45 CSS px`, transform was `translateX(352px)`, background was the semi-transparent `--chat-bg-color`, and animation remained in the filled final state. Retina 2× maps this exactly to the reported `640 × 90 px` block.
- The implemented generated CSS is target-scoped and hash-independent: `html[data-skin][data-skin-target=doubao-work] [class*="greeting-text-"]{overflow:clip!important;}`.
- The regression test also proves that `--chat-bg-color` remains a semi-transparent runtime value and that the generated CSS does not contain the build-specific hash `Q0pGud`.
- Post-fix live CDP probes in the normal and narrow windows both returned `overflow: clip` for the greeting. The official `::after` pseudo-element still reported `width: 320px`, `height: 45px`, `transform: matrix(1, 0, 0, 1, 352, 0)`, and `animation: 0.2s linear both after-byooVd`, so the fix clips only the escaped mask and preserves the original animation.
- The same probe confirmed the active light theme was `gugugaga-administrator`, the target was `doubao-work`, and the official class continued to match the stable `greeting-text-` prefix.
- Pixel replay against the reported mask area measured 4 unique colors and entropy `0.366221`; the equivalent post-fix escaped-mask area measured 274 unique wallpaper colors and entropy `0.597985`, with no flat overlay remaining.

## Visual evidence

- [Normal-window center crop](evidence/greeting-overlay-fixed-normal.png): captured at `1395 × 768` with the light 「咕嘎管理员」 theme; greeting, wallpaper, recommendations, and composer remain intact with no right-side rectangle.
- [Narrow-window center crop](evidence/greeting-overlay-fixed-narrow.png): captured after switching the real app window to `1185 × 768`; the responsive layout remains intact with no overlay.
- Both persisted images are mechanically center-cropped to exclude the sidebar, account identity, recent conversations, and notifications.

## Security and privacy evidence

- Product code changes only generated CSS in `skin-core`; no network, protocol bridge, credentials, headers, cookies, conversation data, attachments, or official app resources were touched.
- The visual run was restricted to the blank homepage and persisted only center-cropped screenshots without account or conversation surfaces.
- `/Applications/DoubaoWork.app` remains unmodified; live application continues through the existing loopback CDP path.
- After validation, the original `gugugaga-snowfield` theme, `1395 × 768` window size, and previously open conversation were restored.

## Deviations and residual risk

- No implementation deviation: one runtime CSS rule and one regression test, with no JS, theme-specific branch, theme asset, app UI, or Web changes.
- Residual external-contract risk: the selector relies on the official semantic CSS Module prefix `greeting-text-`. It deliberately avoids the current full hash, but an official rename of the semantic prefix would require a future compatibility update.
- `overflow:clip` support and computed behavior were observed in the current DoubaoWork Chromium 147 runtime at normal and narrow sizes.
- The temporary structural probe used only class, geometry, and computed styles and was removed after the run; it did not read or persist conversation content.

## Verdict

Implementation and implementer verification are complete. Pending the repository-required fresh-context or human final verdict.

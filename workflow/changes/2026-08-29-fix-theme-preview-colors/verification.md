---
id: "2026-08-29-fix-theme-preview-colors"
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

# Verification: fix theme preview colors

## Automated checks

- Test-first failure: the temporary `gallery-whale-maid` harness and the new core regression failed before implementation because preview fields were plain `u32` values with no alpha member.
- Core regressions: `cargo test -p skin-core --lib --locked` passed all 27 tests.
- The new catalog audit loads all 26 bundled themes, checks every preview-visible color for valid RGB/alpha, and proves every current theme relies on at least one translucent preview color.
- Desktop regressions: `cargo test -p doubao-skin-desktop --locked` passed all 9 tests, including alpha multiplication (`0.16 × 0.60 = 0.096`).
- Scoped lints: both `cargo clippy -p skin-core --lib --locked -- -D warnings` and `cargo clippy -p doubao-skin-desktop --all-targets --locked -- -D warnings` passed.
- The independent temporary harness passed after the fix and was moved to Trash after use.

## Behavioral evidence

- `PreviewColor` now transports RGB and alpha together from `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()`, and `rgba()` declarations.
- The desktop renderer multiplies the theme alpha by the layer/user opacity at every preview paint site instead of replacing the theme alpha.
- `gallery-whale-maid` now resolves its main background to `#bd9999 @ 0.16`, input to white at `0.96`, border to `#7a4e29 @ 0.28`, and placeholder to `#352970 @ 0.60`.
- The complete audit found no additional manifest or CSS declaration errors, so no theme package or generated Web catalog file was changed.

## Visual evidence

- Whale preview: `/Users/idevlab/.codex/visualizations/2026/08/29/theme-whale-blue-preview.png` (SHA-256 `6f6ed8d45799c3884d744ac8c21187cf7e27ce0a87b0ab55f6e173c82add6b08`). The background remains visibly cyan/blue; brown is limited to accents.
- QQ light-blue cross-check: `/Users/idevlab/.codex/visualizations/2026/08/29/theme-qq-blue-preview.png` (SHA-256 `39ab28005cc54f8d5178265c9aac67c6757dfc6d45ccd61feff3b598e9e699bc`).
- A no-background dark theme (`纯暗`) was also inspected in the final packaged window; its text and controls remained readable.

## Security and privacy evidence

- The change only reads existing local theme declarations and paints a static preview. It adds no network request or dynamic CSS execution and does not apply a theme to either official app during QA.

## Deviations and residual risk

- No theme-specific rendering branch and no theme-package recoloring were introduced.
- The full repository Rust gate remains blocked by unrelated concurrent formatting and CLI-test configuration issues described in the fixed-window verification; task-scoped core, desktop, workflow, packaging, and Clippy checks pass.

## Verdict

Implementation self-check passed. Status remains pending until a fresh-context verifier reviews the code, commands, and screenshots.

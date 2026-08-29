---
id: "2026-08-29-dmg-and-whale-theme-icon"
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

# Verification: dmg and whale theme icon

## Automated checks

- Red-first opacity evidence:
  - `runtime_surface_opacity_outranks_variant_and_legacy_background_rules` failed on the old selector because a dark-variant fixed-alpha rule still won.
  - `bundled_theme_css_does_not_claim_runtime_surface_priority` failed on the old tree and reported 16 bundled themes with high-priority controlled background declarations.
  - `live_bootstrap_marks_and_isolates_the_selected_target` initially failed to compile against the old target-less bootstrap, and the restore assertion showed that no target marker was cleaned up.
  - The read-only standard-Doubao probe reproduced the reported 40% mismatch: expected page/sidebar `0.22/0.30`, actual `0.72/0.82` before the fix.
- Green regression checks:
  - `cargo test -p skin-core --locked`: passed, including runtime priority, full-theme audit, target isolation, restore cleanup, theme loading/packing, authoring and CLI tests.
  - `./scripts/check.sh rust`: passed (desktop 9 tests, core 30 tests at that run, authoring 7, bundled paths 1 and CLI 3, followed by clippy).
  - `corepack pnpm --dir apps/web sync`: passed; 26 themes and 26 installable packages regenerated.
  - `./scripts/check.sh web`: passed; Node tests, TypeScript, Next production build and high-severity dependency audit were green.
  - `./scripts/check.sh workflow`: covered by the full gate; 10 artifact sets and policy tests passed.
  - `RUST_TEST_THREADS=1 ./scripts/check.sh all`: passed end to end after the final asset and package changes; 31 core tests, desktop, authoring, CLI, clippy, Web build and audit all passed.
  - `ruby -e 'require "yaml"; YAML.parse_file(".github/workflows/release.yml")'`: passed.
  - `bash -n scripts/build-macos.sh`: passed through the workflow gate.
- Image/package checks:
  - `themes/gallery-whale-maid/icons/main.png` is `1024x1024` RGBA with four transparent corners. Both manifest variants point to `icons/main.png`.
  - The source PNG, generated Web icon and PNG inside `gallery-whale-maid.doubao-skin.zip` share SHA-256 `a5286db3923b5a29e077da9fc409964b48725756373ec73179a79a779f76af92`.
  - `git diff --check` on the scoped files passed and no temporary debug-log marker remains in the touched runtime, desktop, script, theme or workflow paths.

## Behavioral evidence

- Standard Doubao `2.26.10` and Doubao Work `2.26.7` were exercised through their loopback CDP targets without reading message text, cookies, headers, storage or account data.
- Default whale-maid application after navigation/reapplication:
  - Standard Doubao: `data-skin-target="doubao"`, dark appearance, opacity `0.68`, page `rgba(18,20,25,0.374)`, sidebar `rgba(18,20,25,0.51)`, one style node and zero backdrop nodes.
  - Doubao Work: `data-skin-target="doubao-work"`, light appearance, opacity `0.68`, page `rgba(189,153,153,0.374)`, sidebar `rgba(255,255,255,0.51)`, one style node and zero backdrop nodes.
  - Filtered evidence: [standard-doubao-cdp.json](evidence/standard-doubao-cdp.json) and [doubao-work-cdp.json](evidence/doubao-work-cdp.json).
- Three-theme/two-target/three-opacity matrix passed for `doubao-snack-giggle`, `gallery-whale-maid` and `qq-light-blue`: [theme-opacity-matrix.tsv](evidence/theme-opacity-matrix.tsv).
  - Every 40% case computed page/sidebar `0.22/0.30` on both targets.
  - Default and 100% cases increased monotonically to their profile values.
  - Every row retained exactly one style node, zero backdrop nodes and the correct target marker.
- Restore was executed on both targets and left no `data-skin`, `data-skin-target`, style node or backdrop node: [restore-doubao.json](evidence/restore-doubao.json) and [restore-doubao-work.json](evidence/restore-doubao-work.json). Whale-maid was then reapplied to both targets.
- The standard-only `#chat-route-main` neutralization is guarded by `data-skin-target="doubao"`; the Doubao Work probe retained its independent surface colors and target marker.
- All themes found by the structural audit had only the controlled background-variable `!important` removed. The audit additionally found `sakura-night`, which was absent from the plan's enumerated ownership list but was required by the accepted full-theme audit specification.
- Packaging:
  - Final host build produced `Doubao-Skin-macOS-arm64.{zip,dmg}` plus both SHA-256 files; both checksum files verified, the mounted GUI and CLI were `arm64`, strict codesign passed and the bundled icon hash matched the source.
  - Final universal build produced `Doubao-Skin-macOS-universal.{zip,dmg}` plus both SHA-256 files; both checksum files and `hdiutil verify` passed.
  - The read-only universal mount contained only `豆包主题.app` and `Applications -> /Applications`; strict codesign passed, version was `0.1.0`, and GUI plus bundled CLI reported `x86_64 arm64`.
  - No `.dmg-staging.*` directory or `*.tmp.dmg` remained in `dist` after validation and all test mounts detached successfully.

## Visual evidence

- The final icon was generated with built-in ImageGen, then only background/alpha cleanup, one-pixel edge cleanup, resize and centering were performed with ImageMagick.
- Final correction prompt: `Edit the blue whale-maid chibi icon: remove the drool and hungry/open-mouth expression; give her a small closed-mouth, calm knowing smile and bright intelligent eyes; keep the blue whale-tail headpiece, white-blue maid headdress and clean head-and-shoulders composition; no text, brand or watermark; use a flat vivid magenta background for clean chroma-key extraction.`
- ImageGen source: `/Users/idevlab/.codex/generated_images/01a04981-501f-7642-8a9e-312d364fe986/exec-6672ebb7-26e8-4973-b787-d2bad17b3f40.png`.
- Final project asset: [themes/gallery-whale-maid/icons/main.png](../../../themes/gallery-whale-maid/icons/main.png). The character occupies a safe central area and remained recognizable at 128, 52 and 20 px on light and dark backgrounds.
- Fixed 1120x720 theme-tool window: [theme-picker-whale-maid.png](evidence/theme-picker-whale-maid.png). It shows the same wise, closed-mouth icon in the live preview and the blue theme palette rather than the earlier brown/logo placeholder.
- Actual official-client evidence, cropped to remove account and recent-conversation regions:
  - [standard-doubao-whale-maid.png](evidence/standard-doubao-whale-maid.png): standard Doubao dark/blue adaptation with the new icon and no extra fixed brown veil.
  - [doubao-work-whale-maid.png](evidence/doubao-work-whale-maid.png): Doubao Work light-blue adaptation with the same icon and readable content/composer surfaces.

## Security and privacy evidence

- No official app bundle under `/Applications` was modified; live validation used loopback CDP injection only.
- The retained CDP evidence contains only theme/target markers, controlled CSS variables, alpha values and injection-node counts. URLs, page titles, message text, cookies, headers, storage, attachments and workspace content were not saved.
- A preliminary non-empty-window screenshot was discarded and never written to repository evidence. The retained standard and Work screenshots were taken from empty/new-task states and cropped to exclude sidebar/account regions.
- Image generation used only the project whale-maid artwork and project-created icon language as visual references. The final PNG contains no third-party logo, text or watermark.
- No Apple credentials were supplied. Both local bundles use the existing ad-hoc path; this verification does not claim production notarization.
- DMG staging operated only under unique temporary directories. `Applications` inside the image is a symlink; no file was copied into the user's Applications directory.

## Deviations and residual risk

- The user explicitly replaced the accepted “light drool/hungry” expression with “wise, composed, no drool” during implementation. The final asset follows the newer instruction; accepted intent/spec/plan files were not rewritten retroactively.
- Built-in ImageGen returned RGB images with a baked background instead of usable alpha. The final iteration requested a flat magenta key; deterministic ImageMagick chroma-key/decontamination produced the RGBA file. No semantic drawing or third-party asset was added during post-processing.
- The local `pnpm` shim was broken, so the official sync/check operations were run through `corepack pnpm`; outputs are equivalent and both commands passed.
- `doubao-theme check themes/gallery-whale-maid` still reports the repository's existing exact-scope validator mismatch: the checker expects a theme-ID-specific selector while bundled themes use the shared `html[data-skin]` contract. Core bundled-theme loading, packing and Web sync passed, and this unrelated validator contract was not expanded in this change.
- The default parallel `./scripts/check.sh all` was attempted twice. Under the heavy parallel PAK/theme tests, the protocol-bridge upstream fixture missed its three-second client window and then waited in an unbounded test-only `accept/join`; both runs were interrupted after sampling confirmed that state. The exact protocol test passed alone in `0.03s`, and the complete gate passed with `RUST_TEST_THREADS=1`. This is a pre-existing test-harness flake, not a runtime bridge failure, but it remains follow-up debt.
- Current macOS emits a deprecation warning for `hdiutil create/attach` in favor of `diskutil image`. `hdiutil` remains functional and was explicitly selected by the accepted spec; create, verify, read-only mount and detach all passed.
- Production Developer ID/notarytool/stapler behavior was not exercised because credentials were unavailable. The Release path remains conditional and must be verified in an authorized release run.
- Temporary opacity, image-processing and matrix fixtures were moved to `/Users/idevlab/.Trash/doubao-dmg-whale-temp-20260829` and `/Users/idevlab/.Trash/doubao-theme-qa-temp-20260829`; they remain recoverable and are not part of the deliverable.

## Verdict

Implementation evidence is complete for the requested scope, but the artifact intentionally remains `pending`. A fresh-context verifier or human must review the saved evidence and set the final verdict.

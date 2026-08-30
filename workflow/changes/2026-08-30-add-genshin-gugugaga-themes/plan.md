---
id: "2026-08-30-add-genshin-gugugaga-themes"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-30"
based_on: spec.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-30"
---

# Plan: iconic character themes and matching icon sets

## Files and ownership

- `themes/{teyvat-*,gugugaga-*}`: four new theme packages and canonical previews.
- `themes/{doubao-snack-giggle,doubao-dessert-giggle}`: completed PNG icon mappings and alignment rules.
- `crates/skin-core/src/theme.rs`: semantic mapping and bundled-theme regressions.
- `apps/web/scripts` and generated Gallery data: preview/package synchronization and focused tests.
- `workflow/changes/2026-08-30-add-genshin-gugugaga-themes`: the single authoritative artifact chain.

## Order of work

1. Define the semantic icon contract and add a failing regression for incomplete full-color theme mappings.
2. Generate and review each character/background/icon asset independently, rejecting failed transparency or identity candidates.
3. Integrate the four new themes and complete the two existing icon sets.
4. Validate every theme in real DoubaoWork light/dark and normal/narrow states; promote only the final light/normal capture to each canonical `preview.jpg`.
5. Synchronize the 34-theme Gallery and validate desktop/narrow rendering and package contents.
6. During PR review, restore the 28 unchanged packages, prevent ZIP metadata-only rewrites, delete duplicated workflow screenshots, and consolidate the four staged records into this chain.
7. Run the complete local gate and GitHub CI before review handoff.

## Test-first proof

- The focused Rust regression initially failed because the character themes did not expose all full-color PNG mappings; it passed after the manifests and CSS were completed.
- Web source tests protect author-provided previews and byte-stable package preservation when only ZIP metadata differs.
- Package validation compares catalog SHA-256 and size against the files served by the Gallery.

## Visual or integration proof

- Inspect each new theme in actual DoubaoWork windows across both appearances and both supported widths.
- Inspect every generated icon at source size and actual rendered size before integration.
- Inspect the Gallery at 1280 x 720 and 768 x 900, including filtered results and theme detail pages.
- Retain only `themes/<id>/preview.jpg` for each affected theme; intermediate contact sheets and QA captures remain outside Git.

## Risks and mitigations

- Identity drift: establish and reuse one accepted Gugugaga heroine anchor before dependent generations.
- False transparency: reject baked checkerboards and use a deterministic keyed-background cleanup only on accepted generated candidates.
- Private UI leakage: validate in blank tasks and retain only redacted canonical previews.
- Repository bloat: preserve unchanged package bytes and exclude repeated screenshots from workflow evidence.
- Upstream drift: rebase onto current `main`, regenerate from source, and rerun all gates.

## Rollback

Revert the theme source directories, semantic regression, Gallery synchronization changes, and the six corresponding generated packages. The removed review screenshots and superseded workflow records remain recoverable from this branch's pre-cleanup commits if an audit requires them.

## Deviations

The work was originally executed through four separately approved change chains as the user supplied successive visual feedback. Review later determined that they describe one product change, so their requirements and verification were consolidated here without changing the final theme behavior.

## Decision

Accepted by `idevlab` on 2026-08-30 and revised in response to the explicit 2026-08-31 PR review: keep the final product assets and verification facts, remove repeated binary evidence and unrelated package churn.

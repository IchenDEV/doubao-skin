---
id: "2026-08-30-add-genshin-gugugaga-themes"
stage: intent
status: accepted
owner: "codex"
created: "2026-08-30"
source: "user"
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-30"
---

# Intent: add iconic character themes and matching icon sets

## Problem

The Gallery lacked recognizable Genshin and Gugugaga theme packs, while the existing snack and dessert themes still mixed generic SVG glyphs with themed artwork. Follow-up visual review also found that the first Gugugaga treatment did not preserve the supplied character identity closely enough and that duplicated generated packages, screenshots, and workflow records made the pull request unnecessarily large.

## Proposed outcome

Ship four coherent unofficial fan-art themes—two Teyvat-inspired and two Gugugaga-inspired—and complete the full-color icon systems for the snack and dessert themes. Every visible navigation and composer control should use themed character or prop artwork. Keep one canonical preview per affected theme, publish only packages whose theme sources changed, and represent the work through one workflow artifact chain.

## Affected users and systems

- DoubaoWork users applying the six affected themes.
- The Rust theme runtime and semantic icon mapping.
- The Web Gallery catalog, previews, SQLite data, and downloadable packages.
- Repository reviewers evaluating visual quality, provenance, privacy, and binary size.

## Constraints

- Do not modify `/Applications/DoubaoWork.app`; runtime validation uses the loopback bridge.
- Use full-color raster artwork rather than simple line-frame SVG replacements.
- Preserve readable sidebar text, composer controls, and light/dark contrast at normal and narrow widths.
- Do not commit the supplied reference image, official game assets, raw private-window captures, or unclear third-party downloads.
- Treat generated character work as unofficial and not commercially cleared.
- `themes/<id>/preview.jpg` is the sole retained visual proof for each affected theme; do not duplicate screenshots under `workflow/changes`.
- Only the six changed theme ZIPs may differ from `main`; metadata-only ZIP churn is not acceptable.

## Out of scope

- Official endorsement, trademark licensing, or commercial clearance.
- Changing protocol payload behavior, credentials, release signing, or production deployment.
- Redesigning unrelated themes or the newer app icon and greeting animation from `main`.

## Success signals

- Four new theme packs and two completed existing icon packs pass theme validation.
- Search, voice, sidebar expansion, New Task, Scheduled, composer, and primary navigation use matching PNG artwork.
- The final Gugugaga themes retain the accepted penguin-suit heroine identity and recognizable props.
- Real DoubaoWork review passes in light/dark and normal/narrow states.
- The Gallery lists 34 installable themes without broken images or responsive overflow.
- The PR contains six changed ZIP packages, zero workflow screenshots, and one consolidated change directory.

## Open questions

No product question remains. Merge and any later release remain separate human decisions.

## Decision

Accepted by `idevlab` through the four staged reviews on 2026-08-30. On 2026-08-31 the same reviewer requested that the resulting theme work be consolidated and that redundant ZIPs, screenshots, and change records be removed from the Draft PR.

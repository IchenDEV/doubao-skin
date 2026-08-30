---
id: "2026-08-30-add-genshin-gugugaga-themes"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-30"
based_on: intent.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-30"
---

# Spec: iconic character themes and matching icon sets

## Requirements

- Add `teyvat-dandelion-wind`, `teyvat-liyue-lanterns`, `gugugaga-administrator`, and `gugugaga-snowfield` as complete theme-v2 packages.
- Replace the remaining visible SVG controls in `doubao-snack-giggle` and `doubao-dessert-giggle` with theme-matched PNG artwork.
- Each new character theme provides a background, a canonical 1200 x 675 preview, a main icon, and PNG sources covering all exposed semantic icon slots.
- Search/research, voice/read-aloud, sidebar/attach, confirm/send, and connector/tools may share semantically related artwork; unrelated controls may not silently fall back to generic glyphs.
- Raise only New Task and Scheduled artwork by one optical pixel where the themed assets require it.
- Preserve all unrelated packages byte-for-byte when synchronization finds identical archive contents.

## User experience

- Genshin themes must show recognizable wind/Mondstadt and Liyue/Lantern Rite character-and-prop anchors without copying official UI or logos.
- Both Gugugaga themes must depict the same accepted blue-black-haired penguin-suit heroine, including the hood, yellow beak and webbed boots, white belly, and silver/teal fastener details.
- Sidebar labels, selected states, the main prompt, and composer controls remain readable in light and dark appearances.
- Icons remain identifiable at their rendered 20 px size and are optically centered beside labels.

## Technical design

- Theme behavior stays in `themes/<id>/theme.json` and `theme.css`; the Rust runtime resolves semantic fields to data URIs without theme-specific desktop branches.
- `apps/web/scripts/sync-themes.mjs` preserves author-provided previews and compares candidate ZIP entry names, lengths, and CRCs with the checked-in package before replacement. ZIP timestamp-only differences therefore do not enter the diff.
- Gallery synchronization regenerates `themes.db`, catalog metadata, required preview derivatives, and packages for changed theme sources.
- The catalog SHA-256 and byte size must match every checked-in package.

## Security and privacy

- The user attachment remains outside the repository and is used only as a visual identity reference.
- Repository assets are newly generated fan-art compositions; research images and official game resources are not bundled.
- Canonical previews use blank/new-task UI with private conversation and account regions removed or obscured.
- The loopback-only bridge and its plain-text payload boundary remain unchanged.

## Alternatives and non-goals

- Generic SVG recoloring was rejected because it does not provide the requested theme identity.
- Keeping every QA screenshot was rejected because the canonical previews already preserve the final approved visual state.
- Regenerating all 34 ZIPs was rejected because 28 themes have no source change.
- This change does not grant redistribution rights or publish a production release.

## Areas of concern

- Recognizable fan art retains intellectual-property and redistribution risk.
- DoubaoWork DOM changes can break semantic selectors in future releases.
- Archive comparison depends on the system `zip` and `unzip` tools already required by the Gallery synchronization workflow.

## Acceptance criteria

- Theme and Rust regression tests cover full-color PNG mappings and package loading.
- Web tests cover existing-preview preservation and metadata-only ZIP preservation.
- Real-window review covers four new themes in light/dark and normal/narrow states.
- Gallery desktop and 768 px review shows 34 cards, correct search filtering, complete previews, and no relevant console errors.
- The final diff contains exactly six changed ZIP packages and no image files under this workflow change.
- `./scripts/check.sh all`, `git diff --check`, workflow validation, catalog/package hash checks, and GitHub CI pass.

## Decision

Accepted by `idevlab` on 2026-08-30. This consolidated specification is the union of the separately accepted theme-addition, snack/dessert-icon, iconic-fan-art, and Gugugaga-identity requirements, with the 2026-08-31 review request removing redundant repository artifacts.

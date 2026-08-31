---
id: "2026-08-31-theme-package-v3"
stage: verification
status: pending
owner: "codex"
created: "2026-08-31"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: theme package v3

## Automated checks

### Build baseline — 2026-08-31

- `./scripts/check.sh workflow` — passed (`devflow: validated 16 artifact set(s)`; approval and policy cases passed).
- Source inventory: 30 theme directories, 30 `theme.json` files, 30 CSS files, and 117 other resource files.
- Every bundled theme is `schemaVersion: 2`, `version: 1.0.0`, and `appearance: both` at baseline.
- Theme manifest/CSS inventory digest: `95f3ee1b6f9c1beab0070a4ff2b5d97e7287dbb443ecd515226a29ca306a8cb3`.
- Complete theme file inventory digest: `74803bd01d38e2dd8e121ddb326bf56de0a99ca183dd2a0fc880f1e8835e104d`.
- Generated catalog baseline: 30 database rows, 30 catalog themes, 30 package ZIPs, 30 full previews, and 30 card previews.
- Generated catalog ID digest: `4fea09f6c67448e3ebffedcccde68b9bb3a8d865c7b13aa68ecd5040431a5892`.
- Pre-existing WorkBuddy change overlaps seven tracked files and is preserved as an implementation input. Its baseline binary-diff digest is `1cf2a51a9eb216e9942d7c6e456e8e293826ef5618e28c5daa016dd5a002db8d`.

Baseline theme IDs (exactly the accepted Spec set):

`claude-warm`, `codex-catppuccin`, `codex-dracula`, `codex-gruvbox`, `codex-nord`, `codex-one-half`, `codex-solarized`, `cyber-neon`, `doubao-dessert-giggle`, `doubao-snack-giggle`, `forest`, `gallery-cozy-room`, `gallery-crimson-rain`, `gallery-moon-pine`, `gallery-neon-koi`, `gallery-whale-maid`, `github-repository`, `gothic-void`, `huaxia-blue`, `machine-overseer`, `mist-forest`, `ocean-cyan`, `pastel-flower-club`, `pastel-starry-room`, `pastel-tea-party`, `peach-sunset`, `pure-dark`, `qq-light-blue`, `sakura-night`, `violet-night`.

The baseline digest covers current source state only. Later gates compare identity, provenance, preview, resource, and generated-output invariants individually rather than accepting digest changes on trust.

## Behavioral evidence

### Contract and runtime slice

- Added strict Draft 2020-12 schema and normative/mutation fixtures for v3. `cargo test -p skin-core --test theme_v3_schema` — 2/2 passed.
- Added the `theme_package` deep module and CSS AST validator. `cargo test -p skin-core --test theme_package_v3` — 10/10 passed, covering explicit support, merge/null deletion, target appearance changes, effective CSS order, v1/v2 compatibility, future-schema rejection, exact-case/symlink/resource validation, and CSS scope/property/URL/at-rule/reserved-variable attacks.
- Parser dependencies compile and their redistribution notices are recorded: `lightningcss 1.0.0-alpha.72`, `parcel_selectors 0.28.3`, `jsonschema 0.52.1`, and `roxmltree 0.21.1`.
- `cargo test -p skin-core --test authoring` — 9/9 passed. This includes v3 creation, dark-only base semantics, dry-run/write migration, deterministic contract files, preview regeneration, safety rejection, and no-overwrite behavior.
- `cargo test -p skin-core --test doubao_skin_cli` — 4/4 passed. `create` requires explicit targets; `check`, `pack`, `install`, `apply`, `restore`, and `migrate-v3` retain stable JSON/exit contracts under the renamed `doubao-skin` CLI.
- `cargo test -p doubao-skin-desktop --bin doubao-skin` — 12/12 passed. Target switches now filter unsupported installed/store themes, refresh target previews, and distinguish `专属适配` / `共享适配` / `兼容模式`.
- `migrate-v3` dry-run completed for 30/30 bundled directories. After the user explicitly authorized “先批量迁移”, write mode migrated all remaining 29 themes without an intermediate failure.
- The source invariant was first reached for the accepted 30-theme baseline. During Draft PR preparation, `origin/main` added four approved fan themes plus new full-color PNG resources for Snack/Dessert. Conflict integration migrated all six affected v2 manifests, so the current invariant is 34/34 `schemaVersion: 3`, theme version `2.0.0`, `shared.appearance: both`, and explicit `doubao` / `doubao-work` / `workbuddy` target keys.
- Thirty-four CLI `check` operations, Rust `pack` operations, and package integrity checks pass. No package includes an unreferenced root `theme.css`.
- `pnpm --dir apps/web sync` generated 34 database rows, 34 v3 catalog records, and 34 installable packages. Every catalog record has three target support entries and a valid SHA-256 digest.
- `./scripts/check.sh web` — passed against the complete v3 catalog (16 Node tests, TypeScript, dependency audit, and 42-page Next build).
- `cargo clippy -p skin-core --all-targets -- -D warnings` and `cargo clippy -p doubao-skin-desktop --all-targets -- -D warnings` — passed.
- `./scripts/check.sh workflow`, `./scripts/check.sh rust`, `./scripts/check.sh web`, and `./scripts/check.sh all` — passed. The Web gate includes the dependency audit (`No known vulnerabilities found`).
- `git diff --check` — passed.
- Both repository Skills pass `skill-creator` quick validation, and `node apps/web/scripts/sync-skills.mjs --check` passes after their v3/target updates.

### Bulk migration closure

- The migration removed the legacy root CSS files and retained cross-host visual/resource declarations in structured v3 fields. The main-branch PNG upgrade exposed a real compatibility gap: masking raster icons destroys their colors. The trusted runtime adapter now detects raster icon resources and renders them full-color without widening the untrusted v3 CSS whitelist or adding theme-ID branches.
- A full Rust run exposed four expected migration-edge failures. Two old tests were updated from all-v2 to the final all-v3 invariant. The v3 backdrop test now verifies the absence of a second legacy gradient. The real icon-palette regression was fixed by keeping its target-scoped colors and moving SVG-image masking into the trusted engine adapter. v1/v2 compatibility remains locked by the synthetic legacy fixture.
- v3 UI swatches now derive from trusted semantic CSS when a package has no declared CSS; legacy themes continue deriving swatches from their original CSS.
- Final source shape is 34 v3 manifests and zero package CSS files. The four newly merged fan themes and the Snack/Dessert PNG upgrades retain their main-branch images and full-color icon resources.

### Sample package closure

- `gallery-whale-maid` is `schemaVersion: 3`, theme version `2.0.0`, preserves its ID/name/description/store order/provenance/preview/background/icon resources, declares all three targets, and removes the unreferenced root `theme.css`.
- CLI report resolves light/dark for all three targets. Doubao and DoubaoWork are `shared / explicit`; WorkBuddy is `tailored / explicit` because it explicitly removes inherited icons.
- The sample package contains only `theme.json`, `preview.jpg`, `bg.jpg`, and `icons/main.png`; the Rust packer, not Node `zip`, produced it.
- A sample-only Web sync emitted the same target report into SQLite/catalog, produced one installable package, and passed the production Next build. Its source preview SHA-256 remained `d957dfa15e9a9d5123002748050c1a12cfcf1a2e1a0bf9cbc71b4c08da5fa6a5`; sync no longer rewrites author previews. The pre-test generated catalog was then restored; no partial v3 catalog remains checked in.
- Full sync now succeeds for all 34 v3 themes without bypassing Rust validation. The tracked catalog output is a complete 34-theme v3 generation, not the earlier sample-only output.

### Installable test package — 2026-08-31

- `CODESIGN_IDENTITY=- BUNDLE_ALL_THEMES=1 ./scripts/package.sh desktop-macos --universal` and `CODESIGN_IDENTITY=- ./scripts/package.sh cli --universal-macos` completed. The App executable and standalone `doubao-skin` CLI are both universal `x86_64 arm64`; the CLI reports `doubao-skin 0.4.0`.
- The App contains exactly 34 theme directories, 34 `schemaVersion: 3` manifests, and 34 explicit WorkBuddy target declarations. ZIP extraction and read-only DMG mounting each retained all 34 themes.
- ZIP, DMG, and extracted/mounted App `codesign --verify --deep --strict` checks passed. ZIP and DMG integrity checks passed, and the DMG contains the expected `/Applications` link.
- Test artifacts: `Doubao-Skin-macOS-universal.zip` (36,821,522 bytes, SHA-256 `786b1084e3289dadfe4986b1ba7ca1293ca080b59f63f40dbda852c5e3d796d5`), `Doubao-Skin-macOS-universal.dmg` (40,325,628 bytes, SHA-256 `35e01132fed4e6dd9fa4a55f57ab002649c068f7bc1b72b059fc6ae1b8792c98`), and `doubao-skin-cli-macOS-universal.tar.gz` (8,057,915 bytes, SHA-256 `5172cbf5939b5adbb0eb4db8654311c99c7d8b6a4991759e8d691898c303897a`).
- The configured long-lived certificate is present and its certificate fingerprint still matches `C37941DCA5C5E4FDAAB45685C803547EA3AFBCAD3E2534FE23FCA89F5839FC52`, but local `codesign` rejects it as an available identity. The test package therefore uses an explicit ad-hoc hardened-runtime signature (`Signature=adhoc`, no TeamIdentifier). This is acceptable only for local testing; it is not evidence for the stable-signature release gate.

## Visual evidence

### Website

- Browser plugin path used at `http://127.0.0.1:3100/` with a sample-only generated catalog.
- Desktop 1280 × 800: page identity, non-blank content, no framework overlay, no console warnings/errors, WorkBuddy filter URL (`?target=workbuddy#gallery`), selected state, card capability labels, detail capability labels, and archive support declaration passed.
- Mobile 390 × 844: no horizontal overflow (`scrollWidth == clientWidth == 390`), theme card remains visible, filter trigger expands, WorkBuddy remains selected, and all target/type/series options remain keyboard-addressable links.
- Browser evidence showed the intended low-interference chips: 豆包/豆包工作 “支持”, WorkBuddy “专属适配”.

### Real applications — sample only

- Installed applications observed: DoubaoWork 2.27.6 and WorkBuddy 5.3.14. `/Applications/Doubao.app` is absent, so the two Doubao sample scenarios cannot run.
- DoubaoWork light: passed visual inspection for background, readable sidebar/content, controls, composer, and border weight.
- DoubaoWork dark: passed after the composer surface fix. Computed composer values were `rgba(29, 31, 37, 0.76)`, `1px solid rgb(155, 122, 94)`, and `rgb(247, 248, 250)` text; an isolated composer screenshot contained no conversation content.
- WorkBuddy light: passed visual inspection for background, sidebar/content hierarchy, controls, and composer.
- WorkBuddy dark: passed after selecting the application's own `外观 -> 深色` setting. The native host reported `dark cb-dark vscode-dark`, persisted its appearance as `dark`, and retained that state while the theme was applied. The rendered window showed the whale background, solid dark sidebar, readable controls, white foreground text, and a dark composer surface; the composer ancestor resolved to `rgba(29, 31, 37, 0.87)`, a visible accent border, and `rgb(247, 248, 250)` text.
- A real defect found during this probe was fixed: a target preview marked light could make a resolved dark-only WorkBuddy payload emit `color-scheme: light`. Runtime now uses the resolved theme mode; the v3 runtime test locks both light and dark payloads.
- Final restoration passed: DoubaoWork restored 3 responsive pages and WorkBuddy restored 1. Both subsequently reported no `data-skin`, no `data-skin-target`, no theme style, and no backdrop. WorkBuddy's appearance preference was also returned to its original light setting.

## Security and privacy evidence

The approved execution remains limited to repository files, temporary package/install directories, and loopback-CDP theme injection. It does not authorize modification of official application bundles or capture of user content.

- Theme packages reject path traversal, case mismatches, symlinks, fake images, unsafe SVG, external URLs, unscoped selectors, reserved runtime variables, and structure/interaction CSS.
- Real-window evidence remained in the ephemeral test session. No screenshot, page text, cookie, header, account data, attachment, tool payload, or workspace content was written to the repository.
- Official application bundles were not modified. Test-only DOM state was removed, and the normal restore path was verified afterward.

## Deviations and residual risk

- The accepted Plan originally required the six-scenario whale sample gate before writing the other 29 migrations. WorkBuddy dark brought the sample to 4/6; the user then explicitly authorized “先批量迁移”. The accepted Plan records this order deviation. It authorizes the bulk source/catalog build but does not convert the absent Doubao evidence into a pass or relax the final Review/Release Gate.
- Source, catalog, package round trips, and `./scripts/check.sh all` are complete for all 34 v3 themes. This is no longer a mixed v2/v3 source state.
- The original 180-scene baseline matrix plus the 24 scenes introduced by the four newly merged themes, its narrow-window representative matrix, the two unavailable Doubao whale scenarios, and a fresh-context verdict remain pending. They must not be reported as passed.
- The user separately authorized a test package and PR after accepting the Plan. A Draft PR is permitted so review and test-package feedback can proceed while verification remains pending; merge, deployment, Release creation, and the production catalog switch remain unauthorized.

## Verdict

Bulk migration passed its repository, schema, resource, package, catalog, Rust, Web, workflow, security, and ad-hoc test-package gates. Overall verification remains pending rather than passed: Doubao is unavailable and the complete real-window matrix plus fresh-context verdict have not run. The Draft PR does not authorize merge or release.

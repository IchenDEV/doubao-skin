---
id: "2026-08-29-plugin-discovery-well-known"
stage: verification
status: pending
owner: "codex"
created: "2026-08-29"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: "GPT-5.6 Sol (fresh-context verifier)"
verified_at: "2026-08-29"
---

# Verification: plugin discovery well known

## Automated checks

- Red-first proof: `node --test scripts/skill-discovery.test.mjs` failed before implementation with `plugins/doubao-skin must be the canonical plugin package`; after implementation the combined Node run passes 9 tests (5 discovery/package tests and 4 existing filter tests).
- `node scripts/sync-skills.mjs --check` passes. The test suite separately corrupts frontmatter-derived description, `digest`, a published `SKILL.md`, and the generated file list; every corruption is rejected.
- Both migrated Skills pass `skill-creator/scripts/quick_validate.py`. Their final `SKILL.md` and `agents/openai.yaml` SHA-256 values match the pre-migration values exactly.
- `plugin-creator/scripts/validate_plugin.py plugins/doubao-skin` passes the Codex plugin manifest.
- Claude Code 2.1.251 from the official one-shot npm package passes `plugin validate` for both the repository Marketplace and plugin with no warnings.
- `./scripts/check.sh web` passes all 9 Node tests, Well-Known freshness, TypeScript, the Next.js 16.3.3 production build with 38 static/generated pages, dependency audit, and script syntax checks.
- `bash -n scripts/build-macos.sh scripts/check.sh` passes. The host `./scripts/build-macos.sh` completes and produces the arm64 App, ZIP, DMG, and checksum files; the four packaged Skill files are byte-identical to the plugin sources.
- `./scripts/check.sh rust` and `./scripts/check.sh all` were intentionally not run: this change does not modify Rust or theme runtime behavior, and the accepted Plan limits validation to plugin/Web/package integration.

## Behavioral evidence

- Codex 0.150.1 was exercised with temporary `CODEX_HOME=/tmp/doubao-codex-plugin.XLDn17`: local Marketplace add, list, plugin add, list, and remove all succeeded. The installed cache contained only the two manifests and the two Skill directories; no `apps/`, `crates/`, or `themes/` directory was copied. Removal deleted the versioned cache.
- Claude Code 2.1.251 was exercised with temporary `HOME` and npm cache under `/tmp/doubao-claude-plugin.gem3N3`: Marketplace add/list, plugin install/list, uninstall, and Marketplace remove all succeeded. The installed plugin was enabled as `doubao-skin@doubao-skin` version `0.1.0` and its cache was limited to plugin-owned files.
- The checked-in Codex and Claude manifests share name, version, description, author, homepage, repository, license, keywords, and `./skills/`; version `0.1.0` is checked against `[workspace.package]` in `Cargo.toml`.
- A production Next server returned `200` for GET and HEAD on `index.json` and both public `SKILL.md` files. Content types were `application/json` and `text/markdown`; downloaded Skill hashes matched the `sha256:` digests in the Draft 0.2.0 index.
- The guide exposes two-step installation plus update/removal commands for both hosts. The contribution page still states that submissions use GitHub Pull Requests and that the website does not accept direct uploads.

## Visual evidence

- User-connected external Chrome, not the in-app browser, inspected `/guide` at 1440 × 1000 and 390 × 844 in light and emulated dark modes. All four states had `documentElement.scrollWidth === innerWidth`; the two host columns collapse to one 350 px column on the narrow viewport.
- The light desktop, light narrow, dark desktop, and dark narrow screenshots were stored outside the repository and are intentionally not published.
- The first copy button changed from `复制` to `已复制` after activation. The external-browser clipboard API returned an empty string, so exact system clipboard bytes are not claimed as independently read.
- `/contribute` retained its Pull Request/no-direct-upload explanation, and the 390 px home-page smoke retained the theme-library heading with no horizontal overflow.

## Security and privacy evidence

- Both host installation tests used temporary configuration/cache roots and were uninstalled afterward. They did not modify the user's enabled plugin configuration; Codex still displayed the implicitly discovered personal catalog read-only, but only the temporary `CODEX_HOME` received installation state.
- The plugin package contains no MCP server, App, Hook, Agent, executable installer, credentials, conversation content, official app resource, or broad repository subtree.
- Well-Known artifacts are generated from two fixed source paths, preserve raw bytes, use path-absolute public URLs, and are replaced through a temporary sibling directory with rollback on failure. `--check` performs no writes.
- Browser QA used only public local pages, entered no credentials, uploaded nothing, and reset temporary viewport and color-scheme emulation before completion.
- The host package used the existing ad-hoc signing path. No Apple credentials, GitHub writes, public directory submission, Vercel production promotion, or custom-domain change occurred.

## Deviations and residual risk

- `apps/web/src/app/contribute/page.tsx` was an additional live caller of the removed `SKILL_INSTALL_PROMPT` discovered during the accepted path-freeze step. It was updated in the same installation-copy scope so the public creator page does not retain the obsolete `$skill-installer` flow; no contribution behavior changed.
- The host package smoke necessarily performed release compilation even though no Rust tests were run. It replaced the existing generated artifacts under `dist/` with a fresh arm64 App/ZIP/DMG; this is build output, not an application install or release.
- The GitHub Marketplace commands and production Well-Known URL are intentionally preassembled for `IchenDEV/doubao-skin`, but remote installation and `doubao-skin.idevlab.dev` are not claimed until the user pushes the repository and separately authorizes production deployment.
- Agent Skills Discovery remains Draft 0.2.0 rather than an official Codex or Claude installation protocol. The page labels it as a draft and presents each host's Marketplace as the actual installation route.
- The worktree is uncommitted and contains unrelated concurrent changes. Verification is scoped to this change's files; a commit anchor remains required before release preparation.

## Verdict

Fresh-context verdict: **PASS WITH RESIDUALS**. The verifier confirmed the minimal package boundary, manifest parity, one-plugin Marketplace paths, absence of obsolete root Skill references, Draft 0.2.0 URLs/digests, package source path, and public copy. Its read-only sandbox passed `sync-skills.mjs --check` and static JSON/hash invariants but blocked the test suite's temporary-directory tamper fixture with `EPERM`; the same complete 5-test suite passed in the writable implementation context immediately before review. The remaining material residuals are the uncommitted worktree and pending remote GitHub/production publication, both outside this implementation gate.

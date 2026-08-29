---
id: "2026-08-29-rust-theme-skills-cli"
stage: verification
status: pending
owner: "codex"
created: "2026-08-29"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: "Averroes (fresh-context verifier)"
verified_at: "2026-08-29"
---

# Verification: rust theme skills cli

## Automated checks

- The initial authoring/CLI regression run failed before implementation because the authoring module and `doubao-theme` binary did not exist.
- Final `./scripts/check.sh all` passed workflow, formatting, all locked workspace targets, Clippy with warnings denied, and the website gate:
  - 9 desktop UI tests.
  - 31 `skin-core` unit tests.
  - 7 authoring integration tests, 4 CLI integration tests, and 1 bundled-path integration test.
- CLI integration confirms help discovery for all ten stable commands. The isolated success chain covers create → check → preview → pack → install → list; apply/build/remove-build are exercised through pre-side-effect invalid theme or argument paths, and restore uses an unreachable port. Every path asserts its JSON envelope and exit code without touching an installed build. A blocked child path separately proves filesystem creation failure returns code 4.
- Authoring tests cover strict v2 fields, 1200 × 675 preview generation, ID/directory agreement, required resources and CSS scope, both-appearance variants, overwrite refusal, symlink rejection, selective ZIP contents, and the installer's shared 200 MiB compressed-package ceiling.
- Both repository Skills passed `skill-creator/scripts/quick_validate.py`.
- `./scripts/build-macos.sh --universal` produced `dist/Doubao-Skin-macOS-universal.zip` with SHA-256 `95453f379a9e2e99404be226762248280bb2ee1d6262e51354ab5f71fb3e1c03`.
- `lipo -info` reports `x86_64 arm64` for both the GUI and packaged `Contents/Resources/bin/doubao-theme`; `codesign --verify --deep --strict` and `unzip -t` pass.
- The packaged App contains both `SKILL.md` and `agents/openai.yaml` for `create-doubao-theme` and `apply-doubao-theme`.

## Behavioral evidence

- `doubao-theme` implements `list/create/check/preview/pack/install/apply/restore/build/remove-build`, global `--json`, target selection, and watch mode without adding a second theme loader or installer.
- Text-mode `list` prints readable theme names and IDs. JSON mode emits exactly one `{ok, command, result|error}` object while operation logs remain on stderr.
- ID/path resolution checks an explicit directory, user-installed themes, and bundled themes. Packaged CLI lookup finds themes from `Contents/Resources/bin/` as well as the GUI executable location.
- `apply` and `restore` require at least one responsive selected-app page. A closed port or zero cleaned pages exits as an external-operation failure rather than reporting a false success.
- `restore_js()` delegates root-attribute restoration to the recorded runtime state and no longer unconditionally removes `data-skin` or `data-skin-target` after `destroy()`; regressions assert both capture/restore and the absence of destructive duplicate cleanup.
- `pack` checks the actual finished ZIP size against the same 200 MiB constant used by `install_theme_package`, and removes an output that exceeds the installer contract.
- An isolated Skill forward test used the packaged universal CLI and temporary directories to create `松影专注`, check it, regenerate its preview, pack it, install it, and list it. The final catalog contained exactly that one temporary theme; no real user theme directory was written.
- The create Skill mandates `check → preview → check → pack` for every completed creation and constrains unclear artwork to licensed user-supplied material or a generated color/gradient fallback. The apply Skill allows read-only list/check directly and requires an exact target/effect plus explicit approval before install/apply/restore/build/remove-build.

## Visual evidence

- The forward-test `preview.jpg` was inspected at its original 1200 × 675 resolution. It rendered a dark two-column theme mock with the requested low-saturation green accent, readable surfaces, and no explanatory product copy.
- The universal App package itself was not relaunched for this CLI-only text correction because the GUI binary did not change; architecture, signature, archive integrity, CLI output, and embedded Skills were rechecked from the final archive.

## Security and privacy evidence

- Authoring rejects symlinks and path escape, packages only manifest/CSS, referenced assets, icon, and license files, and reuses the existing bounded installer for extraction and verification.
- Live operations remain target-specific and loopback-only. Restore removes only runtime markers/elements owned by this tool, preserves pre-existing root attributes through the runtime snapshot, and fails when it cannot observe actual cleanup.
- CLI integration overrides both CDP ports to an unreachable test port and uses temporary bundled/user theme roots, preventing tests from touching the real clients or user theme library.
- The two Skills explicitly pause before side-effecting commands and do not claim a public native `SKILL.md` import protocol for DoubaoWork.

## Deviations and residual risk

- During an early CLI integration run, before the test-only CDP port overrides were added, a restore test reached the already-open real DoubaoWork loopback port and executed this tool's scoped restore script. It may have reset the active theme but did not read conversation content or delete user files; this was disclosed immediately. All final tests use isolated unreachable ports.
- While adding final `apply` command coverage, an initially valid-theme test revealed that an unreachable debug port still enters the client startup path. That one run may have launched or restarted DoubaoWork, but it observed no responsive page, read no page content, and changed no official App files. The final regression now fails at theme resolution before Live code, and no further real-client apply test was run.
- A complete isolated real-window apply → computed-style → restore acceptance was not rerun because no private-content-free DoubaoWork window was available. Automated ownership/zero-page regressions, existing live tests, and package checks pass, but real Live acceptance remains pending under the Plan's stated fallback.
- The generated universal archive is ad-hoc signed and not notarized. Notarization credentials and release publication were outside this change.
- The worktree is uncommitted and contains unrelated concurrent changes. A commit anchor and fresh-context verdict are still required before marking this artifact passed.

## Verdict

Fresh-context verdict: **PASS WITH RESIDUALS**. Authoring, all ten CLI command contracts, both Skills, and universal package integration satisfy the accepted Spec. Frontmatter remains pending because the accepted Plan explicitly leaves isolated real-window Live apply/restore, commit anchoring, remote Skill installation, and release/notarization as outstanding external or product gates.

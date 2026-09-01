---
id: "2026-09-01-release-v0-5-0"
stage: plan
status: accepted
owner: "codex"
created: "2026-09-01"
based_on: spec.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-09-01"
---

# Plan: release v0 5 0

## Files and ownership

- `Cargo.toml` and generated `Cargo.lock`: set the Rust workspace and inherited package versions to `0.5.0` without changing dependency selections.
- `apps/web/package.json`, `plugins/doubao-skin/.codex-plugin/plugin.json`, and `plugins/doubao-skin/.claude-plugin/plugin.json`: synchronize the three release-validated manifest versions.
- `CHANGELOG.md`: present the never-published `0.4.0` content as `0.5.0` and add the merged remembered-theme/Windows no-console behavior.
- `workflow/changes/2026-09-01-release-v0-5-0/`: preserve the accepted intent/spec/plan and record pre-release plus production evidence in `verification.md`.
- No ownership is assigned to application behavior, themes, dependencies, packaging scripts, or release workflow files unless a failing required gate proves a narrowly scoped release blocker.

## Order of work

1. Reconfirm that PR #16 is merged, `origin/main` is current, `v0.5.0` does not exist locally or remotely, the latest public Release is `v0.3.2`, and the existing version authorities report `0.4.0`.
2. Run a local form of the release tag/version consistency check with expected version `0.5.0` and record that the untouched checkout fails because the manifests still report `0.4.0`.
3. Change only the workspace/Web/plugin versions and changelog. Regenerate local workspace entries in `Cargo.lock`, then inspect the diff to reject unrelated dependency or generated-file churn.
4. Create `verification.md`; run exact version-consistency assertions, workflow validation, release YAML/shell syntax checks, `git diff --check`, targeted package/version checks, and `./scripts/check.sh all`.
5. Commit and push `codex/release-v0-5-0`, then open a Draft release PR linked to this artifact. Wait for every PR check and record the immutable head SHA.
6. Present the pre-release evidence to the human reviewer. The human explicitly decides whether to merge the Draft release PR despite production-only evidence still being unavailable; Codex does not merge it or mark production verified on its own.
7. After merge, fetch `origin/main` again, verify the merge contains only the approved release-prep change plus any explicitly accepted concurrent commits, confirm `main` CI is green, and rerun all version assertions against the exact tag target SHA.
8. Create and push lightweight tag `v0.5.0` at that exact `origin/main` SHA. Wait for the protected GitHub `production` environment and ask the configured human reviewer to approve it; do not rerun, retag, or bypass the environment while approval is pending.
9. After approval, wait for every release job and GitHub Release publication. Download the full asset set into a fresh temporary directory; verify sidecar checksums, macOS ZIP/DMG signatures and architectures, Windows desktop/CLI archive structure, version output, Scoop manifest, and stable latest-download URLs.
10. Perform native smoke checks: launch the released macOS app in a real window and use the Windows VM/VNC to verify the released ARM64 package, `doubao-skin --version`, Scoop/PATH behavior where supported, and absence of recurring terminal windows. Shut down the VM and remove temporary VNC configuration afterward.
11. Append production evidence and the final human/fresh-context verdict to `verification.md` in a documentation-only follow-up. Keep the published tag immutable and report any incomplete job or asset as a failed/partial release rather than success.

## Test-first proof

- Before editing, run an expected-`0.5.0` consistency probe over the workspace, Web package, and both plugin manifests; it must fail on the current `0.4.0` state.
- After editing, the identical probe must pass, `cargo metadata --locked` must resolve workspace packages as `0.5.0`, and no third-party package version in `Cargo.lock` may change.
- Run the repository workflow test and release-tag validation logic locally before the full gate. A deliberately mismatched tag/version input must still fail closed.
- Run `./scripts/check.sh all` once the version/changelog direction is settled. Rerun only checks affected by any subsequent correction.

## Visual or integration proof

- The release-prep metadata change has no new interface design and therefore requires no layout review.
- Launch the locally packaged or final released macOS app and confirm a real native window opens, the bundle version is `0.5.0`, and the stable certificate fingerprint remains unchanged.
- In the Windows VM, extract the final ARM64 desktop archive into a fresh directory, launch it through Explorer, confirm the existing parent-on/child-off theme configuration remains intelligible, and passively observe that no terminal window recurs.
- Verify the final GitHub Release page, latest-download targets, complete asset inventory, and Scoop manifest after publication; CI artifacts alone are not public-release evidence.

## Risks and mitigations

- **Stale tag target:** fetch and compare `origin/main`, PR merge SHA, manifest contents, and main CI immediately before tagging; abort before tag creation on any mismatch.
- **Version drift:** use one exact consistency probe before and after editing and in the tag workflow; reject unrelated Cargo lock changes.
- **Signing secret or identity failure:** keep the existing fail-closed certificate/fingerprint checks and stop at the production job; never print, replace, or weaken secrets to make the release pass.
- **Human production gate:** surface the pending environment review and wait. The user's plan approval and tag request do not substitute for GitHub's configured production reviewer action.
- **Partial release:** require every configured job and asset before declaring success. Do not manually upload a subset to mask a failed workflow.
- **Self-signed macOS package:** verify identity continuity but retain the explicit non-notarized/right-click-open warning.
- **Concurrent `main` changes:** include them only if already reviewed and green; if they materially expand the release beyond the accepted spec, revise the artifact before tagging.

## Rollback

- Before pushing `v0.5.0`, close the Draft PR or revert the release-prep commit through a normal PR if the version is cancelled; no public release state exists yet.
- After pushing the tag but before publication, do not delete or move it automatically. Stop the workflow and request explicit human authorization for any tag deletion/retry because it rewrites public release state.
- After a successful GitHub Release, never retag or replace the immutable release to repair a defect. Publish a new patch version with its own accepted change and changelog entry.
- Temporary downloaded artifacts and VNC configuration are disposable; preserve the guest installation and user settings unless the user explicitly requests cleanup.

## Deviations

None planned. The existing release workflow and stable signing system are reused without modification.

## Decision

Pending explicit human acceptance.

---
id: "2026-09-01-release-v0-5-0"
stage: verification
status: pending
owner: "codex"
created: "2026-09-01"
based_on: plan.md
commit: ""
verification_mode: "human"
verified_by: ""
verified_at: ""
---

# Verification: release v0 5 0

## Automated checks

- Before editing, the expected-`0.5.0` consistency probe failed with status 1 and reported `0.4.0` for `Cargo.toml`, the Web package, and both plugin manifests. This established the red precondition for the release metadata change.
- After editing, the identical probe passed with `0.5.0` in all four authorities. A deliberate expected-`0.5.1` mismatch failed closed.
- `cargo metadata --locked --format-version 1 --no-deps` resolved `skin-core 0.5.0` and `doubao-skin-desktop 0.5.0`.
- The `Cargo.lock` diff contains only the old/new version pair for those two local workspace packages. No package name, source, checksum, dependency list, or third-party version changed.
- Ruby's standard YAML parser accepted `.github/workflows/release.yml`; no release workflow or packaging script was modified.
- `./scripts/check.sh workflow` passed: 22 artifact sets validated, devflow policy cases passed, and portability boundaries passed.
- `./scripts/check.sh all` passed after the version settled. Rust covered desktop 31/31, core 70/70, authoring 9/9, bundled paths 4/4, CLI 4/4, theme package v3 12/12 and schema 2/2; formatting, Clippy and builds passed. Web covered 16/16 tests, skill synchronization, TypeScript, Next.js static generation and the high-severity audit with no known vulnerabilities.
- `git diff --check` passed.

## Behavioral evidence

- The changelog now presents the never-published `0.4.0` content under `0.5.0` dated 2026-09-01 and adds the merged remembered-theme, Windows no-console, startup cleanup and tray-reopen behavior.
- The version change is limited to workspace/Web/plugin metadata, the two generated local Cargo lock entries, changelog text and this accepted change artifact. No application logic, theme, dependency, packaging workflow or signing configuration changed.
- PR #16 is confirmed merged at `ce527264353a3dcde6d3481732c5681667a7e639`; local tag and GitHub Release inventories showed no `v0.4.0` or `v0.5.0`, while the latest published release remained `v0.3.2` during preflight.

## Visual evidence

- No new interface or layout was introduced, so release-prep metadata has no design acceptance surface.
- Real-window evidence for the final `0.5.0` macOS and Windows release packages remains pending until tag-triggered artifacts exist. CI artifacts or prior `0.4.0`-metadata builds are not treated as final release evidence.

## Security and privacy evidence

- No signing secret, certificate material, credential, conversation content, official app resource or local user path was added or printed.
- The existing protected `production` environment, stable community certificate fingerprint checks, strict signature verification and checksum gates remain unchanged.
- Community self-signing is still documented as non-notarized and does not claim Apple trust.

## Deviations and residual risk

- Actions run `33493201113` was a process-gate failure, not a product-build failure: PR #16 was made Ready while its linked verification was pending. Human reviewer `idevlab` later accepted the existing VNC evidence and residual risks; the fix is isolated in PR #18 rather than weakening `devflow` or mixing that artifact update into the version commit.
- GitHub `main` CI, the version PR, tag creation, production-environment approval, release jobs, public asset inventory, fresh downloads, macOS signature checks and Windows/Scoop native smoke checks remain pending.
- Skipping public tag `v0.4.0` is intentional because no such Release exists and the user explicitly selected `v0.5.0`.

## Verdict

Pending. Local release metadata and full repository gates pass, with no third-party lockfile drift or release-workflow change. A final verdict requires the focused version PR to merge, `v0.5.0` to point at the verified `main` SHA, the human production gate and all release jobs to pass, public assets to be downloaded and verified, and final macOS/Windows smoke evidence to be recorded.

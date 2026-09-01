---
id: "2026-09-01-release-v0-5-0"
stage: spec
status: accepted
owner: "codex"
created: "2026-09-01"
based_on: intent.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-09-01"
---

# Spec: release v0 5 0

## Requirements

- Set the workspace version to `0.5.0` in `Cargo.toml` and regenerate `Cargo.lock` so every workspace package entry is consistent.
- Set `apps/web/package.json`, `plugins/doubao-skin/.codex-plugin/plugin.json`, and `plugins/doubao-skin/.claude-plugin/plugin.json` to `0.5.0`.
- Replace the never-published `0.4.0` changelog heading with `0.5.0` and include the user-visible changes delivered since `v0.3.2`, including remembered-theme behavior on Windows and suppression of background terminal windows.
- Keep the release commit limited to version metadata, changelog text, and the linked workflow artifacts; do not add product behavior to the release bump.
- Merge the focused release PR only after the regular CI and workflow-policy checks pass.
- Create `v0.5.0` from the exact clean `origin/main` release commit. The existing `Release` workflow must validate the tag and build every configured desktop, CLI, checksum, and Scoop artifact.
- Publication through the protected `production` environment requires the configured human reviewer; no automation or alternate workflow may bypass that gate.

## User experience

GitHub Releases marks `v0.5.0` as latest. Existing stable download links resolve to the new packages. macOS users retain the documented community self-signed first-open experience, Windows users receive x64/x86/ARM64 desktop archives, and CLI users receive native archives plus the generated Scoop manifest. Older releases and tags remain immutable.

## Technical design

- Reuse the existing workspace-version inheritance and tag/version validation rather than introducing a second version source.
- Use the existing `Release` workflow unchanged unless verification exposes a release-blocking defect. The workflow already imports the stable signing identity, tests the workspace, packages each platform, verifies signatures/checksums, and publishes only after all required jobs succeed.
- Treat `v0.5.0` as the first published version after `v0.3.2`; no `v0.4.0` tag or compatibility shim is created.
- Resolve `origin/main` again after the version PR is merged, verify that all manifests at that SHA report `0.5.0`, and tag that SHA rather than a stale local branch.
- After publication, download the released metadata and representative archives into a fresh temporary directory and verify checksums, package structure, embedded version, and macOS signature continuity. Use the Windows release jobs and a clean Windows/Scoop smoke test for native Windows evidence.

## Security and privacy

- Never print, persist in repository files, or replace the signing certificate, private key, certificate password, temporary keychain password, or GitHub environment credentials.
- Continue to fail closed on missing or mismatched signing secrets, certificate fingerprint, identity, checksum, tag, manifest version, or package contents.
- Keep the protocol bridge bound to loopback and do not include application data, official application resources, credentials, or conversations in release assets or evidence.
- Community self-signing must not be described as Apple Developer ID signing or notarization.

## Alternatives and non-goals

- Creating an unpublished `v0.4.0` first is unnecessary; the user explicitly selected `v0.5.0`, and no public `v0.4.0` compatibility contract exists.
- A new release service, updater, packaging format, dependency upgrade, certificate rotation, notarization flow, or manual asset upload path is out of scope.
- Rewriting a published tag or replacing assets after a successful immutable release is not permitted; defects require a later patch release.

## Areas of concern

- The current source version is `0.4.0` even though no matching tag or Release exists, so all version authorities and changelog presentation must be updated together.
- Another merge into `main` between PR verification and tagging could make evidence stale; the tag target and CI state must be re-resolved immediately before tag creation.
- The GitHub release can pause at the protected `production` environment. That wait is expected and must be surfaced to the human reviewer rather than bypassed.
- The stable community certificate is not Apple-trusted, so a valid signature does not remove the macOS first-open warning.

## Acceptance criteria

- `./scripts/check.sh all`, `./scripts/devflow validate`, release YAML/shell validation, and `git diff --check` pass on the release branch; the merged `main` CI is green.
- The release commit contains exactly `0.5.0` in the workspace, Web, Codex plugin, Claude plugin, and generated Cargo lock entries, and `v0.5.0` resolves to that exact merged commit.
- The `Validate release tag`, macOS, Windows x64/x86/ARM64, CLI, Scoop, and publish jobs succeed after human production approval.
- GitHub Release `v0.5.0` is latest and exposes every asset produced by the workflow with matching checksum sidecars; no incomplete draft or partial asset set remains.
- The macOS app/ZIP/DMG and universal CLI retain the pinned community certificate fingerprint and expected architectures; archive structure checks pass.
- Windows desktop archives contain one top-level `doubao-skin.exe` plus the helper, Windows CLI archives remain CLI-only, and a clean Scoop install reports `doubao-skin 0.5.0`.
- The release verification artifact records the tag SHA, workflow URL, checks, asset inventory, checksums, signature evidence, Windows smoke evidence, deviations, and residual self-signing risk without secrets.

## Decision

Pending explicit human acceptance.

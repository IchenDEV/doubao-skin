---
id: "2026-09-01-release-v0-5-0"
stage: intent
status: accepted
owner: "codex"
created: "2026-09-01"
source: "user"
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-09-01"
---

# Intent: release v0 5 0

## Problem

The Windows auto-theme fix is merged into `main`, while the latest published release is still `v0.3.2` and the source manifests remain at `0.4.0`. Users cannot download the merged Windows behavior from a versioned release, and pushing `v0.5.0` without first synchronizing every authoritative manifest would fail the repository's release gate.

## Proposed outcome

Publish Doubao Skin `v0.5.0` from the exact reviewed `main` commit. Keep the change limited to synchronized version metadata, user-facing release notes, release evidence, and the existing tag-driven multi-platform packaging workflow.

## Affected users and systems

Doubao Skin users on macOS, Windows, and the CLI platforms; repository maintainers; the GitHub Actions `Release` workflow; the protected GitHub `production` environment; GitHub Releases; and Scoop users.

## Constraints

- `Cargo.toml`, `Cargo.lock`, the Web package manifest, and both plugin manifests must all report `0.5.0` before tagging.
- The tag must be exactly `v0.5.0` and point to the clean, CI-passing `main` release commit.
- Reuse the existing stable community signing identity; do not rotate credentials, expose secrets, weaken checksum/signature validation, or claim Apple notarization.
- Preserve the protected `production` environment. Codex may prepare and push the tag after the version PR is merged, but a human reviewer must authorize the production job.
- Publication is complete only when all desktop, CLI, checksum, Scoop, and GitHub Release jobs pass and the published assets are verified.

## Out of scope

- Product feature changes, redesigns, theme migrations, dependency upgrades, or unrelated cleanup.
- Apple Developer ID signing, notarization, App Store distribution, certificate rotation, or bypassing GitHub environment protection.
- Retagging or rewriting an already published release.

## Success signals

- A focused version PR is merged with all repository checks passing.
- Tag `v0.5.0` resolves to the approved `main` release commit and the tag/version validation job passes.
- The release workflow publishes macOS universal, Windows x64/x86/ARM64, CLI, checksum, and Scoop assets without signature or checksum failures.
- Freshly downloaded assets match their published checksums and report `0.5.0` where a version command is available.
- GitHub marks `v0.5.0` as the latest release and the stable latest-download URLs resolve to its assets.

## Open questions

None. The user explicitly selected `0.5.0`; skipping an unpublished `0.4.0` tag is intentional and does not require a compatibility migration.

## Decision

Pending explicit human acceptance.

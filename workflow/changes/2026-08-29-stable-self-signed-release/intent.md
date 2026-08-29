---
id: "2026-08-29-stable-self-signed-release"
stage: intent
status: accepted
owner: "codex"
created: "2026-08-29"
source: "user"
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Intent: stable self signed release

## Problem

The public repository has no GitHub Release, so the website's universal ZIP download target is correct but unavailable. Current release automation falls back to an ad-hoc signature when no Apple Developer certificate is configured; ad-hoc signatures do not provide a stable signing identity across versions.

## Proposed outcome

Publish `v0.1.0` with universal ZIP and DMG assets signed by one long-lived self-signed code-signing certificate. Store the encrypted certificate and its passwords only in GitHub Actions Secrets, pin its fingerprint in the release job, and reuse that exact identity for later versions.

## Affected users and systems

macOS users downloading Doubao Skin, repository maintainers, the GitHub Actions `Release macOS app` workflow, the GitHub `production` environment, and the website download/health checks.

## Constraints

- Never commit or print the private key, certificate password, or temporary keychain password.
- The release job must fail closed when the expected certificate, identity, or fingerprint is missing or different.
- Each release must verify the signed app before packaging and verify the same authority/fingerprint after extracting the final ZIP and mounting the final DMG.
- Keep the existing universal Apple Silicon + Intel packages and fixed asset names.
- A self-signed certificate is not Apple notarization: Gatekeeper may still require right-click Open, and documentation must say so plainly.
- Use the existing `production` environment gate; this intent does not weaken or bypass human release protection.

## Out of scope

- Apple Developer ID enrollment, Apple notarization, App Store distribution, automatic updates, or suppressing Gatekeeper warnings.
- Rotating the signing identity on every release.
- Committing a private key, `.p12`, password, or decoded certificate to Git.

## Success signals

- GitHub Actions imports one stable self-signed certificate and verifies its expected SHA-256 fingerprint before building.
- The ZIP and DMG contain a universal app whose code signature verifies deeply and strictly and reports the expected signing authority.
- GitHub Release `v0.1.0` exposes the ZIP, DMG, and both SHA-256 files at the website's fixed download URLs.
- The tag, release commit, CI result, signing fingerprint, checksums, and downloadable assets are recorded without exposing secrets.
- A second workflow invocation would use the same stored certificate rather than generate a new identity.

## Open questions

None. The requested version is the repository's current workspace version, `0.1.0`.

## Decision

Pending explicit human acceptance.

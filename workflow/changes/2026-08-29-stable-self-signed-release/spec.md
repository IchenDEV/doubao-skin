---
id: "2026-08-29-stable-self-signed-release"
stage: spec
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: intent.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Spec: stable self signed release

## Requirements

- Create one RSA 3072-bit, SHA-256, self-signed X.509 code-signing certificate named `Doubao Skin Community Release`, valid for 20 years.
- Reuse the same certificate and private key for `v0.1.0` and subsequent releases until an explicitly documented rotation.
- Store the encrypted PKCS#12 bundle and passwords in GitHub Actions Secrets; retain one encrypted recovery copy outside the repository with mode `0600`, with its password stored in the user's macOS Keychain.
- Add the expected certificate SHA-256 fingerprint as a GitHub secret and fail the release before compilation if the imported certificate differs.
- Require stable signing for tag and manual release-workflow runs; do not silently fall back to ad-hoc signing in the release workflow.
- Publish `Doubao-Skin-macOS-universal.zip`, `Doubao-Skin-macOS-universal.dmg`, and both `.sha256` files in GitHub Release `v0.1.0`.

## User experience

The website's macOS download button resolves to the new GitHub Release ZIP. Users may still see macOS's unidentified-developer warning because the certificate is community self-signed, not Apple-issued or notarized. The Chinese and English README/release documentation must explain right-click Open without describing the package as Apple-notarized.

## Technical design

- Generate the certificate once with explicit code-signing extended key usage and a stable common name. Export it as a password-protected `.p12`.
- Configure secrets `MACOS_CERTIFICATE`, `MACOS_CERTIFICATE_PASSWORD`, `MACOS_CERTIFICATE_SHA256`, `MACOS_SIGNING_IDENTITY`, and `KEYCHAIN_PASSWORD`. `MACOS_SIGNING_IDENTITY` is the certificate's SHA-1 identity accepted by `codesign`, not a mutable display-name lookup.
- The release job decodes the `.p12`, extracts its public certificate, normalizes and compares the SHA-256 fingerprint, imports it into an ephemeral keychain, marks the self-signed certificate trusted only in that temporary keychain, and confirms the SHA-1 signing identity is available.
- `scripts/build-macos.sh` continues to sign the complete app before ZIP/DMG creation and performs `codesign --verify --deep --strict`.
- Add a small verification script that extracts the signing certificate embedded in an app's code signature and compares its SHA-256 fingerprint with the expected value. Use it on the built app, the app extracted from the ZIP, and the app mounted from the DMG.
- The release workflow verifies both SHA-256 sidecar files before uploading, then creates the tag-matching GitHub Release with a self-signing/Gatekeeper notice.
- Version `v0.1.0` must point at a clean, CI-passing `main` commit whose Cargo workspace version is `0.1.0`.

## Security and privacy

- The private key, decoded certificate, passwords, and temporary keychain are never committed or printed.
- Temporary certificate/keychain material is created under the runner's temporary directory and removed in an `always()` cleanup step.
- The recovery `.p12` is encrypted, stored outside the repository, and permissioned `0600`; its password is stored in macOS Keychain rather than source files or chat.
- Certificate fingerprints and public certificate metadata are non-secret and may be recorded for auditability.
- A fingerprint mismatch, missing secret, missing identity, failed deep signature check, failed archive checksum, or tag/version mismatch stops publication.

## Alternatives and non-goals

- Ad-hoc signing was rejected because its identity is intentionally non-stable.
- Generating a new self-signed certificate in every workflow run was rejected because every version would have a different identity.
- Apple Developer ID and notarization are not part of this request and require Apple-issued credentials.
- The workflow will not auto-rotate an expired or compromised certificate; rotation requires a separately approved change and visible documentation.

## Areas of concern

- Self-signed certificates do not establish Apple trust or notarization and may still be blocked on first open.
- A 20-year certificate increases the importance of protecting and revoking the private key operationally; if compromise is suspected, stable identity must yield to an explicit emergency rotation.
- GitHub Secrets cannot be downloaded later, so the encrypted recovery copy and Keychain password entry are necessary for disaster recovery.
- The repository's `production` environment may pause the workflow for a human reviewer after the tag is pushed.

## Acceptance criteria

- Workflow and shell syntax checks pass; the regular CI remains green.
- A local signing rehearsal verifies the exact certificate fingerprint on a packaged app without exposing secrets.
- GitHub's release job logs show the expected fingerprint/identity checks, universal build, ZIP/DMG checksum checks, and three app-signature checks passing.
- `v0.1.0` is published from the approved `main` SHA with four expected assets.
- Fresh downloads match both sidecar checksums; ZIP extraction and read-only DMG mounting each reveal an app signed by `Doubao Skin Community Release` with the recorded SHA-256 certificate fingerprint.
- `https://github.com/IchenDEV/doubao-skin/releases/latest/download/Doubao-Skin-macOS-universal.zip` returns the released asset.

## Decision

Pending explicit human acceptance.

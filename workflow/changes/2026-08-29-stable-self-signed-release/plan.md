---
id: "2026-08-29-stable-self-signed-release"
stage: plan
status: accepted
owner: "codex"
created: "2026-08-29"
based_on: spec.md
risk: "medium"
approved_by: "idevlab"
approved_at: "2026-08-29"
---

# Plan: stable self signed release

## Files and ownership

- `.github/workflows/release.yml`: require/import the stable certificate, validate identity/fingerprint, verify packaged signatures/checksums, publish release notes, and clean temporary keychain material.
- `scripts/verify-macos-signature.sh`: verify a `.app` deeply and compare the embedded signing certificate's normalized SHA-256 fingerprint.
- `scripts/build-macos.sh`: only the minimum changes needed to expose/verify stable signing metadata; retain existing build/package behavior.
- `docs/releasing.md`, `README.md`, `README.en.md`, `CHANGELOG.md`: document community self-signing, first-open behavior, certificate continuity, and `v0.1.0`.
- `workflow/changes/2026-08-29-stable-self-signed-release/verification.md`: record commands, public fingerprint, GitHub run/tag/release, asset checksums, downloaded-package signature evidence, and residual Gatekeeper risk.
- Outside the repository: one encrypted recovery `.p12` under the user's Application Support directory, one macOS Keychain password entry, GitHub Actions Secrets, and ephemeral local/CI keychains.

## Order of work

1. Reconfirm clean `main`, workspace version `0.1.0`, successful current CI, absence of tag/release `v0.1.0`, and existing website asset names.
2. Implement the signature-verification helper and fail-closed release workflow. Update release and first-open documentation plus the changelog.
3. Run workflow validation, shell syntax checks, formatting, targeted signature-helper error cases, and the repository's smallest relevant checks.
4. Generate the long-lived certificate once in a private temporary directory. Record only public metadata/fingerprints; export an encrypted recovery `.p12`, save its password in macOS Keychain, and remove all unencrypted temporary material.
5. Import the recovery certificate into an ephemeral local keychain and perform a universal `v0.1.0` build. Verify the source app, ZIP app, DMG app, architectures, checksums, CLI, and bundled Skills with the same expected fingerprint.
6. Upload the encrypted certificate and required values to GitHub Actions Secrets without printing them. Confirm secret names exist, not their values.
7. Commit and push the accepted implementation to `main`; wait for CI and the automatic Vercel documentation deployment to pass.
8. Re-resolve the exact clean `origin/main` SHA, create signed-release tag `v0.1.0`, and push it. Wait for any GitHub `production` environment reviewer gate and never bypass it.
9. Wait for the release workflow and Release publication. Download all four assets into a fresh temporary directory, validate sidecar checksums, ZIP/DMG contents, universal architectures, CLI/Skills, and the embedded certificate fingerprint.
10. Confirm the fixed website download URL returns the released ZIP; write verification evidence and push only if doing so does not mutate the already published tag. Keep the immutable release anchored to the recorded SHA.

## Test-first proof

- Before publication, invoke the verification helper with missing arguments, a malformed fingerprint, and an intentionally wrong valid-format fingerprint; all must fail without leaking certificate material.
- Run the helper against a correctly signed local app and require success, then against the same app with a wrong fingerprint and require failure.
- Run the exact protocol-bridge regression repeatedly only if release-related edits touch shared Rust/build behavior; otherwise rely on the already green `main` CI and package-focused checks.
- `./scripts/check.sh workflow`, `sh -n` on changed shell scripts, `git diff --check`, and the full applicable repository gate must pass before tagging.

## Visual or integration proof

- Inspect the locally built application window only if build/signing changes affect the bundle or launch behavior; signing-only workflow edits do not require a new visual design review.
- Launch the signed local app once and confirm a real window appears without signature corruption.
- Mount the local and downloaded DMGs read-only and inspect their top-level contents: `豆包主题.app` and `Applications` symlink.
- Use public HTTP checks to prove the GitHub Release page, ZIP, DMG, checksum assets, and website download URL are reachable.

## Risks and mitigations

- **Private-key exposure:** generate in a `0700` temporary directory, never echo secrets, pipe secret values directly into GitHub CLI, keep only encrypted recovery material, and delete temporary key/cert files.
- **Certificate unavailable to `codesign`:** import into a dedicated keychain, add only that keychain to the job search list, set partition access, trust the self-signed certificate in that keychain, and select by SHA-1 identity.
- **Identity drift:** compare SHA-256 fingerprint before build and after extracting each final package; fail closed on any mismatch.
- **Self-signed Gatekeeper rejection:** never claim notarization; provide right-click Open instructions and validate `codesign`, not `spctl`, as the self-signing acceptance boundary.
- **Irrecoverable GitHub Secret:** retain encrypted recovery `.p12` plus Keychain-stored password outside the repository.
- **Environment approval pause:** report the pending GitHub production gate and wait for the human reviewer; do not recreate tags or workflows to bypass it.
- **Partial release:** verify all four assets and checksums before declaring completion; delete only an incomplete draft release/tag if publication failed and recovery requires a clean retry.

## Rollback

- Before the tag is pushed, revert the implementation commit and remove the new GitHub Secrets if the approach is rejected.
- After the tag is pushed but before a Release is published, stop the workflow and remove the remote/local tag only if the user explicitly authorizes rewriting that unpublished release attempt.
- After publication, do not replace the immutable `v0.1.0` tag. Correct serious defects with a new patch version; mark a compromised release and rotate the certificate through a separately approved emergency change.
- Removing GitHub Secrets disables future signed releases but does not invalidate already signed binaries.

## Deviations

None planned.

## Decision

Pending explicit human acceptance.

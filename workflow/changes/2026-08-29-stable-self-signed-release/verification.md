---
id: "2026-08-29-stable-self-signed-release"
stage: verification
status: pending
owner: "codex"
created: "2026-08-29"
based_on: plan.md
verification_mode: human
---

# Verification: stable self-signed release

## Automated checks

- `./scripts/check.sh workflow`: passed.
- `sh -n scripts/verify-macos-signature.sh`: passed.
- `sh -n scripts/build-macos.sh`: passed.
- `git diff --check`: no whitespace errors.
- All inline shell blocks in `release.yml` pass `sh -n`.
- YAML validation: `yaml.safe_load()` accepts `release.yml`.
- CI run [33247782493](https://github.com/IchenDEV/doubao-skin/actions/runs/33247782493): success.
- Release workflow [33251986190](https://github.com/IchenDEV/doubao-skin/actions/runs/33251986190): success (10m54s).
- Verification script error cases: missing args (exit 2), nonexistent app (exit 1), non-hex fingerprint (exit 2), short fingerprint (exit 2), wrong fingerprint (exit 1), correct fingerprint (exit 0).

## Behavioral evidence

- Tag `v0.1.0` points to commit `fa45de999f116924b844e182923996e4e6537741` on `main`.
- GitHub Release [v0.1.0](https://github.com/IchenDEV/doubao-skin/releases/tag/v0.1.0) published with four assets: ZIP, DMG, and two `.sha256` files.
- `shasum -a 256 -c` passed for both sidecar checksums on downloaded assets.
- ZIP extracted app: `codesign --verify --deep --strict` passed, certificate SHA-256 `6EF66DA353E5593DC972FC399DBE3594C1D0D3F0B5BFC8BBBFC5629E2656AD35` matched.
- DMG: `hdiutil verify` passed, mounted read-only, contains `豆包主题.app` and `Applications -> /Applications`, app signature verified with same fingerprint.
- Universal architectures: x86_64 + arm64 for both GUI and CLI binaries.
- `doubao-theme --help` runs from `Contents/Resources/bin/`.
- Bundled Skills `create-doubao-theme/SKILL.md` and `apply-doubao-theme/SKILL.md` present.
- Website download URL `releases/latest/download/Doubao-Skin-macOS-universal.zip` returns 302 to v0.1.0 asset.

Certificate metadata (public):

| Field | Value |
| --- | --- |
| Common Name | Doubao Skin Community Release |
| Algorithm | RSA 3072-bit, SHA-256, self-signed |
| Validity | 2026-08-29 to 2046-08-29 (20 years) |
| SHA-256 fingerprint | `6EF66DA353E5593DC972FC399DBE3594C1D0D3F0B5BFC8BBBFC5629E2656AD35` |
| SHA-1 signing identity | `0477AC79E4FF2E6768C7DBED72512EA6A1534337` |

Asset checksums:

```text
9bed9f2c2579d5c7394cbb20a788c4f9d8dd4fc51eba1fd496e5db7d454e837d  Doubao-Skin-macOS-universal.zip
661b4e206f71999c678030e0944d8e6ccb3a2759a4b2d3bda02dc5982cc61b4d  Doubao-Skin-macOS-universal.dmg
```

Five GitHub Actions Secrets confirmed present (names only): `MACOS_CERTIFICATE`, `MACOS_CERTIFICATE_PASSWORD`, `MACOS_CERTIFICATE_SHA256`, `MACOS_SIGNING_IDENTITY`, `KEYCHAIN_PASSWORD`. Encrypted recovery `.p12` at `~/Library/Application Support/Doubao Skin/recovery/` (mode 0600); password in macOS Keychain.

## Visual evidence

- Local signed app launched successfully; real window appeared without signature corruption.
- DMG mounted read-only and inspected: top-level contains `豆包主题.app` and `Applications` symlink.

## Security and privacy evidence

- Private key generated in a `0700` temporary directory and deleted immediately after `.p12` export.
- Secrets uploaded to GitHub Actions via `gh secret set` piped from values; never echoed or printed.
- Encrypted recovery `.p12` stored at mode `0600`; password stored in macOS Keychain, not in source files.
- Certificate fingerprints and public metadata are non-secret and recorded for auditability.
- Workflow fails closed on missing secret, fingerprint mismatch, or unavailable identity.
- Temporary keychain and signing material removed in an `always()` cleanup step.

## Deviations and residual risk

Deviations:

- `security add-trusted-cert -d` hangs on headless CI runners (admin authorization prompt). Removed entirely; `find-identity` without `-v` discovers the imported identity, and `codesign` works with untrusted self-signed certificates.
- Tag `v0.1.0` was recreated three times due to CI signing fixes; the published tag points to commit `fa45de9`.

Residual risk:

- Self-signed certificate: macOS Gatekeeper may still block first launch. Users must right-click and choose "Open". Documented in README, release notes, and `docs/releasing.md`.
- The flaky `protocol_bridge::tests::converts_a_full_request_through_an_openai_compatible_upstream` test failed twice during CI (Connection reset by peer) but is unrelated to release changes.

## Verdict

Pending human verification.

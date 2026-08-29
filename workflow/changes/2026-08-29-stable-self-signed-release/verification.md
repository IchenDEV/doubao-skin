---
id: "2026-08-29-stable-self-signed-release"
stage: verification
status: completed
owner: "codex"
created: "2026-08-29"
---

# Verification: stable self-signed release

## Release metadata

| Field | Value |
| --- | --- |
| Tag | `v0.1.0` |
| Release commit | `fa45de999f116924b844e182923996e4e6537741` |
| Release workflow run | [33251986190](https://github.com/IchenDEV/doubao-skin/actions/runs/33251986190) |
| Release page | [v0.1.0](https://github.com/IchenDEV/doubao-skin/releases/tag/v0.1.0) |
| CI run | [33247782493](https://github.com/IchenDEV/doubao-skin/actions/runs/33247782493) — success |
| Workspace version | `0.1.0` |

## Certificate metadata (public)

| Field | Value |
| --- | --- |
| Common Name | Doubao Skin Community Release |
| Algorithm | RSA 3072-bit, SHA-256, self-signed |
| Validity | 2026-08-29 to 2046-08-29 (20 years) |
| SHA-256 fingerprint | `6EF66DA353E5593DC972FC399DBE3594C1D0D3F0B5BFC8BBBFC5629E2656AD35` |
| SHA-1 signing identity | `0477AC79E4FF2E6768C7DBED72512EA6A1534337` |

## GitHub Actions Secrets

Five secrets confirmed present (names only, values never printed):

- `MACOS_CERTIFICATE`
- `MACOS_CERTIFICATE_PASSWORD`
- `MACOS_CERTIFICATE_SHA256`
- `MACOS_SIGNING_IDENTITY`
- `KEYCHAIN_PASSWORD`

Encrypted recovery `.p12` stored at `~/Library/Application Support/Doubao Skin/recovery/community-release.p12` (mode 0600). Its password is in macOS Keychain under "Doubao Skin Community Release P12".

## Asset checksums

```text
9bed9f2c2579d5c7394cbb20a788c4f9d8dd4fc51eba1fd496e5db7d454e837d  Doubao-Skin-macOS-universal.zip
661b4e206f71999c678030e0944d8e6ccb3a2759a4b2d3bda02dc5982cc61b4d  Doubao-Skin-macOS-universal.dmg
```

## Local build verification

- Source app `dist/豆包主题.app`: codesign deep strict verified, certificate SHA-256 matched.
- Universal architectures: x86_64 + arm64 for both GUI and CLI.
- ZIP extracted app: codesign deep strict verified, certificate SHA-256 matched.
- DMG: `hdiutil verify` passed, mounted read-only, contains `豆包主题.app` and `Applications -> /Applications`, app signature verified.
- CLI: `doubao-theme --help` runs.
- Skills: `create-doubao-theme/SKILL.md` and `apply-doubao-theme/SKILL.md` present.
- Wrong-fingerprint test: correctly failed with mismatch error.

## Downloaded release verification

- `shasum -a 256 -c` passed for both ZIP and DMG sidecar checksums.
- ZIP extracted app: codesign deep strict verified, certificate SHA-256 `6EF66DA3...2656AD35` matched.
- DMG: `hdiutil verify` passed, mounted read-only, contains `豆包主题.app` and `Applications -> /Applications`, app signature verified.
- Universal architectures confirmed: x86_64 + arm64.
- CLI and bundled Skills functional.

## Website download URL

```
https://github.com/IchenDEV/doubao-skin/releases/latest/download/Doubao-Skin-macOS-universal.zip
→ 302 → https://github.com/IchenDEV/doubao-skin/releases/download/v0.1.0/Doubao-Skin-macOS-universal.zip
```

## Verification script error cases

| Input | Expected | Actual |
| --- | --- | --- |
| No arguments | Exit 2, usage | Pass |
| Nonexistent app | Exit 1, "does not exist" | Pass |
| Non-hex fingerprint | Exit 2, "must be hexadecimal" | Pass |
| Short fingerprint | Exit 2, "64 hexadecimal characters" | Pass |
| Valid format, wrong fingerprint | Exit 1, mismatch | Pass |
| Correct fingerprint | Exit 0, verified | Pass |

## Deviations from plan

- `security add-trusted-cert -d` hangs on headless CI runners (admin authorization prompt). Removed in favor of user-domain identity lookup (`find-identity` without `-v`). `codesign` works with untrusted self-signed identities when the private key is in the search keychain.
- Tag `v0.1.0` was recreated three times due to CI signing fixes; the published tag points to commit `fa45de9` which includes all fixes.

## Residual risk

- Self-signed certificate: macOS Gatekeeper may still block first launch. Users must right-click and choose "Open". This is documented in README, release notes, and `docs/releasing.md`.
- The flaky `protocol_bridge::tests::converts_a_full_request_through_an_openai_compatible_upstream` test failed twice during CI (Connection reset by peer) but is unrelated to release changes.

## Verdict

Pending human verification.

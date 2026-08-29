# Releasing

## Automated release

The `Release macOS app` workflow runs for tags beginning with `v`.

The job targets the GitHub `production` environment. Configure that environment with a required human reviewer; an agent may prepare the tag and artifacts but cannot authorize the production job.

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow tests the workspace, builds Apple Silicon and Intel versions of both the GUI and `doubao-theme`, combines each binary into the universal app, bundles the two repository Skills, signs the bundle, uploads ZIP and DMG packages with both checksums as workflow artifacts, and creates or updates the matching GitHub Release. A manual workflow run builds artifacts but does not create a GitHub Release.

Each release also publishes a standalone CLI tarball (`doubao-theme-macOS-universal.tar.gz`) with its SHA-256 checksum. This lets users install only the CLI without the desktop app.

## Bundled themes

Release builds include five themes by default to keep the download small: 馋嘴豆包、甜点偷笑、鲸鱼娘、QQ 轻蓝和纯暗。The complete theme library remains in the repository and theme store.

Set `BUNDLE_ALL_THEMES=1` when an internal build needs every checked-in theme. Set `BUNDLED_THEMES` to a space-separated list of theme IDs to create another curated bundle.

## App icon

`assets/app-icon/AppIcon.icon` is the editable Icon Composer source. The macOS package compiles it to `Assets.car` and `AppIcon.icns`, which lets Finder and the Dock select the system, dark, or tinted appearance. The checked-in compiled files are only a fallback for build machines without a compatible full Xcode installation.

## Signing and notarization

Every release uses the same long-lived community self-signed identity. The job refuses to build when its encrypted certificate, pinned SHA-256 fingerprint, or SHA-1 signing identity is missing or mismatched. This creates a continuous release identity across versions, but it is not Apple notarization: macOS may require users to right-click the app and choose Open on first launch.

Configure these GitHub Actions secrets:

| Secret | Value |
| --- | --- |
| `MACOS_CERTIFICATE` | Base64-encoded, password-protected community self-signed `.p12`. |
| `MACOS_CERTIFICATE_PASSWORD` | Password of the `.p12` file. |
| `MACOS_CERTIFICATE_SHA256` | SHA-256 fingerprint of the public certificate, without relying on its display name. |
| `MACOS_SIGNING_IDENTITY` | SHA-1 code-signing identity accepted by `codesign`. |
| `KEYCHAIN_PASSWORD` | Temporary CI keychain password. |

Keep the encrypted recovery `.p12` outside the repository with mode `0600`; keep its password in a protected credential store. Rotate this identity only through a documented emergency release process, publish the new fingerprint, and preserve the old artifact record. Apple Developer ID signing and notarization are a separate release model and must not be mixed with this self-signed workflow.

## Release checklist

- Confirm the linked change artifact passed the PR gate and names the exact release commit.
- Update `CHANGELOG.md` and the workspace version when appropriate.
- Confirm CI passes on `main`.
- Verify the real desktop window and native apply workflow.
- Confirm theme and artwork provenance.
- Create and push the version tag.
- Download both release packages and verify their checksums. Run `hdiutil verify` on the DMG, mount it read-only, confirm it contains “豆皮.app” and `Applications -> /Applications`, verify the mounted app with `./scripts/verify-macos-signature.sh <app> <certificate-sha256>`, then test a clean install.
- Confirm `Contents/Resources/bin/doubao-theme --help` runs and both bundled Skill directories pass validation.
- Extract the standalone CLI tarball, verify `doubao-theme --version` prints the expected version, and confirm `shasum -a 256 -c` against the sidecar checksum.

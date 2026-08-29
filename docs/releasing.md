# Releasing

## Automated release

The `Release macOS app` workflow runs for tags beginning with `v`.

The job targets the GitHub `production` environment. Configure that environment with a required human reviewer; an agent may prepare the tag and artifacts but cannot authorize the production job.

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow tests the workspace, builds Apple Silicon and Intel versions of both the GUI and `doubao-theme`, combines each binary into the universal app, bundles the two repository Skills, signs the bundle, uploads ZIP and DMG packages with both checksums as workflow artifacts, and creates or updates the matching GitHub Release. A manual workflow run builds artifacts but does not create a GitHub Release.

## Bundled themes

Release builds include five themes by default to keep the download small: 馋嘴豆包、甜点偷笑、鲸鱼娘、QQ 轻蓝和纯暗。The complete theme library remains in the repository and theme store.

Set `BUNDLE_ALL_THEMES=1` when an internal build needs every checked-in theme. Set `BUNDLED_THEMES` to a space-separated list of theme IDs to create another curated bundle.

## App icon

`assets/app-icon/AppIcon.icon` is the editable Icon Composer source. The macOS package compiles it to `Assets.car` and `AppIcon.icns`, which lets Finder and the Dock select the system, dark, or tinted appearance. The checked-in compiled files are only a fallback for build machines without a compatible full Xcode installation.

## Signing and notarization

Without repository secrets, the workflow uses an ad-hoc signature. The package is useful for development and community testing, but macOS may require the user to explicitly open it.

For a production-quality notarized release, configure these GitHub Actions secrets:

| Secret | Value |
| --- | --- |
| `MACOS_CERTIFICATE` | Base64-encoded Developer ID Application `.p12`. |
| `MACOS_CERTIFICATE_PASSWORD` | Password of the `.p12` file. |
| `MACOS_SIGNING_IDENTITY` | Full Developer ID Application identity. |
| `KEYCHAIN_PASSWORD` | Temporary CI keychain password. |
| `APPLE_ID` | Apple account used by `notarytool`. |
| `APPLE_TEAM_ID` | Apple Developer Team ID. |
| `APPLE_APP_PASSWORD` | App-specific password for notarization. |

If any notarization credential is supplied, all three notarization values must be present. The packaging script notarizes and staples the app before packaging, then notarizes and staples the DMG. It fails rather than publishing a partly configured release. Builds without these credentials remain ad-hoc signed development artifacts and must not be described as notarized.

## Release checklist

- Confirm the linked change artifact passed the PR gate and names the exact release commit.
- Update `CHANGELOG.md` and the workspace version when appropriate.
- Confirm CI passes on `main`.
- Verify the real desktop window and native apply workflow.
- Confirm theme and artwork provenance.
- Create and push the version tag.
- Download both release packages and verify their checksums. Run `hdiutil verify` on the DMG, mount it read-only, confirm it contains “豆包主题.app” and `Applications -> /Applications`, verify the mounted app with `codesign --verify --deep --strict`, then test a clean install.
- Confirm `Contents/Resources/bin/doubao-theme --help` runs and both bundled Skill directories pass validation.

---
id: "2026-08-30-doupi-app-icon-redesign"
stage: verification
status: pending
owner: "codex"
created: "2026-08-30"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: doupi app icon redesign

## Automated checks

- Canonical source inspection passed with `/Users/chenli/.codex/skills/icon-composer/scripts/inspect-icon.sh assets/app-icon/AppIcon.icon`: the package retains UTI `com.apple.iconcomposer.icon`, valid JSON, three groups and three present assets.
- Icon Composer save/readback showed exactly `Yuba Front Fold`, `Soybean Accent`, and `Yuba Back Sheet`; the former `Doupi`, `Skin Fold`, `Doubao Work Mark`, `01-doubao-work-mark.png`, and `02-skin-fold.svg` names are absent from the canonical package and formal asset directory.
- An isolated Xcode beta `actool` compile succeeded; `evidence/actool-isolated.log` records generation of non-empty `AppIcon.icns`, `Assets.car`, and the partial plist. The isolated products are retained under `evidence/compiled-icon/`.
- `ICON_DEVELOPER_DIR=/Applications/Xcode-beta.app/Contents/Developer ./scripts/build-macos.sh` completed successfully. `evidence/build-macos-host.log` contains `Compiled adaptive app icon from AppIcon.icon`, demonstrating that the build did not use the fallback path.
- After rebasing the PR branch onto `origin/main` at `d1b4347`, `./scripts/check.sh all` passed: workflow validation, 45 Rust tests, clippy/check, 9 web tests, TypeScript, and the Next.js production build all completed successfully. The macOS host package was then rebuilt successfully on that same base.
- The built `dist/豆皮.app/Contents/Info.plist` retains `CFBundleIconFile=AppIcon`, `CFBundleIconName=AppIcon`, bundle identifier `dev.ichen.doubao-skin`, and version `0.3.2 (1)`.
- `codesign --verify --deep --strict --verbose=4 dist/豆皮.app` passed. `xcrun assetutil --validate-file` passed for both the isolated and bundled asset catalogs.
- `AppIcon.icns` is deterministic and the repository/bundle hashes both remain `236dc58b6dad1eb5ac4bfcc4154030fc1d2d369232de8989c319db38ff567a1d`. The repository fallback `Assets.car` is `cfe0f8b65b4943f2070754b2deaa6bb8c0d06ec1c0fb975585599070227077b1`; a fresh post-rebase `actool` build produced `4e4216882000e7f005790e154a1dc970bef1bd8c438c9c087d7b46471e302856` because Apple embeds fresh rendition metadata. Both catalogs pass `assetutil` validation and expose the same icon structure; byte equality is not used as a repeat-build invariant.
- The Default and Dark Icon Composer exports were mechanically resized into both iconsets and the xcassets app-icon set; no Rust, Info.plist, build-script, theme, metadata, or component source was changed.
- At the user's later direction, the approved Default export was also mechanically resized to `apps/web/src/app/icon.png` (128×128 RGBA, SHA-256 `c46d8c8f4bf49664dcdf23fbec7067391b757796bd7970b2db644b33a3a78c29`) and `apps/web/public/app-icon.png` (256×256 RGBA, SHA-256 `6e3576e1640a908491b05ba7da26a740415b768400696a15680492e4663fddc2`).
- `pnpm --dir apps/web sync` and `./scripts/check.sh web` passed: 9 tests passed, TypeScript passed, and the Next.js 16.3.3 production build generated the static `/icon.png` route. The sync command's unrelated theme database/catalog/package rewrites were discarded after confirming those paths were clean before this task.
- The Chinese and English READMEs now reference the tracked 256×256 website icon, describe the shared layered yuba identity, and retain only the two current product screenshots required by `docs/README.md`. All four local image references exist.

## Behavioral evidence

- The exact built bundle was launched successfully. PID `68422` resolved to `/Users/chenli/.codex/worktrees/8fc8/doubao-work-skin/dist/豆皮.app/Contents/MacOS/豆皮` at inspection time.
- The normal application window opened and remained usable; see `evidence/app-window.png`. Its existing zoom control is disabled and the product window is fixed-size, so a manufactured narrow-window run is not applicable to this asset-only change.
- Finder selected the same `dist/豆皮.app` and displayed the new folded-yuba icon and version `0.3.2`; see `evidence/finder.png`.
- The same bundle's About panel displayed the new icon and version `0.3.2 (1)`; see `evidence/about-panel.png`.
- Dock accessibility capture timed out twice, and the app switcher overlay dismissed before the automation could capture it. No system settings or global icon caches were changed to manufacture these results.
- The production-mode website at `http://127.0.0.1:3000/` rendered the new mark in the header at the existing 34×34 desktop and 30×30 narrow sizes. The loaded Next image reported complete with 48×48 natural output; both `/app-icon.png` and the generated `/icon.png?...` route returned HTTP 200 with `image/png`.
- The browser interaction path `首页 → 使用与下载` reached `/guide`, preserved the header mark, and produced no warnings or errors. At a 390×844 viewport the page retained `scrollWidth=390`, so the replacement introduced no horizontal overflow.

## Visual evidence

- Real Icon Composer macOS previews are preserved in `evidence/icon-composer-literal-final-light.jpeg`, `evidence/icon-composer-literal-final-dark.jpeg`, and `evidence/icon-composer-literal-final-mono.jpeg`.
- Native Icon Composer 1024-point exports are `evidence/icon-composer-exports/AppIcon-Default-1024.png` and `evidence/icon-composer-exports/AppIcon-Dark-1024.png`.
- The accepted geometry now reads as several irregular, paper-thin folded sheets with visible layer edges, broad wrinkles/pores, one lifted fold, and two soybeans. The former abstract yellow `d`/ring and blue-purple work mark are absent.
- `evidence/icon-small-sizes.png` compares Default, Dark, and Mono at 16/32/64 px. At 32 and 64 px the layered sheets and folded edge remain clear. At 16 px the stacked silhouette survives but fine wrinkles and the literal food-material cue are necessarily reduced; this remains a human-review point rather than an asserted pass.
- Finder and About-panel screenshots confirm that the system loaded the new bundle icon. Dock and app-switcher screenshots are not available, so the full system-surface acceptance criterion is not yet satisfied.
- In-app Browser screenshots at the default desktop viewport and 390×844 show the folded-yuba mark centered and unclipped in the existing header shell. The page identity, meaningful DOM, absence of framework overlays, console health, resource loading, desktop layout, narrow layout, and one navigation interaction all passed.
- `docs/images/app.png` now shows the verified current 豆皮 desktop window, and `docs/images/gallery.png` shows the verified current website with the folded-yuba mark. Both were visually reopened after replacement; the screenshots contain no clipping or unrelated private content.

## Security and privacy evidence

- All geometry is original and locally authored; the accepted SVG sources and rendered layers are retained under `evidence/source-v5/`.
- No resource was read from or written to `/Applications/DoubaoWork.app` or another official application. The protocol bridge, themes, account data, credentials, signing material, and release configuration were not touched.
- Evidence screenshots contain only the local 豆皮 app, Icon Composer, Finder, the local website, and the necessary system surface; no conversation content or private workspace data was included.

## Deviations and residual risk

- The first abstract yellow design failed the user's literal-recognition test. The accepted spec and plan were revised to use folded yuba sheets, pores, thin edges, and soybeans, then reapproved by `idevlab` before formal assets were overwritten.
- After the user's separate deletion confirmation, the hidden legacy Icon Composer group and its duplicate external layer files were permanently removed. The accepted plan's prose about the group being temporarily hidden and awaiting approval is historical gate text; its frontmatter records the later accepted state.
- `actool` embeds timestamps and rendition identifiers, so separate compiles produce byte-different `Assets.car` files even when `assetutil` reports the same names, sizes, appearances, and layer structure. The repository fallback was synchronized from one successful host build and the isolated compile remains as independent evidence; later rebuilds are expected to differ bytewise and are validated structurally instead.
- Finder and About are verified, but Dock and app-switcher visual captures remain unverified because the Computer Use Dock request timed out and the app-switcher overlay could not be held for capture. The 16 px rendering also loses material texture and should be judged by a human in context.
- Packaging generated local ZIP, DMG, tar, and checksum outputs as part of the existing build script, but no tag, release, upload, publication, or production approval action was performed.
- The accepted intent/spec originally excluded the website favicon. The user later explicitly requested the website icon update and then explicitly rejected a separate intent. Following the repository's established implementation-deviation pattern, the accepted historical artifacts were not rewritten; this verification records the newer instruction, which only synchronizes the two existing website icon files and does not broaden into PWA, social, theme, or deployment work.
- The user subsequently requested a README update. The bilingual README hero, one feature bullet, and the two existing current-product screenshots were updated in the same brand-identity scope; download claims, release links, deployment instructions, and other documentation were not changed.

## Verdict

Pending fresh-context or human verdict. Canonical Icon Composer editing, Default/Dark/Mono source evidence, Apple compilation, bundle integration, signing, launch, Finder, About-panel, website icon integration, production web build, desktop browser, narrow browser, and navigation checks pass. Full visual acceptance remains open for Dock, app switcher, and the human judgment of literal recognition at 16 px.

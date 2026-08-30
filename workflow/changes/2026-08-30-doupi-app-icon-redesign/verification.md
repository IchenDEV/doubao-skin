---
id: "2026-08-30-doupi-app-icon-redesign"
stage: verification
status: passed
owner: "codex"
created: "2026-08-30"
based_on: plan.md
commit: "475aa8aa68c744b43785daec71357e79f213aeb2"
verification_mode: "human"
verified_by: "idevlab"
verified_at: "2026-08-30"
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
- The Default and Dark Icon Composer exports were mechanically resized into both iconsets and the xcassets app-icon set; that icon integration did not change Rust, Info.plist, build-script, theme, metadata, or component source. Later user-requested README screenshot feedback changed only the two themes named below.
- At the user's later direction, the approved Default export was also mechanically resized to `apps/web/src/app/icon.png` (128×128 RGBA, SHA-256 `c46d8c8f4bf49664dcdf23fbec7067391b757796bd7970b2db644b33a3a78c29`) and `apps/web/public/app-icon.png` (256×256 RGBA, SHA-256 `6e3576e1640a908491b05ba7da26a740415b768400696a15680492e4663fddc2`).
- `pnpm --dir apps/web sync` and `./scripts/check.sh web` passed: 9 tests passed, TypeScript passed, and the Next.js 16.3.3 production build generated the static `/icon.png` route. The sync command's unrelated theme database/catalog/package rewrites were discarded after confirming those paths were clean before this task.
- After the user requested stronger background visibility, `pnpm --dir apps/web sync` regenerated the theme database, catalog, previews, and installable packages for `qq-light-blue` and `pastel-starry-room`. Twenty-eight timestamp-only package rewrites were restored; all 30 catalog entries were then mechanically checked against the committed package SHA-256 and byte size.
- `./scripts/check.sh all` passed after the transparency and screenshot revision: workflow policy passed, all 45 Rust tests passed, clippy/check passed, all 9 web tests passed, TypeScript passed, and the Next.js production build generated all 38 static pages. The standalone authoring command `doubao-theme check` still rejects both complex bundled CSS files with `theme.css 必须同时限制在主题 html 和 body 作用域内`; an isolated `git archive HEAD` baseline produced the same exit 3 for both themes, so this pre-existing authoring-check limitation was not introduced or broadened here.
- The Chinese and English READMEs now reference the tracked 256×256 website icon, describe the shared layered yuba identity, retain the current desktop/gallery screenshots, and add the same two privacy-redacted real-window captures. Every local README image reference resolves to a tracked file.

## Behavioral evidence

- The exact built bundle was launched successfully. PID `68422` resolved to `/Users/chenli/.codex/worktrees/8fc8/doubao-work-skin/dist/豆皮.app/Contents/MacOS/豆皮` at inspection time.
- The normal application window opened and remained usable; see `evidence/app-window.png`. Its existing zoom control is disabled and the product window is fixed-size, so a manufactured narrow-window run is not applicable to this asset-only change.
- Finder selected the same `dist/豆皮.app` and displayed the new folded-yuba icon and version `0.3.2`; see `evidence/finder.png`.
- The same bundle's About panel displayed the new icon and version `0.3.2 (1)`; see `evidence/about-panel.png`.
- Dock accessibility capture timed out twice, and the app switcher overlay dismissed before the automation could capture it. No system settings or global icon caches were changed to manufacture these results.
- The production-mode website at `http://127.0.0.1:3000/` rendered the new mark in the header at the existing 34×34 desktop and 30×30 narrow sizes. The loaded Next image reported complete with 48×48 natural output; both `/app-icon.png` and the generated `/icon.png?...` route returned HTTP 200 with `image/png`.
- The browser interaction path `首页 → 使用与下载` reached `/guide`, preserved the header mark, and produced no warnings or errors. At a 390×844 viewport the page retained `scrollWidth=390`, so the replacement introduced no horizontal overflow.
- The local CLI applied the edited `QQ轻蓝` and `星光书房` sources to the real `/Applications/DoubaoWork.app` loopback live target. Each run injected exactly three existing pages. The captured 1396×768 windows show the same actual Doubao Work conversation surface with the expected background, message, and composer styling; no message was sent and no official app file was modified.

## Visual evidence

- Real Icon Composer macOS previews are preserved in `evidence/icon-composer-literal-final-light.jpeg`, `evidence/icon-composer-literal-final-dark.jpeg`, and `evidence/icon-composer-literal-final-mono.jpeg`.
- Native Icon Composer 1024-point exports are `evidence/icon-composer-exports/AppIcon-Default-1024.png` and `evidence/icon-composer-exports/AppIcon-Dark-1024.png`.
- The accepted geometry now reads as several irregular, paper-thin folded sheets with visible layer edges, broad wrinkles/pores, one lifted fold, and two soybeans. The former abstract yellow `d`/ring and blue-purple work mark are absent.
- `evidence/icon-small-sizes.png` compares Default, Dark, and Mono at 16/32/64 px. At 32 and 64 px the layered sheets and folded edge remain clear. At 16 px the stacked silhouette survives but fine wrinkles and the literal food-material cue are necessarily reduced; this remains a human-review point rather than an asserted pass.
- Finder and About-panel screenshots confirm that the system loaded the new bundle icon. Dock and app-switcher screenshots are not available, so the full system-surface acceptance criterion is not yet satisfied.
- In-app Browser screenshots at the default desktop viewport and 390×844 show the folded-yuba mark centered and unclipped in the existing header shell. The page identity, meaningful DOM, absence of framework overlays, console health, resource loading, desktop layout, narrow layout, and one navigation interaction all passed.
- `docs/images/app.png` now shows the verified current 豆皮 desktop window, and `docs/images/gallery.png` shows the verified current website with the folded-yuba mark. Both were visually reopened after replacement; the screenshots contain no clipping or unrelated private content.
- `docs/images/doubao-work-qq-light-blue-redacted.png` and `docs/images/doubao-work-starry-room-redacted.png` are real-window captures, not preview-shell mockups. Both were reopened at their original 1396×768 dimensions after redaction. QQ Light Blue now uses a 0.02 veil with 0.34–0.56 light conversation surfaces; Starry Room uses a 0.02 veil with 0.10–0.48 light conversation surfaces. Their background art, bubbles, composer, conversation list, titles, and messages remain visible.

## Security and privacy evidence

- All geometry is original and locally authored; the accepted SVG sources and rendered layers are retained under `evidence/source-v5/`.
- No file was written inside `/Applications/DoubaoWork.app` or another official application. The existing loopback live path only applied the selected theme CSS to the running page; credentials, signing material, release configuration, attachments, tools, and native headers were not accessed or forwarded.
- Unredacted Doubao Work captures were created only under temporary `/tmp/doubao-readme-v2.*` storage and were never copied into the repository. After final visual review, the temporary directory was moved to macOS Trash and remains recoverable only until the user empties Trash. Following the user's narrowed privacy boundary, the committed PNGs use strong local mosaic redaction only over the company identity, the author's avatar/identity area, and the exact top-bar computer device name. No generative edit or whole-page blur was used; all other pixels come from the real window capture. The live injector then restored all three Doubao Work pages to the default appearance.

## Deviations and residual risk

- The first abstract yellow design failed the user's literal-recognition test. The accepted spec and plan were revised to use folded yuba sheets, pores, thin edges, and soybeans, then reapproved by `idevlab` before formal assets were overwritten.
- After the user's separate deletion confirmation, the hidden legacy Icon Composer group and its duplicate external layer files were permanently removed. The accepted plan's prose about the group being temporarily hidden and awaiting approval is historical gate text; its frontmatter records the later accepted state.
- `actool` embeds timestamps and rendition identifiers, so separate compiles produce byte-different `Assets.car` files even when `assetutil` reports the same names, sizes, appearances, and layer structure. The repository fallback was synchronized from one successful host build and the isolated compile remains as independent evidence; later rebuilds are expected to differ bytewise and are validated structurally instead.
- Finder and About are verified, but Dock and app-switcher visual captures remain unverified because the Computer Use Dock request timed out and the app-switcher overlay could not be held for capture. The 16 px rendering also loses material texture and should be judged by a human in context.
- Packaging generated local ZIP, DMG, tar, and checksum outputs as part of the existing build script, but no tag, release, upload, publication, or production approval action was performed.
- The accepted intent/spec originally excluded the website favicon. The user later explicitly requested the website icon update and then explicitly rejected a separate intent. Following the repository's established implementation-deviation pattern, the accepted historical artifacts were not rewritten; this verification records the newer instruction, which only synchronizes the two existing website icon files and does not broaden into PWA, social, theme, or deployment work.
- The user subsequently requested a README update. The bilingual README hero, one feature bullet, and the two existing current-product screenshots were updated in the same brand-identity scope; download claims, release links, deployment instructions, and other documentation were not changed.
- The user then explicitly requested real transformed-page screenshots with personal information masked. This supersedes the earlier two-screenshot README convention only for a focused bilingual `真实页面效果` / `Real transformed pages` section; no intent was added, matching the user's earlier direction to keep this work in the existing change.
- The first privacy pass masked the whole sidebar and all OCR-visible text, which the user rejected as excessive. The user explicitly narrowed redaction to author identity/avatar, company name, and computer device name, and requested stronger background visibility. The replacement screenshots use only those three local mosaic regions. The accepted plan's expected-untouched theme boundary is superseded only for `qq-light-blue` and `pastel-starry-room`: their foreground veil/surface alpha values and generated catalog artifacts were updated, while no other theme source changed.

## Verdict

Passed by human verifier `idevlab` on 2026-08-30. Canonical Icon Composer editing, Default/Dark/Mono source evidence, Apple compilation, bundle integration, signing, launch, Finder, About-panel, website icon integration, production web build, desktop browser, narrow browser, navigation, two revised live-theme real-window captures, catalog/package consistency, and the narrowed privacy review passed. The verifier accepted the documented residual risk for unavailable Dock and app-switcher captures and reduced material detail at 16 px.

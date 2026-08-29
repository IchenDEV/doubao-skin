---
id: "2026-08-29-support-standard-doubao"
stage: verification
status: pending
owner: "codex"
created: "2026-08-29"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: support standard doubao

## Automated checks

- `./scripts/check.sh rust`
  - Passed formatting and Clippy with warnings denied.
  - Passed 6 desktop tests and 25 `skin-core` tests, including target identity,
    target-scoped restoration, same-document reinjection, and minimum-window
    layout regressions.
- `jq -e . design/theme-standard/layout-spec.json`
  - Passed after recording the sidebar search/source placement, 24 px window
    chrome-to-title gap, centered target switch, and empty trailing titlebar.
- Static UI-copy/element checks found no visible `应用到` label and no
  `install-package` titlebar control in `apps/desktop/src/main.rs`.
- `corepack pnpm --dir apps/web sync && ./scripts/check.sh web && ./scripts/check.sh workflow`
  - Regenerated the catalog from 26 theme manifests.
  - Passed the production Next.js build (30 static pages), dependency audit,
    and workflow validation for all five active change artifacts.
- `./scripts/build-macos.sh`
  - Produced `dist/豆包主题.app` and
    `dist/Doubao-Skin-macOS-arm64.zip`.
  - The app passes `codesign --verify --deep --strict`; its executable is an
    arm64 Mach-O.
  - ZIP SHA-256:
    `a364fbe2bccfeca6b64105733a7ab9c89e9c54263cb449d4f37f932ce2eeb52b`.
  - The executable extracted from the ZIP has the same SHA-256 as the signed
    app bundle executable (`d86c87b2f0c0b2212e6655157bd22df1d79f91f9e766ccc4498a4398e863e0d7`).
- Manifest/catalog consistency checks found 26 themes and 26 manifests whose
  user-facing author is `豆包主题`; package manifests matched their sources.
- `git diff --check` reported no whitespace errors in the scoped files.

## Behavioral evidence

- Tested against installed 豆包 2.26.10 (`com.bot.pc.doubao`) and 豆包工作
  2.26.7 (`com.work.pc.doubao`).
- The app exposes an installation-aware 豆包/豆包工作 target switch, persists
  the selection, updates preview copy, and supports Command-1/Command-2.
- On 豆包, applying `pure-dark` set `data-skin="pure-dark"` on its background,
  launcher, and main chat pages. Reloading the same main-page CDP target kept
  the target id and automatically restored the marker. Restore removed the
  marker from all three pages.
- On 豆包工作, the same apply, same-target reload, reinjection, and three-page
  restore sequence passed.
- Applying and restoring one target did not modify the other target. Existing
  official-app processes were reused while their loopback debug ports were
  available; neither client was restarted during apply/restore.
- After restoration, both official clients were quit through their app menus
  and launched normally. Their process ids changed from 97599/74625 to
  69451/69531, neither process carried a remote-debugging argument, and ports
  9222/9223 were no longer listening.

## Visual evidence

- Native theme-picker window at normal size:
  [theme-picker-normal.jpeg](evidence/theme-picker-normal.jpeg)
- Native theme-picker window at the 720 x 560 minimum size:
  [theme-picker-narrow.jpeg](evidence/theme-picker-narrow.jpeg)
- At normal size, the title starts 24 px after the traffic-light group, the
  豆包/豆包工作 switch is centered on the whole window, and the trailing
  titlebar region is empty. The source switch and search field are in the
  sidebar; installation remains available through the sidebar drop target and
  file chooser instead of a duplicate titlebar button.
- At 720 x 560, the source switch and search field share one leading row while
  the centered target switch, horizontal theme list, preview, restore, and
  apply controls remain visible. The accessibility tree exposes the same
  target/source/search/install controls without a titlebar install action.
- Entering `pure` in the sidebar search reduced the visible library to the
  matching `纯暗` theme, and switching between `我的主题` and `主题商店` kept
  the sidebar controls available.
- The previously rebuilt, signed app from `dist/豆包主题.app` was launched
  directly and exposed both targets and all primary controls. The later
  titlebar/sidebar refinement above was rechecked from the current debug build;
  packaging code did not change and the package build was not repeated.
- Both themes were visually inspected in the corresponding official app. A
  豆包 screenshot was retained only under ignored local verification output;
  no 豆包工作 screenshot was persisted because its sidebar contained existing
  private conversation titles.

## Security and privacy evidence

- CDP listeners were bound to `127.0.0.1`. Target discovery accepted only the
  selected app's known URL identity before admitting generic extension pages.
- The live loop inspected page URL and this tool's `data-skin` marker only; it
  did not read or forward conversation text, cookies, headers, workspace data,
  attachments, tools, or unknown content blocks.
- Restore removes only this tool's stylesheet, icon/background elements,
  observers, and captured root/body attributes. It does not modify either
  official application bundle.
- The protocol bridge and offline clone path remain 豆包工作-only, as required
  by the accepted scope.
- Official-app screenshots and private conversation content were not added to
  the repository or package.

## Deviations and residual risk

- No accepted-scope deviation.
- Real-window QA exposed two defects before completion: same-target navigation
  could lose the theme marker, and the 720 x 560 layout clipped the primary
  controls. Both received regression coverage and were revalidated after the
  fixes.
- 豆包 was logged out during validation, so authenticated-only icon surfaces
  were not exhaustively inspected there. Theme injection, visual styling,
  navigation reinjection, restoration, and target isolation were verified.
- Compatibility evidence is limited to the installed versions above. A
  fresh-context verifier or human must still record the final verdict.

## Verdict

Implementation evidence supports acceptance. Final verdict remains pending for
the required fresh-context verification.

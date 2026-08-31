---
id: "2026-08-31-remember-last-theme"
stage: verification
status: pending
owner: "codex"
created: "2026-09-01"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: remember last theme

## Automated checks

- The configuration red/green proof started with
  `cargo test -p skin-core auto_theme::tests --locked`: the first run failed
  because the module/API did not exist, then all 11 persistence, validation,
  parent-child, audit-session, and supervisor tests passed.
- The live-policy red/green proof started with
  `cargo test -p skin-core stop_on_target_exit_never_relaunches --locked`: the
  first run failed because `PortLossPolicy` did not exist, then the test passed
  with the automatic watcher returning instead of relaunching.
- `cargo test -p doubao-skin-desktop --bin doubao-skin-agent --locked` passed
  all 3 helper tests: nested-bundle path derivation, standalone-development
  rejection, and the registered-application fallback when a managed login-item
  process cannot observe the GUI executable path.
- `cargo test -p doubao-skin-desktop --bin doubao-skin-app --locked` passed all
  18 desktop tests. These include all four `SMAppService` states, unsupported
  mapping, parent-child switch dependency, missing-theme/busy states, matching
  apply generation, and the two-switch UI state contract.
- `cargo test -p skin-core --lib --locked` passed all 51 core tests. The suite
  covers strict schema handling, atomic preservation, per-audit-session launch,
  helper/main ownership, complete stopped-to-running transitions, and the
  target-exit policy alongside existing live/theme regressions.
- `./scripts/check.sh rust` passed again after the live handoff correction:
  formatting, helper 3 tests, desktop 18 tests, core 51 tests, integration
  tests, and Clippy with warnings denied.
- `./scripts/check.sh workflow` passed all 17 artifact sets and the portability
  policy. Main-app detection now verifies each candidate PID's exact executable
  path rather than trusting a process name or command-line substring.
- `./scripts/check.sh all` passed workflow, Rust, lockfile policy, web unit
  tests, skill synchronization, TypeScript, the production Next.js build (42
  static pages), and the dependency audit with no known vulnerabilities.
- `cargo fmt --all -- --check` and `git diff --check` passed.

## Package evidence

- `./scripts/package.sh desktop-macos` produced and verified the arm64 app,
  ZIP, and DMG.
- `./scripts/package.sh desktop-macos --universal` produced and verified the
  final universal app, ZIP, and DMG.
- The main executable and `Contents/Library/LoginItems/豆皮后台服务.app`
  executable both report `x86_64 arm64` via `lipo -archs`.
- Both plists pass `plutil`. The helper has bundle ID
  `dev.ichen.doubao-skin.agent`, executable/display name `豆皮后台服务`,
  `LSUIElement=true`, version `0.4.0`, build `1`, and minimum macOS `12.0`,
  matching the main bundle's version and deployment target.
- Packaging signs the helper first and the main bundle second. The current
  local package uses ad-hoc signing; `codesign --verify --deep --strict` passes
  for the complete nested bundle. This proves structure and sealing, not a
  Developer ID or notarization result.
- Final universal ZIP SHA-256:
  `56b046cc5718e9e2d32c5a5c76e3d2cff8d7364a7222b3287c0616ffaa7791a4`.
- Final universal DMG SHA-256:
  `89193da446e1243945422275bf4ca2b3b84004d2965f5484a34bb9ff0b1936ad`.

## Behavioral evidence

- The packaged main app was launched directly with its production bundle
  structure. With no existing `auto-theme.json`, both switches rendered off,
  the parent was unavailable, the child was visibly dependent, and the group
  showed `请先成功应用一个主题`.
- The packaged app's `--live pure-dark` entry applied the theme through the
  existing loopback CDP session. The first successful injection atomically
  created schema v1 with target `doubao-work`, theme `pure-dark`, null opacity,
  and both requests false. No selection or preview action wrote the file.
- The running official target already owned port 9222, so the test reused it
  without a target restart.
- `/Applications/DoubaoWork.app` passed `codesign --verify --deep --strict`
  before and after live application/restoration. The official bundle was not
  written.
- After explicit user confirmation, the parent switch registered the final
  nested helper through `SMAppService`; status was `enabled`, the managed
  helper ran as PID 57507, and the persisted child request remained false.
- With the main app running, only its PID held 3 loopback CDP connections and
  the helper held 0. Closing the main app caused the helper to take over with
  3 connections. Reopening the main app caused the helper to return to 0 and
  the main app to own the 3 connections again. This closes the real
  main/helper ownership handoff without restarting or exiting Doubao Work.
- Turning the parent switch off wrote `keep_requested=false`, preserved
  `open_at_login=false`, unregistered the service, removed the launchd job,
  and exited the helper. Cleanup restored all 3 admitted pages. The test-created
  config and audit-session marker were moved recoverably to
  `~/.Trash/doubao-skin-auto-theme-final-qa-2026-09-01.json` and
  `~/.Trash/doubao-skin-auto-theme-session-final-qa-2026-09-01.json`; their
  original application-support paths are absent again.

## Visual evidence

- The final packaged app was inspected at its fixed 1120 x 720 window size in
  the current dark system appearance. The settings group fits between preview
  and theme actions; both titles, one-line descriptions, switches, status,
  opacity/actions, and theme list remain visible without overlap or clipping.
- The saved target description correctly rendered as `登录 Mac 后自动打开豆包工作`.
- Enabled/disabled contrast and the parent-child hierarchy were visually
  distinct. No internal path, selector, port, test log, or future-plan copy was
  displayed in the product window.
- The GPUI elements declare stable IDs, `Role::Switch`, `aria_toggled`,
  focusability/tab stops only when enabled, and Enter/Space activation through
  GPUI's click mapping. The local macOS accessibility bridge exposed only the
  GPUI top-level window, so a live VoiceOver/AccessKit child-tree result is not
  claimed; this remains a manual acceptance item.
- Narrow/short branches are regression-tested, but the current product window
  is fixed and non-resizable at 1120 x 720, so those branches cannot be reached
  in the final bundle without changing an unrelated window contract.
- Light appearance was not forced because that would change the user's system
  appearance. The palette implementation and existing light/dark tests remain
  green; only the current dark appearance has final-window evidence.

## Security and privacy evidence

- The helper has no window, Dock icon, listener, IPC channel, or independent
  theme loader. It reads only the strict 豆皮-owned config and uses the shared
  loopback live path.
- Registration uses only `SMAppService` on macOS 13+, and the unsupported path
  stays stable on macOS 12/other platforms. No LaunchAgent plist, cron job,
  administrator daemon, or official-app modification was added.
- Target observation is restricted to the exact executable path. Main-app
  ownership first checks the exact outer executable and uses AppKit's
  `NSRunningApplication` with the fixed main bundle ID as the managed-login-item
  fallback; it does not enumerate window titles or content. Theme data stores
  target ID, theme ID, opacity, request booleans, and audit session ID only; it
  stores no account, title, conversation, cookie, header, attachment, tool, or
  workspace content.
- Real target testing inspected only the target identity URLs and 豆皮-owned
  runtime markers. No official-app screenshot or conversation content was
  saved in the repository.

## Deviations and residual risk

- The first live handoff probe found that exact process-path lookup alone was
  unreliable inside the ServiceManagement-managed helper: it took over but did
  not yield after the main app reopened. The implementation now adds the
  platform-native registered-application fallback and a regression test; the
  rebuilt final package passed the complete takeover/yield sequence above.
- Replacing an already registered ad-hoc bundle in place briefly produced
  macOS `EX_CONFIG`/LWCR cache errors because the CDHash changed without a
  release identity. A normal unregister, cache settlement, and re-register
  created a fresh BTM item and the final binary ran successfully. This is a
  local repeated-test artifact; Developer ID upgrade behavior still belongs to
  the release lane.
- A real logout/login would interrupt the user's desktop. It still requires a
  separate just-in-time permission; the per-audit-session and child-off logic
  are otherwise covered by deterministic tests. No logout was performed.
- The live target-exit/no-reopen case was not exercised because it would close
  the user's running Doubao Work session. The explicit stop policy and complete
  stopped-to-running transition are covered by core regression tests.
- The local package is ad-hoc signed. A stable self-signed or Developer ID
  release may produce different ServiceManagement consent behavior and must be
  rechecked in the release lane.
- Direct official-app launch can still briefly show the default appearance
  before one controlled restart adds CDP arguments; zero-frame restoration is
  explicitly not promised.

## Verdict

Automated, package, normal-size dark-window, current-session Login Item, and
main/helper handoff evidence supports the implementation. Final verdict remains
pending the optional disruptive logout/login acceptance and the required
fresh-context verifier or human verdict.

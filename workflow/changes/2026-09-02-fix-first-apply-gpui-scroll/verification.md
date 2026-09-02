---
id: "2026-09-02-fix-first-apply-gpui-scroll"
stage: verification
status: passed
owner: "codex"
created: "2026-09-02"
based_on: plan.md
commit: "134cb1d"
verification_mode: "human"
verified_by: "user"
verified_at: "2026-09-02"
---

# Verification: fix first apply gpui scroll

## Automated checks

- Red-first evidence was captured before the implementation:
  - `cargo test -p skin-core initial_injection_failure_times_out_with_the_last_error --locked` failed because the old helper returned `None` instead of `Some("未找到可注入页面")` when no page ever became injectable.
  - `cargo test -p skin-core initial_dead_target_retries_before_steady_state_throttle --locked` failed to compile because the required initial/steady-state probe seam did not exist.
  - `cargo test -p skin-core once_mode_requires_a_successful_injection --locked` was added with the same seam so a zero-injection `once` run could not be treated as success.
  - `cargo test -p doubao-skin-desktop target_installations_are_detected_once_and_read_from_memory --locked` failed to compile because no cached installation-state type existed.
- The three targeted `skin-core` tests and the targeted desktop installation-cache test passed after the implementation.
- `cargo test -p skin-core --lib --locked`: 75 passed, 0 failed.
- `cargo test -p doubao-skin-desktop --bin doubao-skin-app --locked`: 33 passed, 0 failed.
- `cargo fmt --all -- --check`, `./scripts/check.sh rust`, and `./scripts/check.sh workflow` passed.
- `./scripts/check.sh all` passed, including workflow/portability checks, all Rust unit and integration tests, Web tests, TypeScript checking, Next.js production build, skill synchronization, and the vulnerability check. The only diagnostic was the pre-existing future-incompatibility warning for `block v0.1.6`.
- `git diff --check` passed.

## Behavioral evidence

- The initial phase now re-probes a dead target on the next existing watcher tick; after the first successful injection it retains the previous 15-tick steady-state throttle.
- The 30-second initial-injection deadline covers missing pages as well as explicit injection failures and returns the latest concrete error when available. `once` can finish only after at least one successful injection.
- The desktop view detects the three target installation states once at initialization. Target-switch and detail rendering read the in-memory value; apply and target-switch execution retain their existing authoritative installation checks.
- A uniquely signed acceptance bundle at `target/acceptance/豆皮验收-8887-fix.app` completed five real Doubao Work cold-start rounds with one Apply click per round and no retry: 9.480 s, 8.943 s, 0.459 s, 8.806 s, and 8.807 s (median 8.806 s). Every round reached `正在使用`; none remained in `正在应用…` or required a second click. The unusually fast third result is preserved as observed rather than normalized.
- Before each measured round the exact Doubao Work main PID was terminated normally and port 9222 was confirmed closed. The old acceptance watcher was stopped first, so only one watcher owned the target.

## Visual evidence

- Baseline: the same 12 alternating GPUI list-scroll actions took 9.181 s. In `target/diagnostics/gpui-scroll.sample.txt`, 420 target-switch render samples plus 167 detail-render samples (587/605 draw samples) entered `registered_macos_bundle` and synchronously waited on the child process path.
- Fixed build: the same 12 actions took 5.014 s, a 45.4% elapsed-time reduction. `target/diagnostics/gpui-scroll-fixed.sample.txt` contains zero `registered_macos_bundle` and zero `osascript` frames; 1,583 of 1,650 main-loop samples were idle and only 51 serviced the main dispatch queue.
- A further 24 alternating scroll actions completed in 9.270 s with the selected theme, 52% opacity, and both switches unchanged.
- The acceptance app currently remains open with target `豆包工作`, filtered theme `甜点偷笑`, opacity 52%, automatic keep off, login open off, and status `正在使用` for human inspection.
- The approved desktop window is intentionally fixed-size (`main_window_is_fixed_at_the_approved_size`), so a manually resized narrow-window pass is not applicable without changing an unrelated product contract. Existing compact-layout and fixed-window regression tests remain green.

## Security and privacy evidence

- No network endpoint, telemetry, persisted schema, privilege, target identity rule, or application-installation resource was added or changed.
- The acceptance loop connected only to the verified loopback CDP endpoint on port 9222. It did not capture chat text, account data, cookies, notifications, or input content.
- `/Applications/DoubaoWork.app` and installed 豆皮 application resources were not modified. The test bundle and profiler output remain under ignored `target/` paths.

## Deviations and residual risk

- The Plan proposed an additional loopback fake-CDP end-to-end harness. The implementation instead used deterministic private state seams for retry/deadline/`once` red-green coverage plus five real cold-start rounds through the actual desktop-to-CDP path. No fake-CDP fixture was added because the real acceptance loop more directly covered the reported symptom and a new protocol harness would expand the test surface.
- Installation status is cached for one window lifetime. If a target is installed or removed while 豆皮 is already open, its label may remain stale until restart; execution-time checks still prevent an invalid apply.
- The third cold-start round completed in 0.459 s while the other four took 8.8-9.5 s. The pre-click UI was `应用主题` and the target had been stopped before the round, but no dedicated launch-timeline trace was collected; this is retained as a timing anomaly, not evidence of failure.
- The user completed human acceptance on 2026-09-02. No release, merge, publication, or installed-app replacement was performed.

## Verdict

Passed. The human verifier explicitly accepted the live build on 2026-09-02 after the automated checks, profiler comparison, five single-click cold starts, and final `甜点偷笑 / 52% / 两个开关关闭 / 正在使用` state were presented for inspection.

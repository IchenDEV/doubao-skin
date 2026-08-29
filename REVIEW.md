# Review instructions

Review the diff against the linked `intent.md`, `spec.md`, `plan.md`, and `verification.md`, not against the PR description alone.

## Passes

1. **Intent and scope** — the change solves the stated problem, satisfies every acceptance criterion, follows the accepted plan, and does not include unrelated work.
2. **Correctness and regression** — trace changed behavior end to end, including failure paths, cleanup, generated artifacts, and the two nearest neighboring workflows.
3. **Security and privacy** — treat CDP, the protocol bridge, package installation, downloaded themes, shell commands, credentials, logs, and conversation content as sensitive surfaces. Reject broader data forwarding, non-loopback listeners, or silent weakening of validation.
4. **Product quality** — for UI and themes, require evidence from the actual macOS window at normal and narrow sizes. A mock panel, build success, or component screenshot is not native-workflow acceptance.
5. **Simplicity and evidence** — prefer a direct change in an existing module. Reject speculative layers and claims not supported by tests, command output, screenshots, or an explicit manual observation.

## Severity

- **Blocker**: data exposure, destructive behavior, release-control bypass, or a change that cannot safely ship.
- **Important**: broken behavior, credible regression, missing acceptance criterion, invalid artifact chain, or absent proof for a risky path.
- **Nit**: local clarity or consistency with no behavioral consequence. Report at most five.

## Do not report

- Formatting that `cargo fmt` owns.
- Generated catalog differences when they match their source theme and pass `pnpm --dir apps/web sync` reproducibility.
- Unrelated pre-existing worktree changes, unless this diff overwrites or depends on them.

The authoring agent cannot approve its own PR. Human approval and the configured production environment gate remain authoritative.

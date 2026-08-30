# AI-native development workflow

This repository uses a version-controlled artifact chain inspired by Anthropic's [AI-Native SDLC playbook](https://claude.com/blog/the-ai-native-sdlc-playbook). The goal is not to add paperwork. It is to preserve intent while implementation becomes faster, give agents a closed verification loop, and keep judgment and production authority with people.

The repository is the source of truth for intent, design, plan, verification, incidents, and eval cases. GitHub is the source of truth for review decisions, CI results, and releases. Every PR links the repository artifact; the artifact links back through its own history.

## State loop

```text
idea / issue / monitor
        │
        ▼
intent.md ── human product decision
        │ accepted
        ▼
 spec.md ─── human product + risk decision
        │ accepted
        ▼
 plan.md ─── human engineering decision
        │ accepted
        ▼
implementation ⇄ local checks ⇄ visual or integration proof
        │
        ▼
verification.md ── fresh-context or human verdict
        │ passed
        ▼
PR review + CI ─── human merge decision
        │
        ▼
preview / release preparation ─── production environment approval
        │
        ▼
production monitor / incident ─── new intent.md + permanent eval
```

Every Gate remains a human decision. A reviewer may edit the artifact directly, or explicitly confirm the current artifact in the active review conversation and let the agent record that decision:

```bash
./scripts/devflow approve <change-id> <intent|spec|plan> <human-approver>
```

The command fills `status`, `approved_by`, and `approved_at`; it does not decide whether an artifact should pass. Agents must not infer approval from silence, a vague acknowledgement, or their own judgment. If the reviewer identity is not already known from the current change, ask once before recording it. High- and critical-risk artifacts still require an approver other than their owner, and Git history preserves the recorded decision.

## Start a change

Use a short lowercase slug:

```bash
./scripts/devflow new improve-theme-install user medium
```

This creates only `workflow/changes/<date>-improve-theme-install/intent.md`. Fill every section. The product owner then accepts or rejects it directly or in the active review conversation. After an explicit acceptance, the agent records it with `devflow approve`.

After acceptance, create each next artifact:

```bash
./scripts/devflow design 2026-08-29-improve-theme-install
# Fill spec.md, resolve concerns, obtain explicit human approval, then record it.

./scripts/devflow plan 2026-08-29-improve-theme-install
# Fill plan.md, name exact files and proof, obtain explicit human approval, then record it.

./scripts/devflow verify 2026-08-29-improve-theme-install
# Implement, iterate, then record the evidence and verifier verdict.
```

`devflow` refuses to create a stage until its prerequisite is accepted. Run this at any time:

```bash
./scripts/devflow validate
```

## What each gate decides

| Gate | Required artifact | Human decision |
| --- | --- | --- |
| Product | `intent.md` | Is this problem real, valuable, bounded, and worth designing? |
| Design | `spec.md` | Does this solve the intent, resolve policy concerns, and define observable acceptance? |
| Engineering | `plan.md` | Is the implementation small, safe, testable, owned, and reversible? |
| Verification | `verification.md` | Do command output and real behavior prove the accepted plan? |
| Merge | PR + CI + `REVIEW.md` | Is residual risk acceptable and is the evidence trustworthy? |
| Production | GitHub `production` environment | Should this exact commit be released now? |

Risk is `low`, `medium`, `high`, or `critical`:

- **Low**: documentation, local copy, or isolated styling with an established test path.
- **Medium**: normal product behavior, theme engine changes, generated catalog changes, or reversible UI work.
- **High**: protocol bridge, CDP lifecycle, package installation, signing, network behavior, privacy boundaries, or migrations.
- **Critical**: credentials, release controls, destructive operations, or any change that could expose private user content.

## Build and test loop

The agent uses the smallest relevant loop while working and the applicable full gate before reporting completion:

```bash
./scripts/check.sh workflow
./scripts/check.sh rust
./scripts/check.sh web
./scripts/check.sh all
```

For a bug, first add a regression that fails for the observed reason. For a native UI or theme change, build success is not acceptance: launch the real app, inspect normal and narrow windows, capture screenshots, compare, adjust, and repeat. For the protocol bridge, use an isolated conversation and an exact prompt marker; verify native input, native user and assistant bubbles, incremental streaming, completion, and cleanup.

Record literal commands and concise outcomes in `verification.md`. Link screenshot paths or PR attachments. If a check is not applicable, say why. A fresh-context verifier, a paired reviewer, or a human sets the final verdict; the implementation session must not claim independent verification it did not perform.

## Pull request and deployment

The PR body must contain a path such as:

```text
workflow/changes/2026-08-29-improve-theme-install
```

CI always validates that the intent, spec, and plan are accepted. A Draft PR may keep verification `pending` while work or evidence is incomplete, but an explicit `failed` verdict still blocks it. Moving the PR to Ready reruns the gate and requires verification to be `passed`. Dependabot remains exempt from the product artifact chain but still runs code checks. Review follows `REVIEW.md`; agent findings inform the decision but never approve the PR.

Configure the GitHub repository with:

1. Pull requests required for `main`.
2. At least one human approval and dismissal of stale approvals.
3. Required `Development workflow`, `Rust workspace`, and `Web application` checks.
4. A `production` environment with a required human reviewer.
5. No direct production credentials in agent or pull-request jobs.

Vercel previews may deploy from branches. Production website deploys follow the protected merge to `main`. The macOS release workflow prepares and publishes a tag only after the `production` environment gate. The release manager verifies the artifact checksum and clean-install path using `docs/releasing.md`.

## Incidents and the maintenance loop

The scheduled `Product health` workflow deterministically checks the public gallery home, a known theme route, and the desktop download headers. It does not run a model or attempt a repair. On failure it opens one deduplicated incident issue containing the failing run.

Create the durable incident and its next intent together:

```bash
./scripts/devflow incident gallery-health monitor
```

The incident remains open until impact, timeline, containment, root cause, and follow-up are recorded. A closed incident must link both its intent and a permanent eval:

```bash
./scripts/devflow add-eval gallery-health workflow/incidents/2026-08-29-gallery-health/incident.md
```

An eval starts as `draft`. Make it `active` only when its scenario, expected behavior, automated oracle, manual oracle, and latest run are real. `./scripts/check.sh workflow` continuously exercises the deterministic workflow policy cases; product evals are run by the relevant test, visual, or integration harness named in each eval. Every escaped defect becomes a case so future changes to code or agent instructions retain the learned behavior.

## Operating metrics

Review monthly, using Git and GitHub timestamps rather than self-reported estimates:

- time from intent creation to acceptance;
- time from accepted intent to accepted spec and plan;
- first-pass CI rate and implementation-to-merge time;
- review cycles per PR and plan deviations;
- escaped incidents, time to containment, and time until a permanent eval exists;
- workflow or agent-rule regressions caught before merge.

If a metric gets worse, change the smallest responsible policy, check, template, or command, then add an eval. Do not add a new process layer without a recurring failure that justifies it.

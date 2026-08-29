# Doubao Skin agent contract

Doubao Skin is a macOS theme tool for the official DoubaoWork app. The repository contains a Rust core (`crates/skin-core`), a GPUI desktop app (`apps/desktop`), a Next.js gallery (`apps/web`), and portable packages under `themes/`.

## Start here

- Read the active change under `workflow/changes/<id>/` before editing product code.
- Run `./scripts/devflow new <slug>` when no change artifact exists. A human must explicitly accept `intent.md`, `spec.md`, and `plan.md`. After that confirmation, the agent records it with `./scripts/devflow approve <change-id> <stage> <human-approver>`; never infer or manufacture an approval from silence, a vague acknowledgement, or the agent's own judgment.
- Keep the artifact chain synchronized when implementation departs from the accepted plan.
- Use a separate worktree only for independent tasks. Work that shares files stays sequential.

## Commands

- Full local gate: `./scripts/check.sh all`
- Workflow policy: `./scripts/check.sh workflow`
- Rust: `./scripts/check.sh rust`
- Web: `pnpm --dir apps/web sync && ./scripts/check.sh web`
- Native app: `cargo run -p doubao-skin-desktop`
- Package: `./scripts/build-macos.sh`

## Boundaries

- Never modify `/Applications/DoubaoWork.app`; live mode injects through loopback CDP and offline mode patches only a clone.
- Never commit credentials, conversation content, official app resources, or assets with unclear redistribution rights.
- Keep the protocol bridge bound to loopback. Forward only the explicitly supported plain-text payload; never forward native headers, cookies, workspace data, attachments, tools, or unknown content blocks.
- Theme behavior belongs in manifests and CSS, not desktop-only branches. After any theme change, regenerate and commit the web catalog with `pnpm --dir apps/web sync`.
- Do not edit generated files under `apps/web/data` or `apps/web/public/themes` by hand.
- Do not add Python sources; CI rejects them.

## Definition of done

- Follow the accepted `plan.md`; add a regression test before fixing a reproducible bug.
- Run the smallest relevant check while iterating, then the full applicable gate before reporting completion.
- Native UI changes require real-window validation at normal and narrow sizes. Theme changes require screenshots in the actual DoubaoWork window. Protocol-bridge acceptance requires native input, native bubbles, streaming, and completion in an isolated conversation.
- Record commands, results, visual evidence, residual risk, and deviations in `verification.md`. A fresh-context verifier or human records the final verdict.
- An agent may prepare a release but may not cross the production approval gate.

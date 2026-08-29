## Change artifact

`workflow/changes/YYYY-MM-DD-short-name`

Risk: low / medium / high / critical

## Outcome

Describe the user-visible or engineering outcome.

## Plan alignment

Describe any deviation from the accepted `plan.md`, or write "None".

## Validation

- [ ] Linked `intent.md`, `spec.md`, and `plan.md` are accepted.
- [ ] Linked `verification.md` passed in a fresh context, pair, or human check.
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test --workspace --all-targets --locked`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] `corepack pnpm --dir apps/web check` when the website changed
- [ ] Real-window or screenshot validation when the native interface changed

## Scope

- [ ] The change is focused and does not include unrelated generated files.
- [ ] No credentials, private conversation content, or local `.vercel` metadata are included.
- [ ] New themes and assets include clear provenance and redistribution terms.

## Human gate

- [ ] I reviewed the intent and residual risk; agent review has not been treated as approval.

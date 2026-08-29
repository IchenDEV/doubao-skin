# Contributing

Thanks for contributing to Doubao Skin. Keep changes focused, preserve the native product workflow, and do not include private conversation content, credentials, or files copied from the official DoubaoWork application.

## Development setup

- macOS 13 or later
- Stable Rust 1.97.1 or later
- Node.js 24.20.0 LTS and pnpm 12.0.0 for the theme gallery
- An installed copy of DoubaoWork for live integration testing

```bash
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
corepack pnpm --dir apps/web install --frozen-lockfile
corepack pnpm --dir apps/web check
```

See [development.md](docs/development.md) for the complete workflow and [architecture.md](docs/architecture.md) for module boundaries.

## Change artifacts

Product work follows the versioned artifact loop in [development-workflow.md](docs/development-workflow.md). Start with an intent rather than an implementation:

```bash
./scripts/devflow new concise-change-name user medium
./scripts/devflow validate
```

Accepted intent, spec, and plan artifacts plus a passed verification record are required before a normal pull request can merge. The command scaffolds and validates artifacts but never records human approval.

## Themes and assets

Theme IDs must match their directory names. Every store theme needs a version, author, category, tags, sort order, 16:9 preview, and clear source and license metadata when it derives from another work.

After editing `themes/`, regenerate the committed website catalog:

```bash
corepack pnpm --dir apps/web sync
```

By submitting code or assets, you confirm that you have the right to contribute them and license your contribution under the license of the component you changed. Do not submit proprietary application resources, brand artwork, fonts, screenshots with private data, or assets whose redistribution terms are unclear.

## Pull requests

- Link the `workflow/changes/<id>` directory in the pull request body.
- Explain the user-visible outcome and the validation performed.
- Add or update tests for behavioral changes.
- Include real-window screenshots when changing the macOS interface.
- Keep generated website catalog changes in the same pull request as their theme source.
- Do not combine unrelated refactors with a feature or fix.

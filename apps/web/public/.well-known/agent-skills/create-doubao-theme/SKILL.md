---
name: create-doubao-theme
description: Create, migrate, refine, validate, preview, and package v3 themes for Doubao, DoubaoWork, and WorkBuddy from natural-language visual requirements.
---

# Create Doubao Theme

Turn a user's visual direction into a valid Doubao theme without requiring them to edit JSON or understand the injection runtime.

## Find the CLI

Resolve one executable before doing any work, in this order:

1. Use the exact path in `DOUBAO_THEME_CLI` when it is set.
2. Use `doubao-theme` from `PATH` when available.
3. Use `/Applications/豆皮.app/Contents/Resources/bin/doubao-theme` when it exists.
4. In this repository, use `cargo run -p skin-core --bin doubao-theme --`.

If none is available, suggest installing the standalone CLI:

```bash
curl -fsSL https://raw.githubusercontent.com/IchenDEV/doubao-skin/main/scripts/install-cli.sh | sh
```

Never substitute a handwritten archive script.

## Create a theme

1. Translate the request into a short Chinese name, one-sentence description, `#RRGGBB` accent, `light`, `dark`, or `both` appearance, and an explicit target list using `doubao`, `doubao-work`, and `workbuddy`. Use only named targets when the request is target-specific. For a generic cross-app theme, use all three and report real-window verification as pending until performed.
2. Use a new lowercase kebab-case directory. Do not delete or overwrite an existing directory.
3. If the user did not provide an author, use `本地用户` and remind them to replace it before publishing.
4. Prefer a pure-color or CSS-gradient theme. Add images, fonts, or icons only when the user supplied them, their source and license are clear, or the current host has an explicitly available generation tool.
5. Run:

   `doubao-theme create <theme-dir> --name <name> --description <description> --accent <#RRGGBB> --appearance <light|dark|both> --targets <comma-separated-targets> --author <author> --json`

6. Make only request-specific edits to the generated `theme.json` and manifest-referenced CSS. Keep shared visual semantics in `shared`, target differences in `targets.<id>`, and behavior in the theme package rather than app-specific branches. Do not add a root `theme.css` unless it is explicitly referenced.
7. Run `check`, then `preview`, then `check` again. Inspect the generated 1200 × 675 preview when visual tools are available.
8. Always finish by running `pack` with a new output path. The required delivery chain is `check → preview → check → pack`; do not report completion without a valid package. Packaging is local and does not publish or install the theme.

## Quality and safety

- Keep `schemaVersion: 3`, the directory name and `id` identical, and `targets` as the only support declaration. Every effective target/appearance must resolve complete composer and content semantics.
- Shared CSS must be scoped to `html[data-skin="<id>"]`; target CSS must also include `data-skin-target`. Use only the visual CSS subset accepted by `check`; express backgrounds, fonts, icons, layout, and motion structurally.
- Do not copy official application resources, unlicensed art, private conversation content, account information, or user workspace data into a theme.
- Do not claim a synthetic preview proves the real DoubaoWork result. Applying a theme belongs to `$apply-doubao-theme`.
- Do not publish, push, open a pull request, or modify the online catalog unless the user explicitly asks for that separate action.

## Migrate an existing v2 theme

1. Run `migrate-v3 <theme-dir> --json` first and present its target/version/resource summary without modifying files.
2. Run `migrate-v3 <theme-dir> --write --json` only when the user authorized migration of that theme or repository scope.
3. Review the result against the v3 guide. Remove obsolete root CSS, keep only manifest-referenced scoped CSS, and use `null` for target resources that must not be inherited.
4. Run the same `check → preview → check → pack` chain. Migration does not prove compatibility; record which target/appearance windows remain unverified.

## Result

Report the theme directory, validation status, preview path, package path, and any material or license still needed. Use ordinary product language, not CDP or injection terminology.

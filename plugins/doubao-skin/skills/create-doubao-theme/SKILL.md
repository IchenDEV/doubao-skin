---
name: create-doubao-theme
description: Create, refine, validate, preview, and package themes for the DoubaoWork desktop app from natural-language visual requirements.
---

# Create Doubao Theme

Turn a user's visual direction into a valid Doubao theme without requiring them to edit JSON or understand the injection runtime.

## Find the CLI

Resolve one executable before doing any work, in this order:

1. Use the exact path in `DOUBAO_THEME_CLI` when it is set.
2. Use `doubao-theme` from `PATH` when available.
3. Use `/Applications/豆包主题.app/Contents/Resources/bin/doubao-theme` when it exists.
4. In this repository, use `cargo run -p skin-core --bin doubao-theme --`.

If none is available, stop and tell the user how to build the repository. Never substitute a handwritten archive script.

## Create a theme

1. Translate the request into a short Chinese name, one-sentence description, `#RRGGBB` accent, and `light`, `dark`, or `both` appearance.
2. Use a new lowercase kebab-case directory. Do not delete or overwrite an existing directory.
3. If the user did not provide an author, use `本地用户` and remind them to replace it before publishing.
4. Prefer a pure-color or CSS-gradient theme. Add images, fonts, or icons only when the user supplied them, their source and license are clear, or the current host has an explicitly available generation tool.
5. Run:

   `doubao-theme create <theme-dir> --name <name> --description <description> --accent <#RRGGBB> --appearance <light|dark|both> --author <author> --json`

6. Make only request-specific edits to the generated `theme.json` and `theme.css`. Keep behavior in the theme package rather than app-specific branches.
7. Run `check`, then `preview`, then `check` again. Inspect the generated 1200 × 675 preview when visual tools are available.
8. Always finish by running `pack` with a new output path. The required delivery chain is `check → preview → check → pack`; do not report completion without a valid package. Packaging is local and does not publish or install the theme.

## Quality and safety

- Keep `schemaVersion: 2`, the directory name and `id` identical, and both `variants.light` and `variants.dark` when appearance is `both`.
- Keep CSS scoped to the theme's `html[data-skin]` and `body`; preserve all required semantic and primary-state variables.
- Do not copy official application resources, unlicensed art, private conversation content, account information, or user workspace data into a theme.
- Do not claim a synthetic preview proves the real DoubaoWork result. Applying a theme belongs to `$apply-doubao-theme`.
- Do not publish, push, open a pull request, or modify the online catalog unless the user explicitly asks for that separate action.

## Result

Report the theme directory, validation status, preview path, package path, and any material or license still needed. Use ordinary product language, not CDP or injection terminology.

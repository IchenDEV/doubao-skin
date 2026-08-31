# Website deployment

The recommended production path is Vercel's GitHub integration. GitHub Actions validates the website; Vercel owns preview and production deployments so the same commit is not deployed twice.

## One-time setup

1. Push the repository to GitHub and import it in Vercel.
2. Set the Vercel project Root Directory to `apps/web`.
3. Select the Next.js framework preset and Node.js 24.
4. Because this app pins pnpm 12, keep the install command as `corepack pnpm install --frozen-lockfile` and the build command as `corepack pnpm run build`. Both are committed in `vercel.json`, so Git deployments do not fall back to Vercel's legacy pnpm shim. Keep `ENABLE_EXPERIMENTAL_COREPACK=1` enabled for Preview and Production.
5. Set the Production Branch to `main`.
6. Repository links are fixed to `https://github.com/IchenDEV/doubao-skin`.
7. The desktop download is fixed to `https://github.com/IchenDEV/doubao-skin/releases/latest/download/Doubao-Skin-macOS-universal.zip`. Keep the large desktop archive on GitHub Releases rather than Vercel; the website does not read an environment-variable override.
8. Add `doubao-skin.idevlab.dev` to the project. With Cloudflare authoritative DNS, use the Vercel-recommended `A doubao-skin 76.76.21.21` record in DNS-only mode.

The local `.vercel` directory contains machine-specific project binding data and is intentionally ignored.

## Normal deployment flow

```text
feature branch push ──▶ GitHub CI ──▶ Vercel Preview URL
merge to main ─────────▶ GitHub CI ──▶ Vercel Production deployment
version tag ───────────▶ macOS Release workflow ──▶ GitHub Release asset
```

The scheduled `Product health` workflow checks the public home page, a known theme detail route, and the desktop download. A failure opens one deduplicated incident issue; diagnosis and repair still go through the normal incident → intent → review loop.

The website always resolves its download button to `Doubao-Skin-macOS-universal.zip` in the latest GitHub Release.

## Direct deployment from the linked workspace

Use this path only when the repository has no GitHub remote or Release yet. Confirm `apps/web/.vercel/project.json` points to `doubao-skin-gallery`, configure `ENABLE_EXPERIMENTAL_COREPACK=1` for both Preview and Production, and set the project install command to `corepack pnpm install --frozen-lockfile`. The website may point at the future GitHub Release asset before that asset exists; do not copy the desktop ZIP into Vercel as a workaround. Then deploy from `apps/web`:

```bash
vercel deploy . --target preview -y
vercel deploy . --prod -y
```

The Preview must serve `/`, `/guide`, `/contribute`, `/robots.txt`, `/sitemap.xml`, `/themes/catalog.json`, one theme detail, one preview image, and one theme package before Production is deployed. The guide's desktop link must resolve to the exact GitHub Latest Release URL; the asset itself is verified during the release workflow, not served by Preview. Attach the custom domain only to the validated project, and change DNS only after the new Production deployment passes the same checks on its Vercel alias.

## Theme changes

Vercel builds from the generated catalog committed under `apps/web`. Before pushing a theme change, run:

```bash
corepack pnpm --dir apps/web install --frozen-lockfile
corepack pnpm --dir apps/web sync
corepack pnpm --dir apps/web check
```

Commit the source theme and generated database, previews, catalog, and theme packages together. This keeps Vercel builds reproducible without reaching outside the configured app root.

## Launch checklist

- CI is green for the production commit.
- The Vercel project uses `apps/web` as its root.
- Preview and Production link to the public GitHub repository and exact universal Release asset.
- The website detects macOS or Windows locally, recommends the matching desktop asset, and keeps all architectures manually selectable.
- The CLI section remains visibly separate from desktop downloads and links Windows to the Scoop manifest while macOS/Linux use the platform-detecting installer.
- Home, guide, contribution, theme detail, and theme-package links work on Preview.
- `robots.txt`, `sitemap.xml`, canonical metadata and structured data use `https://doubao-skin.idevlab.dev`.
- `/themes/catalog.json` returns schema version 1 and a non-empty theme array.
- The custom domain is added only after the Preview deployment passes acceptance, and DNS changes only after Production passes.

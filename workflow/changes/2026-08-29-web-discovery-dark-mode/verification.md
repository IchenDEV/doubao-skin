---
id: "2026-08-29-web-discovery-dark-mode"
stage: verification
status: pending
owner: "codex"
created: "2026-08-29"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: "Averroes (fresh-context verifier)"
verified_at: "2026-08-29"
---

# Verification: web discovery dark mode

## Automated checks

- Final `./scripts/check.sh all` passed workflow, Rust, Clippy, website tests, TypeScript, production build, and dependency audit. Its website section confirmed:
  - 4 Node filter regressions cover type/series AND composition, search, invalid values, legacy `view`, and stable links.
  - TypeScript and the Next.js 16.3.3 production build passed with 34 routes: three top-level pages, 26 theme details, robots, sitemap, icon, and not-found.
  - `pnpm audit --audit-level=high` reported no known vulnerabilities.
- `./scripts/check.sh workflow` passed all 9 artifact sets and approval-policy tests.
- Two consecutive `corepack pnpm --dir apps/web sync` runs each generated 26 themes and 26 installable packages. The combined SHA-256 manifest for all 139 generated files remained `63889cfcbe157223f870c7ecd15c36dc4b52c6dc028288235abd1206e93d7574`.
- Vercel Preview `https://doubao-skin-gallery-jey1xuana-ichendevs-projects.vercel.app` (`dpl_GJnKwJDTpXQATEQvN8fYKDz5A4Ak`) built successfully on Node 24 and reached `READY`.
- Protected Preview checks through `vercel curl` returned the correct status and MIME type for `/`, `/guide`, `/contribute`, `/robots.txt`, `/sitemap.xml`, `/themes/catalog.json`, a theme package, and a preview image.
- Preview catalog checks found schema version 1 and 26 themes. The `peach-sunset` package was a valid ZIP and its actual SHA-256 matched the catalog value `55b917389887358efa3f7fb814c9a1153f48307cab471fd4e21286d26298bcd8`.
- The sitemap contains 29 locations: the three top-level pages plus all 26 theme details. Canonical, Open Graph, Twitter, WebSite, CollectionPage/ItemList, WebPage, and CreativeWork metadata use `https://doubao-skin.idevlab.dev`.
- Built and deployed HTML contains no arm64 desktop URL or `NEXT_PUBLIC_APP_DOWNLOAD_URL`; the guide uses the exact GitHub Latest Release universal asset.
- `.github/workflows/health.yml` parses as YAML and now checks all three top-level pages, one detail, robots, sitemap, the catalog, and the fixed GitHub universal Release asset instead of the retired in-site arm64 ZIP.

## Behavioral evidence

- The desktop theme library renders one left sidebar with two independent facets. Selecting `有背景` and `图库` produced URL `?type=background&series=gallery`, result count `4 套`, four articles, and exactly one `#theme-filter-panel`.
- Search narrows the active facet result. Legacy `?view=codex` remains readable while new links emit only `type` and `series`.
- At 390 × 844, the same single filter panel is disclosed by one `筛选` button; all three top-level navigation links remain visible and the document has no horizontal overflow.
- Desktop and 390px measurements confirmed every facet option, mobile filter trigger, and compact top-level navigation target is 44px high.
- `/guide` exposes one exact download link to `Doubao-Skin-macOS-universal.zip`, labels it `macOS 通用版`, keeps Windows non-clickable as `Coming Soon`, and explains both repository Skills.
- `/contribute` contains the accepted five-step flow—create, preview/check, sync website catalog, Fork/branch/push, Pull Request—explicitly says the website does not accept direct uploads, and links to the full GitHub submission document. The full document instructs creators to run `corepack pnpm --dir apps/web sync` and commit the theme-specific generated files without hand-editing them.
- Theme detail actions lead to `/guide#download`; the home page no longer duplicates the long usage/install guide at the bottom.
- The external Chromium browser reported no warning or error console entries on the tested home, guide, and contribution routes.
- The final theme detail HTML contains page-specific Twitter metadata plus `CreativeWork.genre` matching its visible category; canonical and social URLs remain fixed to the production domain on Preview.

## Visual evidence

- The final protected Preview above was inspected in the user-connected external browser at the normal 1982 × 1159 viewport in both system dark and emulated light modes. Sidebar, search, list rows, type labels, actions, and preview images remained legible without bright dark-mode islands; all 26 theme images completed with non-zero natural width.
- The same Preview was inspected at 390 × 844. The header, collapsed filter panel, theme rows, guide download section, Windows status, and reading layout fit without horizontal scrolling.
- Browser screenshots were emitted into the implementation task for desktop dark, desktop light, mobile library/filter, and mobile guide states. They were not persisted into the repository because the Preview is protected and no product asset depends on them.

## Security and privacy evidence

- Public site, repository, contribution document, and desktop download URLs are fixed constants. Stale Vercel environment variables cannot redirect the public download button.
- The desktop archive remains on GitHub Releases; Vercel serves the website and theme packages but not the large desktop binary.
- JSON-LD is serialized by the narrow `JsonLd` component and escapes `<`; structured data contains only public theme catalog fields and visible product facts.
- Browser QA read public Preview UI and metadata only. It did not enter credentials, upload files, or transmit private user data.
- This implementation deployed Preview only. It did not promote the candidate to Production or change the custom-domain alias.

## Deviations and residual risk

- Vercel Deployment Protection redirects anonymous Preview requests to login. HTTP acceptance used `vercel curl`'s authenticated protection bypass; the user-connected signed-in external browser provided visual acceptance. Production must remain publicly accessible when separately approved.
- Vercel CLI 50.42.0 printed a non-fatal local `pnpm-lock.yaml` parse warning before upload. The cloud install, pnpm supply-chain check, Next build, deployment, and protected HTTP smoke all completed successfully.
- The first full gate run encountered one transient missing terminal event in an unrelated protocol-bridge loopback test. That exact test passed immediately in isolation, and the complete `./scripts/check.sh all` rerun passed all 31 core tests and every remaining gate.
- The GitHub repository and Latest Release asset are future public targets chosen by the user. The exact URL is assembled and verified in rendered HTML, but remote Release availability remains pending until the user uploads/publishes it.
- For the same reason, the scheduled health workflow's GitHub download probe cannot pass until that first universal Release asset exists; the workflow contract is configured but not claimed as remotely verified.
- Production promotion to `doubao-skin.idevlab.dev` is intentionally pending a new explicit approval under the accepted Plan.
- The worktree is uncommitted and contains unrelated concurrent changes. Verification is scoped to the files and deployment above; a commit anchor is still required before marking this artifact passed.
- After the final 26-theme gate and Preview deployment, an independent concurrent task added four new source directories: claude-warm, github-repository, huaxia-blue, and machine-overseer. They are intentionally absent from this change's 26-theme generated catalog and Preview; their owning task must sync and verify them before any later Production promotion.

## Verdict

Fresh-context verdict: **PASS WITH RESIDUALS**. The implementation and Preview satisfy this accepted Spec; frontmatter remains pending because Production promotion, the custom-domain recheck, repository/Release publication, commit anchoring, and the independent four-theme sync are separate outstanding gates.

---
id: "2026-08-29-deploy-online-theme-store"
stage: verification
status: pending
owner: "codex"
created: "2026-08-29"
based_on: plan.md
commit: ""
verification_mode: "fresh-context"
verified_by: ""
verified_at: ""
---

# Verification: deploy online theme store

## Automated checks

- Two consecutive theme syncs generated 26 schema-v1 entries and 26 packages without second-run drift; catalog package sizes and SHA-256 values matched the generated files.
- `./scripts/check.sh web`, `./scripts/check.sh rust`, and `./scripts/check.sh workflow` passed after the generator and public-store changes.
- Vercel Production deployment `dpl_9PJAPtWHRfKKPZX8hfm1Mcvpsb44` reached Ready and owned the platform alias `https://doubao-skin-gallery.vercel.app` plus custom domain `https://doubao-skin.idevlab.dev`.
- Public HTTPS checks returned the theme library, a theme detail, schema-v1 catalog, previews, theme packages, and the then-current arm64 desktop archive without authentication.
- The custom domain resolved through Cloudflare's DNS-only `A doubao-skin 76.76.21.21` record and served a valid Vercel TLS endpoint.

## Behavioral evidence

- The public home page displayed all 26 themes and working detail/package links.
- The local GPUI desktop app, without `DOUBAO_SKIN_THEME_STORE_URL`, opened the online store from the custom-domain catalog and displayed 26 remote themes with thumbnails.
- The Rust default store URL is `https://doubao-skin.idevlab.dev/themes/catalog.json`; the existing environment override remains covered by tests.

## Visual evidence

- The custom-domain website was inspected in the user-connected external browser at desktop and narrow widths during the deployment run.
- The native desktop online-store view was inspected at normal and narrow window sizes and showed the remote catalog without layout breakage.
- No screenshots containing private official-app conversation titles were persisted.

## Security and privacy evidence

- Vercel received only the `apps/web` deployment payload; credentials, cookies, conversations, workspace data, and desktop runtime data were not uploaded.
- Cloudflare scope was limited to the exact `doubao-skin` A record and kept DNS-only. Root DNS, nameservers, and unrelated records were untouched.
- Theme downloads retain HTTP(S), size, hash, path-traversal, and symlink checks in `skin-core`.

## Deviations and residual risk

- This artifact records the successful original production/custom-domain deployment. The later accepted `2026-08-29-web-discovery-dark-mode` Spec intentionally supersedes its arm64 in-site download contract with a fixed GitHub universal Release URL.
- The current production domain still serves the earlier accepted release until the newer Preview receives a separate Production approval. The historical deployment itself remains healthy; its product copy is not the final candidate described by the newer artifact.
- The worktree has no commit anchor and this historical deployment artifact has not received its own fresh-context verdict.

## Verdict

The production deployment, DNS, HTTPS, public theme catalog, and local online-store connection were behaviorally verified. Status remains pending a fresh-context or product-owner verdict and is superseded for future download/IA behavior by the newer web artifact.

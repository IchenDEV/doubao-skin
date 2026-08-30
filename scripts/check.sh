#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

check_workflow() {
  bash -n scripts/devflow scripts/check.sh scripts/install-cli.sh scripts/package.sh \
    scripts/checks/*.sh scripts/package/*.sh
  node --check scripts/package/generate-scoop-manifest.mjs
  ./scripts/devflow validate
  ./scripts/checks/devflow.sh
  ./scripts/checks/portability.sh
}

check_rust() {
  cargo fmt --all -- --check
  cargo test --workspace --all-targets --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
}

check_web() {
  (cd apps/web && corepack pnpm check)
  (cd apps/web && corepack pnpm audit --audit-level=high)
  node --check apps/web/scripts/sync-themes.mjs
  node --check apps/web/scripts/sync-skills.mjs
}

case "${1:-all}" in
  workflow) check_workflow ;;
  rust) check_rust ;;
  web) check_web ;;
  all)
    check_workflow
    check_rust
    check_web
    ;;
  *)
    echo "Usage: ./scripts/check.sh [workflow|rust|web|all]" >&2
    exit 1
    ;;
esac

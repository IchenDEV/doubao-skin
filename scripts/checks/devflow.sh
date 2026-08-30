#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
DEVFLOW="$REPO_ROOT/scripts/devflow"
TEST_ROOT=$(mktemp -d)
TEST_WORKFLOW="$TEST_ROOT/workflow"
mkdir -p "$TEST_WORKFLOW/changes" "$TEST_WORKFLOW/incidents" "$TEST_WORKFLOW/evals"

cleanup() {
  find "$TEST_ROOT" -type f -delete 2>/dev/null || true
  find "$TEST_ROOT" -depth -type d -exec rmdir {} \; 2>/dev/null || true
}
trap cleanup EXIT

flow() {
  DEVFLOW_DATE=2026-01-02 DEVFLOW_WORKFLOW_DIR="$TEST_WORKFLOW" "$DEVFLOW" "$@"
}

replace() {
  file=$1
  before=$2
  after=$3
  temporary="$file.tmp"
  sed "s|$before|$after|g" "$file" > "$temporary"
  mv "$temporary" "$file"
}

expect_failure() {
  label=$1
  shift
  if "$@" >/dev/null 2>&1; then
    echo "test-devflow: expected failure: $label" >&2
    exit 1
  fi
}

fill_artifact() {
  file=$1
  replace "$file" '\[fill\]' 'fixture evidence'
  replace "$file" 'owner: "fixture evidence"' 'owner: "builder"'
}

flow new native-stream user low >/dev/null
change_id=2026-01-02-native-stream
change_dir="$TEST_WORKFLOW/changes/$change_id"
flow validate >/dev/null
expect_failure "design before intent approval" flow design "$change_id"

intent="$change_dir/intent.md"
expect_failure "approval before artifact is complete" flow approve "$change_id" intent product-owner
fill_artifact "$intent"
expect_failure "unsafe approver identity" flow approve "$change_id" intent 'bad"approver'
flow approve "$change_id" intent product-owner >/dev/null
flow design "$change_id" >/dev/null

spec="$change_dir/spec.md"
fill_artifact "$spec"
flow approve "$change_id" spec product-owner >/dev/null
flow plan "$change_id" >/dev/null

plan="$change_dir/plan.md"
fill_artifact "$plan"
flow approve "$change_id" plan tech-lead >/dev/null
flow verify "$change_id" >/dev/null

verification="$change_dir/verification.md"
fill_artifact "$verification"
replace "$verification" 'status: pending' 'status: passed'
replace "$verification" 'commit: ""' 'commit: "abc1234"'
replace "$verification" 'verified_by: ""' 'verified_by: "builder"'
replace "$verification" 'verified_at: ""' 'verified_at: "2026-01-02"'
expect_failure "fresh verifier cannot be implementation owner" flow validate
replace "$verification" 'verified_by: "builder"' 'verified_by: "fresh-verifier"'
flow validate >/dev/null

PR_BODY="Change artifact: workflow/changes/$change_id" DEVFLOW_DATE=2026-01-02 DEVFLOW_WORKFLOW_DIR="$TEST_WORKFLOW" "$DEVFLOW" check-pr >/dev/null
expect_failure "PR without linked artifact" env PR_BODY="No artifact" DEVFLOW_DATE=2026-01-02 DEVFLOW_WORKFLOW_DIR="$TEST_WORKFLOW" "$DEVFLOW" check-pr

flow incident gallery-health monitor >/dev/null
incident="$TEST_WORKFLOW/incidents/2026-01-02-gallery-health/incident.md"
incident_intent="$TEST_WORKFLOW/changes/2026-01-02-gallery-health/intent.md"
fill_artifact "$incident_intent"
expect_failure "high-risk intent requires independent approval" flow approve 2026-01-02-gallery-health intent builder
flow approve 2026-01-02-gallery-health intent incident-owner >/dev/null
fill_artifact "$incident"
replace "$incident" 'status: open' 'status: closed'
replace "$incident" 'closed_at: ""' 'closed_at: "2026-01-02"'
expect_failure "closed incident without regression eval" flow validate

flow add-eval gallery-health incident >/dev/null
eval_file="$TEST_WORKFLOW/evals/2026-01-02-gallery-health.md"
fill_artifact "$eval_file"
replace "$eval_file" 'status: draft' 'status: active'
replace "$eval_file" 'last_run: ""' 'last_run: "2026-01-02"'
replace "$incident" 'regression_eval: ""' 'regression_eval: "workflow/evals/2026-01-02-gallery-health.md"'
flow validate >/dev/null

echo "test-devflow: approval and policy cases passed"

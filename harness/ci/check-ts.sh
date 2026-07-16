#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$REPO_ROOT/harness/ci/advisory.sh"
cd "$REPO_ROOT/ts"

echo "==> pnpm install --frozen-lockfile"
pnpm install --frozen-lockfile

echo "==> TypeScript source shape guard"
run_advisory \
  "typescript-source-shape" \
  "owning TypeScript lane" \
  "record the hotspot and split candidate; build, lint, and import guards remain blocking" \
  node "$REPO_ROOT/harness/depgraph/check-ts-source-shape.mjs" "$REPO_ROOT"
run_advisory \
  "typescript-source-shape-policy" \
  "owning TypeScript lane" \
  "review the structural-pressure baseline without treating it as architecture truth" \
  node "$REPO_ROOT/harness/depgraph/check-ts-source-shape-policy-diff.mjs" "$REPO_ROOT"
if [[ "${ASHA_HARNESS_SELF_TESTS:-1}" == "1" ]]; then
  run_advisory \
    "typescript-source-shape-self-tests" \
    "architecture stewardship" \
    "repair or merge the changed advisory validator fixtures" \
    node "$REPO_ROOT/harness/depgraph/check-ts-source-shape-policy-fixtures.mjs" "$REPO_ROOT"
fi

echo "==> pnpm -r build (untracked workspace outputs)"
pnpm -r build

echo "==> pnpm -r typecheck"
pnpm -r typecheck

echo "==> pnpm -r test"
cd "$REPO_ROOT"
python3 "$REPO_ROOT/harness/identity/execution.py" \
  --execution ts.workspace.tests \
  --attribution gate.typescript-workspace
cd "$REPO_ROOT/ts"

echo "==> pnpm lint"
pnpm lint

echo "==> policy sandbox negative smoke"
if [[ "${ASHA_HARNESS_SELF_TESTS:-1}" == "1" ]]; then
  bash "$REPO_ROOT/harness/lint/ts-eslint/policy-sandbox-smoke.sh"

  echo "==> type-aware lint negative smoke"
  bash "$REPO_ROOT/harness/lint/ts-eslint/type-aware-smoke.sh"
fi

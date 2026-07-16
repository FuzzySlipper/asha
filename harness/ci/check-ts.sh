#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT/ts"

echo "==> pnpm install --frozen-lockfile"
pnpm install --frozen-lockfile

echo "==> TypeScript source shape guard"
node "$REPO_ROOT/harness/depgraph/check-ts-source-shape.mjs" "$REPO_ROOT"
node "$REPO_ROOT/harness/depgraph/check-ts-source-shape-policy-diff.mjs" "$REPO_ROOT"
if [[ "${ASHA_HARNESS_SELF_TESTS:-1}" == "1" ]]; then
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

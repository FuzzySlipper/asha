#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$REPO_ROOT/harness/ci/advisory.sh"

echo "==> Verifying Rust dependency graph"
bash "$REPO_ROOT/harness/depgraph/verify-rust-deps.sh"

echo "==> Rust source shape guard"
run_advisory \
  "rust-source-shape" \
  "owning Rust lane" \
  "record the hotspot and split candidate; dependency and compile guards remain blocking" \
  node "$REPO_ROOT/harness/depgraph/check-rust-source-shape.mjs" "$REPO_ROOT"
run_advisory \
  "rust-source-shape-policy" \
  "owning Rust lane" \
  "review the structural-pressure baseline without treating it as architecture truth" \
  node "$REPO_ROOT/harness/depgraph/check-rust-source-shape-policy-diff.mjs" "$REPO_ROOT"

if [[ "${ASHA_HARNESS_SELF_TESTS:-1}" == "1" ]]; then
  echo "==> Rust source shape fixtures"
  run_advisory \
    "rust-source-shape-self-tests" \
    "architecture stewardship" \
    "repair or merge the changed advisory validator fixtures" \
    node "$REPO_ROOT/harness/depgraph/check-rust-source-shape-fixtures.mjs" "$REPO_ROOT"
  run_advisory \
    "rust-source-shape-policy-self-tests" \
    "architecture stewardship" \
    "repair or merge the changed advisory policy fixtures" \
    node "$REPO_ROOT/harness/depgraph/check-rust-source-shape-policy-fixtures.mjs" "$REPO_ROOT"
fi

echo "==> Verifying TypeScript dependency graph"
bash "$REPO_ROOT/harness/depgraph/verify-ts-deps.sh"

echo "==> Committed path classification"
python3 "$REPO_ROOT/harness/depgraph/check-committed-path-classification.py"
if [[ "${ASHA_HARNESS_SELF_TESTS:-1}" == "1" ]]; then
  python3 "$REPO_ROOT/harness/depgraph/check-committed-path-classification-fixtures.py"
fi

echo "==> Runtime bridge root isolation"
node "$REPO_ROOT/harness/depgraph/check-runtime-bridge-root-isolation.mjs" "$REPO_ROOT"

echo "==> Checking generated TypeScript ESLint boundary config"
python3 "$REPO_ROOT/harness/depgraph/generate-ts-eslint-boundaries.py" --check

echo "==> Checking Agent Code Atlas inventory"
run_advisory \
  "agent-code-atlas" \
  "architecture documentation stewardship" \
  "refresh useful navigation or retire ambient inventory churn" \
  python3 "$REPO_ROOT/harness/code-map/check-agent-code-atlas.py" --check
if [[ "${ASHA_HARNESS_SELF_TESTS:-1}" == "1" ]]; then
  run_advisory \
    "agent-code-atlas-self-tests" \
    "architecture documentation stewardship" \
    "repair the changed advisory inventory validator" \
    python3 "$REPO_ROOT/harness/code-map/check-agent-code-atlas-fixtures.py"
fi

echo "==> Checking generated README workspace counts"
run_advisory \
  "readme-workspace-counts" \
  "architecture documentation stewardship" \
  "refresh the navigation count or remove it if it no longer helps routing" \
  python3 "$REPO_ROOT/harness/code-map/check-readme-workspace-counts.py" --check

if [[ "${ASHA_HARNESS_SELF_TESTS:-1}" == "1" ]]; then
  echo "==> Running README workspace-count fixtures"
  run_advisory \
    "readme-workspace-count-self-tests" \
    "architecture documentation stewardship" \
    "repair the changed advisory count validator" \
    python3 "$REPO_ROOT/harness/code-map/check-readme-workspace-counts-fixtures.py"

  echo "==> Running depgraph negative fixtures"
  bash "$REPO_ROOT/harness/depgraph/check-negative-fixtures.sh"

  echo "==> Smoke-testing TypeScript package generator"
  bash "$REPO_ROOT/harness/depgraph/check-package-generator.sh"
fi

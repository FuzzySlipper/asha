#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "==> Verifying Rust dependency graph"
bash "$REPO_ROOT/harness/depgraph/verify-rust-deps.sh"

echo "==> Verifying TypeScript dependency graph"
bash "$REPO_ROOT/harness/depgraph/verify-ts-deps.sh"

echo "==> Checking generated TypeScript ESLint boundary config"
python3 "$REPO_ROOT/harness/depgraph/generate-ts-eslint-boundaries.py" --check

echo "==> Running depgraph negative fixtures"
bash "$REPO_ROOT/harness/depgraph/check-negative-fixtures.sh"

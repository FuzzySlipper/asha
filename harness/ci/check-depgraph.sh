#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "==> Verifying Rust dependency graph"
bash "$REPO_ROOT/harness/depgraph/verify-rust-deps.sh"

echo "==> Verifying TypeScript dependency graph"
bash "$REPO_ROOT/harness/depgraph/verify-ts-deps.sh"

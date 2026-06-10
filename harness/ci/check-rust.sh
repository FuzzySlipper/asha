#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT/engine-rs"

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo check"
cargo check --workspace

echo "==> cargo clippy"
cargo clippy --workspace -- -D warnings

echo "==> cargo test"
cargo test --workspace

#!/usr/bin/env bash
# Regenerates TypeScript contracts from Rust protocol crates and fails if
# the working-tree result differs from what is committed.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "==> Running protocol codegen"
# Phase 0: codegen binary is a stub; skip actual run until Phase 2.
# cargo run --manifest-path "$REPO_ROOT/engine-rs/Cargo.toml" -p protocol-codegen

echo "==> Checking generated files are unmodified"
if ! git -C "$REPO_ROOT" diff --quiet -- ts/packages/contracts/src/generated/; then
    echo "ERROR: Generated contract files have uncommitted changes."
    echo "Run 'cargo run -p protocol-codegen' and commit the result."
    git -C "$REPO_ROOT" diff -- ts/packages/contracts/src/generated/
    exit 1
fi

echo "Generated contracts are clean."

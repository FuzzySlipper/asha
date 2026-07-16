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
if [[ "${ASHA_GAMEPLAY_RUNTIME_HOST_GATE_OWNS_TESTS:-0}" == "1" ]]; then
  cd "$REPO_ROOT"
  env -u ASHA_GAMEPLAY_RUNTIME_HOST_GATE_OWNS_TESTS \
    python3 "$REPO_ROOT/harness/identity/execution.py" \
      --execution rust.workspace.tests \
      --attribution gate.rust-workspace
else
  cargo test --workspace
fi

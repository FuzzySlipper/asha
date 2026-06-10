#!/usr/bin/env bash
# Verifies the committed TypeScript contracts under
# ts/packages/contracts/src/generated/ exactly match what protocol-codegen
# produces from the Rust protocol crates. Catches two kinds of drift:
#   1. a Rust protocol-source change that was not regenerated, and
#   2. a manual edit to a generated file.
#
# The generator's --check mode compares its deterministic output against the
# files on disk and prints a source-pointing message for any mismatch; it never
# mutates the tree.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "==> Verifying generated TypeScript contracts match Rust protocol source"
if ! cargo run --quiet --manifest-path "$REPO_ROOT/engine-rs/Cargo.toml" \
        -p protocol-codegen -- --check; then
    cat <<'EOF'

Generated contracts are out of sync with the Rust protocol source.
Resolve by choosing the cause of the drift:
  - Border shape really changed: edit the Rust protocol crate(s) under
    engine-rs/crates/protocol/*, then rerun codegen (next step).
  - Source already changed but contracts are stale, OR a generated file was
    hand-edited: run `cargo run -p protocol-codegen` to regenerate, then commit
    the generated diff alongside the source change.
  - The new output is the intended golden: commit it (and any protocol golden
    fixtures) so it becomes the new baseline.

See governance/contract-change-process.md for the full loop.
EOF
    exit 1
fi

# Note: --check compares the generator's deterministic output against the files
# on disk. In CI the working tree is the checked-out commit, so this validates
# exactly the committed contracts — no separate `git diff` guard is needed.

echo "Generated contracts are in sync with the Rust protocol source."

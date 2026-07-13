#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cargo test --locked --offline --manifest-path "$ROOT/engine-rs/Cargo.toml" -p rule-trigger-volume
cargo test --locked --offline --manifest-path "$ROOT/engine-rs/Cargo.toml" -p rule-gameplay-fabric --test owner_events
cargo test --locked --offline --manifest-path "$ROOT/engine-rs/Cargo.toml" -p rule-gameplay-fabric --test reads
cargo test --locked --offline --manifest-path "$ROOT/engine-rs/Cargo.toml" -p rule-project-bundle --test gameplay_bindings
python3 "$ROOT/harness/identity/execution.py" \
  --execution rust.downstream-gameplay-module \
  --attribution gate.trigger-volumes.downstream-module

echo "Trigger-volume conformance passed."

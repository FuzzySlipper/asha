#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cargo test --locked --offline --manifest-path "$ROOT/engine-rs/Cargo.toml" -p rule-trigger-volume
cargo test --locked --offline --manifest-path "$ROOT/engine-rs/Cargo.toml" -p rule-gameplay-fabric --test owner_events
cargo test --locked --offline --manifest-path "$ROOT/engine-rs/Cargo.toml" -p rule-gameplay-fabric --test reads
cargo test --locked --offline --manifest-path "$ROOT/engine-rs/Cargo.toml" -p rule-project-bundle --test gameplay_bindings
cargo test --locked --offline --manifest-path "$ROOT/harness/fixtures/gameplay-module-sdk/downstream-module/Cargo.toml" -- \
  --skip public_static_runtime_provider_lifecycle_releases_each_isolated_cell

echo "Trigger-volume conformance passed."

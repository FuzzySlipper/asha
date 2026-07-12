#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cargo test --locked --offline --manifest-path "$ROOT/engine-rs/Cargo.toml" -p gameplay-runtime-host
cargo test --locked --offline --manifest-path "$ROOT/harness/fixtures/gameplay-module-sdk/downstream-module/Cargo.toml"
pnpm --dir "$ROOT/ts" --filter @asha/runtime-session test
pnpm --dir "$ROOT/ts" --filter @asha/runtime-bridge test

#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FIXTURE="$REPO_ROOT/harness/fixtures/gameplay-module-sdk/downstream-module"
SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

echo "==> Checking public gameplay-module facade"
if rg -n --fixed-strings "engine-rs/crates" "$FIXTURE/Cargo.toml"; then
  echo "Downstream gameplay-module fixture must depend only on the public facade." >&2
  exit 1
fi
cargo test --locked --offline --manifest-path "$FIXTURE/Cargo.toml" -- \
  --skip public_static_runtime_provider_lifecycle_releases_each_isolated_cell

echo "==> Checking gameplay-module scaffold"
"$REPO_ROOT/harness/tools/new-gameplay-module.sh" \
  "$SCRATCH/scaffolded-module" \
  "scaffolded-gameplay-module" \
  "fixture.scaffolded.module"

echo "Gameplay-module SDK public fixture and scaffold passed."

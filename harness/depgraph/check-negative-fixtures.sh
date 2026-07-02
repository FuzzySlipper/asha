#!/usr/bin/env bash
# Negative depgraph smoke tests. These build tiny throwaway workspaces under
# /tmp/asha so the real repo never has to carry intentionally broken packages.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
mkdir -p /tmp/asha
TMP_ROOT="$(mktemp -d /tmp/asha/depgraph-negative.XXXXXX)"
trap 'rm -rf "$TMP_ROOT"' EXIT

expect_failure() {
  local label="$1"
  local expected="$2"
  shift 2

  set +e
  local output
  output="$("$@" 2>&1)"
  local status=$?
  set -e

  if [[ "$status" -eq 0 ]]; then
    printf 'FAIL: %s unexpectedly passed\n' "$label"
    printf '%s\n' "$output"
    exit 1
  fi
  if [[ "$output" != *"$expected"* ]]; then
    printf 'FAIL: %s did not mention expected text: %s\n' "$label" "$expected"
    printf '%s\n' "$output"
    exit 1
  fi
  printf 'negative fixture OK: %s\n' "$label"
}

make_rust_fixture() {
  local root="$1"
  mkdir -p \
    "$root/governance" \
    "$root/engine-rs/crates/foundation/core-a/src" \
    "$root/engine-rs/crates/foundation/core-b/src"
  printf '[workspace]\nmembers = ["crates/foundation/core-a", "crates/foundation/core-b"]\nresolver = "2"\n' > "$root/engine-rs/Cargo.toml"
  printf '[package]\nname = "core-a"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\ncore-b = { path = "../core-b" }\n' > "$root/engine-rs/crates/foundation/core-a/Cargo.toml"
  printf 'pub fn a() {}\n' > "$root/engine-rs/crates/foundation/core-a/src/lib.rs"
  printf '[package]\nname = "core-b"\nversion = "0.1.0"\nedition = "2021"\n' > "$root/engine-rs/crates/foundation/core-b/Cargo.toml"
  printf 'pub fn b() {}\n' > "$root/engine-rs/crates/foundation/core-b/src/lib.rs"
  printf '[crate."engine-rs/crates/foundation/core-a"]\nlane = "rust-foundation"\nmay_depend_on = []\n\n[crate."engine-rs/crates/foundation/core-b"]\nlane = "rust-foundation"\nmay_depend_on = []\n' > "$root/governance/ownership.toml"
}

make_ts_unlisted_fixture() {
  local root="$1"
  mkdir -p "$root/governance" "$root/ts/packages/app/src" "$root/ts/packages/contracts/src"
  printf '{"name":"@asha/app","dependencies":{"@asha/contracts":"workspace:*"}}\n' > "$root/ts/packages/app/package.json"
  printf "import '@asha/contracts';\n" > "$root/ts/packages/app/src/index.ts"
  printf '{"name":"@asha/contracts"}\n' > "$root/ts/packages/contracts/package.json"
  printf 'export {};\n' > "$root/ts/packages/contracts/src/index.ts"
  printf '[package."ts/packages/app"]\nlane = "ts-shell"\nmay_import = []\n\n[package."ts/packages/contracts"]\nlane = "contract-steward"\nmay_import = []\n' > "$root/governance/ownership.toml"
}

make_ts_missing_ownership_fixture() {
  local root="$1"
  mkdir -p "$root/governance" "$root/ts/packages/app/src"
  printf '{"name":"@asha/app"}\n' > "$root/ts/packages/app/package.json"
  printf 'export {};\n' > "$root/ts/packages/app/src/index.ts"
  printf '' > "$root/governance/ownership.toml"
}

RUST_FIXTURE="$TMP_ROOT/rust-unlisted"
make_rust_fixture "$RUST_FIXTURE"
expect_failure \
  "unlisted Rust internal dependency" \
  "depends on unlisted internal crate 'core-b'" \
  bash "$REPO_ROOT/harness/depgraph/verify-rust-deps.sh" "$RUST_FIXTURE"

TS_UNLISTED_FIXTURE="$TMP_ROOT/ts-unlisted"
make_ts_unlisted_fixture "$TS_UNLISTED_FIXTURE"
expect_failure \
  "unlisted TypeScript internal import" \
  "imports unlisted internal package '@asha/contracts'" \
  bash "$REPO_ROOT/harness/depgraph/verify-ts-deps.sh" "$TS_UNLISTED_FIXTURE"

TS_MISSING_FIXTURE="$TMP_ROOT/ts-missing-ownership"
make_ts_missing_ownership_fixture "$TS_MISSING_FIXTURE"
expect_failure \
  "missing TypeScript ownership entry" \
  "has no ownership entry in governance/ownership.toml" \
  bash "$REPO_ROOT/harness/depgraph/verify-ts-deps.sh" "$TS_MISSING_FIXTURE"

echo "Depgraph negative fixtures: OK"

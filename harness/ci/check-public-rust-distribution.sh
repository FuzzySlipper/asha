#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

COMPOSITION_FACADE="$REPO_ROOT/public-rust/runtime-session-composition/src/lib.rs"
WORKSPACE_AUTHORITY="$REPO_ROOT/engine-rs/crates/bridge/runtime-bridge-api/src/authority/workspace_authoring.rs"
CANONICAL_GENERATOR="$REPO_ROOT/harness/fixtures/canonical-project-consumer/src/main.rs"

echo "==> Proving manual workspace bootstrap stays behind the internal owner seam"
if grep -Fq 'pub use runtime_bridge_api::*' "$COMPOSITION_FACADE"; then
  echo "FAIL: the preferred composition facade must use an explicit export list" >&2
  exit 1
fi
if grep -Eq 'WorkspaceAuthoring(OpenRequest|ProjectBundleRef|ProjectIdentity)' "$COMPOSITION_FACADE"; then
  echo "FAIL: the preferred composition facade exposes manual workspace bootstrap vocabulary" >&2
  exit 1
fi
if grep -Eq 'pub fn open_workspace_authoring_(adapter|for_native_transport)' "$WORKSPACE_AUTHORITY"; then
  echo "FAIL: raw EngineBridge exposes a public manual workspace-authoring opener" >&2
  exit 1
fi
if grep -Fq 'asha_runtime_session_composition' "$CANONICAL_GENERATOR"; then
  echo "FAIL: the Engine-owned fixture generator presents its internal owner seam as public composition" >&2
  exit 1
fi

echo "==> Proving exact-revision public Rust Git consumption"
python3 "$REPO_ROOT/harness/public-surface/check-public-rust-distribution.py"

#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

run() { echo "==> $*"; "$@"; }

run env ASHA_GAMEPLAY_RUNTIME_HOST_GATE_OWNS_TESTS=1 "$REPO_ROOT/harness/ci/check-rust.sh"
run "$REPO_ROOT/harness/ci/check-ts.sh"
run "$REPO_ROOT/harness/ci/check-contracts.sh"
run "$REPO_ROOT/harness/ci/check-depgraph.sh"
run "$REPO_ROOT/harness/ci/check-no-den-coupling.sh"
run "$REPO_ROOT/harness/ci/check-vocabulary.sh"
run "$REPO_ROOT/harness/ci/check-consumer-needs.sh"
run "$REPO_ROOT/harness/ci/check-reachability.sh"
run "$REPO_ROOT/harness/ci/check-conformance.sh"
run "$REPO_ROOT/harness/ci/check-bridge.sh"
run "$REPO_ROOT/harness/ci/check-gameplay-module-conformance.sh"
run "$REPO_ROOT/harness/ci/check-gameplay-module-sdk.sh"
run "$REPO_ROOT/harness/ci/check-gameplay-runtime-host.sh"
run "$REPO_ROOT/harness/ci/check-trigger-volumes.sh"
run "$REPO_ROOT/harness/ci/check-replays.sh"
run "$REPO_ROOT/harness/ci/check-render-goldens.sh"

echo ""
echo "All checks passed."

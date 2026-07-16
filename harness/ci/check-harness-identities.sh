#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python3 "$REPO_ROOT/harness/identity/catalog.py"
python3 "$REPO_ROOT/harness/identity/test_catalog.py"
python3 "$REPO_ROOT/harness/identity/test_execution.py"
python3 "$REPO_ROOT/harness/ci/test_ci.py"

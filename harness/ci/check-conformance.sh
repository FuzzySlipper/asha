#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python3 "$REPO_ROOT/harness/identity/catalog.py"
python3 "$REPO_ROOT/harness/conformance/validate.py"
python3 "$REPO_ROOT/harness/conformance/validate.py" --check-fixtures
python3 "$REPO_ROOT/harness/conformance/test_claim_results.py"
python3 "$REPO_ROOT/harness/conformance/run.py"

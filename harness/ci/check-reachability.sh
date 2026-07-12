#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "==> Checking joined public capability reachability"
python3 "$REPO_ROOT/harness/reachability/validate.py"
python3 "$REPO_ROOT/harness/reachability/validate.py" --check-fixtures

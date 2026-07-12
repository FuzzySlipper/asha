#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "==> Checking role-scoped consumer needs manifests"
python3 "$REPO_ROOT/harness/consumer-needs/validate.py" --check-fixtures

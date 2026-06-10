#!/usr/bin/env bash
# Headless render golden check. Builds renderer-three, applies a named
# render-diff fixture, and diffs the renderer's deterministic scene snapshot
# against a committed golden. No GL context / pixel screenshot — the artifact is
# a structural scene snapshot (deterministic and CI-friendly; GPU pixels are
# driver-dependent and headless GL is a heavy native dependency).
#
# Distinct exit stages for repair routing:
#   - SETUP FAILURE   : renderer-three did not build (ts-shell build / deps)
#   - RENDERER/GOLDEN : the golden test fails — its message says RENDERER FAILURE
#                       (apply threw) vs GOLDEN MISMATCH (snapshot drifted)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT/ts"

echo "==> Building renderer-three (setup)"
if ! pnpm --filter @asha/renderer-three build >/tmp/render-golden-build.log 2>&1; then
    echo "SETUP FAILURE: renderer-three did not build" >&2
    cat /tmp/render-golden-build.log >&2
    exit 1
fi

echo "==> Running headless render golden snapshot test"
node --test 'packages/renderer-three/dist/**/golden.test.js'

echo "Render goldens reproduced."

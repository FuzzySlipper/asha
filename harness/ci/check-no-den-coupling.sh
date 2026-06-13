#!/usr/bin/env bash
# Verifies ASHA engine source introduces no dependency or import coupling to Den
# (scene-capability-06 / epic #2313, subtask #2333).
#
# ASHA emits stable, generic diagnostics/artifacts that an external workflow
# system such as Den *may* consume — but ASHA must never depend on Den, name a
# Den package, or import a Den module. This guard greps the Rust crates and TS
# packages for actual coupling syntax (imports, `use`/path references, and
# dependency declarations). It deliberately targets code syntax, not prose, so a
# design note that mentions "Den" in a comment does not trip it.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "==> Verifying ASHA introduces no Den coupling"

# Coupling syntax we reject:
#   Rust:  `use den...`, `den::...`, `extern crate den...`
#   TS:    `from '@den/...'` / `from 'den...'`, `require('den...')`, `import('den...')`
#   Cargo: a `den-...`/`den_...` dependency key
# `[\"']` is a bracket matching either quote char; `\\(` is a literal `(`.
PATTERN="(\<use[[:space:]]+den)|(\<den::)|(extern[[:space:]]+crate[[:space:]]+den)|(from[[:space:]]+[\"']@?den)|(require\\([\"']@?den)|(import\\([\"']@?den)|(^[[:space:]]*den[-_][a-z0-9]+[[:space:]]*=)"

# Search the engine source trees only; skip build output and vendored deps.
matches="$(grep -rEnI "$PATTERN" \
    "$REPO_ROOT/engine-rs" "$REPO_ROOT/ts" \
    --include='*.rs' --include='*.ts' --include='Cargo.toml' --include='package.json' \
    --exclude-dir=target --exclude-dir=node_modules --exclude-dir=dist \
    --exclude-dir=generated 2>/dev/null || true)"

if [ -n "$matches" ]; then
    echo "FAIL: Den coupling found in ASHA engine source:" >&2
    echo "$matches" >&2
    echo "" >&2
    echo "ASHA must not depend on or import Den. Emit generic diagnostics/artifacts;" >&2
    echo "let external systems consume them without ASHA depending on them." >&2
    exit 1
fi

echo "No Den coupling: OK"

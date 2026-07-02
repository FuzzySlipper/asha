#!/usr/bin/env bash
# Smoke-test the TypeScript package generator in a throwaway workspace.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
mkdir -p /tmp/asha
TMP_ROOT="$(mktemp -d /tmp/asha/package-generator.XXXXXX)"
trap 'rm -rf "$TMP_ROOT"' EXIT

mkdir -p "$TMP_ROOT/governance" "$TMP_ROOT/ts"
cp "$REPO_ROOT/ts/tsconfig.base.json" "$TMP_ROOT/ts/tsconfig.base.json"
cp "$REPO_ROOT/ts/pnpm-workspace.yaml" "$TMP_ROOT/ts/pnpm-workspace.yaml"
ln -s "$REPO_ROOT/ts/node_modules" "$TMP_ROOT/ts/node_modules"
printf '# Ownership and lane assignments for all crates and packages.\n\n# ─── TypeScript ──────────────────────────────────────────────────────────────\n' > "$TMP_ROOT/governance/ownership.toml"

node "$REPO_ROOT/ts/scripts/new-package.mjs" generated-tool \
  --repo-root "$TMP_ROOT" \
  --lane ts-tools \
  --type tool \
  --layer tool \
  --may-not-import @asha/native-bridge

test -f "$TMP_ROOT/ts/packages/generated-tool/package.json"
test -f "$TMP_ROOT/ts/packages/generated-tool/tsconfig.json"
test -f "$TMP_ROOT/ts/packages/generated-tool/src/index.ts"
test -f "$TMP_ROOT/ts/packages/generated-tool/src/index.test.ts"
grep -q '\[package."ts/packages/generated-tool"\]' "$TMP_ROOT/governance/ownership.toml"

bash "$REPO_ROOT/harness/depgraph/verify-ts-deps.sh" "$TMP_ROOT"
node "$REPO_ROOT/ts/node_modules/typescript/bin/tsc" --build "$TMP_ROOT/ts/packages/generated-tool/tsconfig.json"
node --test "$TMP_ROOT/ts/packages/generated-tool/dist/index.test.js"

set +e
overwrite_output="$(node "$REPO_ROOT/ts/scripts/new-package.mjs" generated-tool --repo-root "$TMP_ROOT" --lane ts-tools --type tool --layer tool 2>&1)"
overwrite_status=$?
set -e
if [[ "$overwrite_status" -eq 0 || "$overwrite_output" != *"refusing to overwrite"* ]]; then
  printf 'FAIL: package generator did not refuse overwrite\n%s\n' "$overwrite_output"
  exit 1
fi

echo "TypeScript package generator smoke: OK"

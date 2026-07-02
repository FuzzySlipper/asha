#!/usr/bin/env bash
# Negative smoke for the TypeScript type-aware lint ratchet.
#
# Drops a deliberately-illegal file into a package source tree and verifies that
# the enforced type-aware rules reject it. The file is removed before the script
# checks that normal lint still passes.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
SMOKE_FILE="$REPO_ROOT/ts/packages/app/src/__type_aware_smoke__.ts"

cleanup() { rm -f "$SMOKE_FILE"; }
trap cleanup EXIT

cat > "$SMOKE_FILE" <<'TS'
// Intentionally illegal app source — used only by the type-aware lint smoke.
import { RuntimeBridge } from '@asha/runtime-bridge';

export function missingBoundaryReturnType(bridge: RuntimeBridge) {
  return bridge.getCompositionStatus();
}

export function acceptsAny(value: any): void {
  void value;
}

function acceptsVoidCallback(callback: () => void): void {
  callback();
}

export function misusesPromiseCallback(): void {
  acceptsVoidCallback(async () => {
    return Promise.resolve();
  });
}
TS

fail() { echo "TYPE-AWARE LINT SMOKE FAILED: $1" >&2; exit 1; }

echo "==> eslint must reject type-aware lint violations"
set +e
output="$(cd "$REPO_ROOT/ts" && pnpm exec eslint packages/app/src/__type_aware_smoke__.ts 2>&1)"
status=$?
set -e

if [[ "$status" -eq 0 ]]; then
  printf '%s\n' "$output"
  fail "eslint accepted the deliberately illegal type-aware smoke file"
fi

for rule in \
  "@typescript-eslint/consistent-type-imports" \
  "@typescript-eslint/explicit-module-boundary-types" \
  "@typescript-eslint/no-explicit-any" \
  "@typescript-eslint/no-misused-promises"
do
  if [[ "$output" != *"$rule"* ]]; then
    printf '%s\n' "$output"
    fail "eslint output did not mention $rule"
  fi
done
echo "    eslint rejected all enforced type-aware rules (as required)"

cleanup

echo "==> with the file removed, lint must pass again"
(cd "$REPO_ROOT/ts" && pnpm lint) \
  || fail "workspace lint is not clean after removing the smoke file"

echo "Type-aware lint smoke passed."

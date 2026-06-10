# Lane: ts-policy

## Owns
- `ts/packages/script-sdk` — view types, command builder helpers, test harness utilities
- `ts/packages/script-host` — policy pack loader, deterministic invocation, sandbox, command buffer
- `ts/packages/policy-core` — default and no-op policy implementations
- `ts/packages/policy-examples` — example threshold and state-machine policies

## May import
- `@asha/contracts`
- `@asha/script-sdk`
- `@asha/catalog-core`, `@asha/catalog-examples` (approved catalog packages)
- Node built-ins for script-host only (file loading, module resolution)

## Must never import
- `@asha/renderer-babylon`
- `@asha/ui-dom`
- `@asha/wasm-bridge`
- `@asha/electron-main`
- Any browser globals: `Date`, `Math.random`, `document`, `window`, `localStorage`, `fetch`

## Required tests
- Policy function unit tests: given a fixture `PolicyView`, assert returned `PolicyCommand[]`.
- Script-host integration test: load a policy pack, invoke it, collect commands.
- Sandbox test: policy using a forbidden global must fail lint.
- Determinism test: same view input + same seed → same command output across runs.

## Required fixtures
- `harness/fixtures/policy-inputs/` — `PolicyView` snapshots used as test inputs.
- `harness/fixtures/policy-outputs/` — expected `PolicyCommand[]` arrays for golden tests.

## Drift smells reviewers should flag
- Import of `renderer-babylon`, `ui-dom`, `wasm-bridge`, or `electron-main` in any policy package.
- Use of `Date`, `Math.random`, or any browser global inside policy functions.
- Policy function that mutates an object received in its view parameter.
- Script host performing command validation (belongs in Rust).
- Shadow state model accumulating inside a policy package (policy is stateless per tick).
- Manual edit to files in `ts/packages/contracts/src/generated/`.

## Public API changes that require escalation
- Changes to `PolicyView` or `PolicyCommand` types — those come from generated contracts; escalate to contract-steward.
- Changes to the script-host invocation interface — affects every policy pack consumer.

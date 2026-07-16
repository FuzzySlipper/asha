# Proof-apparatus disposition

This is the terminal engine inventory for task #5857. The engine keeps hard
architectural rails and regressions for behavior it owns. It does not maintain
an engine-only declaration that Demo or Studio has been delivered.

| Former artifact family | Disposition | Current owner or replacement |
| --- | --- | --- |
| `harness/consumer-needs/**`, committed validation report, checker, and documentation | Deleted | Concrete requirements and visible acceptance stay in the owning consumer repository and Den |
| `harness/reachability/**`, committed validation report, checker, and documentation | Deleted | Public package/crate allowlists remain engine guardrails; actual usability is exercised by provider or consumer behavior |
| repository-wide `harness/conformance/**`, probe/evidence catalogs, negative meta-fixtures, execution report runner, checker, and documentation | Deleted | Focused synthetic public behavior lives in `asha-testing`; engine providers own behavioral regressions |
| generated semantic identity catalog and catalog self-tests | Deleted | Shared execution definitions retain only command, input, provider identity, claim attribution, and collision-checked receipt IDs |
| `check-asha-demo-input-live.sh` and its execution identity | Deleted | Demo owns visible input, pause, restart, and gameplay acceptance |
| source-token, declaration, and report-shape delivery assertions from those layers | Deleted | Generated-border completeness plus direct accepted/rejected/readback assertions |
| public scene mutation and resolved-input replay synthetic scenario | Migrated to `asha-testing` #5858 | `asha-testing` `9ac2e9672e1a51f051a7640a2cd7a64095f7830c`; native and reference paths ran before upstream removal |
| execution fingerprint/cache machinery | Converted | `harness/identity/execution.py` is now only shared gate execution and ignored receipts; it has no delivery catalog or downstream verdict |
| downstream-shaped gameplay-module fixture | Converted | Engine-owned public gameplay provider regression; it executes bootstrap, invocation, state, snapshot, and replay behavior |
| RuntimeSession evidence-named tests and scripts | Converted | Reference-provider and Rust-provider regressions with direct provenance/behavior assertions |
| persisted voxel, annotation, conversion, and generated voxel-command consumer proofs | Converted | Native provider regressions; report writing and Studio/testing identities were removed, deterministic authority fixtures remain |
| Studio-named voxel conversion fixture/golden | Converted | Product-neutral provider regression fixture/golden |
| public-consumer compatibility test | Converted | Package-root and browser-condition boundary guardrail |
| composed native provider and runtime-host lifecycle proofs | Converted | Engine-owned provider build/lifecycle regressions with bounded failure artifacts |
| clean Git public-Rust consumer fixture | Retained as a local guardrail | Verifies published facade packaging cannot depend on private engine paths; it makes no product-delivery claim |
| generated bridge operation/conformance files | Retained as generated-border inputs | Generated completeness and signature parity, not downstream delivery evidence |
| protocol, replay, render, serialization, and authority goldens | Retained | Reviewed engine-owned product inputs and deterministic/data-loss regressions, not refresh-only reports |
| launchable smoke and performance runners | Retained | Engine-local operational and provider behavior; Demo and Studio still own their visible product acceptance |

## Result ownership

Shared execution receipts, timing, native build logs, and runtime-host failure
tails are ignored under `harness/smoke-out/` and uploaded as CI or task
artifacts. They are not committed refresh obligations.

The remaining blocking distinction is:

- engine local: authority/dependency/wire/generated/data-loss guardrails and
  provider behavior;
- `asha-testing`: focused synthetic public-surface behavior and boundary
  negatives;
- Demo and Studio: visible gameplay and editor acceptance.

An external scenario returns to the engine only after a concrete provider
defect establishes a direct engine-owned regression. Historical report shape or
easier central wiring is not sufficient.

# Real conformance probes

The conformance inventory answers a stricter question than type, bridge-manifest,
or reachability validation: **does an advertised capability reach code that is
actually compiled and executed?** Mock behavior, generated declarations, provider
registration, and source-token reachability are useful earlier gates, but none of
them is real conformance evidence by itself.

## Generated checklist, authored meaning

`harness/conformance/validate.py` derives the required checklist from the live
bridge-operation, reachability, consumer-needs, and public-surface catalogs. The
committed `probe-inventory.json` does not copy the stable operation list. It names
the real test corpora and suites, then supplies hand-authored semantic probes for
the meanings that a catalog cannot infer. Each required semantic claim is bound
to one governed probe and one exact assertion executed by that probe's suite:

- actual downstream gameplay-module invocation;
- event identity bound to frozen typed reads;
- field, selector, quota, and deterministic ordering behavior;
- module-state read providers and provider cardinality;
- stable prefab-part resolution;
- configured ProjectBundle bootstrap and atomic rejection;
- public RuntimeSession projection/readout consumption.

The resulting committed `probe-results.json` is a deterministic structural
artifact. It says whether declarations, public paths, named assertions, and
operation coverage are structurally valid. It deliberately records execution as
`not_run`; source tokens and catalog joins cannot turn that committed file into
fresh execution or product-delivery evidence.

`harness/conformance/run.py` writes the environment-specific execution claim
report under ignored `harness/smoke-out/proof-execution/`. That report joins—but
does not merge—the structural result with shared execution receipts. Every suite
has one execution state:

- `passed`: a current command/input/toolchain/provider/repository-SHA fingerprint
  executed successfully and attributed the suite;
- `failed`: the current execution ran and failed, or omitted its suite attribution;
- `not_run`: no current execution receipt was supplied;
- `unavailable`: the owning consumer checkout is absent;
- `stale`: a receipt exists but its fingerprint or exact repository revision no
  longer matches.

Reports also separate `structuralDeclaration`,
`providerIntegrationExecution`, and `consumerProductAcceptance` verdicts. An
engine-only checkout may pass its structural/provider work while downstream
product acceptance is unavailable. Unavailable and not-run product work never
contributes to pass counts.

## Stable-operation rule

Every stable bridge operation must satisfy both conditions:

1. The generated manifest snapshot marks it native-wired and the exact native
   declaration/export checks find one matching TypeScript binding and Rust
   `#[napi]` function.
2. A named `#[test]` assertion executed by a declared compiled Rust authority or
   native-transport suite calls that exact operation.

Production declarations, helper names, comments, and broad source-substring
matches cannot satisfy operation evidence. The report records the suite, corpus,
source path, and assertion name for every matching operation call.

An operation that lacks native wiring cannot borrow evidence from the reference
bridge or a fake addon. It requires a reviewed temporary exemption with an owner,
specific reason, exit criteria, and evidence that the public facade fails closed.
Removing the exemption before the native probe lands fails CI. Wiring the
operation while retaining the exemption also fails CI as stale policy.

The current inventory records all 69 stable operations as real probes and has no
temporary native-wiring exemptions. The final coverage includes camera input and
projection, voxel pick/selection/mesh evidence, explicit buffer lifetime,
ProjectBundle unload, canonical scene-object commands/readout, and validated
model/material preview projection. A future stable operation must land its native
export and semantic Rust/native assertion in the same change; otherwise this gate
fails rather than quietly reopening an exemption.

## Gameplay pressure tests

The event-bound read probe uses a real fabric coordinator and binds the event
target identity into capability, relationship, prefab-part, owner-query, and
module-state reads. It asserts selected fields, quotas, frozen content, and stable
ordering. This is the Unity-facing gameplay pressure test: a module can react to
an expressive moment without receiving raw Session stores.

The binding probe loads ProjectBundle-shaped prefab authority, resolves a stable
`{ prefab, role }` part identity across two instances, applies per-instance
configuration, initializes all module state in one atomic batch, invokes the real
compiled module, and proves invalid contracts/roles/overrides reject before a
Session is constructed.

The public-consumer probe executes the package-root RuntimeSession, render, HUD,
and readout path used by `asha-demo`. The downstream module probe compiles in its
own Cargo workspace and imports only `asha-gameplay-module-sdk`.

The gameplay-module conformance probe adds the public one-command ProjectBundle
bootstrap, module-state fact playback, save/reload, and verification-replay
report. It is separately reachable through `pulse.conformance` so merely
declaring the conformance crate cannot satisfy delivery.

## Boundary with #5635

This gate owns inventory completeness and the shared vocabulary of real probes,
typed deliveries, stable identities, deterministic ordering, and atomic
bootstrap. Task #5635 remains the owner for gameplay-module-specific state,
save/reload/replay equivalence, and deeper negative/rejection matrices. Those
tests should register through this inventory rather than creating a separate
notion of conformance.

## Commands

Run only catalog/report validation:

```bash
python3 harness/conformance/validate.py
```

Run validation and every currently available real suite:

```bash
./harness/ci/check-conformance.sh
```

To verify the engine-only topology without moving local sibling checkouts, run
the execution reporter with an empty workspace parent:

```bash
python3 harness/conformance/run.py --workspace-parent /path/to/empty-parent
```

This changes only optional-consumer availability classification; engine-owned
provider commands and their normal input paths still execute unchanged.

The full repository gate runs conformance after reachability so a missing public
path is reported before execution. Missing sibling consumer repositories are
reported as unavailable rather than silently omitted or treated as passed.

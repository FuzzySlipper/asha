# Game Rust Authority Extension Model

Status: historical foundation plus implemented gameplay-fabric successor. The
original one-hook `GameRuleModule` slice remains compatibility evidence; new
downstream authority should use the public static gameplay-module SDK and
runtime host described under **Gameplay-Fabric Successor Foundation** below and
in `docs/gameplay-fabric-growth-recipes.md`.

ASHA keeps the central rule:

> Rust owns authority. TypeScript owns expression and projection. Generated
> contracts define the border.

That rule still leaves room for a real game repo to own compiled Rust behavior.
The important distinction is between **generic ASHA authority** and
**authored game authority**.

## Ownership Split

ASHA Rust always owns generic engine truth:

- RuntimeSession lifecycle and canonical state application.
- command validation, accepted domain events, replay records, deterministic
  tick ordering, deterministic RNG, and session hashes.
- transform, collision, spatial queries, pathfinding primitives, generic
  health/lifecycle primitives, capability mutation ownership, and renderer
  projection formats.
- protocol/codegen and generated TypeScript contract packages.
- native/wasm/runtime bridge provider contracts and fail-closed backend
  selection.

Game-owned Rust may own authored rule decisions that are specific to that game:

- weapon effects, ability rules, damage formulas, and hit modifiers.
- quest, interaction, faction, aggro, spawn-condition, wave, and encounter
  rules.
- game-mode scoring/win-condition logic when built from ASHA-provided state
  views and emitted as typed proposals.
- content/package preflight tools and build metadata checks that do not claim
  runtime authority.

The game-owned Rust crate is not a replacement RuntimeSession. It is a compiled
rule contributor that ASHA invokes through a public extension boundary.

## Extension Shape

The durable model should be a boring compiled boundary, not a dynamic plugin
system:

1. Historical compatibility exposed a small `GameRuleModule` Rust trait/API;
   the implemented successor composes real `GameplayModuleBehavior` providers
   with open contracts and declared reads.
2. A game repo builds a Rust crate that implements that trait against generated
   ASHA view/request/receipt types.
3. The game repo declares the compiled rule module in an ASHA game manifest with
   a rule id, semantic version, contract hash, and deterministic capability
   requirements.
4. RuntimeSession loads the manifest, verifies compatibility, and calls the
   compiled module only at declared rule hooks.
5. Rule output is a typed proposal or receipt fragment. ASHA Rust validates it,
   applies accepted events through existing owner matrices, records replay, and
   projects readouts.

The boundary should feel closer to a stable Rust library API plus generated
schemas than to a scripting bridge. A future native host may link the game rule
crate statically or load a compiled artifact, but the invocation contract should
look the same either way.

## Determinism And Replay

Game Rust receives only deterministic inputs:

- generated read-only RuntimeSession/ECRP views,
- explicit tick/session/epoch identifiers,
- deterministic RNG handles or precomputed random draws supplied by ASHA,
- authored content refs whose hashes are part of the loaded ProjectBundle.

Game Rust must not read wall-clock time, ambient randomness, local files, network
state, DOM/browser state, or TypeScript globals during authority hooks.

Replay records must include:

- game rule module id/version/contract hash,
- hook id and deterministic input hash,
- proposal hash,
- ASHA validation/acceptance result,
- resulting domain event/rejection hashes.

Replaying a session must either load the same compatible game rule module or
fail closed with a missing-rule diagnostic. It must not silently substitute
TypeScript behavior or a reference fixture.

## TypeScript Role

Game TypeScript may describe and project:

- authored catalog values and content references,
- UI/control descriptors,
- policy/config choices that become typed proposals,
- HUD/menu/readout projections,
- browser input collection and standalone host shell behavior.

Game TypeScript must not own:

- damage application,
- health/lifecycle mutation,
- collision or pathfinding resolution,
- RuntimeSession restart/session authority,
- rule execution shortcuts,
- arbitrary JSON command hatches,
- generated contract truth.

When a game needs a new authoritative behavior, TypeScript may name the rule and
submit typed intent data. The compiled Rust rule and ASHA RuntimeSession decide
what happened.

## Forbidden Paths

The following paths are hard failures:

- downstream imports of ASHA Rust private crates or TypeScript `src/*` files;
- game TS mutating authoritative state or shadowing RuntimeSession health,
  combat, collision, lifecycle, replay, pathfinding, or generated level truth;
- arbitrary JSON command/action hatches that Rust does not type and validate;
- demo-local replacements for generic collision, combat, lifecycle,
  pathfinding, RuntimeSession, renderer backend, or protocol/codegen authority;
- dynamic JavaScript callbacks in the authority path;
- reference/mock RuntimeSession helpers used as live/product authority.

## Implemented Extension Boundary Slice

Task #4516 adds the first upstream boundary cells for game-owned Rust rule
modules:

- `engine-rs/crates/protocol/protocol-game-extension` defines schema-only Rust
  DTOs for rule module manifests, hook declarations, deterministic weapon-effect
  hook requests, typed proposals, hook receipts, diagnostics, and replay
  evidence.
- `engine-rs/crates/rules/game-rule-extension` defines the public Rust
  `GameRuleModule` trait/API that downstream game crates can compile against.
  The default hook path fails closed with a typed diagnostic, and helper
  receipts explicitly remain pending proposals rather than authority mutations.
- `public-rust/game-rule-extension` is the approved downstream dependency
  facade for ASHA Game Projects. A downstream crate should depend on
  `asha-game-rule-extension = { path = "../asha-engine/public-rust/game-rule-extension" }`
  and import `asha_game_rule_extension`, not `engine-rs/crates/rules/game-rule-extension`.
- `ts/packages/contracts/src/generated/gameExtension.ts` is generated from the
  Rust protocol source and re-exported from `@asha/contracts`, so TypeScript can
  name module refs, hooks, proposals, receipts, and replay evidence without
  demo-local schemas or private generated-file imports.

Task #4517 adds the first RuntimeSession invocation slice:

- `runtime-bridge-api` loads declared game-rule module manifests alongside the
  FPS RuntimeSession load request and fails closed when a requested module or
  hook is missing or incompatible.
- `invoke_game_extension_weapon_effect` invokes a declared Rust
  `GameRuleModule`, validates the returned generated `damageModifier` proposal,
  and applies accepted output through `rule-lifecycle` plus `svc-combat`
  authority. TypeScript does not supply behavior callbacks.
- Replay evidence records module id/version/contract hash, hook id, input hash,
  proposal hash, validation status, accepted combat event hashes, and rejection
  hashes.
- `@asha/runtime-bridge` exposes `invokeGameExtensionWeaponEffect` and
  `RuntimeSessionFacade.submitGameExtensionWeaponEffect` through package-root
  types. Native providers must expose the bounded operation or fail closed.

This slice is intentionally not a dynamic plugin system. It proves the compiled
rule-module invocation path with a narrow reference module and leaves downstream
game-owned compiled modules to the demo/consumer follow-up.

## Gameplay-Fabric Successor Foundation

Task #5630 preserves the compiled Rust boundary while replacing the assumption
that every new game meaning needs another bespoke engine hook. Open,
namespaced/versioned `GameplayContractRef` values now describe events,
proposals, views, facts, and module state. Stable Observe/Guard/Transform/React
families describe invocation roles without granting mutation authority.

`GameplayModuleManifest` and the immutable `svc-gameplay-fabric` registry form
the successor bootstrap boundary. They validate linked provider agreement,
typed codecs, subscriptions, exact authority owners, read-view providers,
namespace ownership, budgets, and ordering before a Session can use the graph.
The original `GameRuleModuleManifest` and weapon-effect trait remain only in
the Wave 1 compatibility quarantine until Demo #5734 removes the last caller. See
`docs/gameplay-fabric-contracts.md` for the implemented boundary and non-goals.

Task #5634 adds the public successor lane at
`public-rust/gameplay-module-sdk`. Downstream crates now implement real
`GameplayModuleBehavior`, use typed handler helpers and declared reads, and
contribute a static provider containing their manifest, codecs, state/view
adapters, configuration schema metadata, and behavior instance. Composition
consumes the same immutable registry builder. The preferred SDK no longer
exports the old weapon trait or its former duplicate adapter. The one retained
bridge adapter is available only through the fabric's `compatibility`
namespace and is not a new-consumer path.

## Remaining Extension Points

ASHA still needs follow-up work before downstream games have the full boundary:

- Move the current bridge-level FPS RuntimeSession envelopes into generated
  `protocol_runtime` contracts instead of the existing explicit transitional
  bridge DTO allowlist.
- Bind static providers and typed configuration from ProjectBundle-authored
  content rather than hard-coded Session setup (#5661).
- Add the full downstream conformance command and real consumer replay proofs
  (#5635/#5636 and Rulebench #5638).

## Minimal `asha-demo` Candidate Slice

The smallest useful proving slice is a game-owned Rust weapon effect:

- `asha-demo` owns a small Rust crate that defines a `demo.primary_fire_effect`
  module.
- The rule reads a generated, read-only hit/effect context supplied by ASHA and
  returns a typed damage modifier proposal, such as `base_damage + close_range_bonus`.
- ASHA RuntimeSession validates that proposal against generic weapon/combat
  rules, applies health/lifecycle changes through existing Rust authority, and
  records replay evidence with the demo rule module id/version/hash.
- Demo TS only submits `primary_fire` and projects the resulting RuntimeSession
  receipt/HUD readout.

`asha-demo` can now prove the first compiled-rule slice by declaring a module
manifest and calling the public RuntimeSession invocation surface. It should
still not become an alternate combat/collision/lifecycle stack.

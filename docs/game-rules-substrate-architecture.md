# Game Rules Substrate Architecture

Status: architecture note for Den task #4525. This plans upstream ASHA main
work for #4524. It does not implement the crates yet.

## Decision

ASHA should add a generic game-rules substrate as upstream Rust authority
infrastructure, separate from downstream game-owned Rust rule modules.

Create these upstream crates:

| Crate | Lane | Role |
|---|---|---|
| `engine-rs/crates/state/core-game-rules` | `rust-state` | Shared ids, bounded value vocabulary, effect IR, modifier/reaction definitions, timing vocabulary, deterministic trace/readout primitives. |
| `engine-rs/crates/protocol/protocol-game-rules` | `contract-steward` | Generated border DTOs for effect catalogs, modifier definitions, validation diagnostics, resolution receipts, trace/readout records, and replay evidence summaries. No authority logic. |
| `engine-rs/crates/services/svc-game-rules` | `rust-service` | Catalog validation and pure effect resolution. It consumes explicit facts supplied by callers and returns pending outcomes/receipts. It does not mutate `SessionState`, `EntityStore`, or `CombatState`. |
| `engine-rs/crates/rules/rule-game-modifier` | `rust-rule` | Stateful modifier lifecycle authority: apply, refresh/stack, deterministic tick, value delta from tick, expiration/removal, and replay/hash contribution. |

Do not create a `rule-game-reaction` crate in the first slice. Reaction
definitions and reaction-window resolution should start in `core-game-rules`,
`protocol-game-rules`, and `svc-game-rules`. Add a separate rule crate only if
reaction state becomes a persisted lifecycle owner instead of a pre-commit
resolution phase.

## Current-State Audit

The existing FPS authority slice is intentionally narrow:

- `svc-combat` owns `HealthState`, `CombatState`, `FireIntentCommand`,
  `CombatEvent::{FireHit, FireMissed, DamageApplied, EntityDefeated}`, and
  `CombatReadout`. `apply_fire_intent()` validates a fire command, resolves
  collision/target hits, mutates health atomically, and emits deterministic
  readouts.
- `rule-lifecycle` composes `svc-entity-authoring` and `svc-combat` for the
  FPS RuntimeSession path. It bootstraps ProjectBundle-shaped definitions,
  applies primary-fire proposals, records health/death lifecycle effects, and
  updates render visibility through explicit rule-owned mutations.
- `svc-entity-authoring` enforces the ECRP owner matrix. TypeScript policies,
  renderers, UI, and downstream repos do not receive raw `EntityStore` access.
- `core-events`, `core-commands`, and the replay crates currently model the
  abstract authority/replay spine, while `svc-combat::CombatEvent` is a
  service-local combat event/readout shape rather than a top-level
  `DomainEvent` variant.
- #4488 / #4516 / #4517 cover compiled game-owned Rust extension modules.
  Those modules are downstream authored rule contributors. They are not the
  upstream generic effect/modifier substrate planned here.

## Boundary Split

`core-game-rules` owns reusable nouns and validation helpers that are safe for
both real-time action games and RPG-style authored actions:

- stable ids for rule catalog entries, effect operations, modifiers, value
  channels, tags, and reaction windows;
- bounded value and delta vocabulary suitable for health, shields, stamina,
  posture, charge, or other game-defined channels;
- effect operation IR for apply delta/damage, restore, spend/grant value,
  apply/remove modifier, schedule periodic effect, cancel/reject resolution,
  and emit trace/projection hints;
- modifier definition/state vocabulary with source, target, duration, cadence,
  stack policy, tags, source hashes, and replay identity;
- generic timing vocabulary based on deterministic ticks or fixed steps, not
  wall-clock time and not turn/round assumptions;
- trace/readout structs that explain resolution but do not authorize mutation.

`protocol-game-rules` mirrors the public border shape for downstream TS content
packages and Studio authoring. It should depend only on `core-ids` and stable
diagnostic vocabularies such as `protocol-diagnostics` if needed. It must not
import `core-game-rules`, services, rules, render, bridge, or TypeScript.

`svc-game-rules` validates catalogs and resolves an `EffectResolutionRequest`
against explicit facts provided by the caller. It returns a receipt containing
pending outcomes, trace entries, diagnostics, and replay hashes. The service
must not commit events or mutate stores. This lets `rule-lifecycle`,
`svc-combat`, future game-owned rule invocations, and tests use the same
interpreter without creating a second authority stack.

`rule-game-modifier` owns persistent modifier lifecycle once a modifier has
been accepted into authority state. It should be the only new crate that commits
modifier apply/refresh/tick/expire facts in the first implementation wave.

## Relationship To `svc-combat`

The generic substrate must not create a competing health/damage stack.

Initial integration preserves `svc-combat` public readouts while adding a
generic effect path underneath them:

1. `svc-combat` keeps fire/raycast/target selection and the compatibility
   `CombatReadout` shape.
2. FPS primary-fire damage now resolves through a generated game-rules
   `ApplyDelta` effect in `svc-game-rules`; the resolved bounded damage amount
   is then passed to `svc-combat::apply_fire_intent()` for the only health
   mutation path.
3. `svc-combat::CombatEvent` remains a compatibility service readout during the
   transition. A later migration can map accepted generic value events back into
   the same public combat readouts.
4. Poison, periodic effects, stacks, and reaction windows must live in the
   game-rules substrate, not as one-off `svc-combat` branches.

## Domain Events And Replay

Accepted generic game-rule facts should eventually become part of the same
replay/audit spine as other authority changes. The recommended path is:

1. `core-game-rules` defines pure event payload types such as value delta
   applied, modifier applied/refreshed/ticked/expired, and reaction adjusted a
   pending effect.
2. A follow-up extends `core-events::DomainEvent` with a typed game-rules event
   variant or narrow dedicated variants after the payload shape lands.
3. `rule-game-modifier` and later rule paths produce those accepted events.
4. `sim-replay` encoding gains deterministic lines for these events and their
   hashes.
5. RuntimeSession readouts expose game-rule traces and replay hashes through
   generated contracts, not ad hoc JSON.

Until that migration lands, `svc-game-rules` receipts are pending evidence only:
useful for validation and preview, but not committed authority.

## Ownership Entries To Add

When the crates land, update `governance/ownership.toml` as follows:

```toml
[crate."engine-rs/crates/state/core-game-rules"]
lane = "rust-state"
may_depend_on = ["core-ids", "core-error", "core-time"]
may_not_depend_on = ["protocol-render", "wasm-api", "render-bridge"]

[crate."engine-rs/crates/protocol/protocol-game-rules"]
lane = "contract-steward"
may_depend_on = ["core-ids", "protocol-diagnostics"]
may_not_depend_on = ["core-state", "core-game-rules", "svc-game-rules", "render-bridge", "wasm-api"]

[crate."engine-rs/crates/services/svc-game-rules"]
lane = "rust-service"
may_depend_on = ["core-ids", "core-error", "core-time", "core-game-rules", "protocol-game-rules", "protocol-diagnostics"]
may_not_depend_on = ["protocol-render", "wasm-api", "render-bridge"]

[crate."engine-rs/crates/rules/rule-game-modifier"]
lane = "rust-rule"
may_depend_on = ["core-ids", "core-error", "core-time", "core-game-rules", "core-events", "svc-game-rules"]
may_not_depend_on = ["protocol-render", "render-bridge", "wasm-api"]
```

Also update:

- `engine-rs/Cargo.toml` workspace members;
- `protocol-codegen` ownership and crate dependencies when
  `protocol-game-rules` is added;
- `runtime-bridge-api` ownership only when #4531 exposes RuntimeSession
  operations;
- `ts/packages/contracts` generated exports through the existing codegen path,
  never by hand-editing generated files.

## Difference From Game-Owned Rust Modules

#4516/#4517 should build a compiled extension boundary for downstream game
repos. That boundary lets a game contribute authored rule decisions with module
id/version/hash metadata.

This substrate is different:

- It is upstream ASHA main, not downstream game code.
- It validates and resolves generic effect/modifier/reaction data that many
  games can use.
- It owns first-class modifier lifecycle and replay shape.
- It gives game-owned Rust modules a durable target for proposals, rather than
  forcing each game module to invent damage, poison, stack, and reaction logic.

Game-owned Rust may decide that a close-range weapon adds a bonus or chooses an
effect bundle. ASHA game-rules authority validates, resolves, commits, and
replays the resulting typed facts.

## Non-Goals

- No ECS framework, hidden scheduler, generic event bus, or dynamic plugin
  registry.
- No TypeScript authority callbacks or `call_rule(json)` style hatches.
- No turn, round, initiative, power, feat, saving throw, class, or action
  economy vocabulary in upstream public APIs.
- No demo-local replacement for generic RuntimeSession, combat, collision,
  lifecycle, pathfinding, protocol/codegen, or replay authority.
- No direct renderer, UI, native transport, wasm bridge, or TypeScript package
  dependencies from the new Rust state/service/rule crates.

## First Proving Fixtures

The accepted fixture environment now lives in
`harness/fixtures/game-rules/*.snapshot.txt` and is exercised by:

```bash
cargo test -p rule-game-modifier --test game_rules_fixtures
```

Those fixtures are the selection environment for future RPG/action-game rule
work: new substrate behavior should be able to explain whether it preserves or
intentionally changes the poisoned-impact and RPG-action readouts before it is
promoted into demo or Studio surfaces.

1. Real-time poisoned impact:
   - input: source, target, hit tags, immediate damage delta, poison modifier
     definition, tick cadence, duration, stack policy;
   - expected facts: hit accepted, immediate value delta applied, modifier
     applied, periodic poison ticks applied deterministically, modifier expires,
     trace explains each step, replay hash is stable.
2. RPG-style declarative effect bundle:
   - input: generic action-like effect bundle with target facts, value delta,
     condition/modifier op, optional reaction window;
   - expected facts: bundle validates before runtime, resolution is deterministic
     from supplied CapabilityState/value facts, and no upstream turn-order
     assumption is required.

## Follow-On Task Fit

The existing #4524 subtasks remain accurate with this split:

- #4526 should create `core-game-rules`.
- #4527 should create `protocol-game-rules` and generated TS exports.
- #4528 should create `svc-game-rules`.
- #4529 should create `rule-game-modifier`.
- #4530 should implement reaction-window definitions/resolution in the
  core/protocol/service substrate first, not a separate rule crate unless
  persisted reaction lifecycle state is required.
- #4531 should expose RuntimeSession operations after the core/protocol/service
  and modifier rule paths exist.
- #4532 should preserve or migrate `svc-combat` without duplicating
  health/damage authority.
- #4533 should bind the poisoned-impact and RPG-style fixtures together as
  end-to-end proof.

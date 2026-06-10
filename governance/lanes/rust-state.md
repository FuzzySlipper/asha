# Lane: rust-state

## Owns
- `engine-rs/crates/state/core-state` — `StateStore`, entity storage, access rules
- `engine-rs/crates/state/core-events` — `DomainEvent` enum, event queue, apply trait
- `engine-rs/crates/state/core-commands` — `InputCommand`, `PolicyCommand`, `SystemCommand` types
- `engine-rs/crates/state/core-snapshot` — snapshot format, version tag, migration scaffold
- `engine-rs/crates/sim/sim-kernel` — tick phases and scheduling
- `engine-rs/crates/sim/sim-validator` — command validation dispatch
- `engine-rs/crates/sim/sim-applier` — sequential event application
- `engine-rs/crates/sim/sim-replay` — replay recording and playback
- `engine-rs/crates/sim/sim-runner` — headless tick driver

## May depend on
Foundation crates (`core-ids`, `core-math`, `core-time`, `core-error`, `core-collections`).
Protocol crates only for replay serialization (`protocol-replay`).

## Must never touch
- Render crates (`render-bridge`, `render-debug`), WASM API, or tool crates.
- TypeScript, DOM, network, filesystem (except snapshot I/O through `svc-serialization`).
- Product-domain rule logic — state crates define *shape*, not *behavior*.
- `Rc<RefCell<_>>` for any authoritative state path.

## Required tests
- Entity create/update/delete fixture.
- Command validation fixture (accept + reject cases).
- Event application fixture — apply a batch, assert resulting state.
- State hash fixture — same events produce the same hash.
- Headless tick test — one full read→propose→validate→apply→project cycle.
- Snapshot round-trip test.

## Required fixtures
- `harness/fixtures/states/` — baseline `StateStore` snapshots used by other lanes.
- `harness/fixtures/commands/` — sample accepted and rejected command payloads.
- `harness/fixtures/events/` — sample `DomainEvent` batches.

## Drift smells reviewers should flag
- `Rc<RefCell<_>>` anywhere in state or sim crates.
- Framework-shaped abstractions over `StateStore` (trait registries, plugin hooks).
- Generic `Event` bus that absorbs all event types.
- Renderer concepts (handles, meshes, materials) appearing in state types.
- Command validation logic moved out of `sim-validator` into state structs.
- Unexplained `.clone()` on authoritative data paths.

## Public API changes that require escalation
- Changes to `DomainEvent` variants — require replay fixture updates.
- Changes to `StateStore` public API — require downstream sim/service/rule compile check.
- Changes to snapshot format — require migration note and snapshot compatibility test.

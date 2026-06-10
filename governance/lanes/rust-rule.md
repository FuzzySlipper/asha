# Lane: rust-rule

## Owns
- `engine-rs/crates/rules/rule-lifecycle` — entity spawn/despawn rules
- `engine-rs/crates/rules/rule-process` — abstract process start/stop/interrupt rules
- `engine-rs/crates/rules/rule-scheduler` — tick-based scheduling and cooldown rules
- `engine-rs/crates/rules/rule-relationship` — ownership, attachment, and dependency rules
- `engine-rs/crates/rules/rule-state-machine` — generic finite state machine transitions
- `engine-rs/crates/render/render-bridge` — converts state/events into retained render diffs
- `engine-rs/crates/render/render-debug` — debug overlay and inspection layer diffs

## May depend on
Foundation crates, `core-state`, `core-events`, `core-commands`, `core-error`.
`protocol-render` for render-bridge and render-debug only.
Services through explicit function calls (no framework inversion).

## Must never touch
- `wasm-api` or any tool crates.
- Product-domain nouns in public APIs during the infrastructure phase (no "soldier", "building", etc.).
- Renderer behavior — render-bridge emits diffs only; it does not render.
- TypeScript, DOM, network.

## Required tests
- Command validation tests: rule accepts valid command, rejects invalid command.
- Event application tests: apply event, assert state change.
- State-machine transition tests: valid and invalid transitions.
- Render-bridge fixture: given a state snapshot, assert the emitted `RenderDiff` set.
- Replay fixture updates when rule behavior changes.

## Required fixtures
- `harness/fixtures/commands/` — accepted and rejected command cases per rule.
- `harness/fixtures/events/` — event batches exercising each rule.
- `harness/fixtures/render-diffs/` — expected render diff output for render-bridge tests.

## Drift smells reviewers should flag
- Product-domain concepts appearing in rule public APIs.
- Rule crate depending on `wasm-api` or render-bridge importing a rule.
- Render-bridge inspecting `StateStore` directly instead of receiving a projection.
- `unsafe` or `Rc<RefCell<_>>` anywhere in this layer.

## Public API changes that require escalation
- Any change to command or event variant shapes — requires downstream sim/protocol review.
- Render diff schema changes — requires protocol-steward review and fixture updates.

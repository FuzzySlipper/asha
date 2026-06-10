# Lane: rust-rule

## Owns
- `engine-rs/crates/rules/rule-lifecycle` — entity spawn/despawn rules
- `engine-rs/crates/rules/rule-process` — abstract process start/stop/interrupt rules
- `engine-rs/crates/rules/rule-scheduler` — tick-based scheduling and cooldown rules
- `engine-rs/crates/rules/rule-relationship` — ownership, attachment, and dependency rules
- `engine-rs/crates/rules/rule-state-machine` — generic finite state machine transitions

The render crates (`render-bridge`, `render-debug`) live in the `rust-render`
lane, not here.

## May depend on
Foundation crates, `core-state`, `core-events`, `core-commands`, `core-error`.
Services through explicit function calls (no framework inversion).

## Must never touch
- `wasm-api`, render crates, or any tool crates.
- Product-domain nouns in public APIs during the infrastructure phase (no "soldier", "building", etc.).
- Renderer behavior or render diffs (those belong to `rust-render`).
- TypeScript, DOM, network.

## Required tests
- Command validation tests: rule accepts valid command, rejects invalid command.
- Event application tests: apply event, assert state change.
- State-machine transition tests: valid and invalid transitions.
- Replay fixture updates when rule behavior changes.

## Required fixtures
- `harness/fixtures/commands/` — accepted and rejected command cases per rule.
- `harness/fixtures/events/` — event batches exercising each rule.

## Drift smells reviewers should flag
- Product-domain concepts appearing in rule public APIs.
- Rule crate depending on `wasm-api`, a render crate, or a render crate importing a rule.
- `unsafe` or `Rc<RefCell<_>>` anywhere in this layer.

## Public API changes that require escalation
- Any change to command or event variant shapes — requires downstream sim/protocol review.

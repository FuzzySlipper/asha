# Lane: rust-render

## Owns
- `engine-rs/crates/render/render-bridge` — converts authoritative state/events into retained render diffs
- `engine-rs/crates/render/render-debug` — debug overlay and inspection layer diffs

## May depend on
Foundation crates, `core-state`, `core-error`, and `protocol-render`.
`render-debug` may also depend on `render-bridge`.
Never `wasm-api` or tool crates.

## Must never touch
- Actual rendering — these crates emit `RenderDiff`s only; the TypeScript renderer (`@asha/renderer-three`) draws.
- `wasm-api`, TypeScript, DOM, network.
- Product-domain nouns in public APIs during the infrastructure phase (no "soldier", "building", etc.).

## Required tests
- Render-bridge fixture: given a state snapshot/projection, assert the emitted `RenderDiff` set.
- Handle stability: the same entity maps to the same `RenderHandle` across diffs; create/update/destroy ordering is correct.

## Required fixtures
- `harness/fixtures/render-diffs/` — input/expected retained render diff output.
- `harness/goldens/render-diffs/` — committed golden diffs where applicable.

## Drift smells reviewers should flag
- Render-bridge smuggling authority decisions into a projection instead of faithfully projecting state.
- A render crate importing a rule/sim crate, or importing `wasm-api`.
- Immediate-mode rendering creeping in (full-scene re-emission instead of create/update/destroy).
- `unsafe` or `Rc<RefCell<_>>` anywhere in this layer.

## Public API changes that require escalation
- Render diff schema changes — require `contract-steward` review, codegen regeneration of `render.ts`, and render-diff fixture/golden updates.

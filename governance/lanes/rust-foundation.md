# Lane: rust-foundation

## Owns
- `engine-rs/crates/foundation/core-ids` — typed, copyable entity ID primitives
- `engine-rs/crates/foundation/core-math` — deterministic math helpers (vectors, matrices, fixed-point)
- `engine-rs/crates/foundation/core-time` — tick counter and time-step primitives (no wall clock)
- `engine-rs/crates/foundation/core-error` — shared error and result types
- `engine-rs/crates/foundation/core-collections` — arena, slot-map, and other low-level collections

## May depend on
External crates only (e.g. `glam`, `slotmap`, `thiserror`).
Foundation crates must not depend on any other workspace crate.

## Must never touch
- State, protocol, sim, service, rule, render, WASM, or tool crates.
- Wall-clock time, filesystem, network, or any I/O.
- Product-domain concepts (entity types, game rules, render concepts).

## Required tests
- Unit tests in each crate covering the public API.
- Property-based tests where invariants are non-trivial (e.g. ID round-trip, arena insert/remove).

## Required fixtures
- None required at this layer. Tests use in-line data.

## Drift smells reviewers should flag
- Any `use` of a higher-level workspace crate.
- Introducing `unsafe` (workspace lint forbids it, but watch for exemption requests).
- Wall-clock (`std::time::SystemTime`, `Instant` for authoritative purposes).
- Domain concepts leaking in (entity names, rule logic, render structs).
- Public API additions without a reviewer note explaining the need.

## Public API changes that require escalation
All public API changes require escalation because foundation crates are imported by
nearly every other crate. A renamed type or changed signature breaks the whole workspace.
Open a PR note listing every downstream crate affected.

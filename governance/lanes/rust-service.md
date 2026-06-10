# Lane: rust-service

## Owns
- `engine-rs/crates/services/svc-rng` — deterministic RNG seeds and streams
- `engine-rs/crates/services/svc-spatial` — spatial transforms and indexing
- `engine-rs/crates/services/svc-collision` — collision shape queries
- `engine-rs/crates/services/svc-physics` — deterministic physics integration step
- `engine-rs/crates/services/svc-pathfinding` — grid/nav-mesh path search and caching
- `engine-rs/crates/services/svc-serialization` — save/load and encode/decode helpers
- `engine-rs/crates/services/svc-volume` — volumetric grid and chunk storage
- `engine-rs/crates/services/svc-mesh` — mesh building and buffer management

## May depend on
Foundation crates (`core-ids`, `core-math`, `core-time`, `core-error`, `core-collections`).
`core-state` where the service needs to read entity data.
External focused libraries (e.g. `rapier`, `pathfinding`, `noise`) that the service calls explicitly.

## Must never touch
- Protocol render crates, `wasm-api`, or `render-bridge`.
- Policy concepts, authored catalogs, or product-domain rule logic.
- Wall-clock time or ambient randomness — all randomness enters through `svc-rng`.
- DOM, network, or filesystem except through `svc-serialization`.

## Required tests
- Local unit tests covering the service's primary query/mutation API.
- Deterministic fixture test: same seed + same inputs → same output, verified by hash or snapshot.
- Performance-sensitive paths should have a criterion benchmark (optional at stub stage).

## Required fixtures
- `harness/fixtures/states/` — minimal state snapshots the service reads during tests.
- Service-specific golden output files when determinism is critical (e.g. RNG streams, path results).

## Drift smells reviewers should flag
- Service importing `protocol-render`, `wasm-api`, or render crates.
- Wall-clock `Instant` or `SystemTime` used for authoritative computation.
- Ambient `rand::thread_rng()` — must go through `svc-rng`.
- Service accumulating policy or rule logic ("if entity is a soldier, do X").
- Cross-service circular imports.

## Public API changes that require escalation
- Changes to deterministic output interfaces — require fixture update and downstream check.
- Removal or rename of any public type used by sim or rule crates.

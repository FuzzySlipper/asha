# Architecture overview

This is a short orientation page. The canonical repository architecture is
`docs/design.md`; Den tasks/docs/messages are the source of truth for current
planning and implementation state.

The durable invariants are:

- Rust owns authority: canonical state, validation, accepted event application,
  deterministic services, replay, serialization, and render projection.
- TypeScript owns expression and projection: constrained policy/catalog packages
  propose commands, while shell/render/UI packages display Rust-projected truth.
- Rust protocol crates define the border, and generated TypeScript contracts are
  not hand-edited.
- Crates and packages are agent assignment cells with machine-readable ownership
  and dependency rules in `governance/ownership.toml`.
- Commands, domain events, render diffs, telemetry, and replay records stay
  distinct; do not collapse them into a generic event bus.
- Services and rules use explicit Rust state access; do not introduce a
  framework-shaped ECS.

For quick routing:

- Use `docs/design.md` for architecture principles, layer model, dependency
  direction, lane expectations, and historical context.
- Use `governance/architecture.md` for governance-specific TS metadata axes and
  current boundary notes.
- Use `README.md` for repository layout, commands, and links to specialized
  docs.

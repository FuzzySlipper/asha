# Agent assignment guide

Each lane maps to one or more crates or packages.
An agent assigned to a lane must stay within its crate/package, use only approved deps,
and pass the required checks before requesting merge.

See governance/lanes/ for per-lane rules and governance/ownership.toml for machine-readable boundaries.

## Acceptance ownership

Agents and reviewers distinguish four kinds of validation:

- A local guardrail blocks an invalid dependency, authority leak, generated-border
  drift, unsafe wire shape, or data-loss path.
- A provider regression executes an engine-owned public/generated seam and checks
  accepted and rejected behavior, readback, call count, or deterministic replay.
- A synthetic conformance check acts as an external consumer and belongs in
  `asha-testing` when it has a distinct public contract.
- Consumer acceptance observes usable gameplay or authoring behavior in the
  owning downstream repository.

Engine changes may be blocked by missing local guardrails or provider regressions.
Do not accept or reject Demo/Studio delivery from source tokens, manifests,
evidence catalogs, or engine-only reports. Do not require a downstream product
proof to close an engine substrate task unless that task explicitly owns a live
cross-repository acceptance run.

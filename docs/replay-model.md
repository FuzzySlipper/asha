# Replay model

## Purpose

Replay is the core audit mechanism for agent-written changes.
It allows a change to be tested against prior behavior without a human running the game.

## What is recorded

| Record | When | Authority |
|---|---|---|
| Proposed commands | every tick input phase | non-authoritative |
| Accepted domain events | after validation | authoritative |
| State hash | at configurable intervals | verification |
| Snapshots | on demand or at checkpoints | verification |

For long-term golden regressions, accepted events plus snapshots/hashes are the stronger authority.

## Canonical replay target

WASM semantics are the replay authority.
Native builds are used for fast iteration and tooling only.
If native and WASM produce different outputs, the divergence must be classified and tested explicitly.

## Determinism requirements

All authoritative randomness comes from `svc-rng` with an explicit seed.
Wall-clock time, ambient randomness, network, filesystem, and DOM access are forbidden inside
the simulation path. Policy code receives deterministic inputs only.

## Replay file format

Defined in `protocol-replay`. A replay file contains:
- Header: engine version, seed, initial snapshot reference
- Steps: sequence of `ReplayStep` (accepted events + optional hash assertion)
- Tail: final state hash

## Running replays

```sh
# Run a named golden replay headlessly
cargo run -p replay-tool -- run harness/goldens/replays/<name>.replay

# Diff two replays
cargo run -p snapshot-diff -- <a>.replay <b>.replay
```

## Divergence reports

When a replay diverges, `sim-replay` produces a `DivergenceReport` that names:
- The step index where divergence began
- The expected vs. actual state hash
- The accepted event that triggered the divergence

This report is machine-readable so an orchestrator can route the failure to the responsible lane.

## Adding a new golden replay

1. Run the scenario headlessly with recording enabled.
2. Save the output to `harness/goldens/replays/<descriptive-name>.replay`.
3. Add the file to the `check-replays.sh` manifest.
4. Verify the replay passes on both native and WASM targets.

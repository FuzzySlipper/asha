# Launchable-voxel performance baseline

A deterministic, **logged** performance scenario over the canonical launch fixture,
run on **one stable host** for trend / regression tracking. It is intentionally *not*
a product performance target and *not* part of the normal CI gate.

> **Same-machine baseline.** Absolute timings are only meaningful relative to other
> runs **on the same host**. Do not compare milliseconds across machines, and do not
> read these numbers as final-product performance — they measure the reference
> (mock-facade) launch/edit/render/save loop in a headless Node process with no GPU.

## What it measures

The harness (`ts/packages/smoke/src/perf.ts`) reuses the smoke building blocks — the
same runtime facade, `ThreeRenderer`, `EditorStore`, and canonical fixture — and runs
the launch→edit→render→save→replay loop, recording:

- **Timings** (`performance.now`, logged/trended, never a gate): `initialize`,
  `world-load`, `render-projection-initial`, `renderer-apply-initial`, `edit-one-cell`,
  `edit-region`, `edit-inverse`, `render-update`, `preview-overlay`, `save`, `reload`,
  `replay`, and an aggregate `edit-render-cycles` loop (mean per cycle = `ms / iterations`).
- **Counters** (the *stable*, comparable fields): peak/leaked render handles, scene
  nodes, overlay cells, material/sprite fallbacks, commands accepted/rejected, total
  render ops applied, replay steps + divergence, outstanding buffers.
- **Structural invariants** (these MAY fail the run hard): `no-handle-leak`,
  `no-preview-remesh`, `bounded-render-ops-per-cycle`, `commands-accepted`,
  `replay-not-diverged`. The exit code reflects **only** these — timings never fail it.

## Running it

```bash
cd ts
ASHA_PERF_HOST=<stable-host-label> pnpm --filter @asha/smoke dev:asha-perf
# native authority path (fails closed honestly if the addon is unavailable):
ASHA_PERF_HOST=<stable-host-label> ASHA_PERF_MODE=authority pnpm --filter @asha/smoke dev:asha-perf
```

Environment knobs (all optional):

| Var | Meaning | Default |
|---|---|---|
| `ASHA_PERF_HOST` | Stable host label — the anchor for same-host comparison | OS hostname |
| `ASHA_PERF_MODE` | `reference` (mock baseline) or `authority` (native path) | `reference` |
| `ASHA_PERF_COMMIT` / `ASHA_PERF_BRANCH` | Override the recorded revision | `git` then `unknown` |

> Set `ASHA_PERF_HOST` to the **same** label every run on a given machine — that is the
> key the trend is grouped by.

## Output

Written under `harness/perf-out/` (gitignored — it is per-host trend data, not a golden):

- `launch-voxel-perf.jsonl` — one JSON record **appended per run** (the trend history).
- `launch-voxel-perf.latest.json` — the latest run, pretty-printed.

Each record is `{ ok, meta, timings, counters, invariants }` (schema in `perf.ts`,
`schema: 1`). `meta` carries `commit / branch / hostLabel / runtimeMode / smokeMode /
fixtureId / fixtureWorldHash` plus host basics (`node / platform / arch / cpus /
cpuModel / totalMemMb`) and a `timestamp`.

## Comparing runs over time

1. **Group by `meta.hostLabel`** (and `runtimeMode`/`smokeMode`). Only compare within a
   group — cross-host millisecond comparison is meaningless.
2. **Anchor on the stable fields.** `meta.fixtureWorldHash`, the counters, and the
   invariant set should be **identical** run-to-run for the same commit; a change there
   is a real structural shift, not noise. Treat those as the regression signal.
3. **Read timings as trends, not thresholds.** Watch a phase's `ms` (or `edit-render-cycles`
   mean) drift across commits on one host. A single run's absolute value is noisy; a
   sustained move across many runs is the signal. There is intentionally **no** committed
   timing golden and **no** CI threshold — wiring one would make CI flaky.

### Field stability cheat-sheet

| Field | Stable enough to assert? | Use |
|---|---|---|
| `counters.*`, `invariants[*].held`, `meta.fixtureWorldHash` | Yes — deterministic | Regression gate (the harness already fails on the invariants) |
| `meta` host/runtime descriptors | Yes (per host) | Grouping key |
| `timings[*].ms`, `edit-render-cycles` mean | No — noisy per run | Trend only, same host, over many runs |
| `meta.timestamp` | No | Ordering only; never compare |

## Why this is not in `check-all.sh`

Timing gates are flaky by nature, so the perf harness is deliberately **separate** from
the offline CI gate. Its correctness contribution — the structural invariants — is
already covered deterministically by `perf.test.ts` (run inside `check-all.sh` via the
smoke package tests), which asserts the record's shape and invariants with an injected
clock and **never** asserts a timing value. The logged timings are for operator/CI-
artifact trend monitoring on a chosen baseline host.

## Limitations

- **Reference baseline by default.** The default run uses the deterministic mock facade,
  so it measures the TS launch/edit/render/save loop, not Rust authority compute. Some
  suggested metrics (dirty-chunk counts, per-chunk meshing time) are not observable
  through the facade in reference mode and are therefore not recorded; they become
  available only when the authority/native path exposes them.
- **No GPU / no pixel work.** `ThreeRenderer` runs headless (structural scene graph only),
  so renderer timings reflect retained-mode bookkeeping, not real draw cost.
- **No product targets.** This task measures; it sets no FPS/frame budgets and makes no
  optimization changes.

Related: `docs/launchable-voxel.md` (the launch hub), `docs/replay-model.md` (durability),
`harness/fixtures/smoke/README.md` (the shared fixture).

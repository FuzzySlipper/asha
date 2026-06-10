# Lane: ts-tools

## Owns
- `ts/packages/devtools` — replay viewer, debug dashboard, catalog validator, state inspector, script lab

## May import
Any `@asha/*` package for inspection purposes.
Tool-level omniscience is intentional and permitted.

## Must never do
- Allow tool-only imports to leak into runtime packages (`app`, `renderer-babylon`, `ui-dom`, etc.).
- Mutate authoritative runtime state at runtime (read-only inspection only).
- Introduce runtime dependencies on devtools from non-tool packages.

## Required tests
- Replay viewer: loads a replay file and renders steps without error.
- State inspector: serializes and displays a `StateStore` snapshot.
- Catalog validator: validates a fixture catalog bundle and reports errors.

## Required fixtures
- `harness/goldens/replays/` — replay files for viewer tests.
- `harness/goldens/snapshots/` — state snapshots for inspector tests.

## Drift smells reviewers should flag
- Any non-tool package (`app`, `renderer-babylon`, `policy-core`, etc.) importing from `devtools`.
- Devtools gaining a code path that submits authority commands at runtime without explicit user action.
- Debug state or overlay logic appearing in `app` instead of `devtools`.

## Public API changes that require escalation
- None required — devtools is the leaf of the import graph. Internal changes do not ripple.

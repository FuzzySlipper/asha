# Navigation Pathfinding Substrate

Task #4041 activates the upstream `svc-pathfinding` lane with a read-only
navigation projection and deterministic path query over voxel authority.

The public Rust import path is:

```rust
use svc_pathfinding::{
    build_nav_projection, find_path, propose_projected_direct_nav_movement,
    NavPathQuery, NavProjectionConfig, ProjectedDirectNavMovementRequest,
};
```

This is projection/query infrastructure only. It does not implement enemy AI,
policy behavior, demo wiring, or movement authority.

## Named Surface

- Projection config: `NavProjectionConfig`
- Read-only projection: `NavProjection`
- Query: `NavPathQuery`
- Readout: `NavPathReadout`
- Outcome: `NavPathOutcome::{Reached, NoPath}`
- Rejections: `NavError::{InvalidAgentHeight, InvalidQueryBudget,
  StartNotWalkable, GoalNotWalkable}`
- Projection-backed direct navigation: `ProjectedDirectNavMovementRequest`,
  `propose_projected_direct_nav_movement`, `ProjectedDirectNavMovementReadout`,
  `ProjectedDirectNavMovementError`

`build_nav_projection` reads a `svc_spatial::VoxelWorld` and marks walkable
cells where the agent has empty vertical clearance and, by default, a solid
floor. `find_path` runs deterministic shortest-path search over the projection
using fixed X/Z neighbor order. The projection is read-only evidence suitable
for future policy views to inspect before proposing movement.

`propose_projected_direct_nav_movement` converts live positions into the
projection grid, queries `find_path`, and proposes one bounded waypoint toward
the next path cell (or the final target when the goal cell is next). The service
keeps no internal cache. Callers that cache externally must invalidate on the
readout/projection `projection_hash`; each movement readout also carries the
`path_hash` and a deterministic `movement_hash`.

## Evidence

The committed fixture uses the #4038 generated tunnel:

- `harness/fixtures/nav/generated-tunnel-path.snapshot.txt`

The focused tests cover:

- reachable path from `exit_hint`-style marker to `player_start`-style marker
- blocked/no-path projection
- deterministic path hash
- invalid query rejection for an unwalkable start
- projection-backed direct navigation obstacle/path following, no-path,
  same-cell reached behavior, invalid inputs/endpoints, and deterministic
  projection/path/movement hashes

Fixture values:

- walkable cells `66`
- projection hash `d1f6ac3e051d6b6e`
- path hash `e8e1ea7a09811ced`

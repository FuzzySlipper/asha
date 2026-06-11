//! Authority-safe picking / edit-anchor validation (voxel-capability-10).
//!
//! The renderer/UI builds a screen→world ray and may do its own visual picking
//! for responsiveness, but a UI-claimed hit is **only a hint**. Before any edit is
//! accepted, Rust revalidates the hint against the authoritative collision
//! projection through the *same shared query service* (no parallel DDA/raycast),
//! and turns a validated anchor into a canonical [`VoxelCommand`]. The UI never
//! mutates authoritative voxel state.
//!
//! ```text
//! TS screen ray + claimed hit ─▶ validate_pick(projection, hint) ─▶ ValidatedAnchor
//!   ▶ place_command / remove_command ─▶ VoxelCommand ─▶ rule_voxel_edit::validate ─▶ apply
//!                                  └─▶ PickRejection (stale / mismatched / no hit)
//! ```

use core_commands::VoxelCommand;
use core_space::{Face, GridId, VoxelCoord};
use core_voxel::VoxelValue;
use svc_collision::{CollisionProjection, Ray, VoxelHit};

/// An untrusted pick proposed by the renderer/UI: the ray it cast plus the voxel
/// and face it believes were hit. Revalidated by [`validate_pick`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RendererPickHint {
    pub ray: Ray,
    pub claimed_voxel: VoxelCoord,
    pub claimed_face: Face,
}

/// A pick that Rust has confirmed against authoritative state. Carries the
/// authoritative [`VoxelHit`] and the resolved edit anchors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidatedAnchor {
    /// The authoritative hit (its `voxel` is the cell a *remove* targets).
    pub hit: VoxelHit,
    /// The empty neighbour across the struck face — the cell a *place* targets.
    pub place_anchor: VoxelCoord,
}

/// Why a renderer pick hint was refused by authoritative revalidation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PickRejection {
    /// The authoritative raycast hit nothing within range — the UI pick was stale
    /// or aimed at empty space.
    NoHit,
    /// The authoritative hit disagrees with the claimed hit (stale projection or a
    /// renderer/authority mismatch). Carries both so a diagnostic overlay can show
    /// the discrepancy.
    HitMismatch {
        authoritative: VoxelHit,
        claimed_voxel: VoxelCoord,
        claimed_face: Face,
    },
}

/// Revalidate a renderer pick hint against the authoritative collision projection.
/// Accepts only when the authoritative nearest hit matches the claimed voxel+face.
pub fn validate_pick(
    projection: &CollisionProjection,
    hint: &RendererPickHint,
    max_distance: f64,
) -> Result<ValidatedAnchor, PickRejection> {
    match projection.raycast(hint.ray, max_distance) {
        None => Err(PickRejection::NoHit),
        Some(hit) if hit.voxel == hint.claimed_voxel && hit.face == hint.claimed_face => {
            Ok(ValidatedAnchor {
                hit,
                place_anchor: hit.voxel.neighbor(hit.face),
            })
        }
        Some(hit) => Err(PickRejection::HitMismatch {
            authoritative: hit,
            claimed_voxel: hint.claimed_voxel,
            claimed_face: hint.claimed_face,
        }),
    }
}

/// Build a canonical *place* command from a validated anchor (sets the empty cell
/// across the struck face). Still subject to [`crate::validate`] (material/resident).
pub fn place_command(grid: GridId, anchor: &ValidatedAnchor, value: VoxelValue) -> VoxelCommand {
    VoxelCommand::SetVoxel {
        grid,
        coord: anchor.place_anchor,
        value,
    }
}

/// Build a canonical *remove* command from a validated anchor (clears the hit cell).
pub fn remove_command(grid: GridId, anchor: &ValidatedAnchor) -> VoxelCommand {
    VoxelCommand::SetVoxel {
        grid,
        coord: anchor.hit.voxel,
        value: VoxelValue::EMPTY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply_all, validate};
    use core_space::{ChunkCoord, ChunkDims, LocalVoxelCoord, VoxelGridSpec, WorldPos, WorldVec};
    use core_voxel::{MaterialCatalog, VoxelMaterialId};
    use svc_spatial::VoxelWorld;
    use svc_volume::VoxelChunk;

    fn spec() -> VoxelGridSpec {
        VoxelGridSpec::new(GridId::new(0), 1.0, ChunkDims::cubic(8).unwrap()).unwrap()
    }

    fn materials() -> MaterialCatalog {
        MaterialCatalog::new([VoxelMaterialId::new(1)])
    }

    fn world_with_solid(local: LocalVoxelCoord) -> VoxelWorld {
        let mut w = VoxelWorld::new(spec());
        let mut chunk = VoxelChunk::from_spec(&spec());
        chunk.set(local, VoxelValue::solid_raw(1)).unwrap();
        w.insert(ChunkCoord::new(0, 0, 0), chunk);
        w.drain_dirty();
        w
    }

    fn ray_plus_x() -> Ray {
        Ray::new(WorldPos::new(0.0, 0.5, 0.5), WorldVec::new(1.0, 0.0, 0.0))
    }

    #[test]
    fn matching_renderer_hint_is_accepted_and_yields_place_remove_commands() {
        let world = world_with_solid(LocalVoxelCoord::new(5, 0, 0)); // world voxel (5,0,0)
        let proj = CollisionProjection::build(&world);
        // The renderer claims exactly what the authoritative raycast finds.
        let truth = proj.raycast(ray_plus_x(), 100.0).unwrap();
        let hint = RendererPickHint {
            ray: ray_plus_x(),
            claimed_voxel: truth.voxel,
            claimed_face: truth.face,
        };
        let anchor = validate_pick(&proj, &hint, 100.0).expect("matching hint accepted");
        assert_eq!(anchor.hit.voxel, VoxelCoord::new(5, 0, 0));
        assert_eq!(anchor.place_anchor, VoxelCoord::new(4, 0, 0)); // across the -X face

        // Place command targets the empty neighbour and survives authority validation.
        let mut world = world;
        let place = place_command(GridId::new(0), &anchor, VoxelValue::solid_raw(1));
        let events = validate(&place, &world, &materials()).unwrap();
        apply_all(&mut world, &events).unwrap();
        assert_eq!(
            world
                .get(ChunkCoord::new(0, 0, 0))
                .unwrap()
                .get(LocalVoxelCoord::new(4, 0, 0)),
            Some(VoxelValue::solid_raw(1)),
        );

        // Remove command clears the hit cell.
        let remove = remove_command(GridId::new(0), &anchor);
        let revents = validate(&remove, &world, &materials()).unwrap();
        apply_all(&mut world, &revents).unwrap();
        assert_eq!(
            world
                .get(ChunkCoord::new(0, 0, 0))
                .unwrap()
                .get(LocalVoxelCoord::new(5, 0, 0)),
            Some(VoxelValue::EMPTY),
        );
    }

    #[test]
    fn stale_renderer_hint_is_rejected_by_authority() {
        let world = world_with_solid(LocalVoxelCoord::new(5, 0, 0));
        let proj = CollisionProjection::build(&world);
        // Renderer claims a wrong voxel/face (e.g. its projection was stale).
        let hint = RendererPickHint {
            ray: ray_plus_x(),
            claimed_voxel: VoxelCoord::new(2, 0, 0), // not what authority sees
            claimed_face: Face::NegX,
        };
        match validate_pick(&proj, &hint, 100.0) {
            Err(PickRejection::HitMismatch {
                authoritative,
                claimed_voxel,
                ..
            }) => {
                assert_eq!(authoritative.voxel, VoxelCoord::new(5, 0, 0));
                assert_eq!(claimed_voxel, VoxelCoord::new(2, 0, 0));
            }
            other => panic!("expected HitMismatch, got {other:?}"),
        }
    }

    #[test]
    fn hint_into_empty_space_is_rejected_no_hit() {
        let world = world_with_solid(LocalVoxelCoord::new(5, 0, 0));
        let proj = CollisionProjection::build(&world);
        // Ray that never enters the solid cell.
        let hint = RendererPickHint {
            ray: Ray::new(WorldPos::new(0.0, 3.5, 0.5), WorldVec::new(1.0, 0.0, 0.0)),
            claimed_voxel: VoxelCoord::new(5, 0, 0),
            claimed_face: Face::NegX,
        };
        assert_eq!(
            validate_pick(&proj, &hint, 100.0),
            Err(PickRejection::NoHit)
        );
    }

    #[test]
    fn validation_does_not_mutate_authority_or_projection() {
        let world = world_with_solid(LocalVoxelCoord::new(5, 0, 0));
        let proj = CollisionProjection::build(&world);
        let before_version = proj.version();
        let before_hash = world.get(ChunkCoord::new(0, 0, 0)).unwrap().content_hash();
        let truth = proj.raycast(ray_plus_x(), 100.0).unwrap();
        let hint = RendererPickHint {
            ray: ray_plus_x(),
            claimed_voxel: truth.voxel,
            claimed_face: truth.face,
        };
        let _ = validate_pick(&proj, &hint, 100.0).unwrap();
        // Pick validation is read-only over authority + projection.
        assert_eq!(proj.version(), before_version);
        assert_eq!(
            world.get(ChunkCoord::new(0, 0, 0)).unwrap().content_hash(),
            before_hash
        );
    }
}

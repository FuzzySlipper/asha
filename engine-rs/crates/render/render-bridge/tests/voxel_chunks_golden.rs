//! Voxel chunk projector golden + reproject tests (#2435).
//!
//! Projects the canonical abstract voxel world (the same shape `fixture-maker`
//! commits in #2434: a 2×2×1 arrangement, solid bottom layers, materials by chunk)
//! into render diffs through the Rust [`VoxelChunkProjector`], pins the multi-chunk
//! seam/material-slot frame to a committed golden, and proves a single dirty-chunk
//! edit reprojects only the expected chunk + resident neighbours.
//!
//! Regenerate the golden with:
//!   BLESS=1 cargo test -p render-bridge --test voxel_chunks_golden

use std::path::PathBuf;

use core_space::{ChunkCoord, ChunkDims, GridId, LocalVoxelCoord, VoxelGridSpec};
use core_voxel::VoxelValue;
use protocol_render::RenderDiff;
use render_bridge::json;
use render_bridge::voxel::VoxelChunkProjector;
use svc_spatial::VoxelWorld;
use svc_volume::VoxelChunk;

/// The canonical grid (matches `fixture_maker::canonical_grid`).
fn grid() -> VoxelGridSpec {
    VoxelGridSpec::new(GridId::new(1), 1.0, ChunkDims::cubic(2).unwrap()).unwrap()
}

const ARRANGEMENT: [(i64, i64, i64); 4] = [(0, 0, 0), (1, 0, 0), (0, 1, 0), (1, 1, 0)];

fn material_for(coord: ChunkCoord) -> u16 {
    [1u16, 2, 3][(coord.x * 2 + coord.y).rem_euclid(3) as usize]
}

/// Build the canonical voxel world (bottom layer of each chunk solid).
fn canonical_world() -> VoxelWorld {
    let spec = grid();
    let dims = spec.chunk_dims();
    let mut world = VoxelWorld::new(spec);
    for (x, y, z) in ARRANGEMENT {
        let coord = ChunkCoord::new(x, y, z);
        let mut chunk = VoxelChunk::from_spec(&spec);
        chunk
            .fill_region(
                LocalVoxelCoord::new(0, 0, 0),
                LocalVoxelCoord::new(dims.x(), dims.y(), 1),
                VoxelValue::solid_raw(material_for(coord)),
            )
            .unwrap();
        world.insert(coord, chunk);
    }
    world
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../harness/fixtures/render-diffs/voxel-chunks.json")
}

#[test]
fn projects_canonical_world_to_committed_golden() {
    let mut world = canonical_world();
    let mut projector = VoxelChunkProjector::new();
    // Drain the insert-dirty set: a full projection of all four chunks.
    let frame = projector.project_dirty(&mut world);
    assert!(projector.diagnostics().is_empty());

    let actual = json::encode_frame(&frame);
    let path = golden_path();
    if std::env::var_os("BLESS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &actual).unwrap();
        return;
    }
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {} ({e}); run with BLESS=1 to create", path.display()));
    assert_eq!(
        actual,
        golden,
        "voxel chunk projection drifted from {} — regenerate with BLESS=1 if intended",
        path.display()
    );
}

#[test]
fn full_projection_creates_a_mesh_per_chunk() {
    let mut world = canonical_world();
    let mut projector = VoxelChunkProjector::new();
    let frame = projector.project_dirty(&mut world);

    let creates = frame
        .ops
        .iter()
        .filter(|o| matches!(o, RenderDiff::Create { .. }))
        .count();
    let payloads = frame
        .ops
        .iter()
        .filter(|o| matches!(o, RenderDiff::ReplaceMeshPayload { .. }))
        .count();
    assert_eq!(creates, 4, "one create per chunk");
    assert_eq!(payloads, 4, "one mesh payload per chunk");
    // Each chunk has a stable handle.
    for (x, y, z) in ARRANGEMENT {
        assert!(projector.handle_of(ChunkCoord::new(x, y, z)).is_some());
    }
}

#[test]
fn single_dirty_chunk_reprojects_only_that_chunk_and_resident_neighbours() {
    let mut world = canonical_world();
    let mut projector = VoxelChunkProjector::new();
    let _ = projector.project_dirty(&mut world); // initial full projection, drains dirty

    let edited = ChunkCoord::new(0, 0, 0);
    let handles_before: Vec<_> = ARRANGEMENT
        .iter()
        .map(|&(x, y, z)| projector.handle_of(ChunkCoord::new(x, y, z)).unwrap())
        .collect();

    // Edit one voxel and invalidate the chunk + its neighbours (authority's job).
    world
        .get_mut(edited)
        .unwrap()
        .set(LocalVoxelCoord::new(0, 0, 1), VoxelValue::solid_raw(2))
        .unwrap();
    world.mark_dirty_with_neighbors(edited);

    let frame = projector.project_dirty(&mut world);

    // Only the edited chunk and its RESIDENT neighbours ((1,0,0) and (0,1,0)) are
    // reprojected; the diagonal chunk (1,1,0) is untouched.
    let touched: std::collections::BTreeSet<u64> = frame
        .ops
        .iter()
        .map(|o| match o {
            RenderDiff::ReplaceMeshPayload { handle, .. } => handle.raw(),
            RenderDiff::Create { handle, .. } => handle.raw(),
            RenderDiff::Destroy { handle } => handle.raw(),
            _ => panic!("unexpected diff in voxel reprojection"),
        })
        .collect();
    let h = |c: ChunkCoord| projector.handle_of(c).unwrap().raw();
    let expected: std::collections::BTreeSet<u64> = [
        h(ChunkCoord::new(0, 0, 0)),
        h(ChunkCoord::new(1, 0, 0)),
        h(ChunkCoord::new(0, 1, 0)),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        touched, expected,
        "only chunk + resident neighbours reproject"
    );
    assert!(
        !touched.contains(&h(ChunkCoord::new(1, 1, 0))),
        "the diagonal chunk must not reproject"
    );

    // Reprojection is ReplaceMeshPayload (no Create/Destroy) — handles are stable.
    assert!(frame
        .ops
        .iter()
        .all(|o| matches!(o, RenderDiff::ReplaceMeshPayload { .. })));
    let handles_after: Vec<_> = ARRANGEMENT
        .iter()
        .map(|&(x, y, z)| projector.handle_of(ChunkCoord::new(x, y, z)).unwrap())
        .collect();
    assert_eq!(
        handles_before, handles_after,
        "chunk handles are stable across edits"
    );
}

#[test]
fn emptying_a_chunk_destroys_its_handle() {
    let mut world = canonical_world();
    let mut projector = VoxelChunkProjector::new();
    let _ = projector.project_dirty(&mut world);

    let target = ChunkCoord::new(1, 1, 0);
    let handle = projector.handle_of(target).unwrap();
    // Clear the whole chunk → no visible geometry.
    let dims = world.grid().chunk_dims();
    world
        .get_mut(target)
        .unwrap()
        .fill_region(
            LocalVoxelCoord::new(0, 0, 0),
            LocalVoxelCoord::new(dims.x(), dims.y(), dims.z()),
            VoxelValue::EMPTY,
        )
        .unwrap();
    world.mark_dirty_with_neighbors(target);

    let frame = projector.project_dirty(&mut world);
    assert!(
        frame
            .ops
            .iter()
            .any(|o| matches!(o, RenderDiff::Destroy { handle: h } if *h == handle)),
        "an emptied chunk's handle is destroyed"
    );
    assert!(projector.handle_of(target).is_none(), "handle is freed");
}

#[test]
fn strategy_label_is_exposed() {
    let projector = VoxelChunkProjector::new();
    assert_eq!(projector.strategy_label(), "visible-face");
}

use core_space::{ChunkCoord, VoxelCoord};
use core_voxel::VoxelMaterialId;
use runtime_bridge_api::{CommandResult, VoxelEditRejection};

use super::voxel_rejections::{NativeCommandResult, NativeVoxelEditRejection};
use super::{initialize_engine, submit_commands};

#[test]
fn every_authoritative_voxel_rejection_converts_to_its_exact_native_dto() {
    let native = NativeCommandResult::from(CommandResult {
        accepted: 2,
        rejected: 4,
        rejections: vec![
            VoxelEditRejection::UnknownMaterial(VoxelMaterialId::new(65535)),
            VoxelEditRejection::EmptyRegion {
                min: VoxelCoord::new(-3, 5, 8),
                max: VoxelCoord::new(-1, 5, 13),
            },
            VoxelEditRejection::ChunkNotResident {
                chunk: ChunkCoord::new(21, -34, 55),
            },
            VoxelEditRejection::GenerationDivergence {
                chunk: ChunkCoord::new(-89, 144, -233),
                expected: 377,
                actual: 610,
            },
        ],
    });

    assert_eq!(native.accepted, 2);
    assert_eq!(native.rejected, 4);
    assert_eq!(native.rejections.len(), 4);

    let NativeVoxelEditRejection::A(unknown_material) = &native.rejections[0] else {
        panic!("unknown material must be the first native rejection DTO");
    };
    assert_eq!(unknown_material.reason, "unknownMaterial");
    assert_eq!(unknown_material.material, 65535);

    let NativeVoxelEditRejection::B(empty_region) = &native.rejections[1] else {
        panic!("empty region must be the second native rejection DTO");
    };
    assert_eq!(empty_region.reason, "emptyRegion");
    assert_eq!(
        [empty_region.min.x, empty_region.min.y, empty_region.min.z],
        [-3, 5, 8]
    );
    assert_eq!(
        [empty_region.max.x, empty_region.max.y, empty_region.max.z],
        [-1, 5, 13]
    );

    let NativeVoxelEditRejection::C(chunk_not_resident) = &native.rejections[2] else {
        panic!("non-resident chunk must be the third native rejection DTO");
    };
    assert_eq!(chunk_not_resident.reason, "chunkNotResident");
    assert_eq!(
        [
            chunk_not_resident.chunk.x,
            chunk_not_resident.chunk.y,
            chunk_not_resident.chunk.z,
        ],
        [21, -34, 55]
    );

    let NativeVoxelEditRejection::D(generation_divergence) = &native.rejections[3] else {
        panic!("generation divergence must be the fourth native rejection DTO");
    };
    assert_eq!(generation_divergence.reason, "generationDivergence");
    assert_eq!(
        [
            generation_divergence.chunk.x,
            generation_divergence.chunk.y,
            generation_divergence.chunk.z,
        ],
        [-89, 144, -233]
    );
    assert_eq!(generation_divergence.expected, 377.0);
    assert_eq!(generation_divergence.actual, 610.0);
}

#[test]
fn native_voxel_rejections_use_generated_tagged_shapes() {
    let handle = initialize_engine(78).expect("engine initializes");

    let unknown_material = submit_commands(
        handle,
        r#"[{"op":"setVoxel","grid":1,"coord":{"x":0,"y":0,"z":0},"value":{"kind":"solid","material":65535}}]"#
            .to_string(),
    )
    .expect("unknown material is a classified rejection");
    assert_eq!(unknown_material.accepted, 0);
    assert_eq!(unknown_material.rejected, 1);
    match unknown_material.rejections.as_slice() {
        [NativeVoxelEditRejection::A(rejection)] => {
            assert_eq!(rejection.reason, "unknownMaterial");
            assert_eq!(rejection.material, 65535);
        }
        _ => panic!("unknown material must use its generated tagged DTO"),
    }

    let empty_region = submit_commands(
        handle,
        r#"[{"op":"fillRegion","grid":1,"min":{"x":1,"y":1,"z":1},"max":{"x":1,"y":1,"z":1},"value":{"kind":"empty"}}]"#
            .to_string(),
    )
    .expect("empty region is a classified rejection");
    assert_eq!(empty_region.accepted, 0);
    assert_eq!(empty_region.rejected, 1);
    match empty_region.rejections.as_slice() {
        [NativeVoxelEditRejection::B(rejection)] => {
            assert_eq!(rejection.reason, "emptyRegion");
            assert_eq!(
                [rejection.min.x, rejection.min.y, rejection.min.z],
                [1, 1, 1]
            );
            assert_eq!(
                [rejection.max.x, rejection.max.y, rejection.max.z],
                [1, 1, 1]
            );
        }
        _ => panic!("empty region must use its generated tagged DTO"),
    }

    let non_resident = submit_commands(
        handle,
        r#"[{"op":"setVoxel","grid":1,"coord":{"x":100,"y":0,"z":0},"value":{"kind":"empty"}}]"#
            .to_string(),
    )
    .expect("non-resident chunk is a classified rejection");
    assert_eq!(non_resident.accepted, 0);
    assert_eq!(non_resident.rejected, 1);
    match non_resident.rejections.as_slice() {
        [NativeVoxelEditRejection::C(rejection)] => {
            assert_eq!(rejection.reason, "chunkNotResident");
            assert_eq!(
                [rejection.chunk.x, rejection.chunk.y, rejection.chunk.z],
                [50, 0, 0]
            );
        }
        _ => panic!("non-resident chunk must use its generated tagged DTO"),
    }
}

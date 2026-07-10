use super::voxel_rejections::NativeVoxelEditRejection;
use super::{initialize_engine, submit_commands};

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

use napi::bindgen_prelude::Either4;
use napi_derive::napi;
use runtime_bridge_api::{CommandResult, VoxelEditRejection};

#[napi(object)]
pub struct NativeCommandResult {
    pub accepted: u32,
    pub rejected: u32,
    pub rejections: Vec<NativeVoxelEditRejection>,
}

#[napi(object)]
pub struct NativeGridCoord {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[napi(object)]
pub struct NativeUnknownMaterialRejection {
    pub reason: String,
    pub material: u32,
}

#[napi(object)]
pub struct NativeEmptyRegionRejection {
    pub reason: String,
    pub min: NativeGridCoord,
    pub max: NativeGridCoord,
}

#[napi(object)]
pub struct NativeChunkNotResidentRejection {
    pub reason: String,
    pub chunk: NativeGridCoord,
}

#[napi(object)]
pub struct NativeGenerationDivergenceRejection {
    pub reason: String,
    pub chunk: NativeGridCoord,
    pub expected: f64,
    pub actual: f64,
}

pub type NativeVoxelEditRejection = Either4<
    NativeUnknownMaterialRejection,
    NativeEmptyRegionRejection,
    NativeChunkNotResidentRejection,
    NativeGenerationDivergenceRejection,
>;

fn native_grid_coord([x, y, z]: [i64; 3]) -> NativeGridCoord {
    NativeGridCoord { x, y, z }
}

fn native_voxel_edit_rejection(value: VoxelEditRejection) -> NativeVoxelEditRejection {
    match value {
        VoxelEditRejection::UnknownMaterial(material) => {
            NativeVoxelEditRejection::A(NativeUnknownMaterialRejection {
                reason: "unknownMaterial".to_string(),
                material: u32::from(material.raw()),
            })
        }
        VoxelEditRejection::EmptyRegion { min, max } => {
            NativeVoxelEditRejection::B(NativeEmptyRegionRejection {
                reason: "emptyRegion".to_string(),
                min: native_grid_coord(min.to_array()),
                max: native_grid_coord(max.to_array()),
            })
        }
        VoxelEditRejection::ChunkNotResident { chunk } => {
            NativeVoxelEditRejection::C(NativeChunkNotResidentRejection {
                reason: "chunkNotResident".to_string(),
                chunk: native_grid_coord(chunk.to_array()),
            })
        }
        VoxelEditRejection::GenerationDivergence {
            chunk,
            expected,
            actual,
        } => NativeVoxelEditRejection::D(NativeGenerationDivergenceRejection {
            reason: "generationDivergence".to_string(),
            chunk: native_grid_coord(chunk.to_array()),
            expected: expected as f64,
            actual: actual as f64,
        }),
    }
}

impl From<CommandResult> for NativeCommandResult {
    fn from(value: CommandResult) -> Self {
        Self {
            accepted: value.accepted,
            rejected: value.rejected,
            rejections: value
                .rejections
                .into_iter()
                .map(native_voxel_edit_rejection)
                .collect(),
        }
    }
}

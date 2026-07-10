//! Bounded Rust-authoritative static mesh ingestion for voxel conversion.
//!
//! Hosts provide source bytes and provenance. This service parses the supported
//! GLB subset, computes the content hash, and returns canonical mesh geometry;
//! it performs no filesystem, network, renderer, or Studio work.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use gltf::{buffer::Source as BufferSource, mesh::Mode};
use protocol_voxel_conversion::{
    VoxelConversionMeshAsset, VoxelConversionMeshAssetGroup, VoxelConversionMeshSourceFormat,
    VoxelConversionMeshSourceImportRequest, VoxelConversionSourceMaterialSlot,
    VoxelConversionSourceRef, VOXEL_CONVERSION_MESH_IMPORT_MAX_ASSET_ID_BYTES,
    VOXEL_CONVERSION_MESH_IMPORT_MAX_INDICES, VOXEL_CONVERSION_MESH_IMPORT_MAX_PRIMITIVE_BYTES,
    VOXEL_CONVERSION_MESH_IMPORT_MAX_SOURCE_BYTES,
    VOXEL_CONVERSION_MESH_IMPORT_MAX_SOURCE_PATH_BYTES, VOXEL_CONVERSION_MESH_IMPORT_MAX_VERTICES,
};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshImportErrorKind {
    InvalidRequest,
    UnsupportedFeature,
    InvalidGeometry,
    QuotaExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshImportError {
    pub kind: MeshImportErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedMeshSource {
    pub source: VoxelConversionSourceRef,
    pub mesh_asset: VoxelConversionMeshAsset,
}

pub fn source_sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub fn import_static_mesh(
    request: &VoxelConversionMeshSourceImportRequest,
) -> Result<ImportedMeshSource, MeshImportError> {
    preflight_import_request(request)?;
    match request.format {
        VoxelConversionMeshSourceFormat::Glb => import_glb(request),
    }
}

pub fn preflight_import_request(
    request: &VoxelConversionMeshSourceImportRequest,
) -> Result<(), MeshImportError> {
    if request.source_asset_id.trim().is_empty()
        || request.asset_version == 0
        || request.source_path.trim().is_empty()
    {
        return Err(error(
            MeshImportErrorKind::InvalidRequest,
            "mesh import requires sourceAssetId, positive assetVersion, and sourcePath",
        ));
    }
    validate_string_bytes(
        "sourceAssetId",
        request.source_asset_id.len() as u64,
        VOXEL_CONVERSION_MESH_IMPORT_MAX_ASSET_ID_BYTES,
    )?;
    validate_string_bytes(
        "sourcePath",
        request.source_path.len() as u64,
        VOXEL_CONVERSION_MESH_IMPORT_MAX_SOURCE_PATH_BYTES,
    )?;
    if let Some(mesh_primitive) = &request.mesh_primitive {
        if mesh_primitive.trim().is_empty() {
            return Err(error(
                MeshImportErrorKind::InvalidRequest,
                "meshPrimitive must be omitted or non-empty",
            ));
        }
        validate_string_bytes(
            "meshPrimitive",
            mesh_primitive.len() as u64,
            VOXEL_CONVERSION_MESH_IMPORT_MAX_PRIMITIVE_BYTES,
        )?;
    }
    if request.source_bytes.is_empty() {
        return Err(error(
            MeshImportErrorKind::InvalidRequest,
            "mesh import source bytes are empty",
        ));
    }
    validate_source_byte_count(request.source_bytes.len() as u64)?;
    Ok(())
}

fn validate_string_bytes(label: &str, count: u64, limit: u64) -> Result<(), MeshImportError> {
    if count > limit {
        return Err(error(
            MeshImportErrorKind::QuotaExceeded,
            format!("mesh import {label} has {count} bytes; hard limit is {limit}"),
        ));
    }
    Ok(())
}

fn validate_source_byte_count(count: u64) -> Result<(), MeshImportError> {
    if count > VOXEL_CONVERSION_MESH_IMPORT_MAX_SOURCE_BYTES {
        return Err(error(
            MeshImportErrorKind::QuotaExceeded,
            format!(
                "mesh import source has {count} bytes; hard limit is {VOXEL_CONVERSION_MESH_IMPORT_MAX_SOURCE_BYTES}"
            ),
        ));
    }
    Ok(())
}

fn validate_cumulative_count(
    current: usize,
    incoming: usize,
    limit: u64,
    label: &str,
) -> Result<(), MeshImportError> {
    let total = (current as u64)
        .checked_add(incoming as u64)
        .ok_or_else(|| {
            error(
                MeshImportErrorKind::QuotaExceeded,
                format!("GLB cumulative {label} count overflowed"),
            )
        })?;
    if total > limit {
        return Err(error(
            MeshImportErrorKind::QuotaExceeded,
            format!("GLB cumulative {label} count {total} exceeds hard limit {limit}"),
        ));
    }
    Ok(())
}

fn import_glb(
    request: &VoxelConversionMeshSourceImportRequest,
) -> Result<ImportedMeshSource, MeshImportError> {
    let parsed = gltf::Gltf::from_slice(&request.source_bytes).map_err(|err| {
        error(
            MeshImportErrorKind::InvalidGeometry,
            format!("invalid GLB 2.0 source: {err}"),
        )
    })?;
    let blob = parsed.blob.as_deref().ok_or_else(|| {
        error(
            MeshImportErrorKind::UnsupportedFeature,
            "GLB source must contain an embedded BIN chunk",
        )
    })?;
    if parsed.document.animations().next().is_some() || parsed.document.skins().next().is_some() {
        return Err(error(
            MeshImportErrorKind::UnsupportedFeature,
            "animated or skinned GLB sources are outside the static mesh import subset",
        ));
    }
    for buffer in parsed.document.buffers() {
        if !matches!(buffer.source(), BufferSource::Bin) {
            return Err(error(
                MeshImportErrorKind::UnsupportedFeature,
                "GLB source may not reference external buffers",
            ));
        }
    }
    let mut meshes = parsed.document.meshes();
    let mesh = meshes.next().ok_or_else(|| {
        error(
            MeshImportErrorKind::InvalidGeometry,
            "GLB source contains no mesh",
        )
    })?;
    if meshes.next().is_some() {
        return Err(error(
            MeshImportErrorKind::UnsupportedFeature,
            "GLB import currently accepts exactly one static mesh",
        ));
    }

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut all_primitives_have_normals = true;
    let mut indices = Vec::new();
    let mut groups = Vec::new();
    let mut material_slots = BTreeMap::<u32, Option<String>>::new();
    let material_count = parsed.document.materials().count() as u32;

    for primitive in mesh.primitives() {
        if primitive.mode() != Mode::Triangles || primitive.morph_targets().next().is_some() {
            return Err(error(
                MeshImportErrorKind::UnsupportedFeature,
                "GLB primitives must be non-morphing indexed triangle lists",
            ));
        }
        let reader = primitive.reader(|buffer| match buffer.source() {
            BufferSource::Bin => Some(blob),
            BufferSource::Uri(_) => None,
        });
        let primitive_positions = reader.read_positions().ok_or_else(|| {
            error(
                MeshImportErrorKind::InvalidGeometry,
                "GLB primitive is missing POSITION data",
            )
        })?;
        let primitive_vertex_count = primitive_positions.len();
        validate_cumulative_count(
            positions.len(),
            primitive_vertex_count,
            VOXEL_CONVERSION_MESH_IMPORT_MAX_VERTICES,
            "vertex",
        )?;
        let vertex_offset = u32::try_from(positions.len()).map_err(|_| {
            error(
                MeshImportErrorKind::QuotaExceeded,
                "GLB vertex offset exceeds u32",
            )
        })?;
        let collected_positions = primitive_positions.collect::<Vec<_>>();
        if collected_positions
            .iter()
            .flatten()
            .any(|component| !component.is_finite())
        {
            return Err(error(
                MeshImportErrorKind::InvalidGeometry,
                "GLB POSITION data contains a non-finite component",
            ));
        }
        positions.extend(collected_positions);

        match reader.read_normals() {
            Some(values) if values.len() == primitive_vertex_count => normals.extend(values),
            None => all_primitives_have_normals = false,
            Some(_) => all_primitives_have_normals = false,
        }
        let primitive_indices = reader.read_indices().ok_or_else(|| {
            error(
                MeshImportErrorKind::UnsupportedFeature,
                "GLB primitives must provide an explicit index accessor",
            )
        })?;
        let primitive_indices = primitive_indices.into_u32();
        let primitive_index_count = primitive_indices.len();
        validate_cumulative_count(
            indices.len(),
            primitive_index_count,
            VOXEL_CONVERSION_MESH_IMPORT_MAX_INDICES,
            "index",
        )?;
        let start = u32::try_from(indices.len()).map_err(|_| {
            error(
                MeshImportErrorKind::QuotaExceeded,
                "GLB index offset exceeds u32",
            )
        })?;
        let local_indices = primitive_indices.collect::<Vec<_>>();
        if local_indices.len() % 3 != 0
            || local_indices
                .iter()
                .any(|index| *index as usize >= positions.len() - vertex_offset as usize)
        {
            return Err(error(
                MeshImportErrorKind::InvalidGeometry,
                "GLB primitive indices are not a valid triangle list",
            ));
        }
        indices.extend(local_indices.into_iter().map(|index| index + vertex_offset));
        let count = u32::try_from(indices.len() - start as usize).map_err(|_| {
            error(
                MeshImportErrorKind::QuotaExceeded,
                "GLB primitive index count exceeds u32",
            )
        })?;
        let material = primitive.material();
        let material_slot = material
            .index()
            .map(|index| index as u32)
            .unwrap_or(material_count + primitive.index() as u32);
        let material_name = material.name().map(str::to_string).or_else(|| {
            material
                .index()
                .map(|index| format!("gltf-material/{index}"))
        });
        material_slots.entry(material_slot).or_insert(material_name);
        groups.push(VoxelConversionMeshAssetGroup {
            material_slot,
            start,
            count,
        });
    }

    if positions.is_empty() || indices.is_empty() || groups.is_empty() {
        return Err(error(
            MeshImportErrorKind::InvalidGeometry,
            "GLB source produced no canonical triangle geometry",
        ));
    }
    if !all_primitives_have_normals || normals.len() != positions.len() {
        normals.clear();
    }

    let source_hash = source_sha256(&request.source_bytes);
    let source = VoxelConversionSourceRef {
        asset_id: request.source_asset_id.clone(),
        asset_kind: "mesh".to_string(),
        asset_version: request.asset_version,
        source_hash,
        mesh_primitive: request.mesh_primitive.clone(),
    };
    let mesh_asset = VoxelConversionMeshAsset {
        asset_id: request.source_asset_id.clone(),
        source_path: Some(request.source_path.clone()),
        positions,
        normals,
        indices,
        groups,
        material_slots: material_slots
            .into_iter()
            .map(
                |(source_material_slot, source_material_id)| VoxelConversionSourceMaterialSlot {
                    source_material_slot,
                    source_material_id,
                },
            )
            .collect(),
    };
    Ok(ImportedMeshSource { source, mesh_asset })
}

fn error(kind: MeshImportErrorKind, message: impl Into<String>) -> MeshImportError {
    MeshImportError {
        kind,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WALL_A: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../harness/fixtures/voxel-conversion/kenney-wall-a.glb"
    ));

    fn request() -> VoxelConversionMeshSourceImportRequest {
        VoxelConversionMeshSourceImportRequest {
            source_asset_id: "mesh/kenney-wall-a".to_string(),
            asset_version: 1,
            source_path: "assets/reference/kenney-wall-a.glb".to_string(),
            format: VoxelConversionMeshSourceFormat::Glb,
            source_bytes: WALL_A.to_vec(),
            mesh_primitive: None,
        }
    }

    #[test]
    fn imports_nontrivial_embedded_glb_with_canonical_metadata() {
        let imported = import_static_mesh(&request()).unwrap();
        assert_eq!(
            imported.source.source_hash,
            "sha256:6fceda24c30d2c22694f232f03fe2115fb1a462046fbbf719a90eea10dc9af00"
        );
        assert_eq!(imported.mesh_asset.positions.len(), 48);
        assert_eq!(imported.mesh_asset.indices.len(), 36);
        assert_eq!(imported.mesh_asset.groups.len(), 2);
        assert_eq!(imported.mesh_asset.material_slots.len(), 2);
        assert!(imported
            .mesh_asset
            .material_slots
            .iter()
            .any(|slot| slot.source_material_id.as_deref() == Some("wall_lines")));
        assert!(imported
            .mesh_asset
            .material_slots
            .iter()
            .any(|slot| slot.source_material_id.as_deref() == Some("concrete")));
    }

    #[test]
    fn rejects_empty_and_malformed_sources_without_geometry() {
        let mut empty = request();
        empty.source_bytes.clear();
        assert_eq!(
            import_static_mesh(&empty).unwrap_err().kind,
            MeshImportErrorKind::InvalidRequest
        );
        let mut malformed = request();
        malformed.source_bytes = vec![0, 1, 2, 3];
        assert_eq!(
            import_static_mesh(&malformed).unwrap_err().kind,
            MeshImportErrorKind::InvalidGeometry
        );
    }

    #[test]
    fn import_preflight_enforces_exact_source_and_string_limits() {
        assert!(validate_source_byte_count(VOXEL_CONVERSION_MESH_IMPORT_MAX_SOURCE_BYTES).is_ok());
        assert_eq!(
            validate_source_byte_count(VOXEL_CONVERSION_MESH_IMPORT_MAX_SOURCE_BYTES + 1)
                .unwrap_err()
                .kind,
            MeshImportErrorKind::QuotaExceeded
        );

        let mut at_limit = request();
        at_limit.source_asset_id =
            "a".repeat(usize::try_from(VOXEL_CONVERSION_MESH_IMPORT_MAX_ASSET_ID_BYTES).unwrap());
        at_limit.source_path = "p"
            .repeat(usize::try_from(VOXEL_CONVERSION_MESH_IMPORT_MAX_SOURCE_PATH_BYTES).unwrap());
        at_limit.mesh_primitive = Some(
            "m".repeat(usize::try_from(VOXEL_CONVERSION_MESH_IMPORT_MAX_PRIMITIVE_BYTES).unwrap()),
        );
        assert!(preflight_import_request(&at_limit).is_ok());

        for over_limit in [
            (
                "sourceAssetId",
                VOXEL_CONVERSION_MESH_IMPORT_MAX_ASSET_ID_BYTES,
            ),
            (
                "sourcePath",
                VOXEL_CONVERSION_MESH_IMPORT_MAX_SOURCE_PATH_BYTES,
            ),
            (
                "meshPrimitive",
                VOXEL_CONVERSION_MESH_IMPORT_MAX_PRIMITIVE_BYTES,
            ),
        ] {
            assert_eq!(
                validate_string_bytes(over_limit.0, over_limit.1 + 1, over_limit.1)
                    .unwrap_err()
                    .kind,
                MeshImportErrorKind::QuotaExceeded
            );
        }
    }

    #[test]
    fn cumulative_geometry_limits_reject_before_collection() {
        assert!(validate_cumulative_count(
            0,
            VOXEL_CONVERSION_MESH_IMPORT_MAX_VERTICES as usize,
            VOXEL_CONVERSION_MESH_IMPORT_MAX_VERTICES,
            "vertex",
        )
        .is_ok());
        assert_eq!(
            validate_cumulative_count(
                1,
                VOXEL_CONVERSION_MESH_IMPORT_MAX_VERTICES as usize,
                VOXEL_CONVERSION_MESH_IMPORT_MAX_VERTICES,
                "vertex",
            )
            .unwrap_err()
            .kind,
            MeshImportErrorKind::QuotaExceeded
        );
        assert!(validate_cumulative_count(
            0,
            VOXEL_CONVERSION_MESH_IMPORT_MAX_INDICES as usize,
            VOXEL_CONVERSION_MESH_IMPORT_MAX_INDICES,
            "index",
        )
        .is_ok());
        assert_eq!(
            validate_cumulative_count(
                0,
                VOXEL_CONVERSION_MESH_IMPORT_MAX_INDICES as usize + 1,
                VOXEL_CONVERSION_MESH_IMPORT_MAX_INDICES,
                "index",
            )
            .unwrap_err()
            .kind,
            MeshImportErrorKind::QuotaExceeded
        );
    }
}

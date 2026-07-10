use std::io::{Error, Write};

use super::*;

struct RequestSizeWriter {
    bytes_written: u64,
}

impl Write for RequestSizeWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let next_size = self
            .bytes_written
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| Error::other("voxel palette update request size overflowed"))?;
        if next_size > VOXEL_PALETTE_UPDATE_MAX_REQUEST_BYTES {
            return Err(Error::other(
                "voxel palette update request exceeds byte limit",
            ));
        }
        self.bytes_written = next_size;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl EngineBridge {
    pub(super) fn voxel_asset_palette_update_request_diagnostics(
        request: &VoxelVolumeAssetPaletteUpdateRequest,
    ) -> Vec<VoxelAssetDiagnostic> {
        let mut diagnostics = Self::voxel_asset_stored_target_diagnostics(
            &request.target_project_bundle,
            &request.target_asset_path,
        );
        Self::validate_palette_update_collection_limits(request, &mut diagnostics);
        if !diagnostics.is_empty() {
            return diagnostics;
        }

        if let Some((reference, byte_count)) = first_oversized_request_string(request) {
            diagnostics.push(Self::voxel_asset_diagnostic(
                VoxelAssetDiagnosticCode::ExportLimitExceeded,
                reference,
                format!(
                    "string has {byte_count} UTF-8 bytes; hard limit is {VOXEL_PALETTE_UPDATE_MAX_STRING_BYTES}"
                ),
            ));
            return diagnostics;
        }

        let mut size_writer = RequestSizeWriter { bytes_written: 0 };
        if serde_json::to_writer(&mut size_writer, request).is_err() {
            diagnostics.push(Self::voxel_asset_diagnostic(
                VoxelAssetDiagnosticCode::ExportLimitExceeded,
                "request",
                format!(
                    "serialized palette update request exceeds the hard {VOXEL_PALETTE_UPDATE_MAX_REQUEST_BYTES}-byte limit"
                ),
            ));
        }
        diagnostics
    }

    fn validate_palette_update_collection_limits(
        request: &VoxelVolumeAssetPaletteUpdateRequest,
        diagnostics: &mut Vec<VoxelAssetDiagnostic>,
    ) {
        if request.max_material_bindings == 0
            || request.max_material_bindings > VOXEL_PALETTE_UPDATE_MAX_MATERIAL_BINDINGS
        {
            diagnostics.push(Self::voxel_asset_diagnostic(
                VoxelAssetDiagnosticCode::ExportLimitExceeded,
                "maxMaterialBindings",
                format!(
                    "maxMaterialBindings must be in 1..={VOXEL_PALETTE_UPDATE_MAX_MATERIAL_BINDINGS}"
                ),
            ));
        }
        push_count_limit(
            diagnostics,
            "asset.materialPalette",
            request.asset.material_palette.len() as u64,
            VOXEL_PALETTE_UPDATE_MAX_MATERIAL_BINDINGS,
        );
        push_count_limit(
            diagnostics,
            "materialPalette",
            request.material_palette.len() as u64,
            VOXEL_PALETTE_UPDATE_MAX_MATERIAL_BINDINGS,
        );
        if request.max_material_bindings > 0
            && request.material_palette.len() as u64 > request.max_material_bindings
        {
            diagnostics.push(Self::voxel_asset_diagnostic(
                VoxelAssetDiagnosticCode::ExportLimitExceeded,
                "materialPalette",
                format!(
                    "material palette has {} entries; request limit is {}",
                    request.material_palette.len(),
                    request.max_material_bindings
                ),
            ));
        }
        push_count_limit(
            diagnostics,
            "asset.representation.sparseRuns",
            request.asset.representation.sparse_runs.len() as u64,
            VOXEL_PALETTE_UPDATE_MAX_SPARSE_RUNS,
        );
        push_count_limit(
            diagnostics,
            "asset.provenance",
            request.asset.provenance.len() as u64,
            VOXEL_PALETTE_UPDATE_MAX_PROVENANCE_REFS,
        );
        push_count_limit(
            diagnostics,
            "asset.validationDiagnostics",
            request.asset.validation_diagnostics.len() as u64,
            VOXEL_PALETTE_UPDATE_MAX_EMBEDDED_DIAGNOSTICS,
        );
        if !diagnostics.is_empty() {
            return;
        }

        let mut represented_voxels = 0_u64;
        for run in &request.asset.representation.sparse_runs {
            represented_voxels = represented_voxels.saturating_add(u64::from(run.length));
            if represented_voxels > VOXEL_PALETTE_UPDATE_MAX_REPRESENTED_VOXELS {
                diagnostics.push(Self::voxel_asset_diagnostic(
                    VoxelAssetDiagnosticCode::ExportLimitExceeded,
                    "asset.representation.representedVoxelCount",
                    format!(
                        "represented voxel count exceeds the hard {VOXEL_PALETTE_UPDATE_MAX_REPRESENTED_VOXELS}-voxel limit"
                    ),
                ));
                break;
            }
        }
    }
}

fn push_count_limit(
    diagnostics: &mut Vec<VoxelAssetDiagnostic>,
    reference: &str,
    count: u64,
    limit: u64,
) {
    if count > limit {
        diagnostics.push(EngineBridge::voxel_asset_diagnostic(
            VoxelAssetDiagnosticCode::ExportLimitExceeded,
            reference,
            format!("item count {count} exceeds the hard limit of {limit}"),
        ));
    }
}

fn first_oversized_request_string(
    request: &VoxelVolumeAssetPaletteUpdateRequest,
) -> Option<(String, usize)> {
    let mut strings = vec![
        (
            "targetProjectBundle".to_string(),
            request.target_project_bundle.as_str(),
        ),
        (
            "targetAssetPath".to_string(),
            request.target_asset_path.as_str(),
        ),
        (
            "expectedCanonicalJsonHash".to_string(),
            request.expected_canonical_json_hash.as_str(),
        ),
        (
            "expectedVoxelDataHash".to_string(),
            request.expected_voxel_data_hash.as_str(),
        ),
        ("asset.assetId".to_string(), request.asset.asset_id.as_str()),
        (
            "asset.mediaType".to_string(),
            request.asset.media_type.as_str(),
        ),
        (
            "asset.grid.coordinateSystem".to_string(),
            request.asset.grid.coordinate_system.as_str(),
        ),
        (
            "asset.contentHashes.canonicalJson".to_string(),
            request.asset.content_hashes.canonical_json.as_str(),
        ),
        (
            "asset.contentHashes.voxelData".to_string(),
            request.asset.content_hashes.voxel_data.as_str(),
        ),
    ];
    push_optional_string(
        &mut strings,
        "asset.authoring.label",
        request.asset.authoring.label.as_deref(),
    );
    push_optional_string(
        &mut strings,
        "asset.authoring.createdBy",
        request.asset.authoring.created_by.as_deref(),
    );
    push_optional_string(
        &mut strings,
        "asset.authoring.sourceTool",
        request.asset.authoring.source_tool.as_deref(),
    );
    push_material_strings(
        &mut strings,
        "asset.materialPalette",
        &request.asset.material_palette,
    );
    push_material_strings(&mut strings, "materialPalette", &request.material_palette);
    for (index, provenance) in request.asset.provenance.iter().enumerate() {
        strings.push((
            format!("asset.provenance[{index}].uri"),
            provenance.uri.as_str(),
        ));
        strings.push((
            format!("asset.provenance[{index}].contentHash"),
            provenance.content_hash.as_str(),
        ));
    }
    for (index, diagnostic) in request.asset.validation_diagnostics.iter().enumerate() {
        strings.push((
            format!("asset.validationDiagnostics[{index}].reference"),
            diagnostic.reference.as_str(),
        ));
        strings.push((
            format!("asset.validationDiagnostics[{index}].message"),
            diagnostic.message.as_str(),
        ));
    }
    strings.into_iter().find_map(|(reference, value)| {
        (value.len() as u64 > VOXEL_PALETTE_UPDATE_MAX_STRING_BYTES)
            .then_some((reference, value.len()))
    })
}

fn push_optional_string<'a>(
    strings: &mut Vec<(String, &'a str)>,
    reference: &str,
    value: Option<&'a str>,
) {
    if let Some(value) = value {
        strings.push((reference.to_string(), value));
    }
}

fn push_material_strings<'a>(
    strings: &mut Vec<(String, &'a str)>,
    reference: &str,
    palette: &'a [VoxelAssetMaterialBinding],
) {
    for (index, binding) in palette.iter().enumerate() {
        strings.push((
            format!("{reference}[{index}].paletteEntryId"),
            binding.palette_entry_id.as_str(),
        ));
        push_optional_string(
            strings,
            &format!("{reference}[{index}].displayName"),
            binding.display_name.as_deref(),
        );
        strings.push((
            format!("{reference}[{index}].materialAssetId"),
            binding.material_asset_id.as_str(),
        ));
        push_optional_string(
            strings,
            &format!("{reference}[{index}].materialCatalogBindingId"),
            binding.material_catalog_binding_id.as_deref(),
        );
    }
}

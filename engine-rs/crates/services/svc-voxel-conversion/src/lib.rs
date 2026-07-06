//! Rust authority service for bounded static-mesh to voxel conversion.
//!
//! # Lane
//!
//! `rust-service` — validates supported Asha static mesh/source assets and
//! produces deterministic voxel-conversion plans, previews, apply receipts, and
//! classified diagnostics. Studio and TypeScript consume the protocol DTOs; they
//! do not own conversion authority.
//!
//! # Current supported source shape
//!
//! This first slice accepts already-loaded static mesh source data: positions,
//! triangles, and source material slots. It intentionally does not import glTF,
//! read renderer buffers, or depend on Three.js/render protocol internals.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use core_voxel::VoxelValue;
use protocol_diagnostics::DiagnosticSeverity;
use protocol_voxel_conversion::{
    VoxelConversionApplyRequest, VoxelConversionBounds, VoxelConversionDiagnostic,
    VoxelConversionDiagnosticCode, VoxelConversionEvidenceKind, VoxelConversionEvidenceRef,
    VoxelConversionMode, VoxelConversionPlan, VoxelConversionPlanRequest, VoxelConversionPreview,
    VoxelConversionPreviewRequest, VoxelConversionPreviewVoxel, VoxelConversionReceipt,
    VoxelConversionSourceRef, VoxelConversionTargetRef,
};

pub const AUTHORITY_VERSION: &str = "svc-voxel-conversion.v0";

/// One supported static mesh source already loaded by Asha authority.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticMeshSource {
    pub asset_id: String,
    pub asset_kind: String,
    pub asset_version: u64,
    pub source_hash: String,
    pub mesh_primitive: Option<String>,
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<MeshTriangle>,
}

/// One triangle with a source material slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshTriangle {
    pub indices: [u32; 3],
    pub source_material_slot: u32,
}

/// Internal sparse authority voxel output. Absence is empty; present voxels are
/// always [`VoxelValue::Solid`] with a validated material id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertedVoxel {
    pub coord: protocol_voxel_conversion::VoxelConversionCoord,
    pub value: VoxelValue,
}

/// Full deterministic conversion output used by preview/apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionOutput {
    pub voxels: Vec<ConvertedVoxel>,
    pub bounds: Option<VoxelConversionBounds>,
    pub output_hash: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedConversion {
    pub plan: VoxelConversionPlan,
    pub output: Option<ConversionOutput>,
}

pub fn plan_conversion(
    request: &VoxelConversionPlanRequest,
    source: &StaticMeshSource,
) -> PlannedConversion {
    let mut diagnostics = Vec::new();
    validate_source_ref(&request.source, source, &mut diagnostics);
    validate_settings(request, source, &mut diagnostics);

    let output = if diagnostics.is_empty() {
        build_output(request, source, &mut diagnostics)
    } else {
        None
    };

    let estimated_bounds = output.as_ref().and_then(|o| o.bounds);
    let estimated_output_voxels = output.as_ref().map_or(0, |o| o.voxels.len() as u64);
    let plan_id = stable_hash(&[
        "plan",
        &request.source.asset_id,
        &request.source.source_hash,
        &settings_fingerprint(request),
    ]);
    let settings_hash = stable_hash(&["settings", &settings_fingerprint(request)]);
    let evidence = vec![evidence_ref(
        VoxelConversionEvidenceKind::Plan,
        format!("asha://voxel-conversion/plan/{plan_id}"),
        &stable_hash(&["plan-evidence", &plan_id, &settings_hash]),
    )];

    PlannedConversion {
        plan: VoxelConversionPlan {
            plan_id,
            source: request.source.clone(),
            target: request.target.clone(),
            settings: request.settings.clone(),
            authority_version: AUTHORITY_VERSION.to_string(),
            expected_source_hash: request.source.source_hash.clone(),
            settings_hash,
            estimated_output_voxels,
            estimated_bounds,
            diagnostics,
            evidence,
        },
        output,
    }
}

pub fn preview_conversion(
    request: &VoxelConversionPreviewRequest,
    planned: &PlannedConversion,
) -> VoxelConversionPreview {
    let expected = plan_hash(&planned.plan);
    if request.plan_id != planned.plan.plan_id || request.expected_plan_hash != expected {
        return VoxelConversionPreview {
            plan_id: request.plan_id.clone(),
            output_hash: String::new(),
            output_voxel_count: 0,
            output_bounds: None,
            sample_voxels: Vec::new(),
            diagnostics: vec![diagnostic(
                VoxelConversionDiagnosticCode::StaleAuthoritySnapshot,
                DiagnosticSeverity::Error,
                "plan",
                "preview request did not match the current authority plan hash",
            )],
            evidence: Vec::new(),
        };
    }

    let Some(output) = &planned.output else {
        return VoxelConversionPreview {
            plan_id: planned.plan.plan_id.clone(),
            output_hash: String::new(),
            output_voxel_count: 0,
            output_bounds: None,
            sample_voxels: Vec::new(),
            diagnostics: planned.plan.diagnostics.clone(),
            evidence: planned.plan.evidence.clone(),
        };
    };

    VoxelConversionPreview {
        plan_id: planned.plan.plan_id.clone(),
        output_hash: output.output_hash.clone(),
        output_voxel_count: output.voxels.len() as u64,
        output_bounds: output.bounds,
        sample_voxels: output
            .voxels
            .iter()
            .map(|voxel| VoxelConversionPreviewVoxel {
                coord: voxel.coord,
                material: voxel
                    .value
                    .material()
                    .expect("converted voxels are solid")
                    .raw(),
            })
            .collect(),
        diagnostics: planned.plan.diagnostics.clone(),
        evidence: vec![evidence_ref(
            VoxelConversionEvidenceKind::Preview,
            format!("asha://voxel-conversion/preview/{}", planned.plan.plan_id),
            &output.output_hash,
        )],
    }
}

pub fn apply_conversion(
    request: &VoxelConversionApplyRequest,
    planned: &PlannedConversion,
) -> VoxelConversionReceipt {
    let preview = preview_conversion(
        &VoxelConversionPreviewRequest {
            plan_id: request.plan_id.clone(),
            expected_plan_hash: request.expected_plan_hash.clone(),
        },
        planned,
    );

    if !preview.diagnostics.is_empty() {
        return rejected_receipt(request.plan_id.clone(), preview.diagnostics);
    }
    if let Some(expected_preview_hash) = &request.expected_preview_hash {
        if expected_preview_hash != &preview.output_hash {
            return rejected_receipt(
                request.plan_id.clone(),
                vec![diagnostic(
                    VoxelConversionDiagnosticCode::ConversionReplayMismatch,
                    DiagnosticSeverity::Error,
                    "preview",
                    "apply request expected a different preview output hash",
                )],
            );
        }
    }

    VoxelConversionReceipt {
        plan_id: request.plan_id.clone(),
        applied: true,
        output_hash: Some(preview.output_hash.clone()),
        output_voxel_count: preview.output_voxel_count,
        output_bounds: preview.output_bounds,
        diagnostics: Vec::new(),
        evidence: vec![evidence_ref(
            VoxelConversionEvidenceKind::ApplyReceipt,
            format!("asha://voxel-conversion/apply/{}", request.plan_id),
            &stable_hash(&["apply", &request.plan_id, &preview.output_hash]),
        )],
    }
}

pub fn plan_hash(plan: &VoxelConversionPlan) -> String {
    stable_hash(&[
        "plan-hash",
        &plan.plan_id,
        &plan.expected_source_hash,
        &plan.settings_hash,
        &plan.authority_version,
    ])
}

fn validate_source_ref(
    reference: &VoxelConversionSourceRef,
    source: &StaticMeshSource,
    diagnostics: &mut Vec<VoxelConversionDiagnostic>,
) {
    if reference.asset_id != source.asset_id
        || reference.asset_kind != source.asset_kind
        || reference.asset_version != source.asset_version
        || reference.mesh_primitive != source.mesh_primitive
        || reference.asset_kind != "mesh"
    {
        diagnostics.push(diagnostic(
            VoxelConversionDiagnosticCode::UnsupportedSourceAsset,
            DiagnosticSeverity::Error,
            &reference.asset_id,
            "source reference does not match a supported loaded static mesh asset",
        ));
    }
    if reference.source_hash != source.source_hash {
        diagnostics.push(diagnostic(
            VoxelConversionDiagnosticCode::SourceHashMismatch,
            DiagnosticSeverity::Error,
            &reference.asset_id,
            "source hash does not match the loaded static mesh authority snapshot",
        ));
    }
    if source.triangles.is_empty() || source.positions.is_empty() {
        diagnostics.push(diagnostic(
            VoxelConversionDiagnosticCode::UnsupportedSourceAsset,
            DiagnosticSeverity::Error,
            &reference.asset_id,
            "static mesh source must contain positions and triangles",
        ));
    }
    for triangle in &source.triangles {
        if triangle
            .indices
            .iter()
            .any(|index| *index as usize >= source.positions.len())
        {
            diagnostics.push(diagnostic(
                VoxelConversionDiagnosticCode::UnsupportedSourceAsset,
                DiagnosticSeverity::Error,
                &reference.asset_id,
                "triangle index is outside the static mesh position buffer",
            ));
            break;
        }
    }
}

fn validate_settings(
    request: &VoxelConversionPlanRequest,
    source: &StaticMeshSource,
    diagnostics: &mut Vec<VoxelConversionDiagnostic>,
) {
    if request.settings.resolution.contains(&0)
        || !request.settings.voxel_size.is_finite()
        || request.settings.voxel_size <= 0.0
        || request
            .settings
            .transform
            .iter()
            .any(|value| !value.is_finite())
    {
        diagnostics.push(diagnostic(
            VoxelConversionDiagnosticCode::UnsupportedSourceAsset,
            DiagnosticSeverity::Error,
            "settings",
            "conversion settings contain non-finite values or zero resolution",
        ));
    }
    if let Err(message) = validate_material_map(request, source) {
        diagnostics.push(diagnostic(
            VoxelConversionDiagnosticCode::InvalidMaterialMap,
            DiagnosticSeverity::Error,
            "materialMap",
            message,
        ));
    }
    if request.settings.mode == VoxelConversionMode::Solid && !is_closed_manifold(source) {
        diagnostics.push(diagnostic(
            VoxelConversionDiagnosticCode::NonManifoldOrAmbiguousSolid,
            DiagnosticSeverity::Error,
            &request.source.asset_id,
            "solid conversion requires each undirected mesh edge to be used exactly twice",
        ));
    }
}

fn validate_material_map(
    request: &VoxelConversionPlanRequest,
    source: &StaticMeshSource,
) -> Result<(), &'static str> {
    let mut map_slots = BTreeSet::new();
    for entry in &request.settings.material_map.entries {
        if !map_slots.insert(entry.source_material_slot) {
            return Err("duplicate source material slot in material map");
        }
    }
    if request
        .settings
        .material_map
        .default_voxel_material
        .is_none()
    {
        for slot in source_material_slots(source) {
            if !map_slots.contains(&slot) {
                return Err("source material slot is unmapped and no default material is set");
            }
        }
    }
    Ok(())
}

fn build_output(
    request: &VoxelConversionPlanRequest,
    source: &StaticMeshSource,
    diagnostics: &mut Vec<VoxelConversionDiagnostic>,
) -> Option<ConversionOutput> {
    let voxels = match request.settings.mode {
        VoxelConversionMode::Surface => surface_voxels(request, source),
        VoxelConversionMode::Solid => solid_voxels(request, source),
    };
    if voxels.len() as u64 > request.settings.max_output_voxels {
        diagnostics.push(diagnostic(
            VoxelConversionDiagnosticCode::OutputLimitExceeded,
            DiagnosticSeverity::Error,
            "maxOutputVoxels",
            "conversion output exceeds the requested maximum voxel count",
        ));
        return None;
    }
    let bounds = bounds_for(&voxels);
    let output_hash = output_hash(&voxels);
    Some(ConversionOutput {
        voxels,
        bounds,
        output_hash,
    })
}

fn surface_voxels(
    request: &VoxelConversionPlanRequest,
    source: &StaticMeshSource,
) -> Vec<ConvertedVoxel> {
    let mapper = CoordMapper::new(request, source);
    let material_map = material_lookup(request);
    let mut voxels = BTreeMap::new();
    for triangle in &source.triangles {
        let material = material_for(&material_map, request, triangle.source_material_slot);
        for index in triangle.indices {
            let coord = mapper.map(source.positions[index as usize]);
            voxels.insert(coord_key(coord), VoxelValue::solid_raw(material));
        }
    }
    voxels
        .into_iter()
        .map(|((x, y, z), value)| ConvertedVoxel {
            coord: protocol_voxel_conversion::VoxelConversionCoord { x, y, z },
            value,
        })
        .collect()
}

fn solid_voxels(
    request: &VoxelConversionPlanRequest,
    source: &StaticMeshSource,
) -> Vec<ConvertedVoxel> {
    let material_map = material_lookup(request);
    let material = source
        .triangles
        .first()
        .map(|triangle| material_for(&material_map, request, triangle.source_material_slot))
        .unwrap_or_else(|| {
            request
                .settings
                .material_map
                .default_voxel_material
                .unwrap_or(1)
        });
    let [rx, ry, rz] = request.settings.resolution;
    let mut voxels = Vec::with_capacity((rx as usize) * (ry as usize) * (rz as usize));
    for z in 0..rz {
        for y in 0..ry {
            for x in 0..rx {
                voxels.push(ConvertedVoxel {
                    coord: protocol_voxel_conversion::VoxelConversionCoord {
                        x: request.target.origin.x + x as i64,
                        y: request.target.origin.y + y as i64,
                        z: request.target.origin.z + z as i64,
                    },
                    value: VoxelValue::solid_raw(material),
                });
            }
        }
    }
    voxels
}

fn source_material_slots(source: &StaticMeshSource) -> BTreeSet<u32> {
    source
        .triangles
        .iter()
        .map(|triangle| triangle.source_material_slot)
        .collect()
}

fn material_lookup(request: &VoxelConversionPlanRequest) -> BTreeMap<u32, u16> {
    request
        .settings
        .material_map
        .entries
        .iter()
        .map(|entry| (entry.source_material_slot, entry.voxel_material))
        .collect()
}

fn material_for(
    material_map: &BTreeMap<u32, u16>,
    request: &VoxelConversionPlanRequest,
    source_slot: u32,
) -> u16 {
    material_map
        .get(&source_slot)
        .copied()
        .or(request.settings.material_map.default_voxel_material)
        .expect("material map was validated before conversion")
}

fn is_closed_manifold(source: &StaticMeshSource) -> bool {
    let mut edges: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for triangle in &source.triangles {
        let [a, b, c] = triangle.indices;
        for (u, v) in [(a, b), (b, c), (c, a)] {
            let edge = if u <= v { (u, v) } else { (v, u) };
            *edges.entry(edge).or_default() += 1;
        }
    }
    !edges.is_empty() && edges.values().all(|count| *count == 2)
}

fn bounds_for(voxels: &[ConvertedVoxel]) -> Option<VoxelConversionBounds> {
    let first = voxels.first()?.coord;
    let mut min = first;
    let mut max = first;
    for voxel in voxels.iter().skip(1) {
        min.x = min.x.min(voxel.coord.x);
        min.y = min.y.min(voxel.coord.y);
        min.z = min.z.min(voxel.coord.z);
        max.x = max.x.max(voxel.coord.x);
        max.y = max.y.max(voxel.coord.y);
        max.z = max.z.max(voxel.coord.z);
    }
    Some(VoxelConversionBounds { min, max })
}

fn output_hash(voxels: &[ConvertedVoxel]) -> String {
    let mut parts = Vec::with_capacity(voxels.len() * 4 + 1);
    parts.push("output".to_string());
    for voxel in voxels {
        parts.push(voxel.coord.x.to_string());
        parts.push(voxel.coord.y.to_string());
        parts.push(voxel.coord.z.to_string());
        parts.push(voxel.value.to_encoded().to_string());
    }
    stable_hash(&parts.iter().map(String::as_str).collect::<Vec<_>>())
}

fn settings_fingerprint(request: &VoxelConversionPlanRequest) -> String {
    let mut parts = vec![
        request.settings.mode.as_str().to_string(),
        request.settings.fit_policy.as_str().to_string(),
        request.settings.origin_policy.as_str().to_string(),
        format!("{:?}", request.settings.resolution),
        request.settings.voxel_size.to_bits().to_string(),
        request.settings.max_output_voxels.to_string(),
        request.target.grid.to_string(),
        format!(
            "{},{},{}",
            request.target.origin.x, request.target.origin.y, request.target.origin.z
        ),
    ];
    for value in request.settings.transform {
        parts.push(value.to_bits().to_string());
    }
    for entry in &request.settings.material_map.entries {
        parts.push(format!(
            "{}:{}",
            entry.source_material_slot, entry.voxel_material
        ));
    }
    if let Some(default) = request.settings.material_map.default_voxel_material {
        parts.push(format!("default:{default}"));
    }
    stable_hash(&parts.iter().map(String::as_str).collect::<Vec<_>>())
}

fn diagnostic(
    code: VoxelConversionDiagnosticCode,
    severity: DiagnosticSeverity,
    reference: impl Into<String>,
    message: impl Into<String>,
) -> VoxelConversionDiagnostic {
    VoxelConversionDiagnostic {
        code,
        severity,
        reference: reference.into(),
        message: message.into(),
    }
}

fn evidence_ref(
    kind: VoxelConversionEvidenceKind,
    uri: String,
    content_hash: &str,
) -> VoxelConversionEvidenceRef {
    VoxelConversionEvidenceRef {
        kind,
        uri,
        content_hash: content_hash.to_string(),
    }
}

fn rejected_receipt(
    plan_id: String,
    diagnostics: Vec<VoxelConversionDiagnostic>,
) -> VoxelConversionReceipt {
    VoxelConversionReceipt {
        plan_id,
        applied: false,
        output_hash: None,
        output_voxel_count: 0,
        output_bounds: None,
        diagnostics,
        evidence: Vec::new(),
    }
}

fn coord_key(coord: protocol_voxel_conversion::VoxelConversionCoord) -> (i64, i64, i64) {
    (coord.x, coord.y, coord.z)
}

struct CoordMapper {
    min: [f32; 3],
    span: [f32; 3],
    target: VoxelConversionTargetRef,
    resolution: [u32; 3],
}

impl CoordMapper {
    fn new(request: &VoxelConversionPlanRequest, source: &StaticMeshSource) -> Self {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for position in &source.positions {
            for axis in 0..3 {
                min[axis] = min[axis].min(position[axis]);
                max[axis] = max[axis].max(position[axis]);
            }
        }
        let span = [
            (max[0] - min[0]).max(f32::EPSILON),
            (max[1] - min[1]).max(f32::EPSILON),
            (max[2] - min[2]).max(f32::EPSILON),
        ];
        Self {
            min,
            span,
            target: request.target.clone(),
            resolution: request.settings.resolution,
        }
    }

    fn map(&self, position: [f32; 3]) -> protocol_voxel_conversion::VoxelConversionCoord {
        let mut out = [0i64; 3];
        for axis in 0..3 {
            let normalized = ((position[axis] - self.min[axis]) / self.span[axis]).clamp(0.0, 1.0);
            let max_index = self.resolution[axis].saturating_sub(1) as f32;
            out[axis] = (normalized * max_index).round() as i64;
        }
        protocol_voxel_conversion::VoxelConversionCoord {
            x: self.target.origin.x + out[0],
            y: self.target.origin.y + out[1],
            z: self.target.origin.z + out[2],
        }
    }
}

fn stable_hash(parts: &[&str]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol_voxel_conversion::{
        VoxelConversionCoord, VoxelConversionFitPolicy, VoxelConversionMaterialMap,
        VoxelConversionMaterialMapEntry, VoxelConversionOriginPolicy, VoxelConversionSettings,
    };

    fn identity() -> [f32; 16] {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn quad_source() -> StaticMeshSource {
        StaticMeshSource {
            asset_id: "mesh/quad".to_string(),
            asset_kind: "mesh".to_string(),
            asset_version: 1,
            source_hash: "sha256:quad".to_string(),
            mesh_primitive: None,
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            triangles: vec![
                MeshTriangle {
                    indices: [0, 1, 2],
                    source_material_slot: 0,
                },
                MeshTriangle {
                    indices: [0, 2, 3],
                    source_material_slot: 1,
                },
            ],
        }
    }

    fn cube_source() -> StaticMeshSource {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let faces = [
            ([0, 1, 2], [0, 2, 3]),
            ([4, 6, 5], [4, 7, 6]),
            ([0, 4, 5], [0, 5, 1]),
            ([1, 5, 6], [1, 6, 2]),
            ([2, 6, 7], [2, 7, 3]),
            ([3, 7, 4], [3, 4, 0]),
        ];
        let triangles = faces
            .into_iter()
            .flat_map(|(a, b)| {
                [
                    MeshTriangle {
                        indices: a,
                        source_material_slot: 0,
                    },
                    MeshTriangle {
                        indices: b,
                        source_material_slot: 0,
                    },
                ]
            })
            .collect();
        StaticMeshSource {
            asset_id: "mesh/cube".to_string(),
            asset_kind: "mesh".to_string(),
            asset_version: 1,
            source_hash: "sha256:cube".to_string(),
            mesh_primitive: None,
            positions,
            triangles,
        }
    }

    fn request_for(
        source: &StaticMeshSource,
        mode: VoxelConversionMode,
        resolution: [u32; 3],
        max_output_voxels: u64,
    ) -> VoxelConversionPlanRequest {
        VoxelConversionPlanRequest {
            source: VoxelConversionSourceRef {
                asset_id: source.asset_id.clone(),
                asset_kind: source.asset_kind.clone(),
                asset_version: source.asset_version,
                source_hash: source.source_hash.clone(),
                mesh_primitive: source.mesh_primitive.clone(),
            },
            target: VoxelConversionTargetRef {
                grid: 7,
                volume_asset_id: Some("voxel/generated".to_string()),
                origin: VoxelConversionCoord { x: 0, y: 0, z: 0 },
            },
            settings: VoxelConversionSettings {
                mode,
                fit_policy: VoxelConversionFitPolicy::Contain,
                origin_policy: VoxelConversionOriginPolicy::TargetMin,
                resolution,
                voxel_size: 1.0,
                max_output_voxels,
                transform: identity(),
                material_map: VoxelConversionMaterialMap {
                    entries: vec![
                        VoxelConversionMaterialMapEntry {
                            source_material_slot: 0,
                            source_material_id: Some("mat/a".to_string()),
                            voxel_material: 3,
                        },
                        VoxelConversionMaterialMapEntry {
                            source_material_slot: 1,
                            source_material_id: Some("mat/b".to_string()),
                            voxel_material: 5,
                        },
                    ],
                    default_voxel_material: None,
                },
            },
        }
    }

    #[test]
    fn synthetic_quad_surface_plans_and_previews_two_material_slots() {
        let source = quad_source();
        let request = request_for(&source, VoxelConversionMode::Surface, [4, 4, 1], 16);
        let planned = plan_conversion(&request, &source);
        assert!(planned.plan.diagnostics.is_empty());
        assert_eq!(planned.plan.estimated_output_voxels, 4);
        assert_eq!(planned.plan.estimated_bounds.unwrap().max.x, 3);

        let preview = preview_conversion(
            &VoxelConversionPreviewRequest {
                plan_id: planned.plan.plan_id.clone(),
                expected_plan_hash: plan_hash(&planned.plan),
            },
            &planned,
        );
        assert_eq!(preview.output_voxel_count, 4);
        assert!(preview
            .sample_voxels
            .iter()
            .any(|voxel| voxel.material == 3));
        assert!(preview
            .sample_voxels
            .iter()
            .any(|voxel| voxel.material == 5));
    }

    #[test]
    fn synthetic_cube_solid_fills_resolution_volume() {
        let source = cube_source();
        let request = request_for(&source, VoxelConversionMode::Solid, [2, 2, 2], 8);
        let planned = plan_conversion(&request, &source);
        assert!(planned.plan.diagnostics.is_empty());
        assert_eq!(planned.plan.estimated_output_voxels, 8);
        assert_eq!(
            planned.output.as_ref().unwrap().voxels[0].value,
            VoxelValue::solid_raw(3)
        );
    }

    #[test]
    fn invalid_material_map_fails_closed_without_output() {
        let source = quad_source();
        let mut request = request_for(&source, VoxelConversionMode::Surface, [4, 4, 1], 16);
        request.settings.material_map.entries.pop();
        let planned = plan_conversion(&request, &source);
        assert!(planned.output.is_none());
        assert_eq!(
            planned.plan.diagnostics[0].code,
            VoxelConversionDiagnosticCode::InvalidMaterialMap
        );
    }

    #[test]
    fn unsupported_topology_rejects_solid_mode() {
        let source = quad_source();
        let request = request_for(&source, VoxelConversionMode::Solid, [2, 2, 2], 8);
        let planned = plan_conversion(&request, &source);
        assert!(planned.output.is_none());
        assert_eq!(
            planned.plan.diagnostics[0].code,
            VoxelConversionDiagnosticCode::NonManifoldOrAmbiguousSolid
        );
    }

    #[test]
    fn oversized_output_rejects_without_best_effort_output() {
        let source = cube_source();
        let request = request_for(&source, VoxelConversionMode::Solid, [2, 2, 2], 7);
        let planned = plan_conversion(&request, &source);
        assert!(planned.output.is_none());
        assert_eq!(
            planned.plan.diagnostics[0].code,
            VoxelConversionDiagnosticCode::OutputLimitExceeded
        );
    }

    #[test]
    fn stale_source_hash_rejects_without_output() {
        let source = cube_source();
        let mut request = request_for(&source, VoxelConversionMode::Solid, [2, 2, 2], 8);
        request.source.source_hash = "sha256:stale".to_string();
        let planned = plan_conversion(&request, &source);
        assert!(planned.output.is_none());
        assert_eq!(
            planned.plan.diagnostics[0].code,
            VoxelConversionDiagnosticCode::SourceHashMismatch
        );
    }

    #[test]
    fn apply_receipt_is_replay_hash_checked() {
        let source = cube_source();
        let request = request_for(&source, VoxelConversionMode::Solid, [2, 2, 2], 8);
        let planned = plan_conversion(&request, &source);
        let preview = preview_conversion(
            &VoxelConversionPreviewRequest {
                plan_id: planned.plan.plan_id.clone(),
                expected_plan_hash: plan_hash(&planned.plan),
            },
            &planned,
        );
        let receipt = apply_conversion(
            &VoxelConversionApplyRequest {
                plan_id: planned.plan.plan_id.clone(),
                expected_plan_hash: plan_hash(&planned.plan),
                expected_preview_hash: Some(preview.output_hash),
            },
            &planned,
        );
        assert!(receipt.applied);
        assert_eq!(receipt.output_voxel_count, 8);
        assert!(receipt.output_hash.is_some());
    }

    #[test]
    fn committed_golden_summaries_cover_success_and_failure_cases() {
        assert_eq!(
            conversion_golden_summary().trim(),
            include_str!(
                "../../../../../harness/goldens/voxel-conversion/conversion-summary.golden"
            )
            .trim()
        );
    }

    fn conversion_golden_summary() -> String {
        let quad = quad_source();
        let quad_plan = plan_conversion(
            &request_for(&quad, VoxelConversionMode::Surface, [4, 4, 1], 16),
            &quad,
        );
        let quad_preview = preview_conversion(
            &VoxelConversionPreviewRequest {
                plan_id: quad_plan.plan.plan_id.clone(),
                expected_plan_hash: plan_hash(&quad_plan.plan),
            },
            &quad_plan,
        );

        let cube = cube_source();
        let cube_plan = plan_conversion(
            &request_for(&cube, VoxelConversionMode::Solid, [2, 2, 2], 8),
            &cube,
        );
        let oversized = plan_conversion(
            &request_for(&cube, VoxelConversionMode::Solid, [2, 2, 2], 7),
            &cube,
        );

        let mut stale_request = request_for(&cube, VoxelConversionMode::Solid, [2, 2, 2], 8);
        stale_request.source.source_hash = "sha256:stale".to_string();
        let stale = plan_conversion(&stale_request, &cube);

        format!(
            "quad.surface.voxels={}\nquad.surface.bounds={}\nquad.surface.materials={}\ncube.solid.voxels={}\ncube.solid.bounds={}\ncube.solid.materials={}\noversized.code={}\nstale.code={}\n",
            quad_preview.output_voxel_count,
            bounds_label(quad_preview.output_bounds),
            material_label(&quad_preview.sample_voxels),
            cube_plan.plan.estimated_output_voxels,
            bounds_label(cube_plan.plan.estimated_bounds),
            output_material_label(cube_plan.output.as_ref().unwrap()),
            oversized.plan.diagnostics[0].code.as_str(),
            stale.plan.diagnostics[0].code.as_str(),
        )
    }

    fn bounds_label(bounds: Option<VoxelConversionBounds>) -> String {
        let Some(bounds) = bounds else {
            return "none".to_string();
        };
        format!(
            "{},{},{}..{},{},{}",
            bounds.min.x, bounds.min.y, bounds.min.z, bounds.max.x, bounds.max.y, bounds.max.z
        )
    }

    fn material_label(voxels: &[VoxelConversionPreviewVoxel]) -> String {
        let materials: BTreeSet<u16> = voxels.iter().map(|voxel| voxel.material).collect();
        materials
            .into_iter()
            .map(|material| material.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    fn output_material_label(output: &ConversionOutput) -> String {
        let materials: BTreeSet<u16> = output
            .voxels
            .iter()
            .map(|voxel| voxel.value.material().unwrap().raw())
            .collect();
        materials
            .into_iter()
            .map(|material| material.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

use std::collections::{BTreeMap, BTreeSet};

use core_space::{ChunkCoord, VoxelCoord};
use core_voxel::VoxelValue;
use svc_spatial::VoxelWorld;

/// Inclusive voxel-space bounds touched by a diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelEditHistoryBounds {
    pub min: VoxelCoord,
    pub max: VoxelCoord,
}

/// Material count change within the bounded diff sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelEditHistoryMaterialDelta {
    /// `None` means empty/air. Solid materials carry their raw material id.
    pub material: Option<u16>,
    pub before_count: u64,
    pub target_count: u64,
    pub delta: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelEditHistoryDiffDiagnostic {
    DiffTruncated { limit: usize, observed: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelEditHistoryDiffOptions {
    pub max_changed_voxels: usize,
    pub include_sample_window: bool,
}

impl VoxelEditHistoryDiffOptions {
    pub const fn new(max_changed_voxels: usize, include_sample_window: bool) -> Self {
        Self {
            max_changed_voxels,
            include_sample_window,
        }
    }
}

/// Bounded diff readout for a replayed target cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoxelEditHistoryDiffSummary {
    pub before_hash: u64,
    pub current_hash: u64,
    pub target_hash: u64,
    pub projected_hash: Option<u64>,
    pub partial: bool,
    /// Exact when `partial == false`; lower-bound observed count when partial.
    pub changed_voxel_count: u64,
    pub touched_bounds: Option<VoxelEditHistoryBounds>,
    pub material_deltas: Vec<VoxelEditHistoryMaterialDelta>,
    pub included_transaction_ids: Vec<u64>,
    pub changed_transaction_count: usize,
    pub replayed_transaction_count: usize,
    pub sample_window_ref: Option<String>,
    pub diagnostics: Vec<VoxelEditHistoryDiffDiagnostic>,
}

pub(super) struct VoxelEditHistoryDiffContext {
    pub current_hash: u64,
    pub before_hash: u64,
    pub target_hash: u64,
    pub included_transaction_ids: Vec<u64>,
    pub changed_transaction_count: usize,
    pub replayed_transaction_count: usize,
    pub options: VoxelEditHistoryDiffOptions,
}

pub(super) fn summarize_world_diff(
    before: &VoxelWorld,
    target: &VoxelWorld,
    context: VoxelEditHistoryDiffContext,
) -> VoxelEditHistoryDiffSummary {
    let mut changed_voxel_count = 0u64;
    let mut touched_bounds: Option<VoxelEditHistoryBounds> = None;
    let mut material_counts: BTreeMap<Option<u16>, (u64, u64)> = BTreeMap::new();
    let mut partial = false;

    for chunk_coord in diff_chunk_coords(before, target) {
        let Some(template_chunk) = before.get(chunk_coord).or_else(|| target.get(chunk_coord))
        else {
            continue;
        };
        for (local, _) in template_chunk.iter() {
            let before_value = before
                .get(chunk_coord)
                .and_then(|chunk| chunk.get(local))
                .unwrap_or(VoxelValue::EMPTY);
            let target_value = target
                .get(chunk_coord)
                .and_then(|chunk| chunk.get(local))
                .unwrap_or(VoxelValue::EMPTY);
            if before_value == target_value {
                continue;
            }

            changed_voxel_count = changed_voxel_count.saturating_add(1);
            if changed_voxel_count as usize > context.options.max_changed_voxels {
                partial = true;
                break;
            }

            let coord = before.grid().chunk_local_to_voxel(chunk_coord, local);
            touched_bounds = Some(expand_bounds(touched_bounds, coord));
            material_counts
                .entry(material_key(before_value))
                .and_modify(|counts| counts.0 = counts.0.saturating_add(1))
                .or_insert((1, 0));
            material_counts
                .entry(material_key(target_value))
                .and_modify(|counts| counts.1 = counts.1.saturating_add(1))
                .or_insert((0, 1));
        }
        if partial {
            break;
        }
    }

    let material_deltas = material_counts
        .into_iter()
        .filter_map(|(material, (before_count, target_count))| {
            if before_count == 0 && target_count == 0 {
                return None;
            }
            Some(VoxelEditHistoryMaterialDelta {
                material,
                before_count,
                target_count,
                delta: target_count as i64 - before_count as i64,
            })
        })
        .collect();
    let diagnostics = if partial {
        vec![VoxelEditHistoryDiffDiagnostic::DiffTruncated {
            limit: context.options.max_changed_voxels,
            observed: changed_voxel_count as usize,
        }]
    } else {
        Vec::new()
    };
    let sample_window_ref = if context.options.include_sample_window {
        touched_bounds.as_ref().map(|bounds| {
            format!(
                "asha://voxel-edit-history/diff-window?min={},{},{}&max={},{},{}&limit={}",
                bounds.min.x,
                bounds.min.y,
                bounds.min.z,
                bounds.max.x,
                bounds.max.y,
                bounds.max.z,
                context.options.max_changed_voxels
            )
        })
    } else {
        None
    };

    VoxelEditHistoryDiffSummary {
        before_hash: context.before_hash,
        current_hash: context.current_hash,
        target_hash: context.target_hash,
        projected_hash: None,
        partial,
        changed_voxel_count,
        touched_bounds,
        material_deltas,
        included_transaction_ids: context.included_transaction_ids,
        changed_transaction_count: context.changed_transaction_count,
        replayed_transaction_count: context.replayed_transaction_count,
        sample_window_ref,
        diagnostics,
    }
}

fn diff_chunk_coords(before: &VoxelWorld, target: &VoxelWorld) -> Vec<ChunkCoord> {
    let mut coords = BTreeSet::new();
    coords.extend(before.resident_chunks().map(|(coord, _)| coord));
    coords.extend(target.resident_chunks().map(|(coord, _)| coord));
    coords.into_iter().collect()
}

fn expand_bounds(
    bounds: Option<VoxelEditHistoryBounds>,
    coord: VoxelCoord,
) -> VoxelEditHistoryBounds {
    match bounds {
        Some(bounds) => VoxelEditHistoryBounds {
            min: VoxelCoord::new(
                bounds.min.x.min(coord.x),
                bounds.min.y.min(coord.y),
                bounds.min.z.min(coord.z),
            ),
            max: VoxelCoord::new(
                bounds.max.x.max(coord.x),
                bounds.max.y.max(coord.y),
                bounds.max.z.max(coord.z),
            ),
        },
        None => VoxelEditHistoryBounds {
            min: coord,
            max: coord,
        },
    }
}

fn material_key(value: VoxelValue) -> Option<u16> {
    value.material().map(|material| material.raw())
}

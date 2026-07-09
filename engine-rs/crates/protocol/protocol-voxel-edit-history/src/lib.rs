//! Protocol border for ASHA-native voxel edit history timelines.
//!
//! # Lane
//!
//! `contract-steward` - owns inert DTOs and stable vocabulary for durable voxel
//! edit history/cursor records plus future runtime revert, undo, redo, and
//! bounded-diff receipts.
//!
//! # Boundary posture
//!
//! These contracts describe accepted Rust-authority edit history and diagnostic
//! preview evidence. They do not own voxel mutation, replay, checkpointing,
//! persistence, Studio state, rendering, collision, or gameplay authority.

#![forbid(unsafe_code)]

use protocol_diagnostics::DiagnosticSeverity;
use serde::{Deserialize, Serialize};

/// Current supported ASHA voxel edit history schema.
pub const VOXEL_EDIT_HISTORY_SCHEMA_VERSION: u32 = 1;

/// Canonical media type for the JSON voxel edit history envelope.
pub const VOXEL_EDIT_HISTORY_MEDIA_TYPE: &str =
    "application/vnd.asha.voxel-edit-history+json;version=1";

/// Canonical filename extension for this JSON envelope.
pub const VOXEL_EDIT_HISTORY_EXTENSION: &str = "avhist.json";

/// Stable durable history entry kind vocabulary.
pub const VOXEL_EDIT_HISTORY_ENTRY_KINDS: &[&str] = &["accepted_transaction", "checkpoint"];

/// Stable cursor kind vocabulary.
pub const VOXEL_EDIT_HISTORY_CURSOR_KINDS: &[&str] = &["applied", "preview"];

/// Stable revert/undo/redo request modes.
pub const VOXEL_EDIT_HISTORY_REVERT_MODES: &[&str] =
    &["preview_revert", "apply_revert", "undo", "redo"];

/// Stable bounded-diff detail levels.
pub const VOXEL_EDIT_HISTORY_DIFF_LEVELS: &[&str] = &["summary", "bounded_samples", "partial"];

/// Stable classified validation/runtime diagnostic codes.
pub const VOXEL_EDIT_HISTORY_DIAGNOSTIC_CODES: &[&str] = &[
    "unsupported_schema_version",
    "unsupported_media_type",
    "invalid_history_id",
    "invalid_transaction_id",
    "invalid_cursor_id",
    "missing_parent_transaction",
    "stale_history_hash",
    "stale_cursor_hash",
    "base_voxel_hash_mismatch",
    "material_catalog_hash_mismatch",
    "checkpoint_hash_mismatch",
    "replay_hash_mismatch",
    "quota_exceeded",
    "target_not_replayable",
    "redo_tail_invalidated",
    "diff_truncated",
    "history_not_loaded",
    "edit_conflict",
];

/// Durable history entry kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoxelEditHistoryEntryKind {
    AcceptedTransaction,
    Checkpoint,
}

impl VoxelEditHistoryEntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            VoxelEditHistoryEntryKind::AcceptedTransaction => "accepted_transaction",
            VoxelEditHistoryEntryKind::Checkpoint => "checkpoint",
        }
    }
}

/// Cursor kind for applied history or non-durable preview evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoxelEditHistoryCursorKind {
    Applied,
    Preview,
}

impl VoxelEditHistoryCursorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            VoxelEditHistoryCursorKind::Applied => "applied",
            VoxelEditHistoryCursorKind::Preview => "preview",
        }
    }
}

/// Revert/undo/redo request mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoxelEditHistoryRevertMode {
    PreviewRevert,
    ApplyRevert,
    Undo,
    Redo,
}

impl VoxelEditHistoryRevertMode {
    pub fn as_str(self) -> &'static str {
        match self {
            VoxelEditHistoryRevertMode::PreviewRevert => "preview_revert",
            VoxelEditHistoryRevertMode::ApplyRevert => "apply_revert",
            VoxelEditHistoryRevertMode::Undo => "undo",
            VoxelEditHistoryRevertMode::Redo => "redo",
        }
    }
}

/// Bounded-diff detail level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoxelEditHistoryDiffLevel {
    Summary,
    BoundedSamples,
    Partial,
}

impl VoxelEditHistoryDiffLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            VoxelEditHistoryDiffLevel::Summary => "summary",
            VoxelEditHistoryDiffLevel::BoundedSamples => "bounded_samples",
            VoxelEditHistoryDiffLevel::Partial => "partial",
        }
    }
}

/// Classified voxel edit history diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoxelEditHistoryDiagnosticCode {
    UnsupportedSchemaVersion,
    UnsupportedMediaType,
    InvalidHistoryId,
    InvalidTransactionId,
    InvalidCursorId,
    MissingParentTransaction,
    StaleHistoryHash,
    StaleCursorHash,
    BaseVoxelHashMismatch,
    MaterialCatalogHashMismatch,
    CheckpointHashMismatch,
    ReplayHashMismatch,
    QuotaExceeded,
    TargetNotReplayable,
    RedoTailInvalidated,
    DiffTruncated,
    HistoryNotLoaded,
    EditConflict,
}

impl VoxelEditHistoryDiagnosticCode {
    pub fn as_str(self) -> &'static str {
        match self {
            VoxelEditHistoryDiagnosticCode::UnsupportedSchemaVersion => {
                "unsupported_schema_version"
            }
            VoxelEditHistoryDiagnosticCode::UnsupportedMediaType => "unsupported_media_type",
            VoxelEditHistoryDiagnosticCode::InvalidHistoryId => "invalid_history_id",
            VoxelEditHistoryDiagnosticCode::InvalidTransactionId => "invalid_transaction_id",
            VoxelEditHistoryDiagnosticCode::InvalidCursorId => "invalid_cursor_id",
            VoxelEditHistoryDiagnosticCode::MissingParentTransaction => {
                "missing_parent_transaction"
            }
            VoxelEditHistoryDiagnosticCode::StaleHistoryHash => "stale_history_hash",
            VoxelEditHistoryDiagnosticCode::StaleCursorHash => "stale_cursor_hash",
            VoxelEditHistoryDiagnosticCode::BaseVoxelHashMismatch => "base_voxel_hash_mismatch",
            VoxelEditHistoryDiagnosticCode::MaterialCatalogHashMismatch => {
                "material_catalog_hash_mismatch"
            }
            VoxelEditHistoryDiagnosticCode::CheckpointHashMismatch => "checkpoint_hash_mismatch",
            VoxelEditHistoryDiagnosticCode::ReplayHashMismatch => "replay_hash_mismatch",
            VoxelEditHistoryDiagnosticCode::QuotaExceeded => "quota_exceeded",
            VoxelEditHistoryDiagnosticCode::TargetNotReplayable => "target_not_replayable",
            VoxelEditHistoryDiagnosticCode::RedoTailInvalidated => "redo_tail_invalidated",
            VoxelEditHistoryDiagnosticCode::DiffTruncated => "diff_truncated",
            VoxelEditHistoryDiagnosticCode::HistoryNotLoaded => "history_not_loaded",
            VoxelEditHistoryDiagnosticCode::EditConflict => "edit_conflict",
        }
    }
}

/// Integer coordinate in stored voxel space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryCoord {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

/// Inclusive stored voxel-space bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryBounds {
    pub min: VoxelEditHistoryCoord,
    pub max: VoxelEditHistoryCoord,
}

/// Material count change in a bounded diff summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryMaterialDelta {
    pub material: u16,
    pub before_count: u64,
    pub after_count: u64,
    pub delta: i64,
}

/// Checkpoint reference that can accelerate replay without changing authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryCheckpointRef {
    pub checkpoint_id: String,
    pub cursor_id: String,
    pub transaction_id: Option<String>,
    pub voxel_state_hash: String,
    pub replay_hash: String,
    pub uri: Option<String>,
}

/// One classified validation/runtime diagnostic for voxel edit history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryDiagnostic {
    pub code: VoxelEditHistoryDiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub reference: String,
    pub message: String,
}

/// Bounded summary of the cells affected by a preview or applied revert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryDiffSummary {
    pub diff_level: VoxelEditHistoryDiffLevel,
    pub partial: bool,
    pub changed_voxel_count: u64,
    pub touched_bounds: Option<VoxelEditHistoryBounds>,
    pub material_deltas: Vec<VoxelEditHistoryMaterialDelta>,
    pub included_transaction_ids: Vec<String>,
    pub before_voxel_hash: String,
    pub current_voxel_hash: String,
    pub target_voxel_hash: String,
    pub projected_voxel_hash: Option<String>,
    pub sample_window_ref: Option<String>,
    pub diagnostics: Vec<VoxelEditHistoryDiagnostic>,
}

/// One accepted durable history transaction or checkpoint record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryEntry {
    pub transaction_id: String,
    pub parent_transaction_id: Option<String>,
    pub cursor_id: String,
    pub parent_cursor_id: Option<String>,
    pub entry_kind: VoxelEditHistoryEntryKind,
    pub operation_label: String,
    pub provenance: String,
    pub command_hash: String,
    pub receipt_hash: String,
    pub before_voxel_hash: String,
    pub after_voxel_hash: String,
    pub projected_voxel_hash: Option<String>,
    pub material_catalog_hash: String,
    pub command_count: u64,
    pub event_count: u64,
    pub touched_bounds: Option<VoxelEditHistoryBounds>,
    pub touched_voxel_count: u64,
    pub checkpoint: Option<VoxelEditHistoryCheckpointRef>,
    pub diff_summary: Option<VoxelEditHistoryDiffSummary>,
    pub diagnostics: Vec<VoxelEditHistoryDiagnostic>,
}

/// Cursor readout for the applied history head or a preview target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryCursor {
    pub cursor_id: String,
    pub cursor_kind: VoxelEditHistoryCursorKind,
    pub applied_transaction_id: Option<String>,
    pub parent_cursor_id: Option<String>,
    pub history_hash: String,
    pub voxel_state_hash: String,
    pub material_catalog_hash: String,
    pub undo_depth: u64,
    pub redo_depth: u64,
    pub entry_count: u64,
    pub checkpoint_count: u64,
}

/// Compact durable timeline readout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistorySummary {
    pub history_id: String,
    pub schema_version: u32,
    pub media_type: String,
    pub target_grid: u64,
    pub target_voxel_volume_asset_id: Option<String>,
    pub base_voxel_hash: String,
    pub material_catalog_hash: String,
    pub cursor: VoxelEditHistoryCursor,
    pub entries: Vec<VoxelEditHistoryEntry>,
    pub retained_redo_transaction_ids: Vec<String>,
    pub history_hash: String,
    pub diagnostics: Vec<VoxelEditHistoryDiagnostic>,
}

/// Request to read bounded durable history around a cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryReadRequest {
    pub history_id: String,
    pub cursor_id: Option<String>,
    pub max_entries: u64,
    pub include_redo_tail: bool,
    pub expected_history_hash: Option<String>,
}

/// Target cursor/transaction for revert-like operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryRevertTarget {
    pub transaction_id: Option<String>,
    pub cursor_id: Option<String>,
    pub cursor_index: Option<u64>,
}

/// Request to preview or apply a revert against the durable timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryRevertRequest {
    pub history_id: String,
    pub mode: VoxelEditHistoryRevertMode,
    pub target: VoxelEditHistoryRevertTarget,
    pub expected_history_hash: String,
    pub expected_cursor_hash: String,
    pub max_replay_steps: u64,
    pub max_diff_voxels: u64,
    pub include_sample_window: bool,
}

/// Request to undo one accepted transaction from the applied cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryUndoRequest {
    pub history_id: String,
    pub expected_history_hash: String,
    pub expected_cursor_hash: String,
    pub max_replay_steps: u64,
    pub max_diff_voxels: u64,
}

/// Request to redo one retained transaction from the redo tail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryRedoRequest {
    pub history_id: String,
    pub expected_history_hash: String,
    pub expected_cursor_hash: String,
    pub max_replay_steps: u64,
    pub max_diff_voxels: u64,
}

/// Non-durable preview/rejected diagnostic evidence from Rust authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryPreviewEvidence {
    pub request_mode: VoxelEditHistoryRevertMode,
    pub target: VoxelEditHistoryRevertTarget,
    pub projected_cursor: Option<VoxelEditHistoryCursor>,
    pub diff_summary: Option<VoxelEditHistoryDiffSummary>,
    pub replay_hash: Option<String>,
    pub diagnostics: Vec<VoxelEditHistoryDiagnostic>,
}

/// Receipt for a previewed, applied, or rejected revert-like operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryRevertReceipt {
    pub request: VoxelEditHistoryRevertRequest,
    pub applied: bool,
    pub preview: bool,
    pub history_id: String,
    pub cursor_before: VoxelEditHistoryCursor,
    pub cursor_after: Option<VoxelEditHistoryCursor>,
    pub durable_entry: Option<VoxelEditHistoryEntry>,
    pub preview_evidence: Option<VoxelEditHistoryPreviewEvidence>,
    pub diff_summary: Option<VoxelEditHistoryDiffSummary>,
    pub replay_hash: Option<String>,
    pub history_hash_before: String,
    pub history_hash_after: Option<String>,
    pub diagnostics: Vec<VoxelEditHistoryDiagnostic>,
}

/// Receipt for undoing one accepted transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryUndoReceipt {
    pub request: VoxelEditHistoryUndoRequest,
    pub receipt: VoxelEditHistoryRevertReceipt,
}

/// Receipt for redoing one retained transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct VoxelEditHistoryRedoReceipt {
    pub request: VoxelEditHistoryRedoRequest,
    pub receipt: VoxelEditHistoryRevertReceipt,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_unique(table: &[&str]) {
        assert!(!table.is_empty());
        let mut sorted = table.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), table.len(), "duplicate in {table:?}");
    }

    #[test]
    fn vocabulary_tables_are_nonempty_and_unique() {
        for table in [
            VOXEL_EDIT_HISTORY_ENTRY_KINDS,
            VOXEL_EDIT_HISTORY_CURSOR_KINDS,
            VOXEL_EDIT_HISTORY_REVERT_MODES,
            VOXEL_EDIT_HISTORY_DIFF_LEVELS,
            VOXEL_EDIT_HISTORY_DIAGNOSTIC_CODES,
        ] {
            assert_unique(table);
        }
    }

    #[test]
    fn enum_tables_match_public_strings() {
        assert_eq!(
            [
                VoxelEditHistoryEntryKind::AcceptedTransaction,
                VoxelEditHistoryEntryKind::Checkpoint,
            ]
            .iter()
            .map(|kind| kind.as_str())
            .collect::<Vec<_>>(),
            VOXEL_EDIT_HISTORY_ENTRY_KINDS
        );
        assert_eq!(
            [
                VoxelEditHistoryCursorKind::Applied,
                VoxelEditHistoryCursorKind::Preview,
            ]
            .iter()
            .map(|kind| kind.as_str())
            .collect::<Vec<_>>(),
            VOXEL_EDIT_HISTORY_CURSOR_KINDS
        );
        assert_eq!(
            [
                VoxelEditHistoryRevertMode::PreviewRevert,
                VoxelEditHistoryRevertMode::ApplyRevert,
                VoxelEditHistoryRevertMode::Undo,
                VoxelEditHistoryRevertMode::Redo,
            ]
            .iter()
            .map(|mode| mode.as_str())
            .collect::<Vec<_>>(),
            VOXEL_EDIT_HISTORY_REVERT_MODES
        );
        assert_eq!(
            [
                VoxelEditHistoryDiffLevel::Summary,
                VoxelEditHistoryDiffLevel::BoundedSamples,
                VoxelEditHistoryDiffLevel::Partial,
            ]
            .iter()
            .map(|level| level.as_str())
            .collect::<Vec<_>>(),
            VOXEL_EDIT_HISTORY_DIFF_LEVELS
        );
        assert_eq!(
            [
                VoxelEditHistoryDiagnosticCode::UnsupportedSchemaVersion,
                VoxelEditHistoryDiagnosticCode::UnsupportedMediaType,
                VoxelEditHistoryDiagnosticCode::InvalidHistoryId,
                VoxelEditHistoryDiagnosticCode::InvalidTransactionId,
                VoxelEditHistoryDiagnosticCode::InvalidCursorId,
                VoxelEditHistoryDiagnosticCode::MissingParentTransaction,
                VoxelEditHistoryDiagnosticCode::StaleHistoryHash,
                VoxelEditHistoryDiagnosticCode::StaleCursorHash,
                VoxelEditHistoryDiagnosticCode::BaseVoxelHashMismatch,
                VoxelEditHistoryDiagnosticCode::MaterialCatalogHashMismatch,
                VoxelEditHistoryDiagnosticCode::CheckpointHashMismatch,
                VoxelEditHistoryDiagnosticCode::ReplayHashMismatch,
                VoxelEditHistoryDiagnosticCode::QuotaExceeded,
                VoxelEditHistoryDiagnosticCode::TargetNotReplayable,
                VoxelEditHistoryDiagnosticCode::RedoTailInvalidated,
                VoxelEditHistoryDiagnosticCode::DiffTruncated,
                VoxelEditHistoryDiagnosticCode::HistoryNotLoaded,
                VoxelEditHistoryDiagnosticCode::EditConflict,
            ]
            .iter()
            .map(|code| code.as_str())
            .collect::<Vec<_>>(),
            VOXEL_EDIT_HISTORY_DIAGNOSTIC_CODES
        );
    }
}

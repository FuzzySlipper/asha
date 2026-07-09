use super::*;

/// Focused behavior test for the `voxelEditHistory` family: schema/media
/// constants, stable vocabularies, accepted durable entries, preview evidence,
/// and bounded diff/readback DTOs are sourced from
/// `protocol-voxel-edit-history` and exposed through generated contracts.
/// Guard for #5280.
#[test]
fn voxel_edit_history_family_emits_history_and_revert_contracts() {
    let history = file("voxelEditHistory.ts");
    assert!(history.contains("import type { DiagnosticSeverity } from './diagnostics.js';"));
    assert!(history.contains("export const VOXEL_EDIT_HISTORY_SCHEMA_VERSION = 1;"));
    assert!(history.contains(
        "export const VOXEL_EDIT_HISTORY_MEDIA_TYPE = \"application/vnd.asha.voxel-edit-history+json;version=1\";"
    ));
    assert!(history.contains("export const VOXEL_EDIT_HISTORY_EXTENSION = \"avhist.json\";"));
    for kind in protocol_voxel_edit_history::VOXEL_EDIT_HISTORY_ENTRY_KINDS {
        assert!(
            history.contains(&format!("'{kind}'")),
            "missing entry kind {kind}"
        );
    }
    for kind in protocol_voxel_edit_history::VOXEL_EDIT_HISTORY_CURSOR_KINDS {
        assert!(
            history.contains(&format!("'{kind}'")),
            "missing cursor kind {kind}"
        );
    }
    for mode in protocol_voxel_edit_history::VOXEL_EDIT_HISTORY_REVERT_MODES {
        assert!(
            history.contains(&format!("'{mode}'")),
            "missing revert mode {mode}"
        );
    }
    for level in protocol_voxel_edit_history::VOXEL_EDIT_HISTORY_DIFF_LEVELS {
        assert!(
            history.contains(&format!("'{level}'")),
            "missing diff level {level}"
        );
    }
    for code in protocol_voxel_edit_history::VOXEL_EDIT_HISTORY_DIAGNOSTIC_CODES {
        assert!(
            history.contains(&format!("'{code}'")),
            "missing code {code}"
        );
    }

    assert!(history.contains("export interface VoxelEditHistoryEntry {"));
    assert!(history.contains("readonly transactionId: string;"));
    assert!(history.contains("readonly parentTransactionId: string | null;"));
    assert!(history.contains("readonly beforeVoxelHash: string;"));
    assert!(history.contains("readonly afterVoxelHash: string;"));
    assert!(history.contains("readonly projectedVoxelHash: string | null;"));
    assert!(history.contains("readonly materialCatalogHash: string;"));
    assert!(history.contains("readonly commandCount: number;"));
    assert!(history.contains("readonly eventCount: number;"));
    assert!(history.contains("readonly checkpoint: VoxelEditHistoryCheckpointRef | null;"));
    assert!(history.contains("export interface VoxelEditHistoryCursor {"));
    assert!(history.contains("readonly cursorKind: VoxelEditHistoryCursorKind;"));
    assert!(history.contains("export interface VoxelEditHistoryDiffSummary {"));
    assert!(history.contains("readonly changedVoxelCount: number;"));
    assert!(history.contains("readonly includedTransactionIds: readonly string[];"));
    assert!(history.contains("export interface VoxelEditHistoryRevertRequest {"));
    assert!(history.contains("export interface VoxelEditHistoryUndoRequest {"));
    assert!(history.contains("export interface VoxelEditHistoryRedoRequest {"));
    assert!(history.contains("export interface VoxelEditHistoryPreviewEvidence {"));
    assert!(history.contains("readonly projectedCursor: VoxelEditHistoryCursor | null;"));
    assert!(history.contains("export interface VoxelEditHistoryRevertReceipt {"));
    assert!(history.contains("readonly durableEntry: VoxelEditHistoryEntry | null;"));
    assert!(history.contains("readonly previewEvidence: VoxelEditHistoryPreviewEvidence | null;"));
}

import type { DiagnosticSeverity } from './diagnostics.js';
export declare const VOXEL_EDIT_HISTORY_SCHEMA_VERSION = 1;
export declare const VOXEL_EDIT_HISTORY_MEDIA_TYPE = "application/vnd.asha.voxel-edit-history+json;version=1";
export declare const VOXEL_EDIT_HISTORY_EXTENSION = "avhist.json";
export type VoxelEditHistoryEntryKind = 'accepted_transaction' | 'checkpoint';
export type VoxelEditHistoryCursorKind = 'applied' | 'preview';
export type VoxelEditHistoryRevertMode = 'preview_revert' | 'apply_revert' | 'undo' | 'redo';
export type VoxelEditHistoryDiffLevel = 'summary' | 'bounded_samples' | 'partial';
export type VoxelEditHistoryDiagnosticCode = 'unsupported_schema_version' | 'unsupported_media_type' | 'invalid_history_id' | 'invalid_transaction_id' | 'invalid_cursor_id' | 'missing_parent_transaction' | 'stale_history_hash' | 'stale_cursor_hash' | 'base_voxel_hash_mismatch' | 'material_catalog_hash_mismatch' | 'checkpoint_hash_mismatch' | 'replay_hash_mismatch' | 'quota_exceeded' | 'target_not_replayable' | 'redo_tail_invalidated' | 'diff_truncated' | 'history_not_loaded' | 'edit_conflict';
export interface VoxelEditHistoryCoord {
    readonly x: number;
    readonly y: number;
    readonly z: number;
}
export interface VoxelEditHistoryBounds {
    readonly min: VoxelEditHistoryCoord;
    readonly max: VoxelEditHistoryCoord;
}
export interface VoxelEditHistoryMaterialDelta {
    readonly material: number;
    readonly beforeCount: number;
    readonly afterCount: number;
    readonly delta: number;
}
export interface VoxelEditHistoryCheckpointRef {
    readonly checkpointId: string;
    readonly cursorId: string;
    readonly transactionId: string | null;
    readonly voxelStateHash: string;
    readonly replayHash: string;
    readonly uri: string | null;
}
export interface VoxelEditHistoryDiagnostic {
    readonly code: VoxelEditHistoryDiagnosticCode;
    readonly severity: DiagnosticSeverity;
    readonly reference: string;
    readonly message: string;
}
export interface VoxelEditHistoryDiffSummary {
    readonly diffLevel: VoxelEditHistoryDiffLevel;
    readonly partial: boolean;
    readonly changedVoxelCount: number;
    readonly touchedBounds: VoxelEditHistoryBounds | null;
    readonly materialDeltas: readonly VoxelEditHistoryMaterialDelta[];
    readonly includedTransactionIds: readonly string[];
    readonly beforeVoxelHash: string;
    readonly currentVoxelHash: string;
    readonly targetVoxelHash: string;
    readonly projectedVoxelHash: string | null;
    readonly sampleWindowRef: string | null;
    readonly diagnostics: readonly VoxelEditHistoryDiagnostic[];
}
export interface VoxelEditHistoryEntry {
    readonly transactionId: string;
    readonly parentTransactionId: string | null;
    readonly cursorId: string;
    readonly parentCursorId: string | null;
    readonly entryKind: VoxelEditHistoryEntryKind;
    readonly operationLabel: string;
    readonly provenance: string;
    readonly commandHash: string;
    readonly receiptHash: string;
    readonly beforeVoxelHash: string;
    readonly afterVoxelHash: string;
    readonly projectedVoxelHash: string | null;
    readonly materialCatalogHash: string;
    readonly commandCount: number;
    readonly eventCount: number;
    readonly touchedBounds: VoxelEditHistoryBounds | null;
    readonly touchedVoxelCount: number;
    readonly checkpoint: VoxelEditHistoryCheckpointRef | null;
    readonly diffSummary: VoxelEditHistoryDiffSummary | null;
    readonly diagnostics: readonly VoxelEditHistoryDiagnostic[];
}
export interface VoxelEditHistoryCursor {
    readonly cursorId: string;
    readonly cursorKind: VoxelEditHistoryCursorKind;
    readonly appliedTransactionId: string | null;
    readonly parentCursorId: string | null;
    readonly historyHash: string;
    readonly voxelStateHash: string;
    readonly materialCatalogHash: string;
    readonly undoDepth: number;
    readonly redoDepth: number;
    readonly entryCount: number;
    readonly checkpointCount: number;
}
export interface VoxelEditHistorySummary {
    readonly historyId: string;
    readonly schemaVersion: number;
    readonly mediaType: string;
    readonly targetGrid: number;
    readonly targetVoxelVolumeAssetId: string | null;
    readonly baseVoxelHash: string;
    readonly materialCatalogHash: string;
    readonly cursor: VoxelEditHistoryCursor;
    readonly entries: readonly VoxelEditHistoryEntry[];
    readonly retainedRedoTransactionIds: readonly string[];
    readonly historyHash: string;
    readonly diagnostics: readonly VoxelEditHistoryDiagnostic[];
}
export interface VoxelEditHistoryReadRequest {
    readonly historyId: string;
    readonly cursorId: string | null;
    readonly maxEntries: number;
    readonly includeRedoTail: boolean;
    readonly expectedHistoryHash: string | null;
}
export interface VoxelEditHistoryRevertTarget {
    readonly transactionId: string | null;
    readonly cursorId: string | null;
    readonly cursorIndex: number | null;
}
export interface VoxelEditHistoryRevertRequest {
    readonly historyId: string;
    readonly mode: VoxelEditHistoryRevertMode;
    readonly target: VoxelEditHistoryRevertTarget;
    readonly expectedHistoryHash: string;
    readonly expectedCursorHash: string;
    readonly maxReplaySteps: number;
    readonly maxDiffVoxels: number;
    readonly includeSampleWindow: boolean;
}
export interface VoxelEditHistoryUndoRequest {
    readonly historyId: string;
    readonly expectedHistoryHash: string;
    readonly expectedCursorHash: string;
    readonly maxReplaySteps: number;
    readonly maxDiffVoxels: number;
}
export interface VoxelEditHistoryRedoRequest {
    readonly historyId: string;
    readonly expectedHistoryHash: string;
    readonly expectedCursorHash: string;
    readonly maxReplaySteps: number;
    readonly maxDiffVoxels: number;
}
export interface VoxelEditHistoryPreviewEvidence {
    readonly requestMode: VoxelEditHistoryRevertMode;
    readonly target: VoxelEditHistoryRevertTarget;
    readonly projectedCursor: VoxelEditHistoryCursor | null;
    readonly diffSummary: VoxelEditHistoryDiffSummary | null;
    readonly replayHash: string | null;
    readonly diagnostics: readonly VoxelEditHistoryDiagnostic[];
}
export interface VoxelEditHistoryRevertReceipt {
    readonly request: VoxelEditHistoryRevertRequest;
    readonly applied: boolean;
    readonly preview: boolean;
    readonly historyId: string;
    readonly cursorBefore: VoxelEditHistoryCursor;
    readonly cursorAfter: VoxelEditHistoryCursor | null;
    readonly durableEntry: VoxelEditHistoryEntry | null;
    readonly previewEvidence: VoxelEditHistoryPreviewEvidence | null;
    readonly diffSummary: VoxelEditHistoryDiffSummary | null;
    readonly replayHash: string | null;
    readonly historyHashBefore: string;
    readonly historyHashAfter: string | null;
    readonly diagnostics: readonly VoxelEditHistoryDiagnostic[];
}
export interface VoxelEditHistoryUndoReceipt {
    readonly request: VoxelEditHistoryUndoRequest;
    readonly receipt: VoxelEditHistoryRevertReceipt;
}
export interface VoxelEditHistoryRedoReceipt {
    readonly request: VoxelEditHistoryRedoRequest;
    readonly receipt: VoxelEditHistoryRevertReceipt;
}
//# sourceMappingURL=voxelEditHistory.d.ts.map
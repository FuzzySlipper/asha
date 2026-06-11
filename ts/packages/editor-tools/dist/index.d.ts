import type { VoxelCommand, VoxelCoord, Face } from '@asha/contracts';
export type ToolMode = 'place' | 'remove' | 'select' | 'inspect';
export type SelectionMode = 'voxel' | 'face';
/** A picked anchor (from collision/picking) — a coord + the struck face. */
export interface VoxelSelection {
    readonly voxel: VoxelCoord;
    readonly face: Face;
}
/** The full persistent editor context. Immutable snapshot; updated via actions. */
export interface EditorContext {
    /** Which voxel grid edits target (GridId raw). */
    readonly grid: number;
    readonly tool: ToolMode;
    /** Brush side length in voxels (>= 1). */
    readonly brushSize: number;
    /** Material id used by the `place` tool (VoxelMaterialId raw). */
    readonly material: number;
    readonly snapping: boolean;
    readonly selectionMode: SelectionMode;
    readonly preview: {
        readonly enabled: boolean;
    };
    /** Current picked anchor, or null when nothing is selected. */
    readonly selection: VoxelSelection | null;
}
/** The initial editor context. */
export declare function initialEditorContext(grid?: number): EditorContext;
export type EditorAction = {
    readonly type: 'setTool';
    readonly tool: ToolMode;
} | {
    readonly type: 'setBrushSize';
    readonly size: number;
} | {
    readonly type: 'setMaterial';
    readonly material: number;
} | {
    readonly type: 'setSnapping';
    readonly snapping: boolean;
} | {
    readonly type: 'setSelectionMode';
    readonly mode: SelectionMode;
} | {
    readonly type: 'setPreviewEnabled';
    readonly enabled: boolean;
} | {
    readonly type: 'setSelection';
    readonly selection: VoxelSelection;
} | {
    readonly type: 'clearSelection';
};
/** Pure reducer. Validates/normalises (e.g. brush size clamped to `>= 1` integer). */
export declare function reduce(state: EditorContext, action: EditorAction): EditorContext;
/** A change listener. */
export type EditorListener = (state: EditorContext) => void;
/**
 * The persistent editor-context store: one instance lives in `app` for the whole
 * session, so context survives camera movement and panel remounts. Devtools can
 * `subscribe` for visibility. Holds no authoritative voxel data.
 */
export declare class EditorStore {
    #private;
    constructor(initial?: EditorContext);
    getState(): EditorContext;
    /** Apply an action; notifies listeners only when the state actually changes. */
    dispatch(action: EditorAction): EditorContext;
    subscribe(listener: EditorListener): () => void;
}
/** The voxel across `face` from `voxel` — the anchor a *place* edit builds on. */
export declare function faceNeighbor(voxel: VoxelCoord, face: Face): VoxelCoord;
/** Half-open `[min, max)` box of side `size` (>= 1) centred on `center`. */
export declare function brushBox(center: VoxelCoord, size: number): {
    min: VoxelCoord;
    max: VoxelCoord;
};
/**
 * The voxel coordinates a brush action would affect — for the non-authoritative
 * preview overlay. `select`/`inspect`, or no selection, affect nothing.
 */
export declare function previewTargets(ctx: EditorContext): VoxelCoord[];
/**
 * Turn the editor context + selection into a generated `VoxelCommand` proposal, or
 * `null` when there is nothing to commit (no selection, or a non-editing tool).
 * Pure — it does not submit; the `app` command path does that on commit.
 */
export declare function proposeCommand(ctx: EditorContext): VoxelCommand | null;
//# sourceMappingURL=index.d.ts.map
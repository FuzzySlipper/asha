export interface EditorCameraSelectionContext {
    readonly selection: {
        readonly voxel: {
            readonly x: number;
            readonly y: number;
            readonly z: number;
        };
    } | null;
}
/**
 * Projects the current editor selection into a world-space camera pivot. The
 * editor owns selection intent only; RuntimeSession still validates and owns
 * the resulting camera mode transaction.
 */
export declare function editorCameraPivot(context: EditorCameraSelectionContext): readonly [number, number, number] | null;
//# sourceMappingURL=camera-navigation.d.ts.map
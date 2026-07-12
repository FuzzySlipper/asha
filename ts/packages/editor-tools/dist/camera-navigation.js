/**
 * Projects the current editor selection into a world-space camera pivot. The
 * editor owns selection intent only; RuntimeSession still validates and owns
 * the resulting camera mode transaction.
 */
export function editorCameraPivot(context) {
    const selection = context.selection;
    if (selection === null)
        return null;
    return [
        selection.voxel.x + 0.5,
        selection.voxel.y + 0.5,
        selection.voxel.z + 0.5,
    ];
}
//# sourceMappingURL=camera-navigation.js.map
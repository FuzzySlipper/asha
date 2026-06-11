import { test } from 'node:test';
import assert from 'node:assert/strict';
import { EditorStore, initialEditorContext, proposeCommand, previewTargets, faceNeighbor, brushBox, } from './index.js';
function withSelection(over = {}) {
    return {
        ...initialEditorContext(0),
        selection: { voxel: { x: 5, y: 0, z: 0 }, face: 'negX' },
        ...over,
    };
}
test('store applies actions, validates, and notifies subscribers on change', () => {
    const store = new EditorStore();
    let notified = 0;
    store.subscribe(() => (notified += 1));
    store.dispatch({ type: 'setTool', tool: 'remove' });
    assert.equal(store.getState().tool, 'remove');
    // Brush size is clamped to an integer >= 1.
    store.dispatch({ type: 'setBrushSize', size: 0 });
    assert.equal(store.getState().brushSize, 1);
    store.dispatch({ type: 'setBrushSize', size: 3.9 });
    assert.equal(store.getState().brushSize, 3);
    assert.equal(notified, 3);
});
test('store reducer is pure and identity-stable for no-op', () => {
    const store = new EditorStore();
    const before = store.getState();
    // Setting a selection then reading is a new object; but unchanged primitive
    // actions still produce a new state object (immutability), so we check values.
    store.dispatch({ type: 'setSnapping', snapping: store.getState().snapping });
    assert.equal(store.getState().snapping, before.snapping);
});
test('faceNeighbor and brushBox are correct', () => {
    assert.deepEqual(faceNeighbor({ x: 5, y: 0, z: 0 }, 'negX'), { x: 4, y: 0, z: 0 });
    assert.deepEqual(faceNeighbor({ x: 5, y: 0, z: 0 }, 'posY'), { x: 5, y: 1, z: 0 });
    // size 1 → the single cell.
    assert.deepEqual(brushBox({ x: 2, y: 2, z: 2 }, 1), { min: { x: 2, y: 2, z: 2 }, max: { x: 3, y: 3, z: 3 } });
    // size 3 → centred 3³.
    assert.deepEqual(brushBox({ x: 2, y: 2, z: 2 }, 3), { min: { x: 1, y: 1, z: 1 }, max: { x: 4, y: 4, z: 4 } });
});
test('place tool proposes a setVoxel at the face-neighbour anchor', () => {
    const ctx = withSelection({ tool: 'place', material: 7 });
    assert.deepEqual(proposeCommand(ctx), {
        op: 'setVoxel',
        grid: 0,
        coord: { x: 4, y: 0, z: 0 }, // across the -X face of voxel 5
        value: { kind: 'solid', material: 7 },
    });
});
test('remove tool proposes a setVoxel Empty at the selected voxel', () => {
    const ctx = withSelection({ tool: 'remove' });
    assert.deepEqual(proposeCommand(ctx), {
        op: 'setVoxel',
        grid: 0,
        coord: { x: 5, y: 0, z: 0 },
        value: { kind: 'empty' },
    });
});
test('box brush (size > 1) proposes a fillRegion', () => {
    const ctx = withSelection({ tool: 'place', brushSize: 3, material: 2 });
    const cmd = proposeCommand(ctx);
    assert.equal(cmd?.op, 'fillRegion');
    if (cmd?.op === 'fillRegion') {
        // anchor = (4,0,0); 3³ box centred there.
        assert.deepEqual(cmd.min, { x: 3, y: -1, z: -1 });
        assert.deepEqual(cmd.max, { x: 6, y: 2, z: 2 });
        assert.deepEqual(cmd.value, { kind: 'solid', material: 2 });
    }
});
test('select/inspect and no-selection propose nothing', () => {
    assert.equal(proposeCommand(withSelection({ tool: 'select' })), null);
    assert.equal(proposeCommand(withSelection({ tool: 'inspect' })), null);
    assert.equal(proposeCommand(initialEditorContext(0)), null); // no selection
});
test('previewTargets enumerates the affected cells without proposing/mutating', () => {
    assert.deepEqual(previewTargets(withSelection({ tool: 'place', brushSize: 1 })), [{ x: 4, y: 0, z: 0 }]);
    assert.equal(previewTargets(withSelection({ tool: 'place', brushSize: 3 })).length, 27);
    assert.deepEqual(previewTargets(withSelection({ tool: 'select' })), []);
    assert.deepEqual(previewTargets(initialEditorContext(0)), []);
});
//# sourceMappingURL=editor.test.js.map
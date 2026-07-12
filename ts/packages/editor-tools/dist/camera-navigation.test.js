import { test } from 'node:test';
import assert from 'node:assert/strict';
import { editorCameraPivot } from './camera-navigation.js';
import { initialEditorContext, reduce } from './index.js';
void test('editor selection projects a pivot without creating camera authority', () => {
    const empty = initialEditorContext();
    assert.equal(editorCameraPivot(empty), null);
    const selected = reduce(empty, {
        type: 'setSelection',
        selection: { voxel: { x: 3, y: 1, z: -5 }, face: 'posY' },
    });
    assert.deepEqual(editorCameraPivot(selected), [3.5, 1.5, -4.5]);
    assert.deepEqual(empty.selection, null);
});
//# sourceMappingURL=camera-navigation.test.js.map
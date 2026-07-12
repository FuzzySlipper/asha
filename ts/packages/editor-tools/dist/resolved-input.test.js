import assert from 'node:assert/strict';
import test from 'node:test';
import { EditorResolvedInputConsumer } from './resolved-input.js';
function action(actionId, phase, pressed) {
    return {
        sequence: 1,
        actionId,
        contextId: 'editor',
        bindingId: `binding-${actionId}`,
        phase,
        value: { kind: 'button', pressed },
    };
}
void test('editor camera and tool code consumes named actions without DOM bindings', () => {
    const consumer = new EditorResolvedInputConsumer();
    assert.equal(consumer.accept(action('editor.camera.forward', 'pressed', true)), true);
    assert.equal(consumer.accept(action('editor.tool.primary', 'pressed', true)), true);
    assert.equal(consumer.accept({
        sequence: 2,
        actionId: 'editor.camera.look',
        contextId: 'editor',
        bindingId: 'editor-look',
        phase: 'changed',
        value: { kind: 'axis2d', x: 4, y: -2 },
    }), true);
    assert.deepEqual(consumer.drain(), {
        cameraForward: 1,
        cameraRight: 0,
        lookDelta: [4, -2],
        primaryToolPressed: true,
        cancelPressed: false,
    });
    assert.equal(consumer.accept(action('gameplay.move.forward', 'pressed', true)), false);
});
//# sourceMappingURL=resolved-input.test.js.map
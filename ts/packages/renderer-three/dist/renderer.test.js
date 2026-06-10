// Runtime tests for the Three.js renderer shell, run with `node --test`.
// The scene graph is built without a GL context (no rendering), so these assert
// registry/scene-graph state directly.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { renderHandle } from '@asha/contracts';
import { ThreeRenderer, RenderApplyError } from './index.js';
function cubeNode(label = 'cube') {
    return {
        geometry: { shape: 'cube' },
        material: { color: [1, 1, 1, 1], wireframe: false },
        transform: { translation: [2, 0, 0], rotation: [0, 0, 0, 1], scale: [1, 1, 1] },
        visible: true,
        layer: 'scene',
        metadata: { source: null, tags: [], label },
    };
}
function createDiff(handle, node) {
    return { op: 'create', handle: renderHandle(handle), parent: null, node };
}
test('create places a node in the scene layer with its transform', () => {
    const r = new ThreeRenderer();
    r.applyDiff(createDiff(1, cubeNode()));
    assert.equal(r.handleCount, 1);
    assert.ok(r.has(renderHandle(1)));
    const obj = r.objectFor(renderHandle(1));
    assert.equal(obj.position.x, 2);
    assert.equal(obj.parent?.name, 'scene');
    assert.equal(obj.name, 'cube');
});
test('update mutates transform and visibility', () => {
    const r = new ThreeRenderer();
    r.applyDiff(createDiff(1, cubeNode()));
    r.applyDiff({
        op: 'update',
        handle: renderHandle(1),
        transform: { translation: [5, 1, 0], rotation: [0, 0, 0, 1], scale: [2, 2, 2] },
        material: null,
        visible: false,
        metadata: null,
    });
    const obj = r.objectFor(renderHandle(1));
    assert.equal(obj.position.x, 5);
    assert.equal(obj.scale.x, 2);
    assert.equal(obj.visible, false);
});
test('destroy removes the node and frees the handle', () => {
    const r = new ThreeRenderer();
    r.applyDiff(createDiff(1, cubeNode()));
    r.applyDiff({ op: 'destroy', handle: renderHandle(1) });
    assert.equal(r.handleCount, 0);
    assert.ok(!r.has(renderHandle(1)));
});
test('duplicate create and stale/unknown handles throw', () => {
    const r = new ThreeRenderer();
    r.applyDiff(createDiff(1, cubeNode()));
    assert.throws(() => r.applyDiff(createDiff(1, cubeNode())), RenderApplyError);
    assert.throws(() => r.applyDiff({
        op: 'update',
        handle: renderHandle(99),
        transform: null,
        material: null,
        visible: null,
        metadata: null,
    }), RenderApplyError);
    assert.throws(() => r.applyDiff({ op: 'destroy', handle: renderHandle(42) }), RenderApplyError);
});
test('debug-layer nodes land in the debug group', () => {
    const r = new ThreeRenderer();
    const node = {
        ...cubeNode('#1'),
        geometry: { shape: 'point' },
        layer: 'debug',
    };
    r.applyDiff(createDiff(1, node));
    assert.equal(r.objectFor(renderHandle(1))?.parent?.name, 'debug');
});
test('applyEncodedFrame decodes through wasm-bridge and sequences create→update→destroy', () => {
    const fixture = JSON.parse(readFileSync(resolve(import.meta.dirname, '../../../../harness/fixtures/render-diffs/sample-frame.json'), 'utf8'));
    const r = new ThreeRenderer();
    r.applyEncodedFrame(fixture);
    // The fixture creates handle 1, updates it, then destroys it.
    assert.equal(r.handleCount, 0);
});
test('applies the Rust render-bridge fixture sequence end-to-end', () => {
    // Rust render bridge → fixture → wasm-bridge decode → renderer apply.
    // Frame 1 creates handles 1 & 2; frame 2 creates 3, updates 1, destroys 2.
    const frames = JSON.parse(readFileSync(resolve(import.meta.dirname, '../../../../harness/fixtures/render-diffs/bridge-sequence.json'), 'utf8'));
    const r = new ThreeRenderer();
    for (const frame of frames) {
        r.applyEncodedFrame(frame);
    }
    assert.equal(r.handleCount, 2);
    assert.ok(r.has(renderHandle(1)));
    assert.ok(r.has(renderHandle(3)));
    assert.ok(!r.has(renderHandle(2)));
    // The update carried the new tag onto handle 1's scene object metadata.
    assert.deepEqual(r.objectFor(renderHandle(1))?.userData.tags, [5]);
});
//# sourceMappingURL=renderer.test.js.map
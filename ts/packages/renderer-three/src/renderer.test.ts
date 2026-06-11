// Runtime tests for the Three.js renderer shell, run with `node --test`.
// The scene graph is built without a GL context (no rendering), so these assert
// registry/scene-graph state directly.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { renderHandle, type RenderDiff, type RenderNode } from '@asha/contracts';
import { ThreeRenderer, RenderApplyError } from './index.js';

function cubeNode(label = 'cube'): RenderNode {
  return {
    geometry: { shape: 'cube' },
    material: { color: [1, 1, 1, 1], wireframe: false },
    transform: { translation: [2, 0, 0], rotation: [0, 0, 0, 1], scale: [1, 1, 1] },
    visible: true,
    layer: 'scene',
    metadata: { source: null, tags: [], label },
  };
}

function createDiff(handle: number, node: RenderNode): RenderDiff {
  return { op: 'create', handle: renderHandle(handle), parent: null, node };
}

test('create places a node in the scene layer with its transform', () => {
  const r = new ThreeRenderer();
  r.applyDiff(createDiff(1, cubeNode()));

  assert.equal(r.handleCount, 1);
  assert.ok(r.has(renderHandle(1)));
  const obj = r.objectFor(renderHandle(1))!;
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

  const obj = r.objectFor(renderHandle(1))!;
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
  assert.throws(
    () =>
      r.applyDiff({
        op: 'update',
        handle: renderHandle(99),
        transform: null,
        material: null,
        visible: null,
        metadata: null,
      }),
    RenderApplyError,
  );
  assert.throws(
    () => r.applyDiff({ op: 'destroy', handle: renderHandle(42) }),
    RenderApplyError,
  );
});

test('debug-layer nodes land in the debug group', () => {
  const r = new ThreeRenderer();
  const node: RenderNode = {
    ...cubeNode('#1'),
    geometry: { shape: 'point' },
    layer: 'debug',
  };
  r.applyDiff(createDiff(1, node));
  assert.equal(r.objectFor(renderHandle(1))?.parent?.name, 'debug');
});

test('applyEncodedFrame decodes through wasm-bridge and sequences create→update→destroy', () => {
  const fixture = JSON.parse(
    readFileSync(
      resolve(import.meta.dirname, '../../../../harness/fixtures/render-diffs/sample-frame.json'),
      'utf8',
    ),
  );
  const r = new ThreeRenderer();
  r.applyEncodedFrame(fixture);
  // The fixture creates handle 1, updates it, then destroys it.
  assert.equal(r.handleCount, 0);
});

test('applies the Rust render-bridge fixture sequence end-to-end', () => {
  // Rust render bridge → fixture → wasm-bridge decode → renderer apply.
  // Frame 1 creates handles 1 & 2; frame 2 creates 3, updates 1, destroys 2.
  const frames = JSON.parse(
    readFileSync(
      resolve(import.meta.dirname, '../../../../harness/fixtures/render-diffs/bridge-sequence.json'),
      'utf8',
    ),
  ) as unknown[];

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

// ── mesh payload upload (ADR 0007 / #2263) ────────────────────────────────────

import * as THREE from 'three';
import type { MeshPayloadDescriptor } from '@asha/contracts';

function meshNode(): RenderNode {
  return {
    geometry: { shape: 'cube' },
    material: { color: [1, 1, 1, 1], wireframe: false },
    transform: { translation: [0, 0, 0], rotation: [0, 0, 0, 1], scale: [1, 1, 1] },
    visible: true,
    layer: 'scene',
    metadata: { source: null, tags: [], label: 'chunk' },
  };
}

// A quad (4 verts, 6 indices) split into two material-slot groups.
function quadPayload(): MeshPayloadDescriptor {
  return {
    layout: {
      vertexCount: 4,
      indexCount: 6,
      indexWidth: 'u32',
      attributes: [
        { name: 'position', components: 3, kind: 'f32' },
        { name: 'normal', components: 3, kind: 'f32' },
      ],
    },
    groups: [
      { materialSlot: 1, start: 0, count: 3 },
      { materialSlot: 2, start: 3, count: 3 },
    ],
    bounds: { min: [0, 0, 0], max: [1, 1, 0] },
    source: {
      kind: 'inline',
      positions: [0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0],
      normals: [0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
      indices: [0, 1, 2, 0, 2, 3],
    },
  };
}

test('replaceMeshPayload uploads a BufferGeometry with groups and material slots', () => {
  const r = new ThreeRenderer();
  const h = renderHandle(1);
  r.applyDiff({ op: 'create', handle: h, parent: null, node: meshNode() });
  r.applyDiff({ op: 'replaceMeshPayload', handle: h, payload: quadPayload() });

  const mesh = r.objectFor(h) as THREE.Mesh;
  const geo = mesh.geometry;
  assert.equal(geo.getAttribute('position').count, 4);
  assert.equal(geo.getAttribute('normal').count, 4);
  assert.equal(geo.getIndex()!.count, 6);
  assert.equal(geo.groups.length, 2);
  assert.deepEqual(
    geo.groups.map((g) => [g.start, g.count, g.materialIndex]),
    [[0, 3, 0], [3, 3, 1]],
  );
  // Two materials, one per group.
  assert.ok(Array.isArray(mesh.material));
  assert.equal((mesh.material as THREE.Material[]).length, 2);
});

test('registered slot colour maps to the group material; unregistered uses a fallback', () => {
  const r = new ThreeRenderer();
  r.registerSlotColor(1, 1, 0, 0); // slot 1 → red
  const h = renderHandle(1);
  r.applyDiff({ op: 'create', handle: h, parent: null, node: meshNode() });
  r.applyDiff({ op: 'replaceMeshPayload', handle: h, payload: quadPayload() });

  const mats = (r.objectFor(h) as THREE.Mesh).material as THREE.MeshBasicMaterial[];
  assert.deepEqual([mats[0]!.color.r, mats[0]!.color.g, mats[0]!.color.b], [1, 0, 0]);
  // Slot 2 was never registered → a deterministic non-red fallback colour.
  assert.notDeepEqual([mats[1]!.color.r, mats[1]!.color.g, mats[1]!.color.b], [1, 0, 0]);
});

test('replaceMeshPayload disposes the previous geometry and material', () => {
  const r = new ThreeRenderer();
  const h = renderHandle(1);
  r.applyDiff({ op: 'create', handle: h, parent: null, node: meshNode() });
  const mesh = r.objectFor(h) as THREE.Mesh;
  const oldGeo = mesh.geometry;
  let disposed = false;
  oldGeo.addEventListener('dispose', () => { disposed = true; });

  r.applyDiff({ op: 'replaceMeshPayload', handle: h, payload: quadPayload() });
  assert.ok(disposed, 'old geometry should be disposed on replace');
  assert.notEqual(mesh.geometry, oldGeo);

  // A second replace disposes the first uploaded geometry too.
  const firstUpload = mesh.geometry;
  let secondDisposed = false;
  firstUpload.addEventListener('dispose', () => { secondDisposed = true; });
  r.applyDiff({ op: 'replaceMeshPayload', handle: h, payload: quadPayload() });
  assert.ok(secondDisposed);
});

test('replaceMeshPayload on an unknown handle throws', () => {
  const r = new ThreeRenderer();
  assert.throws(
    () => r.applyDiff({ op: 'replaceMeshPayload', handle: renderHandle(9), payload: quadPayload() }),
    RenderApplyError,
  );
});

test('handle-source payloads are rejected until runtime buffer wiring exists', () => {
  const r = new ThreeRenderer();
  const h = renderHandle(1);
  r.applyDiff({ op: 'create', handle: h, parent: null, node: meshNode() });
  const p = quadPayload();
  (p as { source: unknown }).source = {
    kind: 'handle',
    buffer: 7,
    positionsByteOffset: 0,
    normalsByteOffset: 48,
    indicesByteOffset: 96,
  };
  assert.throws(() => r.applyDiff({ op: 'replaceMeshPayload', handle: h, payload: p }), RenderApplyError);
});

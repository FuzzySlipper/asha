import assert from 'node:assert/strict';
import { test } from 'node:test';

import type { Transform } from '@asha/contracts';
import {
  DEFAULT_TRANSFORM_MANIPULATOR_SNAPPING,
  beginTransformManipulatorDrag,
  cancelTransformManipulatorDrag,
  projectTransformManipulator,
  transformManipulatorHandleFromId,
  transformManipulatorHandleId,
  updateTransformManipulatorDrag,
  type TransformManipulatorCamera,
  type TransformManipulatorHandle,
} from './transform-manipulator.js';

const SOURCE: Transform = {
  translation: [0, 0, 0],
  rotation: [0, 0, 0, 1],
  scale: [1, 1, 1],
};

const CAMERA: TransformManipulatorCamera = {
  position: [0, 0, 10],
  basis: {
    forward: [0, 0, -1],
    right: [1, 0, 0],
    up: [0, 1, 0],
  },
  fovYDegrees: 60,
  viewport: { width: 800, height: 800 },
};

function drag(handle: TransformManipulatorHandle, pointer: readonly [number, number], source = SOURCE) {
  return beginTransformManipulatorDrag({
    camera: CAMERA,
    handle,
    orientation: 'world',
    pointer,
    revision: 'scene-revision:7',
    snapping: DEFAULT_TRANSFORM_MANIPULATOR_SNAPPING,
    source,
  });
}

void test('manipulator projects stable renderer-neutral handles and maps picks back to intent', () => {
  const frame = projectTransformManipulator({
    active: null,
    hovered: { kind: 'axis', mode: 'translate', axis: 'x' },
    mode: 'translate',
    orientation: 'world',
    transform: SOURCE,
    visible: true,
  });
  assert.equal(frame.ops.length, 6);
  for (const operation of frame.ops) {
    assert.equal(operation.op, 'create');
    if (operation.op !== 'create') continue;
    assert.equal(operation.node.layer, 'debug');
    assert.equal(operation.node.metadata.source, null);
    assert.match(operation.node.metadata.label ?? '', /^transform-manipulator:translate:/);
  }
  const x = { kind: 'axis', mode: 'translate', axis: 'x' } as const;
  assert.deepEqual(transformManipulatorHandleFromId(transformManipulatorHandleId(x)), x);
});

void test('axis and plane translation are camera aware, snapped, and fine-adjustable', () => {
  const axisDrag = drag({ kind: 'axis', mode: 'translate', axis: 'x' }, [400, 400]);
  const axisCandidate = updateTransformManipulatorDrag(axisDrag, CAMERA, [480, 400]);
  assert.ok(axisCandidate.transform.translation[0] > 1);
  assert.equal(axisCandidate.transform.translation[0] % 0.25, 0);
  assert.deepEqual(axisCandidate.transform.translation.slice(1), [0, 0]);

  const fineCandidate = updateTransformManipulatorDrag(axisDrag, CAMERA, [480, 400], { fine: true });
  assert.ok(fineCandidate.transform.translation[0] < axisCandidate.transform.translation[0]);

  const planeDrag = drag({ kind: 'plane', mode: 'translate', plane: 'xy' }, [400, 400]);
  const planeCandidate = updateTransformManipulatorDrag(planeDrag, CAMERA, [480, 320], { snapping: false });
  assert.ok(planeCandidate.transform.translation[0] > 0);
  assert.ok(planeCandidate.transform.translation[1] > 0);
  assert.equal(planeCandidate.transform.translation[2], 0);
});

void test('axis rotation, axis scale, uniform scale, and cancellation preserve explicit preview semantics', () => {
  const rotationDrag = drag({ kind: 'axis', mode: 'rotate', axis: 'z' }, [520, 400]);
  const rotationCandidate = updateTransformManipulatorDrag(rotationDrag, CAMERA, [400, 280]);
  assert.equal(rotationCandidate.previewOnly, true);
  assert.equal(rotationCandidate.revision, 'scene-revision:7');
  assert.ok(Math.abs(rotationCandidate.transform.rotation[2]) > 0.1);
  assert.ok(Math.abs(Math.hypot(...rotationCandidate.transform.rotation) - 1) < 1e-9);

  const axisScaleDrag = drag({ kind: 'axis', mode: 'scale', axis: 'x' }, [400, 400]);
  const axisScaleCandidate = updateTransformManipulatorDrag(axisScaleDrag, CAMERA, [460, 400], { snapping: false });
  assert.ok(axisScaleCandidate.transform.scale[0] > 1);
  assert.deepEqual(axisScaleCandidate.transform.scale.slice(1), [1, 1]);

  const uniformDrag = drag({ kind: 'uniform', mode: 'scale' }, [400, 400]);
  const uniformCandidate = updateTransformManipulatorDrag(uniformDrag, CAMERA, [460, 340], { snapping: false });
  assert.ok(uniformCandidate.transform.scale[0] > 1);
  assert.equal(uniformCandidate.transform.scale[0], uniformCandidate.transform.scale[1]);
  assert.equal(uniformCandidate.transform.scale[1], uniformCandidate.transform.scale[2]);

  const cancelled = cancelTransformManipulatorDrag(uniformDrag);
  assert.deepEqual(cancelled.transform, SOURCE);
  assert.match(cancelled.diagnostics[0] ?? '', /cancelled/);
});

void test('local axes rotate with the source and degeneracies fail safe without invalid transforms', () => {
  const half = Math.sin(Math.PI / 4);
  const rotatedSource: Transform = {
    ...SOURCE,
    rotation: [0, 0, half, half],
  };
  const localDrag = beginTransformManipulatorDrag({
    camera: CAMERA,
    handle: { kind: 'axis', mode: 'translate', axis: 'x' },
    orientation: 'local',
    pointer: [400, 400],
    revision: 'scene-revision:8',
    snapping: DEFAULT_TRANSFORM_MANIPULATOR_SNAPPING,
    source: rotatedSource,
  });
  const candidate = updateTransformManipulatorDrag(localDrag, CAMERA, [400, 320], { snapping: false });
  assert.ok(candidate.transform.translation[1] > 0);
  assert.ok(Math.abs(candidate.transform.translation[0]) < 1e-6);
  assert.ok(candidate.transform.scale.every(value => Number.isFinite(value) && value > 0));

  assert.throws(
    () => beginTransformManipulatorDrag({
      camera: { ...CAMERA, viewport: { width: 0, height: 800 } },
      handle: { kind: 'uniform', mode: 'scale' },
      orientation: 'world',
      pointer: [0, 0],
      revision: 'bad-camera',
      snapping: DEFAULT_TRANSFORM_MANIPULATOR_SNAPPING,
      source: SOURCE,
    }),
    /viewport must be positive/,
  );
});

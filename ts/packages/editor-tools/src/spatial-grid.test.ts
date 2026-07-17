import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import type { SpatialGridSpec } from '@asha/contracts';
import {
  DEFAULT_EDITOR_GRID_DESCRIPTOR,
  snapSpatialGridPoint,
  spatialGridCellBounds,
  spatialGridCellCenter,
  spatialGridCellMin,
  validateEditorGridDescriptor,
  worldToSpatialGridCell,
} from './spatial-grid.js';

interface FixtureCase {
  readonly name: string;
  readonly spec: Pick<SpatialGridSpec, 'origin' | 'spacing'>;
  readonly world: readonly [number, number, number];
  readonly cell: readonly [number, number, number];
  readonly cellMin: readonly [number, number, number];
  readonly cellCenter: readonly [number, number, number];
  readonly cellMax: readonly [number, number, number];
  readonly boundarySnap: readonly [number, number, number];
  readonly centerSnap: readonly [number, number, number];
}

const fixture = JSON.parse(readFileSync(new URL(
  '../../../../harness/fixtures/spatial-grid/conformance.json',
  import.meta.url,
), 'utf8')) as {
  readonly coordinateSystem: SpatialGridSpec['coordinateSystem'];
  readonly cases: readonly FixtureCase[];
};

test('public TypeScript grid math matches the Rust-owned conformance fixture', () => {
  assert.equal(fixture.coordinateSystem, 'rightHandedYUp');
  for (const entry of fixture.cases) {
    const spec: SpatialGridSpec = {
      coordinateSystem: fixture.coordinateSystem,
      ...entry.spec,
    };
    assert.deepEqual(worldToSpatialGridCell(spec, entry.world), entry.cell, entry.name);
    assert.deepEqual(spatialGridCellMin(spec, entry.cell), entry.cellMin, entry.name);
    assert.deepEqual(spatialGridCellCenter(spec, entry.cell), entry.cellCenter, entry.name);
    assert.deepEqual(spatialGridCellBounds(spec, entry.cell), {
      min: entry.cellMin,
      max: entry.cellMax,
    }, entry.name);
    assert.deepEqual(snapSpatialGridPoint(spec, entry.world, 'boundary'), entry.boundarySnap, entry.name);
    assert.deepEqual(snapSpatialGridPoint(spec, entry.world, 'cellCenter'), entry.centerSnap, entry.name);
  }
});

test('default editor grid is an XZ projection in the mandated Y-up world', () => {
  assert.equal(validateEditorGridDescriptor(DEFAULT_EDITOR_GRID_DESCRIPTOR), DEFAULT_EDITOR_GRID_DESCRIPTOR);
  assert.equal(DEFAULT_EDITOR_GRID_DESCRIPTOR.grid.coordinateSystem, 'rightHandedYUp');
  assert.equal(DEFAULT_EDITOR_GRID_DESCRIPTOR.plane, 'xz');
  assert.deepEqual(DEFAULT_EDITOR_GRID_DESCRIPTOR.grid.origin, [0, 0, 0]);
});

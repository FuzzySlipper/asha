import type {
  EditorGridDescriptor,
  SpatialGridSnapAnchor,
  SpatialGridSpec,
} from '@asha/contracts';

export type SpatialGridPoint = readonly [number, number, number];
export type SpatialGridCell = readonly [number, number, number];

export interface SpatialGridCellBounds {
  readonly min: SpatialGridPoint;
  readonly max: SpatialGridPoint;
}

export const DEFAULT_EDITOR_GRID_DESCRIPTOR: EditorGridDescriptor = {
  visible: true,
  grid: {
    coordinateSystem: 'rightHandedYUp',
    origin: [0, 0, 0],
    spacing: [0.25, 0.25, 0.25],
  },
  plane: 'xz',
  snapAnchor: 'boundary',
  style: {
    minorColor: [0.12, 0.2, 0.24, 0.42],
    majorColor: [0.24, 0.34, 0.4, 0.72],
    xAxisColor: [0.95, 0.2, 0.18, 0.9],
    yAxisColor: [0.2, 0.9, 0.3, 0.9],
    zAxisColor: [0.2, 0.45, 1, 0.9],
    majorLineEvery: 4,
    opacity: 1,
    fadeStart: 12,
    fadeEnd: 64,
  },
};

export function validateSpatialGridSpec(spec: SpatialGridSpec): SpatialGridSpec {
  if (spec.coordinateSystem !== 'rightHandedYUp') {
    throw new Error('spatial grid must use the right-handed Y-up coordinate system');
  }
  requireFinitePoint(spec.origin, 'spatial grid origin');
  requireFinitePoint(spec.spacing, 'spatial grid spacing');
  if (spec.spacing.some(value => value <= 0)) {
    throw new Error('spatial grid spacing must be positive on every axis');
  }
  return spec;
}

export function validateEditorGridDescriptor(
  descriptor: EditorGridDescriptor,
): EditorGridDescriptor {
  validateSpatialGridSpec(descriptor.grid);
  const colors = [
    descriptor.style.minorColor,
    descriptor.style.majorColor,
    descriptor.style.xAxisColor,
    descriptor.style.yAxisColor,
    descriptor.style.zAxisColor,
  ];
  if (colors.some(color => color.some(value => !Number.isFinite(value) || value < 0 || value > 1))) {
    throw new Error('editor grid colors must contain finite normalized RGBA channels');
  }
  if (!Number.isSafeInteger(descriptor.style.majorLineEvery) || descriptor.style.majorLineEvery < 1) {
    throw new Error('editor grid major-line cadence must be a positive safe integer');
  }
  if (!Number.isFinite(descriptor.style.opacity)
    || descriptor.style.opacity < 0
    || descriptor.style.opacity > 1) {
    throw new Error('editor grid opacity must be normalized');
  }
  if (!Number.isFinite(descriptor.style.fadeStart)
    || !Number.isFinite(descriptor.style.fadeEnd)
    || descriptor.style.fadeStart < 0
    || descriptor.style.fadeEnd <= descriptor.style.fadeStart) {
    throw new Error('editor grid fade range must be finite and increasing');
  }
  return descriptor;
}

export function worldToSpatialGridCell(
  spec: SpatialGridSpec,
  world: SpatialGridPoint,
): SpatialGridCell {
  validateSpatialGridSpec(spec);
  requireFinitePoint(world, 'world point');
  return tupleMap(world, (value, axis) => {
    const cell = Math.floor((value - spec.origin[axis]!) / spec.spacing[axis]!);
    if (!Number.isSafeInteger(cell)) {
      throw new Error(`world point exceeds safe spatial grid address range on axis ${axis}`);
    }
    return cell;
  });
}

export function spatialGridCellMin(
  spec: SpatialGridSpec,
  cell: SpatialGridCell,
): SpatialGridPoint {
  validateSpatialGridSpec(spec);
  requireSafeCell(cell);
  return tupleMap(cell, (value, axis) => spec.origin[axis]! + value * spec.spacing[axis]!);
}

export function spatialGridCellCenter(
  spec: SpatialGridSpec,
  cell: SpatialGridCell,
): SpatialGridPoint {
  const min = spatialGridCellMin(spec, cell);
  return tupleMap(min, (value, axis) => value + spec.spacing[axis]! * 0.5);
}

export function spatialGridCellBounds(
  spec: SpatialGridSpec,
  cell: SpatialGridCell,
): SpatialGridCellBounds {
  const min = spatialGridCellMin(spec, cell);
  return {
    min,
    max: tupleMap(min, (value, axis) => value + spec.spacing[axis]!),
  };
}

export function snapSpatialGridPoint(
  spec: SpatialGridSpec,
  world: SpatialGridPoint,
  anchor: SpatialGridSnapAnchor,
): SpatialGridPoint {
  validateSpatialGridSpec(spec);
  requireFinitePoint(world, 'world point');
  if (anchor === 'cellCenter') {
    return spatialGridCellCenter(spec, worldToSpatialGridCell(spec, world));
  }
  return tupleMap(world, (value, axis) => {
    const origin = spec.origin[axis]!;
    const spacing = spec.spacing[axis]!;
    const boundary = Math.floor((value - origin) / spacing + 0.5);
    return origin + boundary * spacing;
  });
}

function requireFinitePoint(point: SpatialGridPoint, label: string): void {
  if (point.some(value => !Number.isFinite(value))) {
    throw new Error(`${label} must contain finite values`);
  }
}

function requireSafeCell(cell: SpatialGridCell): void {
  if (cell.some(value => !Number.isSafeInteger(value))) {
    throw new Error('spatial grid cell must contain safe integers');
  }
}

function tupleMap(
  tuple: SpatialGridPoint,
  map: (value: number, axis: 0 | 1 | 2) => number,
): [number, number, number] {
  return [map(tuple[0], 0), map(tuple[1], 1), map(tuple[2], 2)];
}

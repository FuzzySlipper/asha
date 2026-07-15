import {
  renderHandle,
  type CameraBasis,
  type RenderFrameDiff,
  type RenderHandle,
  type Transform,
} from '@asha/contracts';

export type TransformManipulatorMode = 'translate' | 'rotate' | 'scale';
export type TransformManipulatorOrientation = 'local' | 'world';
export type TransformManipulatorAxis = 'x' | 'y' | 'z';
export type TransformManipulatorPlane = 'xy' | 'xz' | 'yz';

export type TransformManipulatorHandle =
  | { readonly kind: 'axis'; readonly mode: TransformManipulatorMode; readonly axis: TransformManipulatorAxis }
  | { readonly kind: 'plane'; readonly mode: 'translate'; readonly plane: TransformManipulatorPlane }
  | { readonly kind: 'uniform'; readonly mode: 'scale' };

export interface TransformManipulatorCamera {
  readonly position: readonly [number, number, number];
  readonly basis: CameraBasis;
  readonly fovYDegrees: number;
  readonly viewport: { readonly width: number; readonly height: number };
}

export interface TransformManipulatorSnapping {
  readonly enabled: boolean;
  readonly rotationDegrees: number;
  readonly scale: number;
  readonly translation: number;
}

export interface TransformManipulatorAppearance {
  readonly active: TransformManipulatorHandle | null;
  readonly hovered: TransformManipulatorHandle | null;
  readonly mode: TransformManipulatorMode;
  readonly orientation: TransformManipulatorOrientation;
  readonly transform: Transform;
  readonly visible: boolean;
}

export interface TransformManipulatorDragInput {
  readonly camera: TransformManipulatorCamera;
  readonly handle: TransformManipulatorHandle;
  readonly orientation: TransformManipulatorOrientation;
  readonly pointer: readonly [number, number];
  readonly revision: string;
  readonly snapping: TransformManipulatorSnapping;
  readonly source: Transform;
}

export interface TransformManipulatorDrag {
  readonly kind: 'transform_manipulator_drag.v0';
  readonly axis: readonly [number, number, number] | null;
  readonly handle: TransformManipulatorHandle;
  readonly orientation: TransformManipulatorOrientation;
  readonly planeNormal: readonly [number, number, number] | null;
  readonly revision: string;
  readonly snapping: TransformManipulatorSnapping;
  readonly source: Transform;
  readonly startAxisParameter: number | null;
  readonly startPlanePoint: readonly [number, number, number] | null;
  readonly startRotationVector: readonly [number, number, number] | null;
  readonly startPointer: readonly [number, number];
}

export interface TransformManipulatorCandidate {
  readonly kind: 'transform_manipulator_candidate.v0';
  readonly diagnostics: readonly string[];
  readonly previewOnly: true;
  readonly revision: string;
  readonly transform: Transform;
}

const AXIS_VECTORS = {
  x: [1, 0, 0],
  y: [0, 1, 0],
  z: [0, 0, 1],
} as const;

const AXIS_COLORS = {
  x: [0.95, 0.2, 0.18, 1],
  y: [0.2, 0.9, 0.3, 1],
  z: [0.2, 0.45, 1, 1],
} as const;

const HANDLE_BASE = 940_000;
const EPSILON = 1e-8;

const HANDLE_SLOTS: readonly TransformManipulatorHandle[] = [
  { kind: 'axis', mode: 'translate', axis: 'x' },
  { kind: 'axis', mode: 'translate', axis: 'y' },
  { kind: 'axis', mode: 'translate', axis: 'z' },
  { kind: 'plane', mode: 'translate', plane: 'xy' },
  { kind: 'plane', mode: 'translate', plane: 'xz' },
  { kind: 'plane', mode: 'translate', plane: 'yz' },
  { kind: 'axis', mode: 'rotate', axis: 'x' },
  { kind: 'axis', mode: 'rotate', axis: 'y' },
  { kind: 'axis', mode: 'rotate', axis: 'z' },
  { kind: 'axis', mode: 'scale', axis: 'x' },
  { kind: 'axis', mode: 'scale', axis: 'y' },
  { kind: 'axis', mode: 'scale', axis: 'z' },
  { kind: 'uniform', mode: 'scale' },
] as const;

export const DEFAULT_TRANSFORM_MANIPULATOR_SNAPPING: TransformManipulatorSnapping = {
  enabled: true,
  rotationDegrees: 15,
  scale: 0.1,
  translation: 0.25,
};

export function transformManipulatorHandleId(handle: TransformManipulatorHandle): RenderHandle {
  const slot = HANDLE_SLOTS.findIndex(candidate => sameHandle(candidate, handle));
  if (slot < 0) throw new Error('unsupported transform manipulator handle');
  return renderHandle(HANDLE_BASE + slot);
}

export function transformManipulatorHandleFromId(handle: RenderHandle): TransformManipulatorHandle | null {
  const slot = (handle as number) - HANDLE_BASE;
  return HANDLE_SLOTS[slot] ?? null;
}

export function projectTransformManipulator(
  appearance: TransformManipulatorAppearance,
): RenderFrameDiff {
  if (!appearance.visible) return { ops: [] };
  const handles = HANDLE_SLOTS.filter(handle => handle.mode === appearance.mode);
  return {
    ops: handles.map(handle => {
      const axis = handle.kind === 'axis' ? orientedAxis(handle.axis, appearance.orientation, appearance.transform) : null;
      const color = handleColor(handle, appearance.active, appearance.hovered);
      const placement = handlePlacement(handle, appearance.transform, axis, appearance.orientation);
      return {
        op: 'create' as const,
        handle: transformManipulatorHandleId(handle),
        parent: null,
        node: {
          geometry: placement.geometry,
          material: { color, wireframe: placement.wireframe },
          transform: placement.transform,
          visible: true,
          layer: 'debug' as const,
          metadata: {
            source: null,
            tags: [],
            label: transformManipulatorHandleLabel(handle),
          },
        },
      };
    }),
  };
}

export function beginTransformManipulatorDrag(
  input: TransformManipulatorDragInput,
): TransformManipulatorDrag {
  validateTransform(input.source);
  validateCamera(input.camera);
  const ray = pointerRay(input.camera, input.pointer);
  const axis = input.handle.kind === 'axis'
    ? orientedAxis(input.handle.axis, input.orientation, input.source)
    : input.handle.kind === 'uniform'
      ? normalize(add(input.camera.basis.right, input.camera.basis.up))
      : null;
  const planeNormal = dragPlaneNormal(
    input.handle,
    axis,
    input.camera,
    input.orientation,
    input.source,
  );
  const startPlanePoint = planeNormal === null
    ? null
    : rayPlaneIntersection(ray.origin, ray.direction, input.source.translation, planeNormal);
  const startAxisParameter = axis === null
    ? null
    : closestAxisParameter(ray.origin, ray.direction, input.source.translation, axis);
  const startRotationVector = input.handle.mode === 'rotate' && startPlanePoint !== null
    ? normalize(subtract(startPlanePoint, input.source.translation))
    : null;
  return {
    kind: 'transform_manipulator_drag.v0',
    axis,
    handle: input.handle,
    orientation: input.orientation,
    planeNormal,
    revision: requireIdentity(input.revision, 'revision'),
    snapping: validateSnapping(input.snapping),
    source: cloneTransform(input.source),
    startAxisParameter,
    startPlanePoint,
    startRotationVector,
    startPointer: input.pointer,
  };
}

export function updateTransformManipulatorDrag(
  drag: TransformManipulatorDrag,
  camera: TransformManipulatorCamera,
  pointer: readonly [number, number],
  options: { readonly fine?: boolean; readonly snapping?: boolean } = {},
): TransformManipulatorCandidate {
  validateCamera(camera);
  const fine = options.fine === true ? 0.1 : 1;
  const snap = options.snapping ?? drag.snapping.enabled;
  const ray = pointerRay(camera, pointer);
  const diagnostics: string[] = [];
  let transform = cloneTransform(drag.source);

  if (drag.handle.mode === 'translate') {
    const delta = translationDelta(drag, ray, camera, pointer, fine, diagnostics);
    const translated = add(drag.source.translation, delta);
    transform = { ...transform, translation: snapVector(translated, drag.snapping.translation, snap) };
  } else if (drag.handle.mode === 'rotate' && drag.axis !== null) {
    const radians = rotationDelta(drag, ray, camera, pointer, fine, diagnostics);
    const snapped = snapScalar(radians, degreesToRadians(drag.snapping.rotationDegrees), snap);
    const deltaRotation = axisAngleQuaternion(drag.axis, snapped);
    transform = {
      ...transform,
      rotation: normalizeQuaternion(
        drag.orientation === 'local'
          ? multiplyQuaternion(drag.source.rotation, deltaRotation)
          : multiplyQuaternion(deltaRotation, drag.source.rotation),
      ),
    };
  } else if (drag.handle.mode === 'scale') {
    const delta = scaleDelta(drag, ray, camera, pointer, fine, diagnostics);
    const next = drag.handle.kind === 'uniform'
      ? drag.source.scale.map(value => value * delta) as [number, number, number]
      : applyAxisScale(drag.source.scale, drag.handle.axis, delta);
    transform = {
      ...transform,
      scale: clampScale(snapVector(next, drag.snapping.scale, snap)),
    };
  }

  return {
    kind: 'transform_manipulator_candidate.v0',
    diagnostics,
    previewOnly: true,
    revision: drag.revision,
    transform,
  };
}

export function cancelTransformManipulatorDrag(drag: TransformManipulatorDrag): TransformManipulatorCandidate {
  return {
    kind: 'transform_manipulator_candidate.v0',
    diagnostics: ['transform manipulator drag cancelled; authoritative source restored'],
    previewOnly: true,
    revision: drag.revision,
    transform: cloneTransform(drag.source),
  };
}

function translationDelta(
  drag: TransformManipulatorDrag,
  ray: Ray,
  camera: TransformManipulatorCamera,
  pointer: readonly [number, number],
  fine: number,
  diagnostics: string[],
): Vec3 {
  if (drag.handle.kind === 'plane' && drag.planeNormal !== null && drag.startPlanePoint !== null) {
    const point = rayPlaneIntersection(ray.origin, ray.direction, drag.source.translation, drag.planeNormal);
    if (point !== null) return scale(subtract(point, drag.startPlanePoint), fine);
  }
  if (drag.axis !== null && drag.startAxisParameter !== null) {
    const parameter = closestAxisParameter(ray.origin, ray.direction, drag.source.translation, drag.axis);
    if (parameter !== null) return scale(drag.axis, (parameter - drag.startAxisParameter) * fine);
  }
  diagnostics.push('used camera-space translation fallback for a near-parallel drag');
  const amount = pointerFallback(camera, drag.startPointer, pointer, drag.source.translation) * fine;
  return scale(drag.axis ?? camera.basis.right, amount);
}

function rotationDelta(
  drag: TransformManipulatorDrag,
  ray: Ray,
  camera: TransformManipulatorCamera,
  pointer: readonly [number, number],
  fine: number,
  diagnostics: string[],
): number {
  if (drag.planeNormal !== null && drag.startRotationVector !== null) {
    const point = rayPlaneIntersection(ray.origin, ray.direction, drag.source.translation, drag.planeNormal);
    if (point !== null) {
      const current = normalize(subtract(point, drag.source.translation));
      const crossValue = cross(drag.startRotationVector, current);
      return Math.atan2(dot(crossValue, drag.axis ?? drag.planeNormal), dot(drag.startRotationVector, current)) * fine;
    }
  }
  diagnostics.push('used screen-space rotation fallback for a near-parallel drag');
  return ((pointer[0] - drag.startPointer[0]) / Math.max(1, camera.viewport.width)) * Math.PI * 2 * fine;
}

function scaleDelta(
  drag: TransformManipulatorDrag,
  ray: Ray,
  camera: TransformManipulatorCamera,
  pointer: readonly [number, number],
  fine: number,
  diagnostics: string[],
): number {
  if (drag.axis !== null && drag.startAxisParameter !== null) {
    const parameter = closestAxisParameter(ray.origin, ray.direction, drag.source.translation, drag.axis);
    if (parameter !== null) {
      const delta = (parameter - drag.startAxisParameter) * fine;
      return Math.max(0.001, 1 + delta);
    }
  }
  diagnostics.push('used screen-space scale fallback for a near-parallel drag');
  const pixels = (pointer[0] - drag.startPointer[0]) - (pointer[1] - drag.startPointer[1]);
  return Math.max(0.001, 1 + pixels / Math.max(1, camera.viewport.height) * 2 * fine);
}

function handlePlacement(
  handle: TransformManipulatorHandle,
  source: Transform,
  axis: Vec3 | null,
  orientation: TransformManipulatorOrientation,
): { readonly geometry: { readonly shape: 'cube' | 'sphere' }; readonly transform: Transform; readonly wireframe: boolean } {
  if (handle.kind === 'uniform') {
    return {
      geometry: { shape: 'sphere' },
      transform: { translation: source.translation, rotation: [0, 0, 0, 1], scale: [0.18, 0.18, 0.18] },
      wireframe: false,
    };
  }
  if (handle.kind === 'plane') {
    const [firstAxis, secondAxis] = planeAxes(handle.plane);
    const first = orientedAxis(firstAxis, orientation, source);
    const second = orientedAxis(secondAxis, orientation, source);
    const translation = add(source.translation, scale(add(first, second), 0.28));
    const scaleValue: Vec3 = handle.plane === 'xy'
      ? [0.22, 0.22, 0.035]
      : handle.plane === 'xz'
        ? [0.22, 0.035, 0.22]
        : [0.035, 0.22, 0.22];
    return {
      geometry: { shape: 'cube' },
      transform: {
        translation,
        rotation: orientation === 'local' ? source.rotation : [0, 0, 0, 1],
        scale: scaleValue,
      },
      wireframe: true,
    };
  }
  const resolvedAxis = axis ?? AXIS_VECTORS[handle.axis];
  const distance = handle.mode === 'rotate' ? 0.78 : 0.62;
  const thickness = handle.mode === 'rotate' ? 0.07 : 0.09;
  return {
    geometry: handle.mode === 'rotate' ? { shape: 'sphere' } : { shape: 'cube' },
    transform: {
      translation: add(source.translation, scale(resolvedAxis, distance)),
      rotation: quaternionFromUnitX(resolvedAxis),
      scale: handle.mode === 'rotate' ? [0.16, 0.16, 0.16] : [0.55, thickness, thickness],
    },
    wireframe: handle.mode === 'rotate',
  };
}

function handleColor(
  handle: TransformManipulatorHandle,
  active: TransformManipulatorHandle | null,
  hovered: TransformManipulatorHandle | null,
): readonly [number, number, number, number] {
  if (active !== null && sameHandle(active, handle)) return [1, 0.85, 0.1, 1];
  if (hovered !== null && sameHandle(hovered, handle)) return [1, 1, 0.55, 1];
  if (handle.kind === 'axis') return AXIS_COLORS[handle.axis];
  if (handle.kind === 'plane') return [0.85, 0.85, 0.85, 0.45];
  return [0.95, 0.95, 0.95, 1];
}

function transformManipulatorHandleLabel(handle: TransformManipulatorHandle): string {
  const target = handle.kind === 'axis' ? handle.axis : handle.kind === 'plane' ? handle.plane : 'uniform';
  return `transform-manipulator:${handle.mode}:${target}`;
}

function dragPlaneNormal(
  handle: TransformManipulatorHandle,
  axis: Vec3 | null,
  camera: TransformManipulatorCamera,
  orientation: TransformManipulatorOrientation,
  source: Transform,
): Vec3 | null {
  if (handle.kind === 'plane') {
    const [first, second] = planeAxes(handle.plane);
    return normalize(cross(
      orientedAxis(first, orientation, source),
      orientedAxis(second, orientation, source),
    ));
  }
  if (handle.mode === 'rotate') return axis;
  if (axis === null) return normalize(camera.basis.forward);
  const perpendicular = cross(axis, camera.basis.forward);
  if (length(perpendicular) <= EPSILON) return normalize(camera.basis.up);
  return normalize(cross(perpendicular, axis));
}

function orientedAxis(axis: TransformManipulatorAxis, orientation: TransformManipulatorOrientation, transform: Transform): Vec3 {
  const world = AXIS_VECTORS[axis];
  return orientation === 'local' ? normalize(rotateVector(transform.rotation, world)) : world;
}

function planeAxes(plane: TransformManipulatorPlane): readonly [TransformManipulatorAxis, TransformManipulatorAxis] {
  if (plane === 'xy') return ['x', 'y'];
  if (plane === 'xz') return ['x', 'z'];
  return ['y', 'z'];
}

function sameHandle(left: TransformManipulatorHandle, right: TransformManipulatorHandle): boolean {
  if (left.kind !== right.kind || left.mode !== right.mode) return false;
  if (left.kind === 'axis' && right.kind === 'axis') return left.axis === right.axis;
  if (left.kind === 'plane' && right.kind === 'plane') return left.plane === right.plane;
  return left.kind === 'uniform' && right.kind === 'uniform';
}

interface Ray { readonly origin: Vec3; readonly direction: Vec3 }
type Vec3 = readonly [number, number, number];
type Quaternion = readonly [number, number, number, number];

function pointerRay(camera: TransformManipulatorCamera, pointer: readonly [number, number]): Ray {
  const x = (pointer[0] / camera.viewport.width) * 2 - 1;
  const y = 1 - (pointer[1] / camera.viewport.height) * 2;
  const tangent = Math.tan(degreesToRadians(camera.fovYDegrees) / 2);
  const aspect = camera.viewport.width / camera.viewport.height;
  return {
    origin: camera.position,
    direction: normalize(add(camera.basis.forward, add(
      scale(camera.basis.right, x * tangent * aspect),
      scale(camera.basis.up, y * tangent),
    ))),
  };
}

function closestAxisParameter(rayOrigin: Vec3, rayDirection: Vec3, axisOrigin: Vec3, axis: Vec3): number | null {
  const offset = subtract(rayOrigin, axisOrigin);
  const a = dot(rayDirection, rayDirection);
  const b = dot(rayDirection, axis);
  const c = dot(axis, axis);
  const d = dot(rayDirection, offset);
  const e = dot(axis, offset);
  const denominator = a * c - b * b;
  if (Math.abs(denominator) <= EPSILON) return null;
  return (a * e - b * d) / denominator;
}

function rayPlaneIntersection(rayOrigin: Vec3, rayDirection: Vec3, planePoint: Vec3, planeNormal: Vec3): Vec3 | null {
  const denominator = dot(rayDirection, planeNormal);
  if (Math.abs(denominator) <= EPSILON) return null;
  const distance = dot(subtract(planePoint, rayOrigin), planeNormal) / denominator;
  return add(rayOrigin, scale(rayDirection, distance));
}

function pointerFallback(
  camera: TransformManipulatorCamera,
  start: readonly [number, number],
  current: readonly [number, number],
  origin: Vec3,
): number {
  const pixelDistance = (current[0] - start[0]) - (current[1] - start[1]);
  const worldPerViewport = 2 * Math.tan(degreesToRadians(camera.fovYDegrees) / 2);
  return pixelDistance / Math.max(1, camera.viewport.height) * worldPerViewport * distance(camera.position, origin);
}

function applyAxisScale(source: Vec3, axis: TransformManipulatorAxis, factor: number): [number, number, number] {
  const next: [number, number, number] = [...source];
  next[axis === 'x' ? 0 : axis === 'y' ? 1 : 2] *= factor;
  return next;
}

function snapVector(value: Vec3, increment: number, enabled: boolean): [number, number, number] {
  return enabled
    ? value.map(component => snapScalar(component, increment, true)) as [number, number, number]
    : [...value];
}

function snapScalar(value: number, increment: number, enabled: boolean): number {
  if (!enabled || increment <= EPSILON) return value;
  return Math.round(value / increment) * increment;
}

function clampScale(value: Vec3): [number, number, number] {
  return value.map(component => Math.max(0.001, component)) as [number, number, number];
}

function validateTransform(transform: Transform): void {
  const values = [...transform.translation, ...transform.rotation, ...transform.scale];
  if (values.some(value => !Number.isFinite(value))) throw new Error('transform manipulator source must be finite');
  if (transform.scale.some(value => value <= 0)) throw new Error('transform manipulator source scale must be positive');
  if (Math.hypot(...transform.rotation) <= EPSILON) throw new Error('transform manipulator source rotation must be nonzero');
}

function validateCamera(camera: TransformManipulatorCamera): void {
  const values = [
    ...camera.position,
    ...camera.basis.forward,
    ...camera.basis.right,
    ...camera.basis.up,
    camera.fovYDegrees,
    camera.viewport.width,
    camera.viewport.height,
  ];
  if (values.some(value => !Number.isFinite(value))) throw new Error('transform manipulator camera must be finite');
  if (camera.viewport.width <= 0 || camera.viewport.height <= 0) throw new Error('transform manipulator viewport must be positive');
  if (camera.fovYDegrees <= 0 || camera.fovYDegrees >= 180) throw new Error('transform manipulator fov must be in 0..180');
}

function validateSnapping(snapping: TransformManipulatorSnapping): TransformManipulatorSnapping {
  for (const value of [snapping.rotationDegrees, snapping.scale, snapping.translation]) {
    if (!Number.isFinite(value) || value <= 0) throw new Error('transform manipulator snap increments must be positive and finite');
  }
  return { ...snapping };
}

function requireIdentity(value: string, label: string): string {
  const trimmed = value.trim();
  if (trimmed.length === 0) throw new Error(`${label} must not be empty`);
  return trimmed;
}

function cloneTransform(transform: Transform): Transform {
  return {
    translation: [...transform.translation],
    rotation: [...transform.rotation],
    scale: [...transform.scale],
  };
}

function add(left: Vec3, right: Vec3): Vec3 {
  return [left[0] + right[0], left[1] + right[1], left[2] + right[2]];
}

function subtract(left: Vec3, right: Vec3): Vec3 {
  return [left[0] - right[0], left[1] - right[1], left[2] - right[2]];
}

function scale(value: Vec3, scalar: number): Vec3 {
  return [value[0] * scalar, value[1] * scalar, value[2] * scalar];
}

function dot(left: Vec3, right: Vec3): number {
  return left[0] * right[0] + left[1] * right[1] + left[2] * right[2];
}

function cross(left: Vec3, right: Vec3): Vec3 {
  return [
    left[1] * right[2] - left[2] * right[1],
    left[2] * right[0] - left[0] * right[2],
    left[0] * right[1] - left[1] * right[0],
  ];
}

function length(value: Vec3): number {
  return Math.hypot(...value);
}

function normalize(value: Vec3): Vec3 {
  const magnitude = length(value);
  if (magnitude <= EPSILON) return [0, 0, 0];
  return scale(value, 1 / magnitude);
}

function distance(left: Vec3, right: Vec3): number {
  return length(subtract(left, right));
}

function rotateVector(rotation: Quaternion, value: Vec3): Vec3 {
  const vector: Quaternion = [value[0], value[1], value[2], 0];
  const inverse: Quaternion = [-rotation[0], -rotation[1], -rotation[2], rotation[3]];
  const result = multiplyQuaternion(multiplyQuaternion(rotation, vector), inverse);
  return [result[0], result[1], result[2]];
}

function axisAngleQuaternion(axis: Vec3, radians: number): Quaternion {
  const half = radians / 2;
  const sine = Math.sin(half);
  return [axis[0] * sine, axis[1] * sine, axis[2] * sine, Math.cos(half)];
}

function multiplyQuaternion(left: Quaternion, right: Quaternion): Quaternion {
  const [ax, ay, az, aw] = left;
  const [bx, by, bz, bw] = right;
  return [
    aw * bx + ax * bw + ay * bz - az * by,
    aw * by - ax * bz + ay * bw + az * bx,
    aw * bz + ax * by - ay * bx + az * bw,
    aw * bw - ax * bx - ay * by - az * bz,
  ];
}

function normalizeQuaternion(value: Quaternion): Quaternion {
  const magnitude = Math.hypot(...value);
  if (magnitude <= EPSILON) return [0, 0, 0, 1];
  return value.map(component => component / magnitude) as [number, number, number, number];
}

function quaternionFromUnitX(axis: Vec3): Quaternion {
  const unit = normalize(axis);
  const cosine = dot([1, 0, 0], unit);
  if (cosine > 1 - EPSILON) return [0, 0, 0, 1];
  if (cosine < -1 + EPSILON) return [0, 1, 0, 0];
  return normalizeQuaternion([0, -unit[2], unit[1], 1 + cosine]);
}

function degreesToRadians(degrees: number): number {
  return degrees * Math.PI / 180;
}

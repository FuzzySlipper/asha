import type { PerspectiveProjection } from '@asha/contracts';
import type { AshaRendererEditorViewportCamera } from './editor-viewport.js';

type CameraVector = readonly [number, number, number];

export type AshaStoredEditorCameraDiagnosticCode =
  | 'coincident_position_target'
  | 'collinear_camera_up'
  | 'invalid_projection'
  | 'non_finite_camera_input';

export interface AshaStoredEditorCameraDiagnostic {
  readonly code: AshaStoredEditorCameraDiagnosticCode;
  readonly message: string;
}

export interface AshaStoredEditorCameraInput {
  readonly position: CameraVector;
  readonly target: CameraVector;
  readonly up: CameraVector;
  readonly projection: PerspectiveProjection;
}

export type AshaStoredEditorCameraResolution =
  | {
      readonly ok: true;
      readonly camera: AshaRendererEditorViewportCamera;
    }
  | {
      readonly ok: false;
      readonly diagnostic: AshaStoredEditorCameraDiagnostic;
    };

const CAMERA_VECTOR_EPSILON = 0.000_001;
const RADIANS_TO_DEGREES = 180 / Math.PI;

function vectorLength(vector: CameraVector): number {
  return Math.hypot(vector[0], vector[1], vector[2]);
}

function canonicalizeZero(value: number): number {
  return Object.is(value, -0) ? 0 : value;
}

function normalize(vector: CameraVector): CameraVector | null {
  const length = vectorLength(vector);
  if (length <= CAMERA_VECTOR_EPSILON) {
    return null;
  }
  return [
    canonicalizeZero(vector[0] / length),
    canonicalizeZero(vector[1] / length),
    canonicalizeZero(vector[2] / length),
  ];
}

function cross(left: CameraVector, right: CameraVector): CameraVector {
  return [
    left[1] * right[2] - left[2] * right[1],
    left[2] * right[0] - left[0] * right[2],
    left[0] * right[1] - left[1] * right[0],
  ];
}

function allFinite(vectors: readonly CameraVector[]): boolean {
  return vectors.every((vector) => vector.every(Number.isFinite));
}

/**
 * Resolves stored editor look-at intent into the canonical renderer-host camera
 * convention. The result is disposable projection state and is never runtime
 * authority or a camera matrix.
 */
export function resolveAshaStoredEditorCamera(
  input: AshaStoredEditorCameraInput,
): AshaStoredEditorCameraResolution {
  if (!allFinite([input.position, input.target, input.up])) {
    return {
      ok: false,
      diagnostic: {
        code: 'non_finite_camera_input',
        message: 'stored editor camera position, target, and up vectors must be finite',
      },
    };
  }
  if (
    !Number.isFinite(input.projection.fovYDegrees)
    || !Number.isFinite(input.projection.near)
    || !Number.isFinite(input.projection.far)
    || input.projection.fovYDegrees <= 0
    || input.projection.fovYDegrees >= 180
    || input.projection.near <= 0
    || input.projection.far <= input.projection.near
  ) {
    return {
      ok: false,
      diagnostic: {
        code: 'invalid_projection',
        message: 'stored editor camera projection requires finite perspective bounds with 0 < fov < 180 and 0 < near < far',
      },
    };
  }

  const forward = normalize([
    input.target[0] - input.position[0],
    input.target[1] - input.position[1],
    input.target[2] - input.position[2],
  ]);
  if (forward === null) {
    return {
      ok: false,
      diagnostic: {
        code: 'coincident_position_target',
        message: 'stored editor camera position and target must be distinct',
      },
    };
  }

  const right = normalize(cross(forward, input.up));
  if (right === null) {
    return {
      ok: false,
      diagnostic: {
        code: 'collinear_camera_up',
        message: 'stored editor camera up vector must not be collinear with its view direction',
      },
    };
  }
  const up = normalize(cross(right, forward));
  if (up === null) {
    return {
      ok: false,
      diagnostic: {
        code: 'collinear_camera_up',
        message: 'stored editor camera could not derive an orthonormal up vector',
      },
    };
  }

  return {
    ok: true,
    camera: {
      source: 'stored_editor',
      pose: {
        position: [...input.position],
        yawDegrees: Math.atan2(forward[0], -forward[2]) * RADIANS_TO_DEGREES,
        pitchDegrees: Math.asin(Math.max(-1, Math.min(1, forward[1]))) * RADIANS_TO_DEGREES,
      },
      basis: { forward, right, up },
      projection: { ...input.projection },
    },
  };
}

import type { CameraSnapshot, CollisionConstrainedCameraInputEnvelope } from '@asha/contracts';

function f32(value: number): number {
  return Math.fround(value);
}

function horizontalMovementBasis(yawDegrees: number): Pick<CameraSnapshot['basis'], 'forward' | 'right'> {
  const yaw = f32((yawDegrees * Math.PI) / 180);
  const sy = f32(Math.sin(yaw));
  const cy = f32(Math.cos(yaw));
  return {
    forward: [sy, 0, f32(-cy)],
    right: [cy, 0, sy],
  };
}

export function collisionCameraAttemptedPose(
  before: CameraSnapshot,
  envelope: CollisionConstrainedCameraInputEnvelope,
): CameraSnapshot['pose'] {
  const cameraInput = envelope.input;
  const yawDegrees = f32(before.pose.yawDegrees + cameraInput.yawDeltaDegrees);
  const pitchDegrees = Math.max(-89, Math.min(89, f32(before.pose.pitchDegrees + cameraInput.pitchDeltaDegrees)));
  const movementBasis = envelope.movementMode === 'grounded'
    ? horizontalMovementBasis(yawDegrees)
    : { forward: before.basis.forward, right: before.basis.right };
  const movementUp = envelope.movementMode === 'grounded' ? [0, 0, 0] as const : before.basis.up;
  const distance = f32(cameraInput.dtSeconds * cameraInput.moveSpeedUnitsPerSecond);
  const movement = (axis: 0 | 1 | 2): number => f32(
    f32(movementBasis.forward[axis] * cameraInput.moveForward) +
    f32(movementBasis.right[axis] * cameraInput.moveRight) +
    f32(movementUp[axis] * cameraInput.moveUp),
  );
  return {
    position: [
      f32(before.pose.position[0] + movement(0) * distance),
      f32(before.pose.position[1] + movement(1) * distance),
      f32(before.pose.position[2] + movement(2) * distance),
    ],
    yawDegrees,
    pitchDegrees,
  };
}

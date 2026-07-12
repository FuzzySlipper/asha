import type { CameraSnapshot, CameraTransitionReadout } from '@asha/contracts';
/**
 * Samples a disposable renderer pose between two authority-accepted snapshots.
 * The returned value is projection state only: callers must never feed it back
 * into RuntimeSession authority or replay evidence.
 */
export declare function sampleCameraTransition(transition: CameraTransitionReadout, elapsedMilliseconds: number): CameraSnapshot;
//# sourceMappingURL=camera-transition.d.ts.map
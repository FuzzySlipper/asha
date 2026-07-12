import type {
  CameraControllerState,
  CameraHandle,
  CameraModeChangeReceipt,
  CameraModeCommand,
  CameraNavigationInput,
  CameraNavigationReceipt,
  CameraPose,
  InputContextChangeReceipt,
  ResolvedInputAction,
} from '@asha/contracts';
import type { RuntimeSessionFacade } from '@asha/runtime-session';

export const CAMERA_INPUT_ACTIONS = {
  firstPerson: 'camera.mode.firstPerson',
  orbit: 'camera.mode.orbit',
  topDown: 'camera.mode.topDown',
  rotate: 'camera.navigation.rotate',
  zoom: 'camera.navigation.zoom',
  panForward: 'camera.navigation.panForward',
  panBackward: 'camera.navigation.panBackward',
  panLeft: 'camera.navigation.panLeft',
  panRight: 'camera.navigation.panRight',
} as const;

export interface ResolvedCameraNavigationOptions {
  readonly session: RuntimeSessionFacade;
  readonly camera: CameraHandle;
  readonly selectedPivot: () => readonly [number, number, number] | null;
  readonly nextTick: () => number;
  readonly contextId?: string;
  readonly transitionMilliseconds?: number;
  readonly rotateDegreesPerPixel?: number;
  readonly wheelUnitsPerPixel?: number;
  readonly panSpeedUnitsPerSecond?: number;
  readonly inputDeltaSeconds?: number;
}

export interface ResolvedCameraModeReceipt {
  readonly kind: 'mode';
  readonly actionId: string;
  readonly accepted: boolean;
  readonly context: InputContextChangeReceipt | null;
  readonly authority: CameraModeChangeReceipt | null;
  readonly rollback: InputContextChangeReceipt | null;
  readonly rejection: 'missingSelectedPivot' | null;
}

export interface ResolvedCameraNavigationReceipt {
  readonly kind: 'navigation';
  readonly actionId: string;
  readonly authority: CameraNavigationReceipt;
}

export type ResolvedCameraActionReceipt =
  | ResolvedCameraModeReceipt
  | ResolvedCameraNavigationReceipt;

function pressed(action: ResolvedInputAction): boolean {
  return action.phase === 'pressed'
    && action.value.kind === 'button'
    && action.value.pressed;
}

export class ResolvedCameraNavigationConsumer {
  readonly #session: RuntimeSessionFacade;
  readonly #camera: CameraHandle;
  readonly #selectedPivot: ResolvedCameraNavigationOptions['selectedPivot'];
  readonly #nextTick: ResolvedCameraNavigationOptions['nextTick'];
  readonly #contextId: string;
  readonly #transitionMilliseconds: number;
  readonly #rotateDegreesPerPixel: number;
  readonly #wheelUnitsPerPixel: number;
  readonly #panSpeedUnitsPerSecond: number;
  readonly #inputDeltaSeconds: number;
  #firstPersonPose: CameraPose | null = null;

  constructor(options: ResolvedCameraNavigationOptions) {
    this.#session = options.session;
    this.#camera = options.camera;
    this.#selectedPivot = options.selectedPivot;
    this.#nextTick = options.nextTick;
    this.#contextId = options.contextId ?? 'cameraNavigation';
    this.#transitionMilliseconds = options.transitionMilliseconds ?? 250;
    this.#rotateDegreesPerPixel = options.rotateDegreesPerPixel ?? 0.2;
    this.#wheelUnitsPerPixel = options.wheelUnitsPerPixel ?? 0.01;
    this.#panSpeedUnitsPerSecond = options.panSpeedUnitsPerSecond ?? 8;
    this.#inputDeltaSeconds = options.inputDeltaSeconds ?? 1 / 60;
  }

  consume(action: ResolvedInputAction): ResolvedCameraActionReceipt | null {
    if (action.actionId === CAMERA_INPUT_ACTIONS.firstPerson && pressed(action)) {
      return this.#returnToFirstPerson(action.actionId);
    }
    if (action.actionId === CAMERA_INPUT_ACTIONS.orbit && pressed(action)) {
      return this.#enterPivotMode(action.actionId, 'orbit');
    }
    if (action.actionId === CAMERA_INPUT_ACTIONS.topDown && pressed(action)) {
      return this.#enterPivotMode(action.actionId, 'topDown');
    }
    const navigation = this.#navigationInput(action);
    if (navigation === null) return null;
    const state = this.#read();
    const authority = this.#session.applyCameraNavigationInput({
      camera: this.#camera,
      expectedRevision: state.revision,
      input: navigation,
      tick: this.#nextTick(),
    });
    return { kind: 'navigation', actionId: action.actionId, authority };
  }

  #read(): CameraControllerState {
    return this.#session.readCameraControllerState({ camera: this.#camera });
  }

  #enterPivotMode(actionId: string, mode: 'orbit' | 'topDown'): ResolvedCameraModeReceipt {
    const pivot = this.#selectedPivot();
    if (pivot === null) {
      return {
        kind: 'mode', actionId, accepted: false, context: null,
        authority: null, rollback: null, rejection: 'missingSelectedPivot',
      };
    }
    const before = this.#read();
    const context = before.mode === 'firstPerson'
      ? this.#session.applyInputContextCommand({ operation: 'push', contextId: this.#contextId })
      : null;
    if (context !== null && !context.accepted) {
      return { kind: 'mode', actionId, accepted: false, context, authority: null, rollback: null, rejection: null };
    }
    if (before.mode === 'firstPerson') this.#firstPersonPose = before.snapshot.pose;
    const command: CameraModeCommand = {
      camera: this.#camera,
      expectedRevision: before.revision,
      target: mode === 'orbit'
        ? {
            mode,
            pivot,
            distance: 8,
            minDistance: 1.5,
            maxDistance: 40,
            yawDegrees: before.snapshot.pose.yawDegrees,
            pitchDegrees: Math.max(-80, Math.min(60, before.snapshot.pose.pitchDegrees)),
          }
        : {
            mode,
            pivot,
            height: 14,
            minHeight: 3,
            maxHeight: 60,
            yawDegrees: before.snapshot.pose.yawDegrees,
            pitchDegrees: -75,
          },
      transition: { durationMilliseconds: this.#transitionMilliseconds, easing: 'smoothStep' },
      tick: this.#nextTick(),
    };
    const authority = this.#session.applyCameraModeCommand(command);
    const rollback = authority.accepted || context === null
      ? null
      : this.#session.applyInputContextCommand({ operation: 'pop', expectedContextId: this.#contextId });
    return {
      kind: 'mode', actionId, accepted: authority.accepted,
      context, authority, rollback, rejection: null,
    };
  }

  #returnToFirstPerson(actionId: string): ResolvedCameraModeReceipt {
    const before = this.#read();
    if (before.mode === 'firstPerson') {
      const authority = this.#session.applyCameraModeCommand({
        camera: this.#camera,
        expectedRevision: before.revision,
        target: { mode: 'firstPerson', pose: before.snapshot.pose },
        transition: null,
        tick: this.#nextTick(),
      });
      return { kind: 'mode', actionId, accepted: authority.accepted, context: null, authority, rollback: null, rejection: null };
    }
    const context = this.#session.applyInputContextCommand({
      operation: 'pop', expectedContextId: this.#contextId,
    });
    if (!context.accepted) {
      return { kind: 'mode', actionId, accepted: false, context, authority: null, rollback: null, rejection: null };
    }
    const authority = this.#session.applyCameraModeCommand({
      camera: this.#camera,
      expectedRevision: before.revision,
      target: { mode: 'firstPerson', pose: this.#firstPersonPose ?? before.snapshot.pose },
      transition: { durationMilliseconds: this.#transitionMilliseconds, easing: 'smoothStep' },
      tick: this.#nextTick(),
    });
    const rollback = authority.accepted
      ? null
      : this.#session.applyInputContextCommand({ operation: 'push', contextId: this.#contextId });
    return {
      kind: 'mode', actionId, accepted: authority.accepted,
      context, authority, rollback, rejection: null,
    };
  }

  #navigationInput(action: ResolvedInputAction): CameraNavigationInput | null {
    let panRight = 0;
    let panForward = 0;
    let yawDeltaDegrees = 0;
    let pitchDeltaDegrees = 0;
    let zoomDelta = 0;
    if (action.actionId === CAMERA_INPUT_ACTIONS.rotate && action.value.kind === 'axis2d') {
      yawDeltaDegrees = action.value.x * this.#rotateDegreesPerPixel;
      pitchDeltaDegrees = action.value.y * this.#rotateDegreesPerPixel;
    } else if (action.actionId === CAMERA_INPUT_ACTIONS.zoom && action.value.kind === 'axis1d') {
      zoomDelta = action.value.value * this.#wheelUnitsPerPixel;
    } else if (action.value.kind === 'button' && action.value.pressed) {
      if (action.actionId === CAMERA_INPUT_ACTIONS.panForward) panForward = 1;
      else if (action.actionId === CAMERA_INPUT_ACTIONS.panBackward) panForward = -1;
      else if (action.actionId === CAMERA_INPUT_ACTIONS.panLeft) panRight = -1;
      else if (action.actionId === CAMERA_INPUT_ACTIONS.panRight) panRight = 1;
      else return null;
    } else {
      return null;
    }
    return {
      panRight,
      panForward,
      yawDeltaDegrees,
      pitchDeltaDegrees,
      zoomDelta,
      dtSeconds: this.#inputDeltaSeconds,
      panSpeedUnitsPerSecond: this.#panSpeedUnitsPerSecond,
    };
  }
}

import type { CameraHandle, FirstPersonCameraInputEnvelope } from '@asha/contracts';
import type { RuntimeActionIntentEnvelope } from '@asha/runtime-session';
export type BrowserFpsKeyCode = 'KeyW' | 'KeyA' | 'KeyS' | 'KeyD' | 'Escape';
export interface BrowserFpsKeyboardInput {
    readonly code: string;
    readonly repeat?: boolean;
    preventDefault?(): void;
}
export interface BrowserFpsMouseMoveInput {
    readonly movementX: number;
    readonly movementY: number;
}
export interface BrowserFpsPointerButtonInput {
    readonly button: number;
    preventDefault?(): void;
}
export type BrowserFpsPointerLockIntent = {
    readonly kind: 'request_pointer_lock';
    readonly reason: 'primary_button' | 'programmatic';
} | {
    readonly kind: 'release_pointer_lock';
    readonly reason: 'escape_key' | 'programmatic';
};
export interface BrowserFpsUnsupportedIntent {
    readonly kind: 'unsupported_primary_fire';
    readonly pressed: boolean;
    readonly triggered: boolean;
    readonly reason: 'no_public_runtime_action_protocol';
}
export type BrowserFpsInputShellMode = 'active' | 'disabled' | 'paused';
export interface BrowserFpsInputShellState {
    readonly mode: BrowserFpsInputShellMode;
}
export interface BrowserFpsInputShellReadout {
    readonly acceptsInput: boolean;
    readonly blockedReason: BrowserFpsInputShellMode | null;
    readonly mode: BrowserFpsInputShellMode;
}
export interface BrowserFpsMovementInput {
    readonly dtSeconds: number;
    readonly moveForward: number;
    readonly moveRight: number;
    readonly moveSpeedUnitsPerSecond: number;
    readonly moveUp: number;
    readonly pitchDeltaDegrees: number;
    readonly yawDeltaDegrees: number;
}
export interface BrowserFpsInputReadout {
    readonly shell: BrowserFpsInputShellReadout;
    readonly pointerLocked: boolean;
    readonly releaseRequestedByEscape: boolean;
    readonly pressedKeys: readonly BrowserFpsKeyCode[];
    readonly moveForward: number;
    readonly moveRight: number;
    readonly pendingMouseDelta: readonly [number, number];
    readonly primaryFirePressed: boolean;
    readonly primaryFireTriggered: boolean;
}
export interface BrowserFpsInputFrame {
    readonly tick: number;
    readonly input: BrowserFpsMovementInput;
    readonly pointerLockIntents: readonly BrowserFpsPointerLockIntent[];
    readonly readout: BrowserFpsInputReadout;
}
export type BrowserFpsRuntimeCommand = {
    readonly kind: 'runtime.apply_first_person_camera_input';
    readonly envelope: FirstPersonCameraInputEnvelope;
};
export type BrowserFpsRuntimeActionCommand = {
    readonly kind: 'runtime.propose_runtime_action_intent';
    readonly envelope: RuntimeActionIntentEnvelope;
};
export interface BrowserFpsCommandFrame {
    readonly tick: number;
    readonly input: BrowserFpsMovementInput;
    readonly runtimeCommand: BrowserFpsRuntimeCommand;
    readonly runtimeActionIntents: readonly BrowserFpsRuntimeActionCommand[];
    readonly pointerLockIntents: readonly BrowserFpsPointerLockIntent[];
    readonly unsupportedIntents: readonly BrowserFpsUnsupportedIntent[];
    readonly readout: BrowserFpsInputReadout;
}
export interface BrowserFpsInputCollectorOptions {
    readonly camera?: CameraHandle;
    readonly shellState?: BrowserFpsInputShellState;
    readonly moveSpeedUnitsPerSecond: number;
    readonly mouseSensitivityDegreesPerPixel: number;
    readonly pointerLocked?: boolean;
}
export interface BrowserFpsDrainInput {
    readonly tick: number;
    readonly dtSeconds: number;
}
export declare class BrowserFpsInputCollector {
    #private;
    constructor(options: BrowserFpsInputCollectorOptions);
    setShellState(state: BrowserFpsInputShellState): BrowserFpsInputReadout;
    setPointerLockActive(active: boolean): BrowserFpsInputReadout;
    requestPointerLock(): readonly BrowserFpsPointerLockIntent[];
    releasePointerLock(): readonly BrowserFpsPointerLockIntent[];
    handleKeyDown(event: BrowserFpsKeyboardInput): readonly BrowserFpsPointerLockIntent[];
    handleKeyUp(event: BrowserFpsKeyboardInput): void;
    handleMouseMove(event: BrowserFpsMouseMoveInput): void;
    handlePointerDown(event: BrowserFpsPointerButtonInput): readonly BrowserFpsPointerLockIntent[];
    handlePointerUp(event: BrowserFpsPointerButtonInput): void;
    reset(): BrowserFpsInputReadout;
    drainInputFrame(input: BrowserFpsDrainInput): BrowserFpsInputFrame;
    drainFrame(input: BrowserFpsDrainInput): BrowserFpsCommandFrame;
    readout(): BrowserFpsInputReadout;
}
//# sourceMappingURL=browser-fps-input.d.ts.map
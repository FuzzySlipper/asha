import type { CameraHandle, CameraModeChangeReceipt, CameraNavigationReceipt, InputContextChangeReceipt, ResolvedInputAction } from '@asha/contracts';
import type { RuntimeSessionFacade } from '@asha/runtime-session';
export declare const CAMERA_INPUT_ACTIONS: {
    readonly firstPerson: "camera.mode.firstPerson";
    readonly orbit: "camera.mode.orbit";
    readonly topDown: "camera.mode.topDown";
    readonly rotate: "camera.navigation.rotate";
    readonly zoom: "camera.navigation.zoom";
    readonly panForward: "camera.navigation.panForward";
    readonly panBackward: "camera.navigation.panBackward";
    readonly panLeft: "camera.navigation.panLeft";
    readonly panRight: "camera.navigation.panRight";
};
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
export type ResolvedCameraActionReceipt = ResolvedCameraModeReceipt | ResolvedCameraNavigationReceipt;
export declare class ResolvedCameraNavigationConsumer {
    #private;
    constructor(options: ResolvedCameraNavigationOptions);
    consume(action: ResolvedInputAction): ResolvedCameraActionReceipt | null;
}
//# sourceMappingURL=resolved-camera-navigation.d.ts.map
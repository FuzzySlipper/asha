import type { CameraControllerState, CameraModeChangeReceipt, CameraModeCommand, CameraNavigationInputEnvelope, CameraNavigationReceipt, CameraSnapshot } from '@asha/contracts';
export declare class MockCameraControllers {
    #private;
    clear(): void;
    create(snapshot: CameraSnapshot): void;
    read(camera: number): CameraControllerState | undefined;
    isFirstPerson(camera: number): boolean;
    syncFirstPerson(snapshot: CameraSnapshot): void;
    applyMode(command: CameraModeCommand): CameraModeChangeReceipt | undefined;
    applyNavigation(input: CameraNavigationInputEnvelope): CameraNavigationReceipt | undefined;
}
//# sourceMappingURL=mock-camera-controller.d.ts.map
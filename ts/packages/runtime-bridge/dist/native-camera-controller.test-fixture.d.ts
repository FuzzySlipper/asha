import type { NativeAddon } from '@asha/native-bridge';
export declare const CAMERA_MODE_COMMAND: {
    camera: import("@asha/contracts").CameraHandle;
    expectedRevision: number;
    target: {
        mode: "orbit";
        pivot: [number, number, number];
        distance: number;
        minDistance: number;
        maxDistance: number;
        yawDegrees: number;
        pitchDegrees: number;
    };
    transition: {
        durationMilliseconds: number;
        easing: "smoothStep";
    };
    tick: number;
};
export declare const CAMERA_NAVIGATION_INPUT: {
    camera: import("@asha/contracts").CameraHandle;
    expectedRevision: number;
    tick: number;
    input: {
        panRight: number;
        panForward: number;
        yawDeltaDegrees: number;
        pitchDeltaDegrees: number;
        zoomDelta: number;
        dtSeconds: number;
        panSpeedUnitsPerSecond: number;
    };
};
export declare function createNativeCameraControllerHandlers(calls: string[], hashA: string, hashB: string, hashC: string): Pick<NativeAddon, 'applyCameraModeCommand' | 'applyCameraNavigationInput' | 'readCameraControllerState'>;
//# sourceMappingURL=native-camera-controller.test-fixture.d.ts.map
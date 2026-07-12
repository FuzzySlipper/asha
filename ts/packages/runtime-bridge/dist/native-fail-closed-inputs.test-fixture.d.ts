import type { CollisionConstrainedCameraInputEnvelope, InputContextCommand, InputSessionConfigureRequest, ModelMaterialPreviewRequest, RawInputSample, RecordedInputAction } from '@asha/contracts';
import type { NativeAddon } from '@asha/native-bridge';
export declare const INPUT_SESSION_CONFIGURE_REQUEST: InputSessionConfigureRequest;
export declare const INPUT_CONTEXT_COMMAND: InputContextCommand;
export declare const RAW_INPUT_SAMPLE: RawInputSample;
export declare const RECORDED_INPUT_ACTION: RecordedInputAction;
export declare function createNativeInputHandlers(hashA: string, hashB: string, hashC: string): Partial<NativeAddon>;
export declare const MODEL_MATERIAL_PREVIEW_REQUEST: ModelMaterialPreviewRequest;
export declare const CAMERA_CREATE_REQUEST: {
    readonly initialPose: {
        readonly position: readonly [0, 1.6, 0];
        readonly yawDegrees: 0;
        readonly pitchDegrees: 0;
    };
    readonly projection: {
        readonly fovYDegrees: 60;
        readonly near: 0.1;
        readonly far: 1000;
    };
    readonly viewport: {
        readonly width: 1280;
        readonly height: 720;
    };
};
export declare const CAMERA_INPUT: {
    readonly camera: import("@asha/contracts").CameraHandle;
    readonly tick: 1;
    readonly input: {
        readonly moveForward: 1;
        readonly moveRight: 0;
        readonly moveUp: 0;
        readonly yawDeltaDegrees: 15;
        readonly pitchDeltaDegrees: -5;
        readonly dtSeconds: number;
        readonly moveSpeedUnitsPerSecond: 3;
    };
};
export declare const COLLISION_CAMERA_INPUT: CollisionConstrainedCameraInputEnvelope;
//# sourceMappingURL=native-fail-closed-inputs.test-fixture.d.ts.map
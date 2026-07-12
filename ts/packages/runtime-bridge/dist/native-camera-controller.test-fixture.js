import { CAMERA_CREATE_REQUEST, CAMERA_INPUT } from './native-fail-closed-inputs.test-fixture.js';
export const CAMERA_MODE_COMMAND = {
    camera: CAMERA_INPUT.camera,
    expectedRevision: 0,
    target: {
        mode: 'orbit',
        pivot: [0, 1, -4],
        distance: 6,
        minDistance: 2,
        maxDistance: 20,
        yawDegrees: 20,
        pitchDegrees: -30,
    },
    transition: { durationMilliseconds: 250, easing: 'smoothStep' },
    tick: 2,
};
export const CAMERA_NAVIGATION_INPUT = {
    camera: CAMERA_INPUT.camera,
    expectedRevision: 1,
    tick: 3,
    input: {
        panRight: 0.5,
        panForward: 0,
        yawDeltaDegrees: 5,
        pitchDeltaDegrees: 0,
        zoomDelta: 1,
        dtSeconds: 0.25,
        panSpeedUnitsPerSecond: 4,
    },
};
export function createNativeCameraControllerHandlers(calls, hashA, hashB, hashC) {
    const firstPersonController = () => ({
        schemaVersion: 1,
        revision: 0,
        camera: CAMERA_INPUT.camera,
        mode: 'firstPerson',
        pivot: null,
        distance: null,
        minDistance: null,
        maxDistance: null,
        snapshot: {
            camera: CAMERA_INPUT.camera,
            tick: 0,
            pose: CAMERA_CREATE_REQUEST.initialPose,
            basis: { forward: [0, 0, -1], right: [1, 0, 0], up: [0, 1, 0] },
            projection: CAMERA_CREATE_REQUEST.projection,
            viewport: CAMERA_CREATE_REQUEST.viewport,
        },
        stateHash: hashA,
    });
    const orbitController = () => ({
        ...firstPersonController(),
        revision: 1,
        mode: 'orbit',
        pivot: CAMERA_MODE_COMMAND.target.mode === 'orbit' ? CAMERA_MODE_COMMAND.target.pivot : null,
        distance: 6,
        minDistance: 2,
        maxDistance: 20,
        stateHash: hashB,
    });
    return {
        applyCameraModeCommand: (_handle, commandJson) => {
            calls.push(`cameraMode:${commandJson}`);
            return JSON.stringify({
                accepted: true,
                before: firstPersonController(),
                after: orbitController(),
                transition: null,
                terrainConstrained: false,
                rejection: null,
                receiptHash: hashC,
            });
        },
        applyCameraNavigationInput: (_handle, inputJson) => {
            calls.push(`cameraNavigation:${inputJson}`);
            return JSON.stringify({
                accepted: true,
                before: orbitController(),
                after: { ...orbitController(), revision: 2, distance: 5, stateHash: hashC },
                terrainConstrained: false,
                rejection: null,
                receiptHash: hashA,
            });
        },
        readCameraControllerState: (_handle, requestJson) => {
            calls.push(`cameraControllerRead:${requestJson}`);
            return JSON.stringify(orbitController());
        },
    };
}
//# sourceMappingURL=native-camera-controller.test-fixture.js.map
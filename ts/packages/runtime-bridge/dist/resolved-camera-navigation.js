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
};
function pressed(action) {
    return action.phase === 'pressed'
        && action.value.kind === 'button'
        && action.value.pressed;
}
export class ResolvedCameraNavigationConsumer {
    #session;
    #camera;
    #selectedPivot;
    #nextTick;
    #contextId;
    #transitionMilliseconds;
    #rotateDegreesPerPixel;
    #wheelUnitsPerPixel;
    #panSpeedUnitsPerSecond;
    #inputDeltaSeconds;
    #firstPersonPose = null;
    constructor(options) {
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
    consume(action) {
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
        if (navigation === null)
            return null;
        const state = this.#read();
        const authority = this.#session.applyCameraNavigationInput({
            camera: this.#camera,
            expectedRevision: state.revision,
            input: navigation,
            tick: this.#nextTick(),
        });
        return { kind: 'navigation', actionId: action.actionId, authority };
    }
    #read() {
        return this.#session.readCameraControllerState({ camera: this.#camera });
    }
    #enterPivotMode(actionId, mode) {
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
        if (before.mode === 'firstPerson')
            this.#firstPersonPose = before.snapshot.pose;
        const command = {
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
    #returnToFirstPerson(actionId) {
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
    #navigationInput(action) {
        let panRight = 0;
        let panForward = 0;
        let yawDeltaDegrees = 0;
        let pitchDeltaDegrees = 0;
        let zoomDelta = 0;
        if (action.actionId === CAMERA_INPUT_ACTIONS.rotate && action.value.kind === 'axis2d') {
            yawDeltaDegrees = action.value.x * this.#rotateDegreesPerPixel;
            pitchDeltaDegrees = action.value.y * this.#rotateDegreesPerPixel;
        }
        else if (action.actionId === CAMERA_INPUT_ACTIONS.zoom && action.value.kind === 'axis1d') {
            zoomDelta = action.value.value * this.#wheelUnitsPerPixel;
        }
        else if (action.value.kind === 'button' && action.value.pressed) {
            if (action.actionId === CAMERA_INPUT_ACTIONS.panForward)
                panForward = 1;
            else if (action.actionId === CAMERA_INPUT_ACTIONS.panBackward)
                panForward = -1;
            else if (action.actionId === CAMERA_INPUT_ACTIONS.panLeft)
                panRight = -1;
            else if (action.actionId === CAMERA_INPUT_ACTIONS.panRight)
                panRight = 1;
            else
                return null;
        }
        else {
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
//# sourceMappingURL=resolved-camera-navigation.js.map
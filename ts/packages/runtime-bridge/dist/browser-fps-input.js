export class BrowserFpsInputCollector {
    #camera;
    #moveSpeedUnitsPerSecond;
    #mouseSensitivityDegreesPerPixel;
    #keys = new Set();
    #pointerLockIntents = [];
    #shellMode;
    #pointerLocked;
    #releaseRequestedByEscape = false;
    #mouseX = 0;
    #mouseY = 0;
    #primaryFirePressed = false;
    #primaryFireTriggered = false;
    #primaryFireReleased = false;
    constructor(options) {
        if (options.moveSpeedUnitsPerSecond < 0 || !Number.isFinite(options.moveSpeedUnitsPerSecond)) {
            throw new Error('moveSpeedUnitsPerSecond must be a finite non-negative number');
        }
        if (!Number.isFinite(options.mouseSensitivityDegreesPerPixel)) {
            throw new Error('mouseSensitivityDegreesPerPixel must be finite');
        }
        this.#camera = options.camera ?? null;
        this.#shellMode = options.shellState?.mode ?? 'active';
        this.#moveSpeedUnitsPerSecond = options.moveSpeedUnitsPerSecond;
        this.#mouseSensitivityDegreesPerPixel = options.mouseSensitivityDegreesPerPixel;
        this.#pointerLocked = options.pointerLocked ?? false;
    }
    setShellState(state) {
        if (this.#shellMode !== state.mode) {
            this.#shellMode = state.mode;
            this.#clearTransientInput();
        }
        return this.readout();
    }
    setPointerLockActive(active) {
        this.#pointerLocked = active;
        if (active) {
            this.#releaseRequestedByEscape = false;
        }
        return this.readout();
    }
    requestPointerLock() {
        if (!this.#acceptsInput()) {
            return [];
        }
        const intent = { kind: 'request_pointer_lock', reason: 'programmatic' };
        this.#pointerLockIntents.push(intent);
        return [intent];
    }
    releasePointerLock() {
        const intent = { kind: 'release_pointer_lock', reason: 'programmatic' };
        this.#pointerLockIntents.push(intent);
        return [intent];
    }
    handleKeyDown(event) {
        const key = fpsKeyCode(event.code);
        if (key === null) {
            return [];
        }
        event.preventDefault?.();
        if (!this.#acceptsInput()) {
            return [];
        }
        if (key === 'Escape') {
            this.#releaseRequestedByEscape = true;
            if (!this.#pointerLocked) {
                return [];
            }
            const intent = { kind: 'release_pointer_lock', reason: 'escape_key' };
            this.#pointerLockIntents.push(intent);
            return [intent];
        }
        this.#keys.add(key);
        return [];
    }
    handleKeyUp(event) {
        const key = fpsKeyCode(event.code);
        if (key === null || key === 'Escape') {
            return;
        }
        event.preventDefault?.();
        if (!this.#acceptsInput()) {
            return;
        }
        this.#keys.delete(key);
    }
    handleMouseMove(event) {
        if (!this.#acceptsInput() || !this.#pointerLocked) {
            return;
        }
        if (!Number.isFinite(event.movementX) || !Number.isFinite(event.movementY)) {
            return;
        }
        this.#mouseX += event.movementX;
        this.#mouseY += event.movementY;
    }
    handlePointerDown(event) {
        event.preventDefault?.();
        if (event.button !== 0) {
            return [];
        }
        if (!this.#acceptsInput()) {
            return [];
        }
        this.#primaryFirePressed = true;
        this.#primaryFireTriggered = true;
        if (this.#pointerLocked) {
            return [];
        }
        const intent = { kind: 'request_pointer_lock', reason: 'primary_button' };
        this.#pointerLockIntents.push(intent);
        return [intent];
    }
    handlePointerUp(event) {
        if (event.button !== 0) {
            return;
        }
        event.preventDefault?.();
        if (!this.#acceptsInput()) {
            return;
        }
        const wasPressed = this.#primaryFirePressed;
        this.#primaryFirePressed = false;
        if (wasPressed) {
            this.#primaryFireReleased = true;
        }
    }
    reset() {
        this.#keys.clear();
        this.#pointerLockIntents.length = 0;
        this.#releaseRequestedByEscape = false;
        this.#clearTransientInput();
        return this.readout();
    }
    drainInputFrame(input) {
        validateDrainInput(input);
        const frame = this.#buildInputFrame(input);
        this.#resetDrainedFrameState();
        this.#primaryFireTriggered = false;
        this.#primaryFireReleased = false;
        return frame;
    }
    drainFrame(input) {
        validateDrainInput(input);
        if (this.#camera === null) {
            throw new Error('camera is required to drain a RuntimeSession browser FPS command frame');
        }
        const inputFrame = this.#buildInputFrame(input);
        const runtimeCommand = {
            kind: 'runtime.apply_first_person_camera_input',
            envelope: {
                camera: this.#camera,
                tick: input.tick,
                input: inputFrame.input,
            },
        };
        const runtimeActionIntents = this.#drainRuntimeActionIntents(input.tick);
        const frame = {
            tick: input.tick,
            input: inputFrame.input,
            runtimeCommand,
            runtimeActionIntents,
            pointerLockIntents: inputFrame.pointerLockIntents,
            unsupportedIntents: [],
            readout: inputFrame.readout,
        };
        this.#resetDrainedFrameState();
        this.#primaryFireTriggered = false;
        this.#primaryFireReleased = false;
        return frame;
    }
    readout() {
        const shell = this.#shellReadout();
        return {
            shell,
            pointerLocked: this.#pointerLocked,
            releaseRequestedByEscape: this.#releaseRequestedByEscape,
            pressedKeys: [...this.#keys].sort(),
            moveForward: directional(this.#keys.has('KeyW'), this.#keys.has('KeyS')),
            moveRight: directional(this.#keys.has('KeyD'), this.#keys.has('KeyA')),
            pendingMouseDelta: [this.#mouseX, this.#mouseY],
            primaryFirePressed: this.#primaryFirePressed,
            primaryFireTriggered: this.#primaryFireTriggered,
        };
    }
    #drainRuntimeActionIntents(tick) {
        const intents = [];
        if (!this.#acceptsInput() || this.#camera === null) {
            return intents;
        }
        if (this.#primaryFireTriggered) {
            intents.push({
                kind: 'runtime.propose_runtime_action_intent',
                envelope: {
                    kind: 'runtime_action_intent.v0',
                    action: 'primary_fire',
                    phase: 'pressed',
                    camera: this.#camera,
                    tick,
                    source: 'browser_fps_pointer',
                    pressed: true,
                },
            });
        }
        if (this.#primaryFireReleased) {
            intents.push({
                kind: 'runtime.propose_runtime_action_intent',
                envelope: {
                    kind: 'runtime_action_intent.v0',
                    action: 'primary_fire',
                    phase: 'released',
                    camera: this.#camera,
                    tick,
                    source: 'browser_fps_pointer',
                    pressed: false,
                },
            });
        }
        return intents;
    }
    #movementInput(input) {
        if (!this.#acceptsInput()) {
            return {
                dtSeconds: input.dtSeconds,
                moveForward: 0,
                moveRight: 0,
                moveSpeedUnitsPerSecond: this.#moveSpeedUnitsPerSecond,
                moveUp: 0,
                pitchDeltaDegrees: 0,
                yawDeltaDegrees: 0,
            };
        }
        return {
            dtSeconds: input.dtSeconds,
            moveForward: directional(this.#keys.has('KeyW'), this.#keys.has('KeyS')),
            moveRight: directional(this.#keys.has('KeyD'), this.#keys.has('KeyA')),
            moveSpeedUnitsPerSecond: this.#moveSpeedUnitsPerSecond,
            moveUp: 0,
            pitchDeltaDegrees: -this.#mouseY * this.#mouseSensitivityDegreesPerPixel,
            yawDeltaDegrees: this.#mouseX * this.#mouseSensitivityDegreesPerPixel,
        };
    }
    #buildInputFrame(input) {
        return {
            tick: input.tick,
            input: this.#movementInput(input),
            pointerLockIntents: [...this.#pointerLockIntents],
            readout: this.readout(),
        };
    }
    #shellReadout() {
        const acceptsInput = this.#acceptsInput();
        return {
            acceptsInput,
            blockedReason: acceptsInput ? null : this.#shellMode,
            mode: this.#shellMode,
        };
    }
    #acceptsInput() {
        return this.#shellMode === 'active';
    }
    #clearTransientInput() {
        this.#keys.clear();
        this.#mouseX = 0;
        this.#mouseY = 0;
        this.#primaryFirePressed = false;
        this.#primaryFireTriggered = false;
        this.#primaryFireReleased = false;
    }
    #resetDrainedFrameState() {
        this.#pointerLockIntents.length = 0;
        this.#mouseX = 0;
        this.#mouseY = 0;
    }
}
function fpsKeyCode(code) {
    switch (code) {
        case 'KeyW':
        case 'KeyA':
        case 'KeyS':
        case 'KeyD':
        case 'Escape':
            return code;
        default:
            return null;
    }
}
function directional(positive, negative) {
    if (positive === negative) {
        return 0;
    }
    return positive ? 1 : -1;
}
function validateDrainInput(input) {
    if (!Number.isSafeInteger(input.tick) || input.tick < 0) {
        throw new Error('tick must be a non-negative safe integer');
    }
    if (!Number.isFinite(input.dtSeconds) || input.dtSeconds < 0) {
        throw new Error('dtSeconds must be a finite non-negative number');
    }
}
//# sourceMappingURL=browser-fps-input.js.map
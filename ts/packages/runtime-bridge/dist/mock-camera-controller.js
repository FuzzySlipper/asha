import { CAMERA_CONTROLLER_STATE_SCHEMA_VERSION } from '@asha/contracts';
import { fnv1a64 } from './mock-primitives.js';
function hash(value) {
    return `fnv1a64:${fnv1a64(JSON.stringify(value))}`;
}
function basisFromPose(pose) {
    const yaw = (pose.yawDegrees * Math.PI) / 180;
    const pitch = (pose.pitchDegrees * Math.PI) / 180;
    const cosPitch = Math.cos(pitch);
    const forward = [
        Math.sin(yaw) * cosPitch,
        Math.sin(pitch),
        -Math.cos(yaw) * cosPitch,
    ];
    const right = [Math.cos(yaw), 0, Math.sin(yaw)];
    const up = [
        -Math.sin(yaw) * Math.sin(pitch),
        Math.cos(pitch),
        Math.cos(yaw) * Math.sin(pitch),
    ];
    return { forward, right, up };
}
function state(revision, mode, pivot, distance, minDistance, maxDistance, snapshot) {
    const stateHash = hash({ revision, mode, pivot, distance, minDistance, maxDistance, snapshot });
    return {
        schemaVersion: CAMERA_CONTROLLER_STATE_SCHEMA_VERSION,
        revision,
        camera: snapshot.camera,
        mode,
        pivot,
        distance,
        minDistance,
        maxDistance,
        snapshot,
        stateHash,
    };
}
function rejectedMode(before, rejection) {
    return {
        accepted: false,
        before,
        after: before,
        transition: null,
        terrainConstrained: false,
        rejection,
        receiptHash: hash({ accepted: false, stateHash: before.stateHash, rejection }),
    };
}
function rejectedNavigation(before, rejection) {
    return {
        accepted: false,
        before,
        after: before,
        terrainConstrained: false,
        rejection,
        receiptHash: hash({ accepted: false, stateHash: before.stateHash, rejection }),
    };
}
function finitePose(pose) {
    return pose.position.every(Number.isFinite)
        && Number.isFinite(pose.yawDegrees)
        && Number.isFinite(pose.pitchDegrees)
        && pose.pitchDegrees >= -89
        && pose.pitchDegrees <= 89;
}
function validMetric(pivot, value, minimum, maximum) {
    return pivot.every(Number.isFinite)
        && [value, minimum, maximum].every(Number.isFinite)
        && minimum > 0
        && minimum <= value
        && value <= maximum
        && maximum <= 10_000;
}
function orbitPose(pivot, distance, yawDegrees, pitchDegrees) {
    const basis = basisFromPose({ position: pivot, yawDegrees, pitchDegrees });
    return {
        position: [
            pivot[0] - basis.forward[0] * distance,
            pivot[1] - basis.forward[1] * distance,
            pivot[2] - basis.forward[2] * distance,
        ],
        yawDegrees,
        pitchDegrees,
    };
}
function topDownPose(pivot, height, yawDegrees, pitchDegrees) {
    const basis = basisFromPose({ position: pivot, yawDegrees, pitchDegrees });
    return orbitPose(pivot, height / Math.max(-basis.forward[1], 0.001), yawDegrees, pitchDegrees);
}
export class MockCameraControllers {
    #states = new Map();
    clear() {
        this.#states.clear();
    }
    create(snapshot) {
        this.#states.set(snapshot.camera, state(0, 'firstPerson', null, null, null, null, snapshot));
    }
    read(camera) {
        return this.#states.get(camera);
    }
    isFirstPerson(camera) {
        return this.#states.get(camera)?.mode === 'firstPerson';
    }
    syncFirstPerson(snapshot) {
        const before = this.#states.get(snapshot.camera);
        if (!before || before.mode !== 'firstPerson')
            return;
        this.#states.set(snapshot.camera, state(before.revision + 1, 'firstPerson', null, null, null, null, snapshot));
    }
    applyMode(command) {
        const before = this.#states.get(command.camera);
        if (!before)
            return undefined;
        if (command.expectedRevision !== before.revision)
            return rejectedMode(before, 'staleRevision');
        if (command.transition !== null
            && (command.transition.durationMilliseconds <= 0
                || command.transition.durationMilliseconds > 10_000)) {
            return rejectedMode(before, 'invalidInput');
        }
        const target = command.target;
        let mode;
        let pivot;
        let distance;
        let minDistance;
        let maxDistance;
        let pose;
        if (target.mode === 'firstPerson') {
            if (!finitePose(target.pose))
                return rejectedMode(before, 'invalidTarget');
            mode = 'firstPerson';
            pivot = null;
            distance = null;
            minDistance = null;
            maxDistance = null;
            pose = target.pose;
        }
        else if (target.mode === 'orbit') {
            if (!validMetric(target.pivot, target.distance, target.minDistance, target.maxDistance)
                || !Number.isFinite(target.yawDegrees)
                || !Number.isFinite(target.pitchDegrees)
                || target.pitchDegrees < -89
                || target.pitchDegrees > 89)
                return rejectedMode(before, 'invalidTarget');
            mode = 'orbit';
            pivot = target.pivot;
            distance = target.distance;
            minDistance = target.minDistance;
            maxDistance = target.maxDistance;
            pose = orbitPose(target.pivot, target.distance, target.yawDegrees, target.pitchDegrees);
        }
        else {
            if (!validMetric(target.pivot, target.height, target.minHeight, target.maxHeight)
                || !Number.isFinite(target.yawDegrees)
                || !Number.isFinite(target.pitchDegrees)
                || target.pitchDegrees < -89
                || target.pitchDegrees > -30)
                return rejectedMode(before, 'invalidTarget');
            mode = 'topDown';
            pivot = target.pivot;
            distance = target.height;
            minDistance = target.minHeight;
            maxDistance = target.maxHeight;
            pose = topDownPose(target.pivot, target.height, target.yawDegrees, target.pitchDegrees);
        }
        const snapshot = { ...before.snapshot, tick: command.tick, pose, basis: basisFromPose(pose) };
        const after = state(before.revision + 1, mode, pivot, distance, minDistance, maxDistance, snapshot);
        const transition = command.transition === null ? null : {
            from: before.snapshot,
            to: after.snapshot,
            durationMilliseconds: command.transition.durationMilliseconds,
            easing: command.transition.easing,
            transitionHash: hash({ from: before.stateHash, to: after.stateHash, transition: command.transition }),
        };
        const receipt = {
            accepted: true,
            before,
            after,
            transition,
            terrainConstrained: false,
            rejection: null,
            receiptHash: hash({ accepted: true, before: before.stateHash, after: after.stateHash, transition }),
        };
        this.#states.set(command.camera, after);
        return receipt;
    }
    applyNavigation(input) {
        const before = this.#states.get(input.camera);
        if (!before)
            return undefined;
        if (input.expectedRevision !== before.revision)
            return rejectedNavigation(before, 'staleRevision');
        if (before.mode === 'firstPerson')
            return rejectedNavigation(before, 'incompatibleMode');
        const value = input.input;
        if (![value.panRight, value.panForward, value.yawDeltaDegrees, value.pitchDeltaDegrees,
            value.zoomDelta, value.dtSeconds, value.panSpeedUnitsPerSecond].every(Number.isFinite)
            || value.dtSeconds < 0 || value.dtSeconds > 1
            || value.panSpeedUnitsPerSecond < 0 || value.panSpeedUnitsPerSecond > 1_000) {
            return rejectedNavigation(before, 'invalidInput');
        }
        const priorPivot = before.pivot;
        if (priorPivot === null || before.distance === null
            || before.minDistance === null || before.maxDistance === null) {
            return rejectedNavigation(before, 'invalidTarget');
        }
        const yawDegrees = before.snapshot.pose.yawDegrees + value.yawDeltaDegrees;
        const pitchDegrees = before.mode === 'orbit'
            ? Math.max(-89, Math.min(89, before.snapshot.pose.pitchDegrees + value.pitchDeltaDegrees))
            : Math.max(-89, Math.min(-30, before.snapshot.pose.pitchDegrees + value.pitchDeltaDegrees));
        const panBasis = basisFromPose({ position: priorPivot, yawDegrees, pitchDegrees: 0 });
        const panDistance = value.dtSeconds * value.panSpeedUnitsPerSecond;
        const pivot = [
            priorPivot[0] + (panBasis.right[0] * value.panRight + panBasis.forward[0] * value.panForward) * panDistance,
            priorPivot[1],
            priorPivot[2] + (panBasis.right[2] * value.panRight + panBasis.forward[2] * value.panForward) * panDistance,
        ];
        const distance = Math.max(before.minDistance, Math.min(before.maxDistance, before.distance - value.zoomDelta));
        const pose = before.mode === 'orbit'
            ? orbitPose(pivot, distance, yawDegrees, pitchDegrees)
            : topDownPose(pivot, distance, yawDegrees, pitchDegrees);
        const snapshot = { ...before.snapshot, tick: input.tick, pose, basis: basisFromPose(pose) };
        const after = state(before.revision + 1, before.mode, pivot, distance, before.minDistance, before.maxDistance, snapshot);
        const receipt = {
            accepted: true,
            before,
            after,
            terrainConstrained: false,
            rejection: null,
            receiptHash: hash({ accepted: true, before: before.stateHash, after: after.stateHash }),
        };
        this.#states.set(input.camera, after);
        return receipt;
    }
}
//# sourceMappingURL=mock-camera-controller.js.map
const COUNTER_ORDER = [
    'entityCount',
    'activeCapabilityCount',
    'residentChunkCount',
    'dirtyChunkCount',
    'renderDiffCount',
    'renderHandleCount',
    'drawCallCount',
    'activeAudioSourceCount',
    'activeBillboardCount',
    'activeParticleCount',
    'droppedFeedbackCount',
];
export class AshaLiveTelemetryCollector {
    #expectedCounters;
    #maxFrameTimeSamples;
    #frameTimeHistory = [];
    #sampleSequence = 0;
    #snapshot = null;
    constructor(options) {
        this.#expectedCounters = new Set(options.expectedCounters);
        this.#maxFrameTimeSamples = boundedInteger(options.maxFrameTimeSamples ?? 60, 1, 240, 'maxFrameTimeSamples');
    }
    sample(input) {
        if (!Number.isSafeInteger(input.authorityTick) || input.authorityTick < 0) {
            throw new Error('authorityTick must be a non-negative safe integer');
        }
        const diagnostics = [];
        const metrics = [];
        if (validMetric(input.frameTimeMs)) {
            this.#frameTimeHistory.push(input.frameTimeMs);
            if (this.#frameTimeHistory.length > this.#maxFrameTimeSamples) {
                this.#frameTimeHistory.splice(0, this.#frameTimeHistory.length - this.#maxFrameTimeSamples);
            }
            metrics.push(metric('frameTimeMs', input.frameTimeMs, 'durationMs', 'ms'));
        }
        else {
            diagnostics.push({
                code: 'invalidSample',
                counter: 'frameTimeMs',
                message: 'frameTimeMs must be finite and non-negative',
            });
        }
        for (const counter of COUNTER_ORDER) {
            const value = input.counters[counter];
            if (value === null || value === undefined) {
                if (this.#expectedCounters.has(counter)) {
                    diagnostics.push({
                        code: 'counterUnavailable',
                        counter,
                        message: `${counter} is unavailable from the current owner adapter`,
                    });
                }
                continue;
            }
            if (!validMetric(value)) {
                diagnostics.push({
                    code: 'invalidSample',
                    counter,
                    message: `${counter} must be finite and non-negative`,
                });
                continue;
            }
            metrics.push(metric(counter, value, 'gauge', 'count'));
        }
        this.#sampleSequence += 1;
        this.#snapshot = {
            schemaVersion: 1,
            authorityTick: input.authorityTick,
            sampleSequence: this.#sampleSequence,
            metrics,
            frameTimeHistoryMs: [...this.#frameTimeHistory],
            diagnostics,
        };
        return this.readSnapshot();
    }
    readSnapshot() {
        if (this.#snapshot === null) {
            throw new Error('live telemetry has not sampled any owner counters');
        }
        return {
            ...this.#snapshot,
            metrics: [...this.#snapshot.metrics],
            frameTimeHistoryMs: [...this.#snapshot.frameTimeHistoryMs],
            diagnostics: [...this.#snapshot.diagnostics],
        };
    }
    tryReadSnapshot() {
        return this.#snapshot === null ? null : this.readSnapshot();
    }
}
export class AshaTelemetryOverlayHost {
    #collector;
    #sink;
    #active = new Map();
    #diagnostics = [];
    #renderedSnapshots = 0;
    constructor(options) {
        this.#collector = options.collector;
        this.#sink = options.sink;
    }
    applyPresentation(frame) {
        const diagnostics = [];
        let applied = 0;
        for (const operation of frame.ops) {
            if (operation.domain !== 'telemetryOverlay') {
                continue;
            }
            const diagnostic = this.#applyOperation(operation);
            if (diagnostic === null) {
                applied += 1;
            }
            else {
                diagnostics.push(diagnostic);
                this.#diagnostics.push(diagnostic);
            }
        }
        return { applied, diagnostics, readout: this.readout() };
    }
    sample(input, elapsedMs) {
        if (!Number.isFinite(elapsedMs) || elapsedMs < 0) {
            throw new Error('elapsedMs must be finite and non-negative');
        }
        const snapshot = this.#collector.sample(input);
        for (const [rawHandle, overlay] of this.#active) {
            if (!overlay.descriptor.visible) {
                continue;
            }
            if (overlay.lastRenderedMs === null
                || elapsedMs - overlay.lastRenderedMs >= overlay.descriptor.refreshIntervalMs) {
                this.#sink.render(rawHandle, overlay.descriptor, snapshot);
                overlay.lastRenderedMs = elapsedMs;
                this.#renderedSnapshots += 1;
            }
        }
        return snapshot;
    }
    setVisible(handle, visible) {
        const overlay = this.#active.get(handle);
        if (overlay === undefined) {
            return false;
        }
        overlay.descriptor = { ...overlay.descriptor, visible };
        overlay.lastRenderedMs = null;
        this.#sink.render(handle, overlay.descriptor, this.#collector.tryReadSnapshot());
        return true;
    }
    toggleVisible(handle) {
        const overlay = this.#active.get(handle);
        if (overlay === undefined) {
            return null;
        }
        const visible = !overlay.descriptor.visible;
        this.setVisible(handle, visible);
        return visible;
    }
    readout() {
        return {
            activeOverlays: this.#active.size,
            renderedSnapshots: this.#renderedSnapshots,
            diagnostics: [...this.#diagnostics],
        };
    }
    cleanup() {
        for (const rawHandle of this.#active.keys()) {
            this.#sink.destroy(rawHandle);
        }
        this.#active.clear();
    }
    #applyOperation(operation) {
        const rawHandle = operation.op.handle;
        try {
            if (operation.op.op === 'create') {
                if (this.#active.has(rawHandle)) {
                    return diagnostic(operation, 'duplicateHandle', 'overlay handle is already active');
                }
                this.#active.set(rawHandle, {
                    descriptor: operation.op.descriptor,
                    lastRenderedMs: null,
                });
                this.#sink.render(operation.op.handle, operation.op.descriptor, this.#collector.tryReadSnapshot());
                return null;
            }
            const active = this.#active.get(rawHandle);
            if (active === undefined) {
                return diagnostic(operation, 'unknownHandle', 'overlay handle is not active');
            }
            if (operation.op.op === 'update') {
                active.descriptor = applyPatch(active.descriptor, operation.op.patch);
                active.lastRenderedMs = null;
                this.#sink.render(operation.op.handle, active.descriptor, this.#collector.tryReadSnapshot());
            }
            else {
                this.#active.delete(rawHandle);
                this.#sink.destroy(operation.op.handle);
            }
            return null;
        }
        catch (error) {
            return diagnostic(operation, 'hostFailure', error instanceof Error ? error.message : String(error));
        }
    }
}
function metric(counter, value, kind, unit) {
    return { counter, kind, value, unit };
}
function validMetric(value) {
    return Number.isFinite(value) && value >= 0;
}
function boundedInteger(value, min, max, name) {
    if (!Number.isInteger(value) || value < min || value > max) {
        throw new Error(`${name} must be an integer between ${min} and ${max}`);
    }
    return value;
}
function applyPatch(descriptor, patch) {
    return {
        title: patch.title ?? descriptor.title,
        corner: patch.corner ?? descriptor.corner,
        refreshIntervalMs: patch.refreshIntervalMs ?? descriptor.refreshIntervalMs,
        maxFrameTimeSamples: patch.maxFrameTimeSamples ?? descriptor.maxFrameTimeSamples,
        visible: patch.visible ?? descriptor.visible,
    };
}
function diagnostic(operation, code, message) {
    return {
        code,
        sequence: operation.meta.sequence,
        handle: operation.op.handle,
        message,
        origin: operation.meta.origin,
    };
}
//# sourceMappingURL=telemetry-host.js.map
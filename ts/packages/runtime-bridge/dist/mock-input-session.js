import { RuntimeBridgeError } from './bridge.js';
export class MockInputSession {
    #initialized = false;
    #catalog = null;
    #catalogHash = '';
    #contextState = null;
    #replayedRecordHashes = new Set();
    initialize() {
        this.#initialized = true;
        this.#catalog = null;
        this.#catalogHash = '';
        this.#contextState = null;
        this.#replayedRecordHashes.clear();
    }
    configure(request) {
        this.#requireInitialized('configureInputSession');
        const actionIds = new Set(request.catalog.actions.map((action) => action.actionId));
        const contexts = new Map(request.catalog.contexts.map((context) => [context.contextId, context]));
        const bindingIds = new Set();
        const bindingKeys = new Set();
        const invalid = request.catalog.schemaVersion !== 1
            || actionIds.size !== request.catalog.actions.length
            || contexts.size !== request.catalog.contexts.length
            || request.initialContexts.some((id, index) => !contexts.has(id) || request.initialContexts.indexOf(id) !== index)
            || request.catalog.bindings.some((binding) => {
                const key = `${binding.contextId}\u0000${binding.platformKind}\u0000${binding.control}`;
                const rejected = bindingIds.has(binding.bindingId) || bindingKeys.has(key)
                    || !actionIds.has(binding.actionId) || !contexts.has(binding.contextId)
                    || binding.extension !== null || !Number.isFinite(binding.scale);
                bindingIds.add(binding.bindingId);
                bindingKeys.add(key);
                return rejected;
            });
        if (invalid)
            throw new RuntimeBridgeError('invalid_input', 'input catalog validation failed');
        const catalog = {
            ...request.catalog,
            actions: [...request.catalog.actions].sort((left, right) => left.actionId.localeCompare(right.actionId)),
            contexts: [...request.catalog.contexts].sort((left, right) => left.contextId.localeCompare(right.contextId)),
            bindings: [...request.catalog.bindings].sort((left, right) => left.bindingId.localeCompare(right.bindingId)),
        };
        const catalogHash = hash(catalog);
        this.#catalog = catalog;
        this.#catalogHash = catalogHash;
        this.#contextState = this.#buildContextState(0, request.initialContexts);
        this.#replayedRecordHashes.clear();
        return { catalogHash, contextState: this.#contextState };
    }
    applyContextCommand(command) {
        const current = this.#requireContextState('applyInputContextCommand');
        const ids = current.activeContexts.map((context) => context.contextId);
        let nextIds;
        if (command.operation === 'push') {
            if (ids.includes(command.contextId) || !this.#contextDefinition(command.contextId)) {
                return this.#rejectedChange(current, 'unknown or duplicate input context');
            }
            nextIds = [...ids, command.contextId];
        }
        else if (command.operation === 'pop') {
            if (ids.at(-1) !== command.expectedContextId) {
                return this.#rejectedChange(current, 'input context stack mismatch');
            }
            nextIds = ids.slice(0, -1);
        }
        else {
            if (new Set(command.contextIds).size !== command.contextIds.length
                || command.contextIds.some((id) => !this.#contextDefinition(id))) {
                return this.#rejectedChange(current, 'unknown or duplicate input context');
            }
            nextIds = command.contextIds;
        }
        const state = this.#buildContextState(current.revision + 1, nextIds);
        this.#contextState = state;
        return { accepted: true, state, diagnostics: [] };
    }
    resolve(sample) {
        const state = this.#requireContextState('submitRawInput');
        const catalog = this.#catalog;
        const inputHash = hash(sample);
        const active = [...state.activeContexts].sort((left, right) => {
            const leftPriority = this.#contextDefinition(left.contextId).priority;
            const rightPriority = this.#contextDefinition(right.contextId).priority;
            return rightPriority - leftPriority || right.stackOrder - left.stackOrder
                || left.contextId.localeCompare(right.contextId);
        });
        for (const context of active) {
            const definition = this.#contextDefinition(context.contextId);
            const binding = catalog.bindings.find((candidate) => candidate.contextId === context.contextId
                && candidate.platformKind === sample.platformKind && candidate.control === sample.control);
            if (binding) {
                const action = catalog.actions.find((candidate) => candidate.actionId === binding.actionId);
                if (!action.acceptedPhases.includes(sample.phase)) {
                    return this.#receipt(sample.sequence, false, true, null, inputHash, [{
                            code: 'unsupportedPhase', path: 'sample.phase', message: 'input phase is not accepted by action',
                        }]);
                }
                return this.#receipt(sample.sequence, true, true, {
                    sequence: sample.sequence,
                    actionId: binding.actionId,
                    contextId: binding.contextId,
                    bindingId: binding.bindingId,
                    phase: sample.phase,
                    value: scaledValue(sample.value, binding.scale),
                }, inputHash, []);
            }
            if (definition.consumesLowerPriority) {
                return this.#receipt(sample.sequence, false, true, null, inputHash, [{
                        code: 'consumedByContext', path: 'contextState.activeContexts',
                        message: `input consumed by '${context.contextId}'`,
                    }]);
            }
        }
        return this.#receipt(sample.sequence, false, false, null, inputHash, [{
                code: 'unboundInput', path: 'sample.control', message: `no active context binds '${sample.control}'`,
            }]);
    }
    readContextState() {
        return this.#requireContextState('readInputContextState');
    }
    replay(record) {
        const state = this.#requireContextState('replayResolvedInputAction');
        const diagnostics = [];
        if (record.schemaVersion !== 1)
            diagnostics.push({
                code: 'unsupportedReplaySchema', path: 'record.schemaVersion', message: 'unsupported replay schema',
            });
        if (record.catalogHash !== this.#catalogHash)
            diagnostics.push({
                code: 'catalogHashMismatch', path: 'record.catalogHash', message: 'record catalog hash mismatch',
            });
        if (record.contextHash !== state.stateHash)
            diagnostics.push({
                code: 'contextHashMismatch', path: 'record.contextHash', message: 'record context hash mismatch',
            });
        if (record.recordHash !== hash(recordPayload(record)))
            diagnostics.push({
                code: 'replayRecordHashMismatch', path: 'record.recordHash', message: 'record hash mismatch',
            });
        const definition = this.#catalog.actions.find((action) => action.actionId === record.action.actionId);
        const binding = this.#catalog.bindings.find((item) => item.bindingId === record.action.bindingId);
        if (!definition)
            diagnostics.push({
                code: 'unknownAction', path: 'record.action.actionId', message: 'unknown recorded action',
            });
        if (!binding || binding.actionId !== record.action.actionId || binding.contextId !== record.action.contextId) {
            diagnostics.push({
                code: 'conflictingBinding', path: 'record.action.bindingId', message: 'record binding mismatch',
            });
        }
        if (!state.activeContexts.some((context) => context.contextId === record.action.contextId)) {
            diagnostics.push({
                code: 'unknownContext', path: 'record.action.contextId', message: 'record context is not active',
            });
        }
        else if (binding) {
            const winnerDiagnostic = this.#recordWinnerDiagnostic(binding);
            if (winnerDiagnostic !== null)
                diagnostics.push(winnerDiagnostic);
        }
        if (definition && !definition.acceptedPhases.includes(record.action.phase))
            diagnostics.push({
                code: 'unsupportedPhase', path: 'record.action.phase', message: 'record phase is not accepted',
            });
        if (diagnostics.length === 0 && this.#replayedRecordHashes.has(record.recordHash))
            diagnostics.push({
                code: 'replayAlreadyDelivered', path: 'record.recordHash', message: 'record already delivered',
            });
        const accepted = diagnostics.length === 0;
        if (accepted)
            this.#replayedRecordHashes.add(record.recordHash);
        const action = accepted ? record.action : null;
        const core = {
            accepted, action, diagnostics, catalogHash: this.#catalogHash,
            contextHash: state.stateHash, recordHash: record.recordHash,
        };
        return { ...core, replayHash: hash(core) };
    }
    #requireInitialized(operation) {
        if (!this.#initialized)
            throw new RuntimeBridgeError('not_initialized', `${operation} before initializeEngine`);
    }
    #requireContextState(operation) {
        this.#requireInitialized(operation);
        if (this.#contextState === null || this.#catalog === null) {
            throw new RuntimeBridgeError('invalid_input', `${operation} before configureInputSession`);
        }
        return this.#contextState;
    }
    #contextDefinition(contextId) {
        return this.#catalog?.contexts.find((context) => context.contextId === contextId);
    }
    #recordWinnerDiagnostic(recordedBinding) {
        const state = this.#contextState;
        const active = [...state.activeContexts].sort((left, right) => {
            const leftPriority = this.#contextDefinition(left.contextId).priority;
            const rightPriority = this.#contextDefinition(right.contextId).priority;
            return rightPriority - leftPriority || right.stackOrder - left.stackOrder
                || left.contextId.localeCompare(right.contextId);
        });
        for (const activeContext of active) {
            const candidate = this.#catalog.bindings.find((binding) => binding.contextId === activeContext.contextId
                && binding.platformKind === recordedBinding.platformKind
                && binding.control === recordedBinding.control);
            if (candidate) {
                return candidate.bindingId === recordedBinding.bindingId ? null : {
                    code: 'conflictingBinding', path: 'record.action.bindingId',
                    message: `record shadowed by '${candidate.bindingId}'`,
                };
            }
            if (this.#contextDefinition(activeContext.contextId).consumesLowerPriority)
                return {
                    code: 'consumedByContext', path: 'record.action.contextId',
                    message: `record consumed by '${activeContext.contextId}'`,
                };
        }
        return { code: 'unknownContext', path: 'record.action.contextId', message: 'record context is unreachable' };
    }
    #buildContextState(revision, contextIds) {
        const activeContexts = contextIds.map((contextId, stackOrder) => ({ contextId, stackOrder }));
        const state = { schemaVersion: 1, revision, activeContexts };
        return { ...state, stateHash: hash(state) };
    }
    #rejectedChange(state, message) {
        return { accepted: false, state, diagnostics: [{ code: 'contextStackMismatch', path: 'command', message }] };
    }
    #receipt(sequence, accepted, consumed, action, inputHash, diagnostics) {
        const receipt = { sequence, accepted, consumed, action, diagnostics, catalogHash: this.#catalogHash,
            contextHash: this.#contextState.stateHash, inputHash };
        const resolutionHash = hash(receipt);
        const record = action === null ? null : recordedAction(action, this.#catalogHash, this.#contextState.stateHash);
        return { ...receipt, resolutionHash, record };
    }
}
function recordPayload(record) {
    return {
        schemaVersion: record.schemaVersion,
        action: record.action,
        catalogHash: record.catalogHash,
        contextHash: record.contextHash,
    };
}
function recordedAction(action, catalogHash, contextHash) {
    const payload = { schemaVersion: 1, action, catalogHash, contextHash };
    return { ...payload, recordHash: hash(payload) };
}
function scaledValue(value, scale) {
    if (value.kind === 'button')
        return value;
    if (value.kind === 'axis1d')
        return { kind: 'axis1d', value: value.value * scale };
    return { kind: 'axis2d', x: value.x * scale, y: value.y * scale };
}
function hash(value) {
    let current = 0xcbf29ce484222325n;
    const text = JSON.stringify(value);
    for (let index = 0; index < text.length; index += 1) {
        current ^= BigInt(text.charCodeAt(index));
        current = (current * 0x100000001b3n) & 0xffffffffffffffffn;
    }
    return `fnv1a64:${current.toString(16).padStart(16, '0')}`;
}
//# sourceMappingURL=mock-input-session.js.map
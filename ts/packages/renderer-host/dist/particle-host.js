export class AshaParticleHost {
    #maxActiveEmitters;
    #maxParticles;
    #resolveEntityPosition;
    #resolveResource;
    #sink;
    #emitters = new Map();
    #burstEmitters = new Map();
    #particles = new Map();
    #seenSignals = new Set();
    #spriteUrls = new Map();
    #diagnostics = [];
    #nextParticleId = 1;
    #emittedBursts = 0;
    #droppedParticles = 0;
    constructor(options) {
        this.#maxActiveEmitters = options.maxActiveEmitters ?? 64;
        this.#maxParticles = options.maxParticles ?? 4_096;
        this.#resolveEntityPosition = options.resolveEntityPosition;
        this.#resolveResource = options.resolveResource;
        this.#sink = options.sink;
    }
    async applyPresentation(frame) {
        const diagnostics = [];
        let applied = 0;
        for (const operation of frame.ops) {
            if (operation.domain !== 'particle') {
                continue;
            }
            const diagnostic = await this.#applyOperation(operation);
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
    advance(deltaSeconds) {
        if (!Number.isFinite(deltaSeconds) || deltaSeconds < 0 || deltaSeconds > 1) {
            const diagnostic = hostDiagnostic('invalidDescriptor', 'particle frame delta must be finite and between zero and one second');
            this.#diagnostics.push(diagnostic);
            return { applied: 0, diagnostics: [diagnostic], readout: this.readout() };
        }
        const diagnostics = [];
        for (const emitter of this.#emitters.values()) {
            if (!emitter.descriptor.visible) {
                continue;
            }
            emitter.emissionCarry += emitter.descriptor.ratePerSecond * deltaSeconds;
            const count = Math.floor(emitter.emissionCarry);
            emitter.emissionCarry -= count;
            const diagnostic = this.#spawn(emitter, count, 0);
            if (diagnostic !== null) {
                diagnostics.push(diagnostic);
            }
        }
        for (const particle of [...this.#particles.values()]) {
            particle.ageSeconds += deltaSeconds;
            if (particle.ageSeconds >= particle.lifetimeSeconds) {
                this.#destroyParticle(particle);
                continue;
            }
            const acceleration = particle.descriptor.acceleration;
            particle.velocity[0] += acceleration[0] * deltaSeconds;
            particle.velocity[1] += acceleration[1] * deltaSeconds;
            particle.velocity[2] += acceleration[2] * deltaSeconds;
            particle.position[0] += particle.velocity[0] * deltaSeconds;
            particle.position[1] += particle.velocity[1] * deltaSeconds;
            particle.position[2] += particle.velocity[2] * deltaSeconds;
            this.#sink.update(projectParticle(particle));
        }
        this.#cleanupFinishedBursts();
        this.#diagnostics.push(...diagnostics);
        return { applied: this.#particles.size, diagnostics, readout: this.readout() };
    }
    readout() {
        return {
            activeEmitters: this.#emitters.size,
            activeParticles: this.#particles.size,
            loadedSprites: this.#spriteUrls.size,
            emittedBursts: this.#emittedBursts,
            droppedParticles: this.#droppedParticles,
            diagnostics: [...this.#diagnostics],
        };
    }
    cleanup() {
        for (const particle of [...this.#particles.values()]) {
            this.#destroyParticle(particle);
        }
        this.#emitters.clear();
        this.#burstEmitters.clear();
        this.#seenSignals.clear();
    }
    dispose() {
        this.cleanup();
        this.#spriteUrls.clear();
        this.#diagnostics.length = 0;
    }
    async #applyOperation(operation) {
        try {
            switch (operation.op.op) {
                case 'emit':
                    return await this.#emit(operation.meta, operation.op);
                case 'create':
                    return await this.#create(operation.meta, operation.op);
                case 'update':
                    return await this.#update(operation.meta, operation.op);
                case 'destroy':
                    return this.#destroy(operation.meta, operation.op);
            }
        }
        catch (error) {
            return operationDiagnostic(error instanceof AshaParticleResourceError ? error.code : 'hostFailure', operation.meta, operationHandle(operation.op), error instanceof Error ? error.message : String(error));
        }
    }
    async #emit(meta, op) {
        if (this.#seenSignals.has(op.signalId)) {
            return null;
        }
        const spriteUrl = await this.#prepareSprite(op.descriptor.sprite);
        const emitter = createEmitter(`signal:${op.signalId}`, null, meta.origin, op.descriptor, spriteUrl);
        const diagnostic = this.#spawn(emitter, op.descriptor.burstCount, meta.sequence, spriteUrl);
        if (diagnostic?.code === 'anchorMissing') {
            return diagnostic;
        }
        this.#seenSignals.add(op.signalId);
        this.#burstEmitters.set(emitter.key, emitter);
        this.#emittedBursts += 1;
        return diagnostic;
    }
    async #create(meta, op) {
        const rawHandle = op.handle;
        if (this.#emitters.has(rawHandle)) {
            return operationDiagnostic('duplicateHandle', meta, op.handle, 'particle emitter handle is already active');
        }
        if (this.#emitters.size >= this.#maxActiveEmitters) {
            return operationDiagnostic('budgetExceeded', meta, op.handle, 'particle emitter budget is exhausted');
        }
        const spriteUrl = await this.#prepareSprite(op.descriptor.sprite);
        const emitter = createEmitter(`handle:${rawHandle}`, op.handle, meta.origin, op.descriptor, spriteUrl);
        this.#emitters.set(rawHandle, emitter);
        return this.#spawn(emitter, op.descriptor.burstCount, meta.sequence, spriteUrl);
    }
    async #update(meta, op) {
        const emitter = this.#emitters.get(op.handle);
        if (emitter === undefined) {
            return operationDiagnostic('unknownHandle', meta, op.handle, 'particle emitter handle is not active');
        }
        const descriptor = applyParticlePatch(emitter.descriptor, op.patch);
        emitter.spriteUrl = await this.#prepareSprite(descriptor.sprite);
        emitter.descriptor = descriptor;
        return null;
    }
    #destroy(meta, op) {
        const emitter = this.#emitters.get(op.handle);
        if (emitter === undefined) {
            return operationDiagnostic('unknownHandle', meta, op.handle, 'particle emitter handle is not active');
        }
        this.#emitters.delete(op.handle);
        for (const id of [...emitter.particleIds]) {
            const particle = this.#particles.get(id);
            if (particle !== undefined) {
                this.#destroyParticle(particle);
            }
        }
        return null;
    }
    #spawn(emitter, requested, sequence, preparedSpriteUrl) {
        if (requested <= 0 || !emitter.descriptor.visible) {
            return null;
        }
        const anchor = resolveAnchor(emitter.descriptor.anchor, this.#resolveEntityPosition);
        if (anchor === null) {
            return operationDiagnostic('anchorMissing', { sequence, origin: emitter.origin }, emitter.handle, 'particle entity anchor is unavailable');
        }
        const emitterRemaining = Math.max(0, emitter.descriptor.maxParticles - emitter.particleIds.size);
        const hostRemaining = Math.max(0, this.#maxParticles - this.#particles.size);
        const count = Math.min(requested, emitterRemaining, hostRemaining);
        this.#droppedParticles += requested - count;
        const spriteUrl = preparedSpriteUrl ?? emitter.spriteUrl;
        for (let index = 0; index < count; index += 1) {
            const particle = this.#newParticle(emitter, anchor, spriteUrl);
            emitter.particleIds.add(particle.id);
            this.#particles.set(particle.id, particle);
            this.#sink.create(projectParticle(particle));
        }
        return count < requested
            ? operationDiagnostic('budgetExceeded', { sequence, origin: emitter.origin }, emitter.handle, `particle budget dropped ${requested - count} particles`)
            : null;
    }
    #newParticle(emitter, anchor, spriteUrl) {
        const descriptor = emitter.descriptor;
        const lifetime = randomRange(emitter, descriptor.lifetimeSeconds[0], descriptor.lifetimeSeconds[1]);
        const velocity = [
            randomRange(emitter, descriptor.velocityMin[0], descriptor.velocityMax[0]),
            randomRange(emitter, descriptor.velocityMin[1], descriptor.velocityMax[1]),
            randomRange(emitter, descriptor.velocityMin[2], descriptor.velocityMax[2]),
        ];
        return {
            id: this.#nextParticleId++,
            emitterKey: emitter.key,
            descriptor,
            spriteUrl,
            ageSeconds: 0,
            lifetimeSeconds: lifetime,
            position: [...anchor],
            velocity,
        };
    }
    #destroyParticle(particle) {
        this.#particles.delete(particle.id);
        this.#sink.destroy(particle.id);
        this.#emitters.get(Number(particle.emitterKey.slice(7)))?.particleIds.delete(particle.id);
        this.#burstEmitters.get(particle.emitterKey)?.particleIds.delete(particle.id);
    }
    #cleanupFinishedBursts() {
        for (const [key, emitter] of this.#burstEmitters) {
            if (emitter.particleIds.size === 0) {
                this.#burstEmitters.delete(key);
            }
        }
    }
    async #prepareSprite(sprite) {
        const key = spriteKey(sprite);
        const existing = this.#spriteUrls.get(key);
        if (existing !== undefined) {
            return existing;
        }
        const prepared = this.#resolveResource(sprite).then(async (resource) => {
            if (resource === null) {
                throw new AshaParticleResourceError('spriteLoadFailed', `particle sprite ${sprite.asset} is unavailable`);
            }
            await validateResourceHash(resource.bytes, sprite.contentHash);
            return resource.url;
        });
        this.#spriteUrls.set(key, prepared);
        try {
            return await prepared;
        }
        catch (error) {
            this.#spriteUrls.delete(key);
            throw error;
        }
    }
}
function createEmitter(key, handle, origin, descriptor, spriteUrl) {
    return {
        descriptor,
        spriteUrl,
        key,
        handle,
        origin,
        randomState: normalizeSeed(descriptor.seed),
        emissionCarry: 0,
        particleIds: new Set(),
    };
}
function normalizeSeed(seed) {
    const normalized = Math.trunc(seed) >>> 0;
    return normalized === 0 ? 0x9e3779b9 : normalized;
}
function randomRange(emitter, min, max) {
    let value = emitter.randomState;
    value ^= value << 13;
    value ^= value >>> 17;
    value ^= value << 5;
    emitter.randomState = value >>> 0;
    return min + (max - min) * (emitter.randomState / 0x1_0000_0000);
}
function resolveAnchor(anchor, resolveEntityPosition) {
    if (anchor.kind === 'world') {
        return anchor.position;
    }
    const base = resolveEntityPosition(anchor.entity);
    return base === null
        ? null
        : [
            base[0] + anchor.offset[0],
            base[1] + anchor.offset[1],
            base[2] + anchor.offset[2],
        ];
}
function projectParticle(particle) {
    const age = Math.min(1, particle.ageSeconds / particle.lifetimeSeconds);
    return {
        id: particle.id,
        position: [...particle.position],
        size: interpolateScalar(particle.descriptor.sizeCurve, age),
        color: interpolateColor(particle.descriptor.colorCurve, age),
        frameIndex: particle.descriptor.sprite.frameCount === 1
            ? 0
            : Math.floor(particle.ageSeconds * particle.descriptor.flipbookFramesPerSecond)
                % particle.descriptor.sprite.frameCount,
        frameCount: particle.descriptor.sprite.frameCount,
        spriteUrl: particle.spriteUrl,
    };
}
function interpolateScalar(keys, age) {
    const [left, right] = curvePair(keys, age);
    const blend = curveBlend(left.age, right.age, age);
    return left.value + (right.value - left.value) * blend;
}
function interpolateColor(keys, age) {
    const [left, right] = curvePair(keys, age);
    const blend = curveBlend(left.age, right.age, age);
    return [0, 1, 2, 3].map((index) => left.color[index] + (right.color[index] - left.color[index]) * blend);
}
function curvePair(keys, age) {
    for (let index = 1; index < keys.length; index += 1) {
        const right = keys[index];
        if (age <= right.age) {
            return [keys[index - 1], right];
        }
    }
    return [keys[keys.length - 1], keys[keys.length - 1]];
}
function curveBlend(start, end, age) {
    return end === start ? 0 : (age - start) / (end - start);
}
function applyParticlePatch(descriptor, patch) {
    return {
        anchor: patch.anchor ?? descriptor.anchor,
        sprite: patch.sprite ?? descriptor.sprite,
        ratePerSecond: patch.ratePerSecond ?? descriptor.ratePerSecond,
        burstCount: patch.burstCount ?? descriptor.burstCount,
        lifetimeSeconds: patch.lifetimeSeconds ?? descriptor.lifetimeSeconds,
        velocityMin: patch.velocityMin ?? descriptor.velocityMin,
        velocityMax: patch.velocityMax ?? descriptor.velocityMax,
        acceleration: patch.acceleration ?? descriptor.acceleration,
        sizeCurve: patch.sizeCurve ?? descriptor.sizeCurve,
        colorCurve: patch.colorCurve ?? descriptor.colorCurve,
        flipbookFramesPerSecond: patch.flipbookFramesPerSecond ?? descriptor.flipbookFramesPerSecond,
        seed: descriptor.seed,
        maxParticles: patch.maxParticles ?? descriptor.maxParticles,
        visible: patch.visible ?? descriptor.visible,
    };
}
function operationHandle(op) {
    return op.op === 'emit' ? null : op.handle;
}
function operationDiagnostic(code, meta, handle, message) {
    return { code, sequence: meta.sequence, handle, message, origin: meta.origin };
}
function hostDiagnostic(code, message) {
    return { code, sequence: 0, handle: null, message, origin: null };
}
function spriteKey(sprite) {
    return `${sprite.asset}:${sprite.contentHash}`;
}
async function validateResourceHash(bytes, expected) {
    if (globalThis.crypto?.subtle === undefined) {
        throw new AshaParticleResourceError('contentHashMismatch', 'Web Crypto SHA-256 is unavailable for particle sprites');
    }
    const digest = await globalThis.crypto.subtle.digest('SHA-256', bytes);
    const actual = [...new Uint8Array(digest)]
        .map((byte) => byte.toString(16).padStart(2, '0'))
        .join('');
    if (actual !== expected) {
        throw new AshaParticleResourceError('contentHashMismatch', `particle sprite hash ${actual} does not match ${expected}`);
    }
}
class AshaParticleResourceError extends Error {
    code;
    constructor(code, message) {
        super(message);
        this.code = code;
    }
}
//# sourceMappingURL=particle-host.js.map
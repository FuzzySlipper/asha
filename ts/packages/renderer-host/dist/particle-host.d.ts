import type { ParticleProjectionDiagnostic, ParticleProjectionReadout, ParticleSpriteRef, PresentationFrameDiff } from '@asha/contracts';
type Vec3 = readonly [number, number, number];
export interface AshaParticleResource {
    readonly bytes: ArrayBuffer;
    readonly url: string;
}
export type AshaParticleResourceResolver = (sprite: ParticleSpriteRef) => Promise<AshaParticleResource | null>;
export type AshaParticleEntityPositionResolver = (entity: number) => Vec3 | null;
export interface AshaParticleBillboard {
    readonly id: number;
    readonly position: Vec3;
    readonly size: number;
    readonly color: readonly [number, number, number, number];
    readonly frameIndex: number;
    readonly frameCount: number;
    readonly spriteUrl: string;
}
export interface AshaParticleBillboardSink {
    create(particle: AshaParticleBillboard): void;
    update(particle: AshaParticleBillboard): void;
    destroy(id: number): void;
}
export interface AshaParticleHostOptions {
    readonly maxActiveEmitters?: number;
    readonly maxParticles?: number;
    readonly resolveEntityPosition: AshaParticleEntityPositionResolver;
    readonly resolveResource: AshaParticleResourceResolver;
    readonly sink: AshaParticleBillboardSink;
}
export interface AshaParticleFrameReceipt {
    readonly applied: number;
    readonly diagnostics: readonly ParticleProjectionDiagnostic[];
    readonly readout: ParticleProjectionReadout;
}
export declare class AshaParticleHost {
    #private;
    constructor(options: AshaParticleHostOptions);
    applyPresentation(frame: PresentationFrameDiff): Promise<AshaParticleFrameReceipt>;
    advance(deltaSeconds: number): AshaParticleFrameReceipt;
    readout(): ParticleProjectionReadout;
    cleanup(): void;
    dispose(): void;
}
export {};
//# sourceMappingURL=particle-host.d.ts.map
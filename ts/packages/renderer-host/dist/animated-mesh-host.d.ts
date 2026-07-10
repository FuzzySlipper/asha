import type { RenderFrameDiff, RenderHandle } from '@asha/contracts';
export type AshaRendererHostDiagnosticCode = 'animated_mesh_manifest_invalid' | 'animated_mesh_resource_unavailable' | 'animated_mesh_content_hash_mismatch' | 'animated_mesh_clip_unavailable' | 'animated_mesh_frame_rejected' | 'animated_mesh_handle_unavailable' | 'animation_not_started' | 'animation_paused' | 'animation_stopped';
export interface AshaRendererHostDiagnostic {
    readonly code: AshaRendererHostDiagnosticCode;
    readonly message: string;
    readonly asset: string | null;
    readonly handle: RenderHandle | null;
}
export declare class AshaRendererHostError extends Error {
    readonly diagnostics: readonly AshaRendererHostDiagnostic[];
    constructor(diagnostics: readonly AshaRendererHostDiagnostic[]);
}
export interface AshaRendererAnimatedMeshResourceDescriptor {
    readonly asset: string;
    readonly resourceUrl: string;
    readonly contentHash: `sha256:${string}`;
    readonly clipIds: readonly string[];
    readonly licenseUrl: string | null;
}
export interface AshaRendererAnimatedMeshResourceManifest {
    readonly kind: 'asha_renderer_animated_mesh_resources.v0';
    readonly resources: readonly AshaRendererAnimatedMeshResourceDescriptor[];
}
export type AshaRendererAnimatedMeshResourceResolver = (descriptor: AshaRendererAnimatedMeshResourceDescriptor) => Promise<ArrayBuffer>;
export declare const ASHA_RENDERER_HOST_KENNEY_ANIMATED_MESH_RESOURCE: AshaRendererAnimatedMeshResourceDescriptor;
export declare const ASHA_RENDERER_HOST_ANIMATED_MESH_FIXTURE_MANIFEST: AshaRendererAnimatedMeshResourceManifest;
export interface AshaRendererAnimatedMeshFrameReceipt {
    readonly applied: boolean;
    readonly diagnostics: readonly AshaRendererHostDiagnostic[];
}
export interface AshaRendererAnimatedMeshPoseSample {
    readonly rootTranslation: readonly [number, number, number];
    readonly rootRotation: readonly [number, number, number, number];
    readonly rootScale: readonly [number, number, number];
    readonly hierarchyNodeCount: number;
    readonly hierarchyTranslationSum: readonly [number, number, number];
    readonly hierarchyRotationSum: readonly [number, number, number, number];
    readonly hierarchyScaleSum: readonly [number, number, number];
}
export interface AshaRendererAnimatedMeshPlaybackReadout {
    readonly handle: RenderHandle;
    readonly asset: string | null;
    readonly status: 'unavailable' | 'not_started' | 'playing' | 'paused' | 'stopped';
    readonly selectedClip: string | null;
    readonly mixerTimeSeconds: number;
    readonly actionTimeSeconds: number | null;
    readonly commandSelected: boolean;
    readonly running: boolean;
    readonly paused: boolean;
    readonly loop: 'once' | 'repeat' | 'pingPong' | null;
    readonly speed: number | null;
    readonly weight: number | null;
    readonly poseSample: AshaRendererAnimatedMeshPoseSample | null;
    readonly diagnostics: readonly AshaRendererHostDiagnostic[];
    readonly projectionOnly: true;
}
export interface AshaRendererAnimatedMeshProjection {
    readonly kind: 'asha_renderer_animated_mesh_projection.v0';
    readonly applyFrame: (frame: RenderFrameDiff) => AshaRendererAnimatedMeshFrameReceipt;
    readonly advance: (deltaSeconds: number) => AshaRendererAnimatedMeshFrameReceipt;
    readonly playback: (handle: RenderHandle) => AshaRendererAnimatedMeshPlaybackReadout;
    readonly snapshot: () => string;
}
export interface AshaRendererAnimatedMeshProjectionOptions {
    readonly manifest: AshaRendererAnimatedMeshResourceManifest;
    readonly resolveResource?: AshaRendererAnimatedMeshResourceResolver;
}
export declare function createAshaRendererAnimatedMeshProjection(options: AshaRendererAnimatedMeshProjectionOptions): Promise<AshaRendererAnimatedMeshProjection>;
export declare function loadRendererAnimatedMeshSource(manifest: AshaRendererAnimatedMeshResourceManifest, resolver?: AshaRendererAnimatedMeshResourceResolver): Promise<unknown>;
export declare function animationPlaybackReadout(handle: RenderHandle, readout: BackendAnimatedMeshPlaybackReadout | undefined): AshaRendererAnimatedMeshPlaybackReadout;
interface BackendAnimatedMeshPlaybackReadout {
    readonly asset: string;
    readonly status: 'not_started' | 'playing' | 'paused' | 'stopped';
    readonly currentClip: string | null;
    readonly mixerTimeSeconds: number;
    readonly actionTimeSeconds: number | null;
    readonly commandSelected: boolean;
    readonly running: boolean;
    readonly paused: boolean;
    readonly loop: 'once' | 'repeat' | 'pingPong' | null;
    readonly speed: number | null;
    readonly weight: number | null;
    readonly poseSample: AshaRendererAnimatedMeshPoseSample;
    readonly diagnostics: readonly string[];
}
export {};
//# sourceMappingURL=animated-mesh-host.d.ts.map
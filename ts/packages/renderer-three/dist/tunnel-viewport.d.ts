import { type CameraProjectionSnapshot, type CollisionAxis, type RenderFrameDiff } from '@asha/contracts';
import type { GeneratedTunnelReadout } from '@asha/runtime-bridge';
export declare const FIRST_PERSON_TUNNEL_VIEWPORT_FIXTURE_NAME = "generated-tunnel-first-person-viewport";
export type TunnelViewportVec3 = readonly [number, number, number];
export type TunnelViewportColor = readonly [number, number, number, number];
export type TunnelViewportMaterialRole = 'wall' | 'floor' | 'accent' | 'playerMarker' | 'exitMarker';
export interface TunnelViewportMaterialPalette {
    readonly wall: TunnelViewportColor;
    readonly floor: TunnelViewportColor;
    readonly accent: TunnelViewportColor;
    readonly playerMarker: TunnelViewportColor;
    readonly exitMarker: TunnelViewportColor;
}
export interface FirstPersonTunnelViewportCollisionDebug {
    readonly collided: boolean;
    readonly blockedAxes: readonly CollisionAxis[];
    readonly worldHash: string;
    readonly collisionProjectionHash: string;
    readonly movementHash: string;
}
export interface FirstPersonTunnelViewportInput {
    readonly tunnel: GeneratedTunnelReadout;
    readonly camera: CameraProjectionSnapshot;
    readonly materials?: Partial<TunnelViewportMaterialPalette>;
    readonly collision?: FirstPersonTunnelViewportCollisionDebug | null;
}
export interface FirstPersonTunnelViewportSummary {
    readonly kind: 'first_person_tunnel_viewport.v0';
    readonly fixture: typeof FIRST_PERSON_TUNNEL_VIEWPORT_FIXTURE_NAME;
    readonly presetId: GeneratedTunnelReadout['generator']['presetId'];
    readonly seed: GeneratedTunnelReadout['generator']['seed'];
    readonly camera: {
        readonly camera: CameraProjectionSnapshot['camera'];
        readonly tick: number;
        readonly position: TunnelViewportVec3;
        readonly yawDegrees: number;
        readonly pitchDegrees: number;
        readonly projectionHash: string;
        readonly viewport: {
            readonly width: number;
            readonly height: number;
        };
    };
    readonly tunnel: {
        readonly dims: GeneratedTunnelReadout['volume']['tunnelDims'];
        readonly solidVoxels: number;
        readonly spawnMarkers: readonly string[];
        readonly materialRoles: readonly string[];
    };
    readonly debug: {
        readonly generatorHash: string;
        readonly outputHash: string;
        readonly renderProjectionHash: string;
        readonly collisionProjectionHash: string;
        readonly replayHash: string;
        readonly collision: FirstPersonTunnelViewportCollisionDebug | null;
    };
    readonly scene: {
        readonly frameHash: string;
        readonly structuralHash: string;
        readonly opCount: number;
        readonly instanceCount: number;
    };
    readonly nonClaims: readonly [
        'not_runtime_authority',
        'not_collision_authority',
        'not_local_generation',
        'not_pixel_golden'
    ];
}
export declare function createGeneratedTunnelViewportFrame(tunnel: GeneratedTunnelReadout, materials?: Partial<TunnelViewportMaterialPalette>): RenderFrameDiff;
export declare function summarizeFirstPersonTunnelViewport(input: {
    readonly tunnel: GeneratedTunnelReadout;
    readonly camera: CameraProjectionSnapshot;
    readonly frame: RenderFrameDiff;
    readonly structuralSnapshot?: string;
    readonly collision?: FirstPersonTunnelViewportCollisionDebug | null;
}): FirstPersonTunnelViewportSummary;
//# sourceMappingURL=tunnel-viewport.d.ts.map
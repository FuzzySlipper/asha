export interface RuntimeSessionEcrpRenderTargetIdentity {
    readonly kind: 'runtime_session.ecrp_render_target.v0';
    readonly targetId: string;
    readonly entity: number;
    readonly definitionStableId: string;
    readonly displayName: string;
    readonly source: {
        readonly projectBundle: string;
        readonly relativePath: string;
    };
    readonly role: 'player' | 'enemy' | 'neutral';
    readonly projection: 'first_person_camera' | 'target_cube' | 'spawn_marker';
    readonly renderLabel: string;
    readonly renderHandle: number | null;
    readonly visible: boolean;
    readonly position: readonly [number, number, number];
    readonly yawDegrees: number;
    readonly pitchDegrees: number;
    readonly scale: readonly [number, number, number] | null;
    readonly targetHash: string;
}
//# sourceMappingURL=ecrp-render-target.d.ts.map
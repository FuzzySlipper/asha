import type { BillboardProjectionDiagnostic, BillboardProjectionReadout, PresentationFrameDiff } from '@asha/contracts';
type Vec3 = readonly [number, number, number];
export interface AshaBillboardResource {
    readonly bytes: ArrayBuffer;
    readonly url?: string;
}
export type AshaBillboardResourceResolver = (asset: string) => Promise<AshaBillboardResource | null>;
export type AshaBillboardEntityPositionResolver = (entity: number) => Vec3 | null;
export interface AshaBillboardScreenProjection {
    readonly xPixels: number;
    readonly yPixels: number;
    readonly depth: number;
    readonly distance: number;
    readonly insideViewport: boolean;
    readonly occluded: boolean;
}
export type AshaBillboardWorldProjector = (position: Vec3) => AshaBillboardScreenProjection;
export type AshaBillboardLocalizer = (key: string, fallback: string, argumentsByName: Readonly<Record<string, string>>) => string;
export interface AshaBillboardElementStyle {
    backgroundColor: string;
    backgroundImage: string;
    backgroundPosition: string;
    backgroundRepeat: string;
    backgroundSize: string;
    borderRadius: string;
    color: string;
    display: string;
    fontFamily: string;
    fontSize: string;
    left: string;
    lineHeight: string;
    pointerEvents: string;
    position: string;
    top: string;
    transform: string;
    whiteSpace: string;
    zIndex: string;
}
export interface AshaBillboardElement {
    readonly style: AshaBillboardElementStyle;
    textContent: string | null;
    setAttribute(name: string, value: string): void;
    remove(): void;
}
export interface AshaBillboardContainer {
    appendChild(element: AshaBillboardElement): unknown;
}
export type AshaBillboardElementFactory = () => AshaBillboardElement;
export type AshaBillboardFontLoader = (family: string, bytes: ArrayBuffer) => Promise<void>;
export interface AshaBillboardHostOptions {
    readonly container: AshaBillboardContainer;
    readonly createElement?: AshaBillboardElementFactory;
    readonly loadFont?: AshaBillboardFontLoader;
    readonly localize?: AshaBillboardLocalizer;
    readonly projectWorld: AshaBillboardWorldProjector;
    readonly resolveEntityPosition: AshaBillboardEntityPositionResolver;
    readonly resolveResource?: AshaBillboardResourceResolver;
}
export interface AshaBillboardFrameReceipt {
    readonly applied: number;
    readonly diagnostics: readonly BillboardProjectionDiagnostic[];
    readonly readout: BillboardProjectionReadout;
}
export declare class AshaBillboardHost {
    #private;
    constructor(options: AshaBillboardHostOptions);
    applyPresentation(frame: PresentationFrameDiff): Promise<AshaBillboardFrameReceipt>;
    refreshLayout(): readonly BillboardProjectionDiagnostic[];
    readout(): BillboardProjectionReadout;
    cleanup(): void;
    dispose(): void;
}
export {};
//# sourceMappingURL=billboard-host.d.ts.map
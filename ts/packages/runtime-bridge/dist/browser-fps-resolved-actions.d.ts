import type { ResolvedInputAction } from '@asha/contracts';
export interface BrowserFpsResolvedFrame {
    readonly moveForward: number;
    readonly moveRight: number;
    readonly pitchDeltaPixels: number;
    readonly yawDeltaPixels: number;
    readonly primaryFirePressed: boolean;
}
export declare class BrowserFpsResolvedActionConsumer {
    #private;
    accept(action: ResolvedInputAction): void;
    drain(): BrowserFpsResolvedFrame;
    reset(): void;
}
//# sourceMappingURL=browser-fps-resolved-actions.d.ts.map
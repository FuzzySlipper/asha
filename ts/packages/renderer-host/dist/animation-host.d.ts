import type { AnimationProjectionDiagnostic, AnimationProjectionReadout, PresentationFrameDiff } from '@asha/contracts';
import type { AshaRendererAnimatedMeshProjection } from './animated-mesh-host.js';
export interface AshaAnimationFrameReceipt {
    readonly applied: number;
    readonly diagnostics: readonly AnimationProjectionDiagnostic[];
    readonly readout: AnimationProjectionReadout;
}
export declare class AshaAnimationHost {
    #private;
    constructor(projection: AshaRendererAnimatedMeshProjection);
    applyPresentation(frame: PresentationFrameDiff): AshaAnimationFrameReceipt;
    advance(deltaSeconds: number): AshaAnimationFrameReceipt;
    readout(): AnimationProjectionReadout;
}
//# sourceMappingURL=animation-host.d.ts.map
import type { AshaRendererAnimatedMeshProjection, AshaRendererSurface } from '@asha/renderer-host';
import type { RuntimeSessionFacade } from '@asha/runtime-session';
export interface RendererHostRuntimeSessionConsumerReadout {
    readonly projectionStatus: ReturnType<AshaRendererAnimatedMeshProjection['playback']>['status'];
    readonly surfaceStatus: ReturnType<AshaRendererSurface['animatedMeshPlayback']>['status'];
}
export declare function consumeRuntimeSessionAnimationIntent(session: Pick<RuntimeSessionFacade, 'readAnimationIntent'>, projection: AshaRendererAnimatedMeshProjection, surface: AshaRendererSurface): RendererHostRuntimeSessionConsumerReadout;
//# sourceMappingURL=renderer-host-runtime-session-consumer-proof.d.ts.map
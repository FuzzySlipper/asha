import type {
  AshaRendererAnimatedMeshProjection,
  AshaRendererSurface,
} from '@asha/renderer-host';
import type {
  RuntimeSessionAnimationIntentReadout,
  RuntimeSessionFacade,
} from '@asha/runtime-session';

export interface RendererHostRuntimeSessionConsumerReadout {
  readonly projectionStatus: ReturnType<AshaRendererAnimatedMeshProjection['playback']>['status'];
  readonly surfaceStatus: ReturnType<AshaRendererSurface['animatedMeshPlayback']>['status'];
}

export function consumeRuntimeSessionAnimationIntent(
  session: Pick<RuntimeSessionFacade, 'readAnimationIntent'>,
  projection: AshaRendererAnimatedMeshProjection,
  surface: AshaRendererSurface,
): RendererHostRuntimeSessionConsumerReadout {
  const intent: RuntimeSessionAnimationIntentReadout = session.readAnimationIntent();
  projection.applyFrame(intent.frame);
  surface.applyFrame(intent.frame);
  return {
    projectionStatus: projection.playback(intent.instanceHandle).status,
    surfaceStatus: surface.animatedMeshPlayback(intent.instanceHandle).status,
  };
}

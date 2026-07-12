import type {
  AnimationControllerProjectionState,
  AnimationProjectionDiagnostic,
  AnimationProjectionHandle,
  AnimationProjectionReadout,
  AnimationResolvedMotion,
  PresentationFrameDiff,
  PresentationOp,
  RenderHandle,
} from '@asha/contracts';
import type {
  AshaRendererAnimatedMeshProjection,
  AshaRendererAnimationControllerClip,
} from './animated-mesh-host.js';

type AnimationPresentationOp = Extract<PresentationOp, { readonly domain: 'animation' }>;

export interface AshaAnimationFrameReceipt {
  readonly applied: number;
  readonly diagnostics: readonly AnimationProjectionDiagnostic[];
  readonly readout: AnimationProjectionReadout;
}

interface AnimationControllerRealization {
  readonly handle: AnimationProjectionHandle;
  readonly target: RenderHandle;
  readonly asset: string;
  readonly tickDurationSeconds: number;
  controller: AnimationControllerProjectionState;
  presented: readonly AshaRendererAnimationControllerClip[];
  interpolation: AnimationWeightInterpolation | null;
}

interface AnimationWeightInterpolation {
  readonly from: readonly AshaRendererAnimationControllerClip[];
  readonly to: readonly AshaRendererAnimationControllerClip[];
  readonly durationSeconds: number;
  elapsedSeconds: number;
}

export class AshaAnimationHost {
  readonly #projection: AshaRendererAnimatedMeshProjection;
  readonly #controllers = new Map<AnimationProjectionHandle, AnimationControllerRealization>();
  readonly #diagnostics: AnimationProjectionDiagnostic[] = [];
  #sampledFrames = 0;
  #compatibilityFallbacks = 0;

  constructor(projection: AshaRendererAnimatedMeshProjection) {
    this.#projection = projection;
  }

  applyPresentation(frame: PresentationFrameDiff): AshaAnimationFrameReceipt {
    const diagnostics: AnimationProjectionDiagnostic[] = [];
    let applied = 0;
    for (const operation of frame.ops) {
      if (operation.domain !== 'animation') {
        continue;
      }
      const diagnostic = this.#applyOperation(operation);
      if (diagnostic === null) {
        applied += 1;
      } else {
        diagnostics.push(diagnostic);
        this.#diagnostics.push(diagnostic);
      }
    }
    return { applied, diagnostics, readout: this.readout() };
  }

  advance(deltaSeconds: number): AshaAnimationFrameReceipt {
    if (!Number.isFinite(deltaSeconds) || deltaSeconds < 0) {
      throw new Error('animation host deltaSeconds must be finite and non-negative');
    }
    const diagnostics: AnimationProjectionDiagnostic[] = [];
    for (const realization of this.#controllers.values()) {
      const interpolation = realization.interpolation;
      if (interpolation === null) {
        continue;
      }
      interpolation.elapsedSeconds = Math.min(
        interpolation.durationSeconds,
        interpolation.elapsedSeconds + deltaSeconds,
      );
      const progress = interpolation.durationSeconds === 0
        ? 1
        : interpolation.elapsedSeconds / interpolation.durationSeconds;
      realization.presented = interpolateWeights(interpolation.from, interpolation.to, progress);
      try {
        this.#projection.setAnimationControllerWeights(
          realization.target,
          realization.presented,
        );
      } catch (cause) {
        const diagnostic = animationDiagnostic(
          'hostFailure',
          0,
          realization.handle,
          realization.target,
          errorMessage(cause),
          null,
        );
        diagnostics.push(diagnostic);
        this.#diagnostics.push(diagnostic);
      }
      if (progress === 1) {
        realization.interpolation = null;
      }
    }
    this.#projection.advance(deltaSeconds);
    this.#sampledFrames += 1;
    return { applied: this.#controllers.size, diagnostics, readout: this.readout() };
  }

  readout(): AnimationProjectionReadout {
    return {
      activeControllers: this.#controllers.size,
      sampledFrames: this.#sampledFrames,
      compatibilityFallbacks: this.#compatibilityFallbacks,
      diagnostics: [...this.#diagnostics],
    };
  }

  #applyOperation(operation: AnimationPresentationOp): AnimationProjectionDiagnostic | null {
    const { op, meta } = operation;
    if (op.op === 'create') {
      if (this.#controllers.has(op.handle)) {
        return animationDiagnostic('duplicateHandle', meta.sequence, op.handle, op.descriptor.target, 'animation handle already exists', meta.origin);
      }
      const validation = validateController(op.descriptor.controller);
      if (validation !== null || op.descriptor.tickDurationMillis === 0) {
        return animationDiagnostic('invalidDescriptor', meta.sequence, op.handle, op.descriptor.target, validation ?? 'tick duration must be non-zero', meta.origin);
      }
      if (!this.#projection.hasAnimationTarget(op.descriptor.target)) {
        return animationDiagnostic('unknownTarget', meta.sequence, op.handle, op.descriptor.target, 'animation target is unavailable', meta.origin);
      }
      const playback = this.#projection.playback(op.descriptor.target);
      if (playback.asset === null) {
        return animationDiagnostic('assetMissing', meta.sequence, op.handle, op.descriptor.target, 'animation target has no loaded asset', meta.origin);
      }
      if (playback.asset !== op.descriptor.asset) {
        return animationDiagnostic('incompatibleRig', meta.sequence, op.handle, op.descriptor.target, 'animation descriptor asset does not match the target rig', meta.origin);
      }
      const weights = controllerWeights(op.descriptor.controller);
      if (!this.#projection.hasAnimationClips(op.descriptor.target, weights.map((clip) => clip.clip))) {
        return animationDiagnostic('clipMissing', meta.sequence, op.handle, op.descriptor.target, 'controller references an unavailable clip', meta.origin);
      }
      try {
        this.#projection.setAnimationControllerWeights(op.descriptor.target, weights);
      } catch (cause) {
        return hostDiagnostic(cause, meta.sequence, op.handle, op.descriptor.target, meta.origin);
      }
      this.#controllers.set(op.handle, {
        handle: op.handle,
        target: op.descriptor.target,
        asset: op.descriptor.asset,
        tickDurationSeconds: op.descriptor.tickDurationMillis / 1_000,
        controller: op.descriptor.controller,
        presented: weights,
        interpolation: null,
      });
      return null;
    }
    const realization = this.#controllers.get(op.handle);
    if (realization === undefined) {
      return animationDiagnostic('unknownHandle', meta.sequence, op.handle, null, 'animation handle is unavailable', meta.origin);
    }
    if (op.op === 'destroy') {
      try {
        this.#projection.clearAnimationControllerWeights(realization.target);
      } catch (cause) {
        return hostDiagnostic(cause, meta.sequence, op.handle, realization.target, meta.origin);
      }
      this.#controllers.delete(op.handle);
      return null;
    }
    const validation = validateController(op.controller);
    if (validation !== null) {
      return animationDiagnostic('invalidDescriptor', meta.sequence, op.handle, realization.target, validation, meta.origin);
    }
    if (op.controller.revision < realization.controller.revision) {
      return animationDiagnostic('staleRevision', meta.sequence, op.handle, realization.target, 'controller revision moved backward', meta.origin);
    }
    if (
      op.controller.revision === realization.controller.revision
      && op.controller.stateHash !== realization.controller.stateHash
    ) {
      return animationDiagnostic('staleRevision', meta.sequence, op.handle, realization.target, 'controller state changed without a revision change', meta.origin);
    }
    const target = controllerWeights(op.controller);
    if (!this.#projection.hasAnimationClips(realization.target, target.map((clip) => clip.clip))) {
      return animationDiagnostic('clipMissing', meta.sequence, op.handle, realization.target, 'controller references an unavailable clip', meta.origin);
    }
    realization.controller = op.controller;
    realization.interpolation = {
      from: realization.presented,
      to: target,
      durationSeconds: realization.tickDurationSeconds,
      elapsedSeconds: 0,
    };
    return null;
  }
}

function validateController(controller: AnimationControllerProjectionState): string | null {
  const motions = [controller.motion, controller.transition?.targetMotion].filter(
    (motion): motion is AnimationResolvedMotion => motion !== undefined,
  );
  for (const motion of motions) {
    if (
      motion.clipA.length === 0
      || motion.blendWeightMilli < 0
      || motion.blendWeightMilli > 1_000
      || motion.speedMilli <= 0
      || (motion.clipB === null && motion.blendWeightMilli !== 0)
    ) {
      return 'controller motion is invalid';
    }
  }
  const transition = controller.transition;
  if (
    transition !== null
    && (transition.durationTicks === 0 || transition.elapsedTicks > transition.durationTicks)
  ) {
    return 'controller transition progress is invalid';
  }
  return null;
}

function controllerWeights(
  controller: AnimationControllerProjectionState,
): readonly AshaRendererAnimationControllerClip[] {
  const transition = controller.transition;
  if (transition === null) {
    return motionWeights(controller.motion);
  }
  const progress = transition.elapsedTicks / transition.durationTicks;
  return mergeWeights([
    ...motionWeights(controller.motion).map((clip) => ({ ...clip, weight: clip.weight * (1 - progress) })),
    ...motionWeights(transition.targetMotion).map((clip) => ({ ...clip, weight: clip.weight * progress })),
  ]);
}

function motionWeights(motion: AnimationResolvedMotion): readonly AshaRendererAnimationControllerClip[] {
  const highWeight = motion.clipB === null ? 0 : motion.blendWeightMilli / 1_000;
  const clips: AshaRendererAnimationControllerClip[] = [{
    clip: motion.clipA,
    weight: 1 - highWeight,
    speed: motion.speedMilli / 1_000,
  }];
  if (motion.clipB !== null && highWeight > 0) {
    clips.push({ clip: motion.clipB, weight: highWeight, speed: motion.speedMilli / 1_000 });
  }
  return clips;
}

function mergeWeights(
  clips: readonly AshaRendererAnimationControllerClip[],
): readonly AshaRendererAnimationControllerClip[] {
  const merged = new Map<string, AshaRendererAnimationControllerClip>();
  for (const clip of clips) {
    if (clip.weight <= 0) {
      continue;
    }
    const prior = merged.get(clip.clip);
    merged.set(clip.clip, {
      clip: clip.clip,
      weight: (prior?.weight ?? 0) + clip.weight,
      speed: clip.speed,
    });
  }
  return [...merged.values()].sort((left, right) => left.clip.localeCompare(right.clip));
}

function interpolateWeights(
  from: readonly AshaRendererAnimationControllerClip[],
  to: readonly AshaRendererAnimationControllerClip[],
  progress: number,
): readonly AshaRendererAnimationControllerClip[] {
  const clips = new Set([...from.map((clip) => clip.clip), ...to.map((clip) => clip.clip)]);
  return mergeWeights([...clips].map((clip) => {
    const prior = from.find((value) => value.clip === clip);
    const next = to.find((value) => value.clip === clip);
    return {
      clip,
      weight: (prior?.weight ?? 0) + ((next?.weight ?? 0) - (prior?.weight ?? 0)) * progress,
      speed: next?.speed ?? prior?.speed ?? 1,
    };
  }));
}

function hostDiagnostic(
  cause: unknown,
  sequence: number,
  handle: AnimationProjectionHandle,
  target: RenderHandle,
  origin: AnimationPresentationOp['meta']['origin'],
): AnimationProjectionDiagnostic {
  const message = errorMessage(cause);
  const code = message.includes('missing clip') ? 'clipMissing' : message.includes('handle') ? 'unknownTarget' : 'hostFailure';
  return animationDiagnostic(code, sequence, handle, target, message, origin);
}

function animationDiagnostic(
  code: AnimationProjectionDiagnostic['code'],
  sequence: number,
  handle: AnimationProjectionHandle | null,
  target: RenderHandle | null,
  message: string,
  origin: AnimationPresentationOp['meta']['origin'],
): AnimationProjectionDiagnostic {
  return { code, sequence, handle, target, message, origin };
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

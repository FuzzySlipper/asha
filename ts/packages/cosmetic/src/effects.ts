import type { PresentationOp, RenderFrameDiff } from '@asha/contracts';

export type CosmeticEffectKind = 'screen_flash' | 'hit_spark' | 'view_kick';

export type CosmeticEffectDiagnosticCode =
  | 'invalidDuration'
  | 'invalidIntensity'
  | 'invalidStartTick'
  | 'missingEffectId';

export type CosmeticSource =
  | {
      readonly kind: 'render_frame_diff';
      readonly renderOpCount: number;
      readonly renderOpKinds: readonly string[];
    }
  | {
      readonly kind: 'local_ui_event';
      readonly eventId: string;
    }
  | {
      readonly kind: 'particle_projection';
      readonly signalId: string;
      readonly originId: string | null;
    };

export type CosmeticEffectDescriptor = {
  readonly effectId: string;
  readonly kind: CosmeticEffectKind;
  readonly source: CosmeticSource;
  readonly startsAtTick: number;
  readonly durationTicks: number;
  readonly intensity: number;
  readonly color: readonly [number, number, number, number] | null;
  readonly anchor: readonly [number, number, number] | null;
  readonly replayScope: 'excluded_from_replay_truth';
};

export type CosmeticEffectViewModel = {
  readonly effectId: string;
  readonly kind: CosmeticEffectKind;
  readonly active: boolean;
  readonly progress: number;
  readonly opacity: number;
  readonly intensity: number;
  readonly color: readonly [number, number, number, number] | null;
  readonly anchor: readonly [number, number, number] | null;
};

export type CosmeticFrameViewModel = {
  readonly kind: 'cosmetic_frame_view_model.v0';
  readonly tick: number;
  readonly effects: readonly CosmeticEffectViewModel[];
  readonly diagnostics: readonly CosmeticEffectDiagnostic[];
  readonly nonAuthority: CosmeticNonAuthorityReadout;
};

export type CosmeticEffectDiagnostic = {
  readonly code: CosmeticEffectDiagnosticCode;
  readonly effectId: string | null;
  readonly detail: string;
};

export type CosmeticNonAuthorityReadout = {
  readonly kind: 'cosmetic_non_authority_readout.v0';
  readonly commandCount: 0;
  readonly replayRecordCount: 0;
  readonly authoritativeMutationCount: 0;
  readonly rendererBackendCoupling: false;
  readonly runtimeTruth: 'not_authoritative';
};

export type CosmeticAuthorityBoundary = {
  readonly packageRole: '@asha/cosmetic';
  readonly owns: readonly ['transient_effect_descriptors', 'local_view_models'];
  readonly consumes: readonly ['generated_render_frame_diff_descriptors', 'generated_particle_projection', 'local_ui_events'];
  readonly doesNotProduce: readonly ['authority_commands', 'replay_records', 'state_mutations', 'renderer_backend_calls'];
};

export type ScreenFlashInput = {
  readonly effectId: string;
  readonly renderFrame: RenderFrameDiff;
  readonly startsAtTick: number;
  readonly durationTicks: number;
  readonly intensity: number;
  readonly color?: readonly [number, number, number, number] | null;
};

export type HitSparkInput = {
  readonly effectId: string;
  readonly sourceEventId: string;
  readonly startsAtTick: number;
  readonly durationTicks: number;
  readonly intensity: number;
  readonly anchor: readonly [number, number, number];
  readonly color?: readonly [number, number, number, number] | null;
};

export type ParticleHitSparkAdapterInput = {
  readonly operation: Extract<PresentationOp, { readonly domain: 'particle' }>;
  readonly startsAtTick: number;
  readonly ticksPerSecond: number;
  readonly resolveEntityPosition?: (
    entity: number,
  ) => readonly [number, number, number] | null;
};

export const COSMETIC_AUTHORITY_BOUNDARY: CosmeticAuthorityBoundary = {
  packageRole: '@asha/cosmetic',
  owns: ['transient_effect_descriptors', 'local_view_models'],
  consumes: [
    'generated_render_frame_diff_descriptors',
    'generated_particle_projection',
    'local_ui_events',
  ],
  doesNotProduce: ['authority_commands', 'replay_records', 'state_mutations', 'renderer_backend_calls'],
};

export const COSMETIC_NON_AUTHORITY_READOUT: CosmeticNonAuthorityReadout = {
  kind: 'cosmetic_non_authority_readout.v0',
  commandCount: 0,
  replayRecordCount: 0,
  authoritativeMutationCount: 0,
  rendererBackendCoupling: false,
  runtimeTruth: 'not_authoritative',
};

export function createScreenFlashDescriptor(input: ScreenFlashInput): CosmeticEffectDescriptor {
  return {
    effectId: input.effectId,
    kind: 'screen_flash',
    source: renderFrameSource(input.renderFrame),
    startsAtTick: input.startsAtTick,
    durationTicks: input.durationTicks,
    intensity: input.intensity,
    color: input.color ?? [1, 1, 1, 1],
    anchor: null,
    replayScope: 'excluded_from_replay_truth',
  };
}

export function createHitSparkDescriptor(input: HitSparkInput): CosmeticEffectDescriptor {
  return {
    effectId: input.effectId,
    kind: 'hit_spark',
    source: {
      kind: 'local_ui_event',
      eventId: input.sourceEventId,
    },
    startsAtTick: input.startsAtTick,
    durationTicks: input.durationTicks,
    intensity: input.intensity,
    color: input.color ?? [1, 0.85, 0.35, 1],
    anchor: input.anchor,
    replayScope: 'excluded_from_replay_truth',
  };
}

export function adaptParticleBurstToHitSparkDescriptor(
  input: ParticleHitSparkAdapterInput,
): CosmeticEffectDescriptor | null {
  if (input.operation.op.op !== 'emit') {
    return null;
  }
  const descriptor = input.operation.op.descriptor;
  const anchor = descriptor.anchor.kind === 'world'
    ? descriptor.anchor.position
    : resolveAttachedParticleAnchor(
        descriptor.anchor.entity,
        descriptor.anchor.offset,
        input.resolveEntityPosition,
      );
  if (anchor === null || !Number.isFinite(input.ticksPerSecond) || input.ticksPerSecond <= 0) {
    return null;
  }
  const lifetime = descriptor.lifetimeSeconds[1];
  return {
    effectId: `particle:${input.operation.op.signalId}`,
    kind: 'hit_spark',
    source: {
      kind: 'particle_projection',
      signalId: input.operation.op.signalId,
      originId: input.operation.meta.origin?.id ?? null,
    },
    startsAtTick: input.startsAtTick,
    durationTicks: Math.max(1, Math.ceil(lifetime * input.ticksPerSecond)),
    intensity: Math.min(1, descriptor.burstCount / 16),
    color: descriptor.colorCurve[0]?.color ?? [1, 0.85, 0.35, 1],
    anchor,
    replayScope: 'excluded_from_replay_truth',
  };
}

function resolveAttachedParticleAnchor(
  entity: number,
  offset: readonly [number, number, number],
  resolveEntityPosition:
    | ((entity: number) => readonly [number, number, number] | null)
    | undefined,
): readonly [number, number, number] | null {
  const position = resolveEntityPosition?.(entity) ?? null;
  return position === null
    ? null
    : [
        position[0] + offset[0],
        position[1] + offset[1],
        position[2] + offset[2],
      ];
}

export function projectCosmeticFrame(
  descriptors: readonly CosmeticEffectDescriptor[],
  tick: number,
): CosmeticFrameViewModel {
  const diagnostics = descriptors.flatMap(validateDescriptor);
  const validDescriptors = [...descriptors]
    .filter((descriptor) => validateDescriptor(descriptor).length === 0)
    .sort(compareDescriptors);
  const effects = validDescriptors.map((descriptor) => projectEffect(descriptor, tick));

  return {
    kind: 'cosmetic_frame_view_model.v0',
    tick,
    effects,
    diagnostics,
    nonAuthority: COSMETIC_NON_AUTHORITY_READOUT,
  };
}

export function validateCosmeticEffectDescriptor(
  descriptor: CosmeticEffectDescriptor,
): readonly CosmeticEffectDiagnostic[] {
  return validateDescriptor(descriptor);
}

export function readCosmeticAuthorityBoundary(): CosmeticAuthorityBoundary {
  return COSMETIC_AUTHORITY_BOUNDARY;
}

function renderFrameSource(frame: RenderFrameDiff): CosmeticSource {
  return {
    kind: 'render_frame_diff',
    renderOpCount: frame.ops.length,
    renderOpKinds: frame.ops.map((op) => op.op),
  };
}

function projectEffect(descriptor: CosmeticEffectDescriptor, tick: number): CosmeticEffectViewModel {
  const elapsedTicks = tick - descriptor.startsAtTick;
  const progress = clamp(elapsedTicks / descriptor.durationTicks, 0, 1);
  const active = elapsedTicks >= 0 && elapsedTicks < descriptor.durationTicks;
  const fadeOut = 1 - progress;
  const opacity = active ? roundToThree(clamp(descriptor.intensity * fadeOut, 0, 1)) : 0;

  return {
    effectId: descriptor.effectId,
    kind: descriptor.kind,
    active,
    progress: roundToThree(progress),
    opacity,
    intensity: descriptor.intensity,
    color: descriptor.color,
    anchor: descriptor.anchor,
  };
}

function validateDescriptor(descriptor: CosmeticEffectDescriptor): readonly CosmeticEffectDiagnostic[] {
  const diagnostics: CosmeticEffectDiagnostic[] = [];

  if (descriptor.effectId.trim().length === 0) {
    diagnostics.push({
      code: 'missingEffectId',
      effectId: null,
      detail: 'Cosmetic effect id must not be blank',
    });
  }
  if (!Number.isSafeInteger(descriptor.startsAtTick) || descriptor.startsAtTick < 0) {
    diagnostics.push({
      code: 'invalidStartTick',
      effectId: descriptor.effectId,
      detail: 'Cosmetic effect start tick must be a non-negative safe integer',
    });
  }
  if (!Number.isSafeInteger(descriptor.durationTicks) || descriptor.durationTicks <= 0) {
    diagnostics.push({
      code: 'invalidDuration',
      effectId: descriptor.effectId,
      detail: 'Cosmetic effect duration must be a positive safe integer tick count',
    });
  }
  if (!Number.isFinite(descriptor.intensity) || descriptor.intensity < 0 || descriptor.intensity > 1) {
    diagnostics.push({
      code: 'invalidIntensity',
      effectId: descriptor.effectId,
      detail: 'Cosmetic effect intensity must be in the inclusive range 0..1',
    });
  }

  return diagnostics;
}

function compareDescriptors(left: CosmeticEffectDescriptor, right: CosmeticEffectDescriptor): number {
  if (left.startsAtTick !== right.startsAtTick) {
    return left.startsAtTick - right.startsAtTick;
  }
  return left.effectId.localeCompare(right.effectId);
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function roundToThree(value: number): number {
  return Math.round(value * 1000) / 1000;
}

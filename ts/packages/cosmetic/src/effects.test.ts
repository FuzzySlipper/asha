import { test } from 'node:test';
import assert from 'node:assert/strict';

import type { PresentationOp, RenderFrameDiff } from '@asha/contracts';

import {
  COSMETIC_NON_AUTHORITY_READOUT,
  adaptParticleBurstToHitSparkDescriptor,
  createHitSparkDescriptor,
  createScreenFlashDescriptor,
  projectCosmeticFrame,
  readCosmeticAuthorityBoundary,
  validateCosmeticEffectDescriptor,
  type CosmeticEffectDescriptor,
} from './index.js';

const EMPTY_RENDER_FRAME: RenderFrameDiff = { ops: [] };

void test('screen flash descriptor consumes generated render frame descriptors only', () => {
  const descriptor = createScreenFlashDescriptor({
    effectId: 'screen-flash/hit-confirm',
    renderFrame: EMPTY_RENDER_FRAME,
    startsAtTick: 4,
    durationTicks: 8,
    intensity: 0.75,
  });

  assert.equal(descriptor.kind, 'screen_flash');
  assert.equal(descriptor.replayScope, 'excluded_from_replay_truth');
  assert.deepEqual(descriptor.source, {
    kind: 'render_frame_diff',
    renderOpCount: 0,
    renderOpKinds: [],
  });
  assert.deepEqual(validateCosmeticEffectDescriptor(descriptor), []);
});

void test('cosmetic frame projection is deterministic and sorted by tick then id', () => {
  const later = createHitSparkDescriptor({
    effectId: 'spark/b',
    sourceEventId: 'ui/fire-2',
    startsAtTick: 6,
    durationTicks: 4,
    intensity: 1,
    anchor: [2, 0, 1],
  });
  const earlier = createHitSparkDescriptor({
    effectId: 'spark/a',
    sourceEventId: 'ui/fire-1',
    startsAtTick: 4,
    durationTicks: 8,
    intensity: 0.5,
    anchor: [1, 0, 1],
  });

  const frame = projectCosmeticFrame([later, earlier], 6);

  assert.equal(frame.kind, 'cosmetic_frame_view_model.v0');
  assert.deepEqual(
    frame.effects.map((effect) => effect.effectId),
    ['spark/a', 'spark/b'],
  );
  assert.deepEqual(
    frame.effects.map((effect) => ({
      active: effect.active,
      progress: effect.progress,
      opacity: effect.opacity,
    })),
    [
      { active: true, progress: 0.25, opacity: 0.375 },
      { active: true, progress: 0, opacity: 1 },
    ],
  );
  assert.deepEqual(frame.diagnostics, []);
  assert.equal(frame.nonAuthority, COSMETIC_NON_AUTHORITY_READOUT);
});

void test('invalid cosmetic descriptors fail closed with diagnostics and no active view model', () => {
  const invalid = {
    ...createScreenFlashDescriptor({
      effectId: '',
      renderFrame: EMPTY_RENDER_FRAME,
      startsAtTick: 0,
      durationTicks: 1,
      intensity: 1,
    }),
    durationTicks: 0,
    startsAtTick: -1,
    intensity: 2,
  } satisfies CosmeticEffectDescriptor;

  const frame = projectCosmeticFrame([invalid], 0);
  const codes = frame.diagnostics.map((diagnostic) => diagnostic.code);

  assert.deepEqual(frame.effects, []);
  assert.deepEqual(codes, ['missingEffectId', 'invalidStartTick', 'invalidDuration', 'invalidIntensity']);
});

void test('cosmetic boundary does not expose authority commands or replay records', () => {
  const frame = projectCosmeticFrame(
    [
      createScreenFlashDescriptor({
        effectId: 'screen-flash/no-authority',
        renderFrame: EMPTY_RENDER_FRAME,
        startsAtTick: 1,
        durationTicks: 4,
        intensity: 0.25,
      }),
    ],
    2,
  );
  const boundary = readCosmeticAuthorityBoundary();

  assert.deepEqual(boundary.doesNotProduce, [
    'authority_commands',
    'replay_records',
    'state_mutations',
    'renderer_backend_calls',
  ]);
  assert.deepEqual(frame.nonAuthority, {
    kind: 'cosmetic_non_authority_readout.v0',
    commandCount: 0,
    replayRecordCount: 0,
    authoritativeMutationCount: 0,
    rendererBackendCoupling: false,
    runtimeTruth: 'not_authoritative',
  });
  assert.equal('commands' in frame, false);
  assert.equal('replayRecords' in frame, false);
});

void test('particle burst adapts one way into the existing hit-spark view model', () => {
  const operation: Extract<PresentationOp, { readonly domain: 'particle' }> = {
    domain: 'particle',
    meta: {
      sequence: 0,
      origin: {
        kind: 'gameplayEvent',
        id: 'combat.primary-fire.feedback:44',
        authorityTick: 9,
        causationId: 'combat.primary-fire:44',
        correlationId: 'fps.session:1',
      },
    },
    op: {
      op: 'emit',
      signalId: 'impact:44',
      descriptor: {
        anchor: { kind: 'entityAttached', entity: 42, offset: [0, 1, 0] },
        sprite: { asset: 'sprite/spark', contentHash: 'aabb', frameCount: 1 },
        ratePerSecond: 0,
        burstCount: 12,
        lifetimeSeconds: [0.2, 0.8],
        velocityMin: [-1, 1, -1],
        velocityMax: [1, 3, 1],
        acceleration: [0, -4, 0],
        sizeCurve: [{ age: 0, value: 0.2 }, { age: 1, value: 0 }],
        colorCurve: [
          { age: 0, color: [1, 0.8, 0.2, 1] },
          { age: 1, color: [1, 0.2, 0, 0] },
        ],
        flipbookFramesPerSecond: 0,
        seed: 44,
        maxParticles: 32,
        visible: true,
      },
    },
  };
  const adapted = adaptParticleBurstToHitSparkDescriptor({
    operation,
    startsAtTick: 9,
    ticksPerSecond: 60,
    resolveEntityPosition: () => [10, 11, 12],
  });
  assert.deepEqual(adapted, {
    effectId: 'particle:impact:44',
    kind: 'hit_spark',
    source: {
      kind: 'particle_projection',
      signalId: 'impact:44',
      originId: 'combat.primary-fire.feedback:44',
    },
    startsAtTick: 9,
    durationTicks: 48,
    intensity: 0.75,
    color: [1, 0.8, 0.2, 1],
    anchor: [10, 12, 12],
    replayScope: 'excluded_from_replay_truth',
  });
  assert.equal('commands' in (adapted ?? {}), false);
});

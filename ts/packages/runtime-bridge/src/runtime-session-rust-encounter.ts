import {
  buildEncounterDirectorReadout,
  type EncounterDirectorReadout,
  type EncounterDirectorState,
} from '@asha/runtime-session';
import {
  RuntimeBridgeError,
  type FpsEncounterDirectorSnapshot,
  type FpsEncounterLifecycleInput,
  type FpsEncounterStateReadout,
  type FpsEncounterTransitionResult,
} from './bridge.js';
import type { lifecycleStatusToEncounterLifecycle } from './runtime-session-lifecycle.js';

export function fpsEncounterLifecycleInput(
  lifecycle: ReturnType<typeof lifecycleStatusToEncounterLifecycle>,
): FpsEncounterLifecycleInput {
  return {
    outcomeKind: lifecycle.outcomeKind,
    terminal: lifecycle.terminal,
    enemyDead: lifecycle.enemyDead,
    playerDead: lifecycle.playerDead,
    lifecycleHash: lifecycle.lifecycleHash,
  };
}

export function encounterReadoutFromFpsSnapshot(input: {
  readonly snapshot: FpsEncounterDirectorSnapshot;
  readonly sequenceId: number;
  readonly tick: number;
  readonly sessionSeed: number;
  readonly sessionHash: string;
}): EncounterDirectorReadout {
  return buildEncounterDirectorReadout({
    state: fpsEncounterStateToReadoutState(input.snapshot.state),
    sequenceId: input.sequenceId,
    tick: input.tick,
    sessionSeed: input.sessionSeed,
    sessionHash: input.sessionHash,
    lifecycle: input.snapshot.lifecycle,
    authority: {
      source: input.snapshot.backend === 'native_rust' ? 'rust_bridge' : 'reference_bridge',
      backend: input.snapshot.backend,
      surface: input.snapshot.authoritySurface,
      mutationOwner: input.snapshot.mutationOwner,
      readSets: input.snapshot.readSets,
      workspaceTrace: input.snapshot.workspaceTrace,
    },
  });
}

export function encounterTransitionResultForReceipt(result: FpsEncounterTransitionResult): {
  readonly accepted: boolean;
  readonly state: EncounterDirectorState;
  readonly eventKind?: NonNullable<FpsEncounterTransitionResult['eventKind']>;
  readonly rejectionReason?: NonNullable<FpsEncounterTransitionResult['rejectionReason']>;
} {
  return {
    accepted: result.accepted,
    state: fpsEncounterStateToReadoutState(result.state),
    ...(result.eventKind === null ? {} : { eventKind: result.eventKind }),
    ...(result.rejectionReason === null ? {} : { rejectionReason: result.rejectionReason }),
  };
}

export function fpsEncounterStateToReadoutState(
  state: FpsEncounterStateReadout,
): EncounterDirectorState {
  return {
    presetId: requireGeneratedTunnelEncounterPreset(state.presetId),
    status: state.status,
    spawnedEnemyIds: generatedTunnelEncounterIds(state.spawnedEnemyIds),
    defeatedEnemyIds: generatedTunnelEncounterIds(state.defeatedEnemyIds),
    revision: state.revision,
    lastTransition: state.lastTransition,
  };
}

function requireGeneratedTunnelEncounterPreset(value: string): EncounterDirectorState['presetId'] {
  if (value !== 'generated-tunnel-small-encounter') {
    throw new RuntimeBridgeError('internal', `unsupported Rust encounter preset '${value}'`);
  }
  return value;
}

function generatedTunnelEncounterIds(ids: readonly string[]): EncounterDirectorState['spawnedEnemyIds'] {
  return ids.map((id) => {
    if (id !== 'encounter.generated_tunnel_small.wave_1.enemy_001') {
      throw new RuntimeBridgeError('internal', `unsupported Rust encounter instance '${id}'`);
    }
    return id;
  });
}

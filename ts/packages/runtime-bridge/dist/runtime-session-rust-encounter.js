import { buildEncounterDirectorReadout, } from '@asha/runtime-session';
import { RuntimeBridgeError, } from './bridge.js';
export function fpsEncounterLifecycleInput(lifecycle) {
    return {
        outcomeKind: lifecycle.outcomeKind,
        terminal: lifecycle.terminal,
        enemyDead: lifecycle.enemyDead,
        playerDead: lifecycle.playerDead,
        lifecycleHash: lifecycle.lifecycleHash,
    };
}
export function encounterReadoutFromFpsSnapshot(input) {
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
export function encounterTransitionResultForReceipt(result) {
    return {
        accepted: result.accepted,
        state: fpsEncounterStateToReadoutState(result.state),
        ...(result.eventKind === null ? {} : { eventKind: result.eventKind }),
        ...(result.rejectionReason === null ? {} : { rejectionReason: result.rejectionReason }),
    };
}
export function fpsEncounterStateToReadoutState(state) {
    return {
        presetId: requireGeneratedTunnelEncounterPreset(state.presetId),
        status: state.status,
        spawnedEnemyIds: generatedTunnelEncounterIds(state.spawnedEnemyIds),
        defeatedEnemyIds: generatedTunnelEncounterIds(state.defeatedEnemyIds),
        revision: state.revision,
        lastTransition: state.lastTransition,
    };
}
function requireGeneratedTunnelEncounterPreset(value) {
    if (value !== 'generated-tunnel-small-encounter') {
        throw new RuntimeBridgeError('internal', `unsupported Rust encounter preset '${value}'`);
    }
    return value;
}
function generatedTunnelEncounterIds(ids) {
    return ids.map((id) => {
        if (id !== 'encounter.generated_tunnel_small.wave_1.enemy_001') {
            throw new RuntimeBridgeError('internal', `unsupported Rust encounter instance '${id}'`);
        }
        return id;
    });
}
//# sourceMappingURL=runtime-session-rust-encounter.js.map
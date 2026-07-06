const DEFAULT_ENEMY_STABLE_ID = 'actor/generated-tunnel-enemy';
const DEFAULT_PLAYER_STABLE_ID = 'actor/demo-player';
export function readRuntimeSessionPlayableEncounterTick(session, request) {
    const shell = {
        paused: request.shell?.paused ?? false,
    };
    const enemyStableId = request.enemyStableId ?? DEFAULT_ENEMY_STABLE_ID;
    const playerStableId = request.playerStableId ?? DEFAULT_PLAYER_STABLE_ID;
    const blockedBase = {
        kind: 'runtime_session.playable_encounter_tick.v0',
        status: 'blocked',
        tick: request.tick ?? 0,
        shell,
        lifecycleBefore: null,
        lifecycleAfter: null,
        enemy: {
            stableId: enemyStableId,
            entity: null,
            position: null,
        },
        player: {
            stableId: playerStableId,
            camera: request.targetCamera,
        },
        autonomousPolicy: null,
        movementSummary: null,
        combatSummary: null,
        nonClaims: ['not_shell_scheduler', 'not_ui_authority', 'not_demo_local_authority'],
    };
    if (session === null) {
        return { ...blockedBase, blockedReason: 'missing_backend' };
    }
    const lifecycleBefore = session.readLifecycleStatus();
    const blockedByLifecycle = lifecycleBlockReason(lifecycleBefore, shell.paused);
    if (blockedByLifecycle !== null) {
        return {
            ...blockedBase,
            blockedReason: blockedByLifecycle,
            tick: request.tick ?? lifecycleBefore.tick,
            lifecycleBefore,
            lifecycleAfter: lifecycleBefore,
        };
    }
    if (request.targetCamera === null) {
        return {
            ...blockedBase,
            blockedReason: 'missing_player',
            tick: request.tick ?? lifecycleBefore.tick,
            lifecycleBefore,
            lifecycleAfter: lifecycleBefore,
        };
    }
    const ecrp = session.readEcrpRuntimeReadout();
    const enemy = findEntityByStableId(ecrp.entities, enemyStableId);
    const enemyTransform = enemy === null ? null : findTransform(enemy.capabilities);
    if (enemy === null || enemyTransform === null) {
        return {
            ...blockedBase,
            blockedReason: 'missing_enemy',
            tick: request.tick ?? lifecycleBefore.tick,
            lifecycleBefore,
            lifecycleAfter: lifecycleBefore,
        };
    }
    const policyReadout = session.runAutonomousPolicyTick({
        targetCamera: request.targetCamera,
        ...(request.tick === undefined ? {} : { tick: request.tick }),
        enemy: {
            id: enemyStableId,
            position: enemyTransform.position,
        },
        target: {
            id: playerStableId,
            ...(request.targetPosition === undefined || request.targetPosition === null ? {} : { position: request.targetPosition }),
        },
        combat: {
            primaryFireRangeUnits: request.combat?.primaryFireRangeUnits ?? 2.4,
            lineOfSight: request.combat?.lineOfSight ?? 'clear',
        },
    });
    const lifecycleAfter = session.readLifecycleStatus();
    return {
        kind: 'runtime_session.playable_encounter_tick.v0',
        status: 'advanced',
        blockedReason: null,
        tick: policyReadout.tick,
        shell,
        lifecycleBefore,
        lifecycleAfter,
        enemy: {
            stableId: enemyStableId,
            entity: enemy.entity,
            position: enemyTransform.position,
        },
        player: {
            stableId: playerStableId,
            camera: request.targetCamera,
        },
        autonomousPolicy: policyReadout,
        movementSummary: policyReadout.movementSummary,
        combatSummary: policyReadout.combatSummary,
        nonClaims: ['not_shell_scheduler', 'not_ui_authority', 'not_demo_local_authority'],
    };
}
function lifecycleBlockReason(lifecycle, paused) {
    if (paused) {
        return 'paused';
    }
    if (lifecycle.player.dead) {
        return 'player_dead';
    }
    if (lifecycle.enemy.dead) {
        return 'enemy_dead';
    }
    return null;
}
function findEntityByStableId(entities, stableId) {
    return entities.find((entity) => entity.definitionStableId === stableId) ?? null;
}
function findTransform(capabilities) {
    const transform = capabilities.find((capability) => capability.kind === 'transform');
    return transform ?? null;
}
//# sourceMappingURL=playable-encounter-tick.js.map
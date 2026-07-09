import type { FpsRuntimeSessionLoadRequest } from './bridge.js';

export function fpsLoadRequest(): FpsRuntimeSessionLoadRequest {
  return {
    projectBundle: 'custom-demo',
    definitions: [
      {
        entity: 101,
        stableId: 'actor/custom-player',
        displayName: 'Custom Player',
        sourcePath: 'catalogs/actors/player.entity.json',
        tags: ['player'],
        role: 'player' as const,
        transform: { translation: [0, 1.5, 0] as const, rotation: [0, 0, 0, 1] as const, scale: [1, 1, 1] as const },
        bounds: { min: [2.2, 1, 1] as const, max: [2.8, 2, 2] as const },
        renderVisible: true,
        staticCollider: false,
        health: { current: 88, max: 88 },
        weapon: { weaponId: 'weapon.custom.primary', damage: 75, rangeUnits: 16, ammo: 3, cooldownTicksAfterFire: 4 },
        policyBinding: null,
      },
      {
        entity: 777,
        stableId: 'actor/custom-enemy',
        displayName: 'Custom Enemy',
        sourcePath: 'catalogs/actors/enemy.entity.json',
        tags: ['enemy'],
        role: 'enemy' as const,
        transform: { translation: [0, 1.5, 5.2] as const, rotation: [0, 0, 0, 1] as const, scale: [1, 1, 1] as const },
        bounds: { min: [2.2, 1, 5] as const, max: [2.8, 2, 5.8] as const },
        renderVisible: true,
        staticCollider: false,
        health: { current: 75, max: 75 },
        weapon: null,
        policyBinding: {
          bindingId: 'binding.enemy.custom.v0',
          policyId: 'policy.enemy.custom.v0',
          viewKind: 'runtime_session.nav_policy_view.v0',
          viewVersion: 'v0',
          allowedIntents: ['runtime.intent.primary_fire.v0'],
          runtimeMoment: 'runtime.tick.enemy_policy.v0',
        },
      },
    ],
    gameRuleModules: [],
  };
}

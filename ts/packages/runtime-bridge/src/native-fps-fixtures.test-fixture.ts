import { sceneId, sceneNodeId } from '@asha/contracts';
import type { FlatSceneDocument } from '@asha/contracts';
import type { FpsRuntimeSessionLoadRequest } from './bridge.js';

export function entitySceneDocument(input: {
  readonly id: number;
  readonly instances: readonly {
    readonly entity: number;
    readonly definitionId: string;
    readonly instanceId?: string;
    readonly spawnMarkerId?: string | null;
    readonly translation?: readonly [number, number, number];
    readonly rotation?: readonly [number, number, number, number];
  }[];
}): FlatSceneDocument {
  return {
    schemaVersion: 3,
    id: sceneId(input.id),
    metadata: { name: 'RuntimeSession test scene', authoringFormatVersion: 3 },
    dependencies: [],
    nodes: input.instances.map((instance, childOrder) => ({
      id: sceneNodeId(instance.entity),
      parent: null,
      childOrder,
      label: instance.definitionId,
      tags: [],
      transform: {
        translation: instance.translation ?? [0, 0, 0],
        rotation: instance.rotation ?? [0, 0, 0, 1],
        scale: [1, 1, 1],
      },
      kind: {
        kind: 'entityInstance',
        instance: {
          instanceId: instance.instanceId ?? `${instance.definitionId}.instance`,
          reference: { kind: 'entityDefinition', stableId: instance.definitionId },
          spawnMarkerId: instance.spawnMarkerId ?? null,
        },
      },
    })),
  };
}

export function fpsLoadRequest(): FpsRuntimeSessionLoadRequest {
  return {
    projectBundle: 'custom-demo',
    bootstrapResolutionRegistry: {
      schemaVersion: 1,
      entityDefinitionIds: ['actor/custom-player', 'actor/custom-enemy'],
      prefabIds: [],
      spawnMarkerIds: [],
      generatorPresets: [],
      catalogIds: [],
    },
    sceneDocument: entitySceneDocument({
      id: 77,
      instances: [
        { entity: 101, definitionId: 'actor/custom-player', translation: [0, 1.5, 0] },
        { entity: 777, definitionId: 'actor/custom-enemy', translation: [0, 1.5, 5.2] },
      ],
    }),
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

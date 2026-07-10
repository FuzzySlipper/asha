import type { EnemyPolicyProposal, EnemyPolicyVec3 } from '@asha/runtime-session';
import { type NavPathReadout, type NavPathScenario } from '@asha/runtime-session';
import type { RuntimeSessionEcrpEntityState, RuntimeSessionEcrpProjectState, RuntimeSessionEcrpTransformState, RuntimeSessionHashValue, RuntimeSessionLifecycleRole } from '@asha/runtime-session';
export declare const RUNTIME_SESSION_ENEMY_MOVEMENT_AUTHORITY: {
    readonly navServiceCrate: "svc-pathfinding";
    readonly runtimeTransformAuthorityCrate: "core-scene";
    readonly lifecycleRuleCrate: "rule-lifecycle";
    readonly replayUnit: "runtime_session.enemy.direct_nav_movement.v0";
};
export declare function buildRuntimeSessionEnemyNavPath(input: {
    readonly scenario?: NavPathScenario;
    readonly enemyPosition?: EnemyPolicyVec3;
    readonly targetPosition?: EnemyPolicyVec3;
    readonly queryFixturePath: (scenario?: NavPathScenario) => NavPathReadout;
}): NavPathReadout;
export declare function transformForAutonomousMovementProposal(input: {
    readonly projectState: RuntimeSessionEcrpProjectState | null;
    readonly proposal: Extract<EnemyPolicyProposal, {
        readonly kind: 'enemy_policy.move_toward_target.v0';
    }>;
    readonly runtimeTransforms: ReadonlyMap<number, RuntimeSessionEcrpTransformState>;
    readonly enemyDead: boolean;
}): {
    readonly entity: number;
    readonly transform: RuntimeSessionEcrpTransformState;
} | null;
export declare function ecrpActorPosition(input: {
    readonly projectState: RuntimeSessionEcrpProjectState | null;
    readonly runtimeTransforms: ReadonlyMap<number, RuntimeSessionEcrpTransformState>;
    readonly role: RuntimeSessionLifecycleRole;
}): EnemyPolicyVec3 | null;
export declare function ecrpEntityTransform(input: {
    readonly entity: RuntimeSessionEcrpEntityState;
    readonly runtimeTransforms: ReadonlyMap<number, RuntimeSessionEcrpTransformState>;
}): RuntimeSessionEcrpTransformState | null;
export declare function runtimeTransformHashRecord(transforms: ReadonlyMap<number, RuntimeSessionEcrpTransformState>): RuntimeSessionHashValue;
//# sourceMappingURL=runtime-session-enemy-authority.d.ts.map
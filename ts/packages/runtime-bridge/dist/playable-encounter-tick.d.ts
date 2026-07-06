import type { CameraHandle } from '@asha/contracts';
import type { RuntimeSessionAutonomousPolicyTickReadout, RuntimeSessionFacade, RuntimeSessionLifecycleStatusReadout } from './runtime-session.js';
export type RuntimeSessionPlayableEncounterTickBlockedReason = 'missing_backend' | 'paused' | 'player_dead' | 'enemy_dead' | 'missing_enemy' | 'missing_player';
export interface RuntimeSessionPlayableEncounterTickRequest {
    readonly targetCamera: CameraHandle | null;
    readonly targetPosition?: readonly [number, number, number] | null;
    readonly tick?: number;
    readonly shell?: {
        readonly paused?: boolean;
    };
    readonly enemyStableId?: string;
    readonly playerStableId?: string;
    readonly combat?: {
        readonly primaryFireRangeUnits?: number;
        readonly lineOfSight?: 'clear' | 'blocked';
    };
}
export interface RuntimeSessionPlayableEncounterTickReadout {
    readonly kind: 'runtime_session.playable_encounter_tick.v0';
    readonly status: 'advanced' | 'blocked';
    readonly blockedReason: RuntimeSessionPlayableEncounterTickBlockedReason | null;
    readonly tick: number;
    readonly shell: {
        readonly paused: boolean;
    };
    readonly lifecycleBefore: RuntimeSessionLifecycleStatusReadout | null;
    readonly lifecycleAfter: RuntimeSessionLifecycleStatusReadout | null;
    readonly enemy: {
        readonly stableId: string;
        readonly entity: number | null;
        readonly position: readonly [number, number, number] | null;
    };
    readonly player: {
        readonly stableId: string;
        readonly camera: CameraHandle | null;
    };
    readonly autonomousPolicy: RuntimeSessionAutonomousPolicyTickReadout | null;
    readonly movementSummary: RuntimeSessionAutonomousPolicyTickReadout['movementSummary'] | null;
    readonly combatSummary: RuntimeSessionAutonomousPolicyTickReadout['combatSummary'] | null;
    readonly nonClaims: readonly [
        'not_shell_scheduler',
        'not_ui_authority',
        'not_demo_local_authority'
    ];
}
type RuntimeSessionPlayableEncounterTickFacade = Pick<RuntimeSessionFacade, 'readEcrpRuntimeReadout' | 'readLifecycleStatus' | 'runAutonomousPolicyTick'>;
export declare function readRuntimeSessionPlayableEncounterTick(session: RuntimeSessionPlayableEncounterTickFacade | null, request: RuntimeSessionPlayableEncounterTickRequest): RuntimeSessionPlayableEncounterTickReadout;
export {};
//# sourceMappingURL=playable-encounter-tick.d.ts.map
import type { CameraHandle } from '@asha/contracts';
import type { NavPolicyViewReadout } from './nav-readout.js';
import type { RuntimeActionIntentEnvelope } from './runtime-action.js';
export type EnemyPolicyVec3 = readonly [number, number, number];
export type EnemyPolicyLineOfSight = 'clear' | 'blocked';
export interface EnemyPolicyActorView {
    readonly id: string;
    readonly position: EnemyPolicyVec3;
}
export interface EnemyPolicyTargetView extends EnemyPolicyActorView {
    readonly camera: CameraHandle;
}
export interface EnemyPolicyCombatView {
    readonly primaryFireRangeUnits: number;
    readonly lineOfSight: EnemyPolicyLineOfSight;
}
export interface EnemyPolicyView {
    readonly kind: 'enemy_policy_view.v0';
    readonly tick: number;
    readonly enemy: EnemyPolicyActorView;
    readonly target: EnemyPolicyTargetView;
    readonly nav: NavPolicyViewReadout;
    readonly combat: EnemyPolicyCombatView;
    readonly readOnly: true;
    readonly proposalOnly: true;
}
export interface GeneratedTunnelEnemyPolicyFixtureInput {
    readonly tick?: number;
    readonly enemy?: Partial<EnemyPolicyActorView>;
    readonly target: Partial<EnemyPolicyTargetView> & {
        readonly camera: CameraHandle;
    };
    readonly nav: NavPolicyViewReadout;
    readonly combat?: Partial<EnemyPolicyCombatView>;
}
export interface GeneratedTunnelEnemyPolicyFixture {
    readonly kind: 'generated_tunnel_enemy_policy_fixture.v0';
    readonly view: EnemyPolicyView;
    readonly frame: EnemyPolicyProposalFrame;
    readonly nonClaims: readonly EnemyPolicyFixtureNonClaim[];
}
export type EnemyPolicyFixtureNonClaim = 'not_policy_runtime' | 'not_authority' | 'not_local_state_mutation' | 'not_dom_or_network_capable';
export interface EnemyPolicyMoveProposal {
    readonly kind: 'enemy_policy.move_toward_target.v0';
    readonly actor: string;
    readonly target: string;
    readonly from: EnemyPolicyVec3;
    readonly nextWaypoint: EnemyPolicyVec3 | null;
    readonly pathHash: string;
    readonly authority: 'rust_runtime_must_validate';
}
export interface EnemyPolicyPrimaryFireProposal {
    readonly kind: 'enemy_policy.primary_fire_intent.v0';
    readonly actor: string;
    readonly target: string;
    readonly intent: RuntimeActionIntentEnvelope;
    readonly distanceUnits: number;
    readonly authority: 'rust_runtime_must_validate';
}
export type EnemyPolicyProposal = EnemyPolicyMoveProposal | EnemyPolicyPrimaryFireProposal;
export interface EnemyPolicyProposalFrame {
    readonly kind: 'enemy_policy_proposal_frame.v0';
    readonly tick: number;
    readonly proposals: readonly EnemyPolicyProposal[];
    readonly diagnostics: readonly EnemyPolicyDiagnostic[];
    readonly proposalHash: string;
}
export type EnemyPolicyDiagnosticCode = 'blocked_nav_path' | 'target_out_of_range' | 'line_of_sight_blocked' | 'invalid_policy_view';
export interface EnemyPolicyDiagnostic {
    readonly code: EnemyPolicyDiagnosticCode;
    readonly detail: string;
}
export type EnemyPolicyForbiddenCapability = 'clock' | 'random' | 'network' | 'dom' | 'filesystem' | 'process' | 'dynamic_code' | 'module_import';
export interface EnemyPolicySourceDiagnostic {
    readonly code: 'forbidden_capability_reference';
    readonly capability: EnemyPolicyForbiddenCapability;
    readonly token: string;
    readonly detail: string;
}
export declare function createGeneratedTunnelEnemyPolicyFixture(input: GeneratedTunnelEnemyPolicyFixtureInput): GeneratedTunnelEnemyPolicyFixture;
export declare function createEnemyPolicyView(input: GeneratedTunnelEnemyPolicyFixtureInput): EnemyPolicyView;
export declare function proposeEnemyPolicyFrame(view: EnemyPolicyView): EnemyPolicyProposalFrame;
export declare function validateEnemyPolicySource(source: string): readonly EnemyPolicySourceDiagnostic[];
//# sourceMappingURL=enemy-policy.d.ts.map
import type { EditorControl } from './index.js';
import type { TimeControlCommand } from '@asha/contracts';
export interface HudHealthInput {
    readonly entity: number;
    readonly current: number;
    readonly max: number;
    readonly dead: boolean;
}
export interface HudStatusInput {
    readonly id: string;
    readonly tone: 'info' | 'warning' | 'danger';
    readonly text: string;
}
export interface HudProjectionInput {
    readonly health: HudHealthInput;
    readonly status: readonly HudStatusInput[];
    readonly nonClaims: readonly string[];
    readonly menuOpen?: boolean;
}
export interface HudHealthProjection {
    readonly entity: number;
    readonly current: number;
    readonly max: number;
    readonly dead: boolean;
    readonly ratio: number;
    readonly label: string;
}
export interface HudMenuProjection {
    readonly open: boolean;
    readonly controls: readonly EditorControl[];
}
export interface HudProjection {
    readonly kind: 'hud_projection.v0';
    readonly health: HudHealthProjection;
    readonly status: readonly HudStatusInput[];
    readonly nonClaims: readonly string[];
    readonly menu: HudMenuProjection;
}
export type HudMenuIntent = {
    readonly kind: 'ui.pause_intent';
    readonly source: 'hud_menu';
} | {
    readonly kind: 'runtime.restart_session_intent';
    readonly source: 'hud_menu';
} | {
    readonly kind: 'ui.open_options_intent';
    readonly source: 'hud_menu';
} | {
    readonly kind: 'ui.exit_to_menu_intent';
    readonly source: 'hud_menu';
} | {
    readonly kind: 'ui.resume_intent';
    readonly source: 'hud_menu';
};
export declare function hudIntentToTimeControlCommand(intent: HudMenuIntent): TimeControlCommand | null;
export type GameHudHealthRole = 'player' | 'target' | 'ally' | 'neutral';
export interface GameHudHealthBarInput extends HudHealthInput {
    readonly id: string;
    readonly role: GameHudHealthRole;
    readonly title: string;
}
export interface GameHudHealthBarProjection extends HudHealthProjection {
    readonly id: string;
    readonly role: GameHudHealthRole;
    readonly title: string;
}
export interface GameHudCombatCountersInput {
    readonly shotsFired: number;
    readonly hits: number;
    readonly misses: number;
    readonly damageDealt?: number;
    readonly damageTaken?: number;
    readonly restartCount?: number;
    readonly actionTick?: number;
}
export interface GameHudCombatCountersProjection extends GameHudCombatCountersInput {
    readonly accuracyRatio: number;
    readonly label: string;
}
export interface GameHudInputStatusInput {
    readonly pointerLocked: boolean;
    readonly movementEnabled: boolean;
    readonly fireEnabled: boolean;
    readonly paused: boolean;
}
export interface GameHudInputStatusProjection extends GameHudInputStatusInput {
    readonly lockLabel: string;
    readonly movementLabel: string;
    readonly fireLabel: string;
    readonly pauseLabel: string;
}
export interface GameHudPoseLabelsInput {
    readonly position: string;
    readonly facing: string;
    readonly camera: string;
}
export interface GameHudEventRowInput {
    readonly id: string;
    readonly tone: HudStatusInput['tone'];
    readonly text: string;
    readonly detail?: string;
}
export interface GameHudMenuControlInput {
    readonly id: 'hud-pause' | 'hud-resume' | 'hud-restart' | 'hud-options' | 'hud-exit';
    readonly label: string;
    readonly value: string;
    readonly disabled?: boolean;
}
export interface GameHudProjectionInput {
    readonly healthBars: readonly GameHudHealthBarInput[];
    readonly combat: GameHudCombatCountersInput;
    readonly input: GameHudInputStatusInput;
    readonly pose: GameHudPoseLabelsInput;
    readonly status: readonly HudStatusInput[];
    readonly events: readonly GameHudEventRowInput[];
    readonly menuOpen?: boolean;
    readonly menuControls?: readonly GameHudMenuControlInput[];
}
export interface GameHudProjection {
    readonly kind: 'game_hud_projection.v0';
    readonly healthBars: readonly GameHudHealthBarProjection[];
    readonly combat: GameHudCombatCountersProjection;
    readonly input: GameHudInputStatusProjection;
    readonly pose: GameHudPoseLabelsInput;
    readonly status: readonly HudStatusInput[];
    readonly events: readonly GameHudEventRowInput[];
    readonly menu: HudMenuProjection;
}
export declare function buildHudProjection(input: HudProjectionInput): HudProjection;
export declare function hudControlToIntent(controlId: string): HudMenuIntent | null;
export declare function buildGameHudProjection(input: GameHudProjectionInput): GameHudProjection;
//# sourceMappingURL=hud.d.ts.map
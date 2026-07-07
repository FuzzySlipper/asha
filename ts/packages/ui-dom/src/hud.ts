import type { EditorControl } from './index.js';

// ── HUD/menu projection (pure rusty-view-style model; proposals only) ─────────

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

export type HudMenuIntent =
  | { readonly kind: 'ui.pause_intent'; readonly source: 'hud_menu' }
  | { readonly kind: 'runtime.restart_session_intent'; readonly source: 'hud_menu' }
  | { readonly kind: 'ui.open_options_intent'; readonly source: 'hud_menu' }
  | { readonly kind: 'ui.exit_to_menu_intent'; readonly source: 'hud_menu' }
  | { readonly kind: 'ui.resume_intent'; readonly source: 'hud_menu' };

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

export function buildHudProjection(input: HudProjectionInput): HudProjection {
  validateHealth(input.health);
  const health = projectHudHealth(input.health);
  const controls: EditorControl[] = [
    {
      id: 'hud-resume',
      role: 'button',
      label: 'Resume',
      value: 'resume',
      disabled: input.menuOpen !== true,
    },
    {
      id: 'hud-restart',
      role: 'button',
      label: 'Restart session',
      value: 'restart',
    },
    {
      id: 'hud-options',
      role: 'button',
      label: 'Options',
      value: 'options',
    },
    {
      id: 'hud-exit',
      role: 'button',
      label: 'Exit',
      value: 'exit',
    },
  ];
  return {
    kind: 'hud_projection.v0',
    health,
    status: [...input.status],
    nonClaims: [...input.nonClaims],
    menu: {
      open: input.menuOpen ?? false,
      controls,
    },
  };
}

export function hudControlToIntent(controlId: string): HudMenuIntent | null {
  switch (controlId) {
    case 'hud-pause':
      return { kind: 'ui.pause_intent', source: 'hud_menu' };
    case 'hud-resume':
      return { kind: 'ui.resume_intent', source: 'hud_menu' };
    case 'hud-restart':
      return { kind: 'runtime.restart_session_intent', source: 'hud_menu' };
    case 'hud-options':
      return { kind: 'ui.open_options_intent', source: 'hud_menu' };
    case 'hud-exit':
      return { kind: 'ui.exit_to_menu_intent', source: 'hud_menu' };
    default:
      return null;
  }
}

export function buildGameHudProjection(input: GameHudProjectionInput): GameHudProjection {
  if (input.healthBars.length === 0) {
    throw new Error('Game HUD requires at least one health bar');
  }
  const healthBars = input.healthBars.map(projectGameHudHealthBar);
  const menuControls = input.menuControls ?? defaultGameHudMenuControls(input.menuOpen ?? false);
  return {
    kind: 'game_hud_projection.v0',
    healthBars,
    combat: projectCombatCounters(input.combat),
    input: projectInputStatus(input.input),
    pose: { ...input.pose },
    status: [...input.status],
    events: input.events.map((event) => ({ ...event })),
    menu: {
      open: input.menuOpen ?? false,
      controls: menuControls.map(gameMenuControlDescriptor),
    },
  };
}

function validateHealth(health: HudHealthInput): void {
  if (!Number.isFinite(health.current) || !Number.isFinite(health.max) || health.max <= 0 || health.current < 0) {
    throw new Error('HUD health must satisfy finite 0 <= current and max > 0');
  }
  if (health.current > health.max) {
    throw new Error('HUD health current must not exceed max');
  }
}

function projectHudHealth(health: HudHealthInput): HudHealthProjection {
  const ratio = health.max === 0 ? 0 : health.current / health.max;
  return {
    entity: health.entity,
    current: health.current,
    max: health.max,
    dead: health.dead,
    ratio,
    label: health.dead ? `Health ${health.current}/${health.max} defeated` : `Health ${health.current}/${health.max}`,
  };
}

function projectGameHudHealthBar(health: GameHudHealthBarInput): GameHudHealthBarProjection {
  validateHealth(health);
  const projected = projectHudHealth(health);
  return {
    ...projected,
    id: health.id,
    role: health.role,
    title: health.title,
    label: health.dead
      ? `${health.title} health ${health.current}/${health.max} defeated`
      : `${health.title} health ${health.current}/${health.max}`,
  };
}

function projectCombatCounters(input: GameHudCombatCountersInput): GameHudCombatCountersProjection {
  const shotsFired = nonNegativeUiNumber(input.shotsFired, 'shotsFired');
  const hits = nonNegativeUiNumber(input.hits, 'hits');
  const misses = nonNegativeUiNumber(input.misses, 'misses');
  const damageDealt = optionalNonNegativeUiNumber(input.damageDealt, 'damageDealt');
  const damageTaken = optionalNonNegativeUiNumber(input.damageTaken, 'damageTaken');
  const restartCount = optionalNonNegativeUiNumber(input.restartCount, 'restartCount');
  const actionTick = optionalNonNegativeUiNumber(input.actionTick, 'actionTick');
  const accuracyRatio = shotsFired === 0 ? 0 : hits / shotsFired;
  return {
    shotsFired,
    hits,
    misses,
    ...(damageDealt === undefined ? {} : { damageDealt }),
    ...(damageTaken === undefined ? {} : { damageTaken }),
    ...(restartCount === undefined ? {} : { restartCount }),
    ...(actionTick === undefined ? {} : { actionTick }),
    accuracyRatio,
    label: `Shots ${shotsFired}, hits ${hits}, misses ${misses}`,
  };
}

function projectInputStatus(input: GameHudInputStatusInput): GameHudInputStatusProjection {
  return {
    ...input,
    lockLabel: input.pointerLocked ? 'Pointer locked' : 'Pointer unlocked',
    movementLabel: input.movementEnabled ? 'Movement enabled' : 'Movement disabled',
    fireLabel: input.fireEnabled ? 'Fire enabled' : 'Fire disabled',
    pauseLabel: input.paused ? 'Paused' : 'Running',
  };
}

function defaultGameHudMenuControls(menuOpen: boolean): readonly GameHudMenuControlInput[] {
  return [
    { id: 'hud-pause', label: 'Pause', value: 'pause', disabled: menuOpen },
    { id: 'hud-resume', label: 'Resume', value: 'resume', disabled: !menuOpen },
    { id: 'hud-restart', label: 'Restart session', value: 'restart' },
    { id: 'hud-options', label: 'Options', value: 'options' },
    { id: 'hud-exit', label: 'Exit', value: 'exit' },
  ];
}

function gameMenuControlDescriptor(control: GameHudMenuControlInput): EditorControl {
  return {
    id: control.id,
    role: 'button',
    label: control.label,
    value: control.value,
    ...(control.disabled === undefined ? {} : { disabled: control.disabled }),
  };
}

function nonNegativeUiNumber(value: number, field: string): number {
  if (!Number.isFinite(value) || value < 0) {
    throw new Error(`Game HUD ${field} must be a finite non-negative number`);
  }
  return value;
}

function optionalNonNegativeUiNumber(value: number | undefined, field: string): number | undefined {
  return value === undefined ? undefined : nonNegativeUiNumber(value, field);
}

export function hudIntentToTimeControlCommand(intent) {
    if (intent.kind === 'ui.pause_intent')
        return { operation: 'pause' };
    if (intent.kind === 'ui.resume_intent')
        return { operation: 'resume' };
    return null;
}
export function buildHudProjection(input) {
    validateHealth(input.health);
    const health = projectHudHealth(input.health);
    const controls = [
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
export function hudControlToIntent(controlId) {
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
export function buildGameHudProjection(input) {
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
function validateHealth(health) {
    if (!Number.isFinite(health.current) || !Number.isFinite(health.max) || health.max <= 0 || health.current < 0) {
        throw new Error('HUD health must satisfy finite 0 <= current and max > 0');
    }
    if (health.current > health.max) {
        throw new Error('HUD health current must not exceed max');
    }
}
function projectHudHealth(health) {
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
function projectGameHudHealthBar(health) {
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
function projectCombatCounters(input) {
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
function projectInputStatus(input) {
    return {
        ...input,
        lockLabel: input.pointerLocked ? 'Pointer locked' : 'Pointer unlocked',
        movementLabel: input.movementEnabled ? 'Movement enabled' : 'Movement disabled',
        fireLabel: input.fireEnabled ? 'Fire enabled' : 'Fire disabled',
        pauseLabel: input.paused ? 'Paused' : 'Running',
    };
}
function defaultGameHudMenuControls(menuOpen) {
    return [
        { id: 'hud-pause', label: 'Pause', value: 'pause', disabled: menuOpen },
        { id: 'hud-resume', label: 'Resume', value: 'resume', disabled: !menuOpen },
        { id: 'hud-restart', label: 'Restart session', value: 'restart' },
        { id: 'hud-options', label: 'Options', value: 'options' },
        { id: 'hud-exit', label: 'Exit', value: 'exit' },
    ];
}
function gameMenuControlDescriptor(control) {
    return {
        id: control.id,
        role: 'button',
        label: control.label,
        value: control.value,
        ...(control.disabled === undefined ? {} : { disabled: control.disabled }),
    };
}
function nonNegativeUiNumber(value, field) {
    if (!Number.isFinite(value) || value < 0) {
        throw new Error(`Game HUD ${field} must be a finite non-negative number`);
    }
    return value;
}
function optionalNonNegativeUiNumber(value, field) {
    return value === undefined ? undefined : nonNegativeUiNumber(value, field);
}
//# sourceMappingURL=hud.js.map
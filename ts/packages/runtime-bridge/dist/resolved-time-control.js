export const TIME_CONTROL_INPUT_ACTIONS = {
    pause: 'runtime.time.pause',
    resume: 'runtime.time.resume',
    stepOne: 'runtime.time.step_one',
};
export function timeControlCommandFromResolvedAction(action) {
    if (action.phase !== 'pressed'
        || action.value.kind !== 'button'
        || !action.value.pressed)
        return null;
    if (action.actionId === TIME_CONTROL_INPUT_ACTIONS.pause)
        return { operation: 'pause' };
    if (action.actionId === TIME_CONTROL_INPUT_ACTIONS.resume)
        return { operation: 'resume' };
    if (action.actionId === TIME_CONTROL_INPUT_ACTIONS.stepOne) {
        return { operation: 'stepTicks', ticks: 1 };
    }
    return null;
}
export class ResolvedTimeControlConsumer {
    session;
    constructor(session) {
        this.session = session;
    }
    consume(action) {
        const command = timeControlCommandFromResolvedAction(action);
        return command === null ? null : this.session.applyTimeControlCommand(command);
    }
}
/**
 * Downstream composition for the common pause-menu flow. Rust still owns both
 * validated state transitions; this consumer only sequences their public verbs.
 */
export class ResolvedPauseContextConsumer {
    session;
    menuContextId;
    constructor(session, menuContextId = 'menu') {
        this.session = session;
        this.menuContextId = menuContextId;
    }
    consume(action) {
        const command = timeControlCommandFromResolvedAction(action);
        if (command?.operation === 'pause')
            return this.#pause(action.actionId);
        if (command?.operation === 'resume')
            return this.#resume(action.actionId);
        return null;
    }
    #pause(actionId) {
        const context = this.session.applyInputContextCommand({
            operation: 'push', contextId: this.menuContextId,
        });
        if (!context.accepted)
            return { actionId, accepted: false, context, time: null, rollback: null };
        const time = this.session.applyTimeControlCommand({ operation: 'pause' });
        const rollback = time.accepted ? null : this.session.applyInputContextCommand({
            operation: 'pop', expectedContextId: this.menuContextId,
        });
        return { actionId, accepted: time.accepted, context, time, rollback };
    }
    #resume(actionId) {
        const context = this.session.applyInputContextCommand({
            operation: 'pop', expectedContextId: this.menuContextId,
        });
        if (!context.accepted)
            return { actionId, accepted: false, context, time: null, rollback: null };
        const time = this.session.applyTimeControlCommand({ operation: 'resume' });
        const rollback = time.accepted ? null : this.session.applyInputContextCommand({
            operation: 'push', contextId: this.menuContextId,
        });
        return { actionId, accepted: time.accepted, context, time, rollback };
    }
}
//# sourceMappingURL=resolved-time-control.js.map
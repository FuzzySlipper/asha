import type {
  InputContextChangeReceipt,
  ResolvedInputAction,
  TimeControlCommand,
  TimeControlReceipt,
} from '@asha/contracts';
import type { RuntimeSessionFacade } from '@asha/runtime-session';

export const TIME_CONTROL_INPUT_ACTIONS = {
  pause: 'runtime.time.pause',
  resume: 'runtime.time.resume',
  stepOne: 'runtime.time.step_one',
} as const;

export function timeControlCommandFromResolvedAction(
  action: ResolvedInputAction,
): TimeControlCommand | null {
  if (action.phase !== 'pressed'
    || action.value.kind !== 'button'
    || !action.value.pressed) return null;
  if (action.actionId === TIME_CONTROL_INPUT_ACTIONS.pause) return { operation: 'pause' };
  if (action.actionId === TIME_CONTROL_INPUT_ACTIONS.resume) return { operation: 'resume' };
  if (action.actionId === TIME_CONTROL_INPUT_ACTIONS.stepOne) {
    return { operation: 'stepTicks', ticks: 1 };
  }
  return null;
}

export class ResolvedTimeControlConsumer {
  constructor(private readonly session: RuntimeSessionFacade) {}

  consume(action: ResolvedInputAction): TimeControlReceipt | null {
    const command = timeControlCommandFromResolvedAction(action);
    return command === null ? null : this.session.applyTimeControlCommand(command);
  }
}

export interface ResolvedPauseContextReceipt {
  readonly actionId: string;
  readonly accepted: boolean;
  readonly context: InputContextChangeReceipt;
  readonly time: TimeControlReceipt | null;
  readonly rollback: InputContextChangeReceipt | null;
}

/**
 * Downstream composition for the common pause-menu flow. Rust still owns both
 * validated state transitions; this consumer only sequences their public verbs.
 */
export class ResolvedPauseContextConsumer {
  constructor(
    private readonly session: RuntimeSessionFacade,
    private readonly menuContextId = 'menu',
  ) {}

  consume(action: ResolvedInputAction): ResolvedPauseContextReceipt | null {
    const command = timeControlCommandFromResolvedAction(action);
    if (command?.operation === 'pause') return this.#pause(action.actionId);
    if (command?.operation === 'resume') return this.#resume(action.actionId);
    return null;
  }

  #pause(actionId: string): ResolvedPauseContextReceipt {
    const context = this.session.applyInputContextCommand({
      operation: 'push', contextId: this.menuContextId,
    });
    if (!context.accepted) return { actionId, accepted: false, context, time: null, rollback: null };
    const time = this.session.applyTimeControlCommand({ operation: 'pause' });
    const rollback = time.accepted ? null : this.session.applyInputContextCommand({
      operation: 'pop', expectedContextId: this.menuContextId,
    });
    return { actionId, accepted: time.accepted, context, time, rollback };
  }

  #resume(actionId: string): ResolvedPauseContextReceipt {
    const context = this.session.applyInputContextCommand({
      operation: 'pop', expectedContextId: this.menuContextId,
    });
    if (!context.accepted) return { actionId, accepted: false, context, time: null, rollback: null };
    const time = this.session.applyTimeControlCommand({ operation: 'resume' });
    const rollback = time.accepted ? null : this.session.applyInputContextCommand({
      operation: 'push', contextId: this.menuContextId,
    });
    return { actionId, accepted: time.accepted, context, time, rollback };
  }
}

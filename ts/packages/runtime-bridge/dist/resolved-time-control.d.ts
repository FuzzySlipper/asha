import type { InputContextChangeReceipt, ResolvedInputAction, TimeControlCommand, TimeControlReceipt } from '@asha/contracts';
import type { RuntimeSessionFacade } from '@asha/runtime-session';
export declare const TIME_CONTROL_INPUT_ACTIONS: {
    readonly pause: "runtime.time.pause";
    readonly resume: "runtime.time.resume";
    readonly stepOne: "runtime.time.step_one";
};
export declare function timeControlCommandFromResolvedAction(action: ResolvedInputAction): TimeControlCommand | null;
export declare class ResolvedTimeControlConsumer {
    private readonly session;
    constructor(session: RuntimeSessionFacade);
    consume(action: ResolvedInputAction): TimeControlReceipt | null;
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
export declare class ResolvedPauseContextConsumer {
    #private;
    private readonly session;
    private readonly menuContextId;
    constructor(session: RuntimeSessionFacade, menuContextId?: string);
    consume(action: ResolvedInputAction): ResolvedPauseContextReceipt | null;
}
//# sourceMappingURL=resolved-time-control.d.ts.map
import type { TimeControlCommand, TimeControlReceipt, TimeControlState } from '@asha/contracts';
import type { StepResult } from './bridge.js';
export declare class MockTimeController {
    #private;
    initialize(): void;
    read(): TimeControlState;
    apply(command: TimeControlCommand): TimeControlReceipt;
    step(tick: number): StepResult;
}
//# sourceMappingURL=mock-time-control.d.ts.map
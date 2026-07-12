export declare const TIME_CONTROL_STATE_SCHEMA_VERSION = 1;
export type TimeControlMode = 'paused' | 'running';
export type TimeControlCommand = {
    readonly operation: 'pause';
} | {
    readonly operation: 'resume';
} | {
    readonly operation: 'setSpeedMultiplier';
    readonly multiplier: number;
} | {
    readonly operation: 'stepTicks';
    readonly ticks: number;
};
export type TimeControlRejection = 'alreadyPaused' | 'alreadyRunning' | 'invalidSpeedMultiplier' | 'invalidStepCount' | 'notPausedForExactStep';
export interface TimeControlState {
    readonly schemaVersion: number;
    readonly mode: TimeControlMode;
    readonly speedMultiplier: number;
    readonly revision: number;
    readonly authorityTick: number;
    readonly stateHash: string;
}
export interface TimeControlReceipt {
    readonly accepted: boolean;
    readonly before: TimeControlState;
    readonly after: TimeControlState;
    readonly exactTicksAdvanced: number;
    readonly rejection: TimeControlRejection | null;
    readonly receiptHash: string;
}
//# sourceMappingURL=timeControl.d.ts.map
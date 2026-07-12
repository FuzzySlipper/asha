import { type EncounterDirectorReadout, type EncounterDirectorState } from '@asha/runtime-session';
import { type FpsEncounterDirectorSnapshot, type FpsEncounterLifecycleInput, type FpsEncounterStateReadout, type FpsEncounterTransitionResult } from './bridge.js';
import type { lifecycleStatusToEncounterLifecycle } from './runtime-session-lifecycle.js';
export declare function fpsEncounterLifecycleInput(lifecycle: ReturnType<typeof lifecycleStatusToEncounterLifecycle>): FpsEncounterLifecycleInput;
export declare function encounterReadoutFromFpsSnapshot(input: {
    readonly snapshot: FpsEncounterDirectorSnapshot;
    readonly sequenceId: number;
    readonly tick: number;
    readonly sessionSeed: number;
    readonly sessionHash: string;
}): EncounterDirectorReadout;
export declare function encounterTransitionResultForReceipt(result: FpsEncounterTransitionResult): {
    readonly accepted: boolean;
    readonly state: EncounterDirectorState;
    readonly eventKind?: NonNullable<FpsEncounterTransitionResult['eventKind']>;
    readonly rejectionReason?: NonNullable<FpsEncounterTransitionResult['rejectionReason']>;
};
export declare function fpsEncounterStateToReadoutState(state: FpsEncounterStateReadout): EncounterDirectorState;
//# sourceMappingURL=runtime-session-rust-encounter.d.ts.map
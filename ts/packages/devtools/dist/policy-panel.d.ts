import type { PolicyWorldOutcome, PolicyWorldRejection } from '@asha/contracts';
/** One policy's execution metadata for the run (id/version + counts). */
export interface PolicyExecutionInput {
    readonly policyId: string;
    readonly version: number;
    readonly proposedCount: number;
    readonly violationCount: number;
}
/** The deterministic input to the policy-run summary: executions + authority outcomes. */
export interface PolicyRunInput {
    readonly tick: number;
    /** Per-policy execution metadata, in policy order. */
    readonly executions: readonly PolicyExecutionInput[];
    /** The authority outcome for each proposed command, in proposal order. */
    readonly outcomes: readonly PolicyWorldOutcome[];
    /** The deterministic replay handle authority assigned this tick (opaque id/hash). */
    readonly replayHandle: string;
}
/** One policy's row in the run summary. */
export interface PolicyRunRow {
    readonly policyId: string;
    readonly version: number;
    readonly proposedCount: number;
    readonly violationCount: number;
}
/** The classified policy-run summary the panel/agent reads. */
export interface PolicyRunSummary {
    readonly tick: number;
    readonly policyCount: number;
    readonly totalProposed: number;
    readonly accepted: number;
    readonly rejected: number;
    readonly violations: number;
    /** Rejection counts grouped by classified reason, in sorted reason order. */
    readonly rejectionsByReason: ReadonlyArray<{
        readonly reason: PolicyWorldRejection;
        readonly count: number;
    }>;
    readonly replayHandle: string;
    readonly rows: readonly PolicyRunRow[];
}
/** Build the policy-run summary. Pure and deterministic. */
export declare function buildPolicyRunSummary(input: PolicyRunInput): PolicyRunSummary;
/** Deterministic, greppable rendering of the policy-run summary (golden-friendly). */
export declare function formatPolicyRunSummary(view: PolicyRunSummary): string[];
//# sourceMappingURL=policy-panel.d.ts.map
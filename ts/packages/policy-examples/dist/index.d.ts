import { type Policy, type SignalId, type TagId } from '@asha/script-sdk';
/** Configuration for {@link thresholdPolicy}. */
export interface ThresholdConfig {
    /** The tag whose bearers are counted. */
    readonly watchTag: TagId;
    /** The count at or above which the signal is raised. */
    readonly threshold: number;
    /** The signal proposed when the threshold is reached. */
    readonly raiseSignal: SignalId;
}
/**
 * A threshold policy: when at least `threshold` entities carry `watchTag`, it
 * proposes defining `raiseSignal`. It is deterministic and idempotent — once the
 * signal is already defined in the view, it proposes nothing further, so
 * re-running on the resulting state is a fixed point.
 *
 * This is the canonical fixture proving the Phase 3 loop: a policy reads a
 * read-only view and returns a generated `PolicyCommand`.
 */
export declare function thresholdPolicy(config: ThresholdConfig): Policy;
/**
 * The named fixture instance used by tests and the `harness/fixtures` golden
 * inputs/outputs: raise signal `1` once at least three entities carry tag `1`.
 */
export declare const tagCountThreshold: Policy;
//# sourceMappingURL=index.d.ts.map
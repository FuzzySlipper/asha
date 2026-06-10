// @asha/policy-examples — abstract, constrained policy fixtures.
//
// Policies here use only the Phase 1/2 abstract vocabulary (Entity, Subject,
// Process, Mode, Signal, Tag). They are pure functions of the read-only view,
// propose commands only, and obey the ts-policy sandbox (no wall-clock, no
// ambient randomness, no DOM/renderer/bridge/Electron/filesystem/network).
import { commands, signalId, tagId, } from '@asha/script-sdk';
/**
 * A threshold policy: when at least `threshold` entities carry `watchTag`, it
 * proposes defining `raiseSignal`. It is deterministic and idempotent — once the
 * signal is already defined in the view, it proposes nothing further, so
 * re-running on the resulting state is a fixed point.
 *
 * This is the canonical fixture proving the Phase 3 loop: a policy reads a
 * read-only view and returns a generated `PolicyCommand`.
 */
export function thresholdPolicy(config) {
    return (view) => {
        // Idempotent: nothing to do once the signal exists.
        if (view.signals.includes(config.raiseSignal)) {
            return [];
        }
        const bearers = view.entities.filter((e) => e.tags.includes(config.watchTag)).length;
        if (bearers < config.threshold) {
            return [];
        }
        return [commands.defineSignal(config.raiseSignal)];
    };
}
/**
 * The named fixture instance used by tests and the `harness/fixtures` golden
 * inputs/outputs: raise signal `1` once at least three entities carry tag `1`.
 */
export const tagCountThreshold = thresholdPolicy({
    watchTag: tagId(1),
    threshold: 3,
    raiseSignal: signalId(1),
});
//# sourceMappingURL=index.js.map
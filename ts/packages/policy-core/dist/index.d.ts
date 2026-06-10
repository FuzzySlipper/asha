import type { Policy } from '@asha/script-sdk';
/**
 * The no-op policy: the smallest possible constrained policy.
 *
 * It accepts the read-only view (which it does not read or mutate) and
 * deterministically proposes no commands. It is the baseline fixture for
 * script-host invocation and for sandbox/lint checks — if anything about the
 * policy lane breaks, the no-op policy is the first place it shows.
 */
export declare const noopPolicy: Policy;
//# sourceMappingURL=index.d.ts.map
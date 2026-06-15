// @asha/devtools — policy-run summary read model (#2487).
//
// Observational, **read-only** projection of one policy tick: which policies ran
// (id + version), how many commands each proposed, the authority accept/reject
// outcome for each proposal, classified rejection reasons, sandbox violations, and
// the deterministic replay handle. Built from the TS tick-stage execution metadata
// plus the authority-returned `PolicyWorldOutcome`s — TS never decides acceptance,
// it reflects it. Same input → same summary (the loop is deterministic).

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
  readonly rejectionsByReason: ReadonlyArray<{ readonly reason: PolicyWorldRejection; readonly count: number }>;
  readonly replayHandle: string;
  readonly rows: readonly PolicyRunRow[];
}

/** Build the policy-run summary. Pure and deterministic. */
export function buildPolicyRunSummary(input: PolicyRunInput): PolicyRunSummary {
  let accepted = 0;
  const rejectionCounts = new Map<PolicyWorldRejection, number>();
  for (const outcome of input.outcomes) {
    if (outcome.status === 'accepted') {
      accepted += 1;
    } else {
      rejectionCounts.set(outcome.rejection, (rejectionCounts.get(outcome.rejection) ?? 0) + 1);
    }
  }
  const rejectionsByReason = [...rejectionCounts.entries()]
    .map(([reason, count]) => ({ reason, count }))
    .sort((a, b) => a.reason.localeCompare(b.reason));

  const rows: PolicyRunRow[] = input.executions.map((e) => ({
    policyId: e.policyId,
    version: e.version,
    proposedCount: e.proposedCount,
    violationCount: e.violationCount,
  }));

  return {
    tick: input.tick,
    policyCount: input.executions.length,
    totalProposed: input.outcomes.length,
    accepted,
    rejected: input.outcomes.length - accepted,
    violations: input.executions.reduce((sum, e) => sum + e.violationCount, 0),
    rejectionsByReason,
    replayHandle: input.replayHandle,
    rows,
  };
}

/** Deterministic, greppable rendering of the policy-run summary (golden-friendly). */
export function formatPolicyRunSummary(view: PolicyRunSummary): string[] {
  const lines: string[] = [
    `policyRun tick=${view.tick} policies=${view.policyCount} proposed=${view.totalProposed} ` +
      `accepted=${view.accepted} rejected=${view.rejected} violations=${view.violations} ` +
      `replay=${view.replayHandle}`,
  ];
  for (const row of view.rows) {
    lines.push(
      `  policy ${row.policyId} v${row.version} proposed=${row.proposedCount} violations=${row.violationCount}`,
    );
  }
  for (const r of view.rejectionsByReason) {
    lines.push(`  rejected ${r.reason}=${r.count}`);
  }
  return lines;
}

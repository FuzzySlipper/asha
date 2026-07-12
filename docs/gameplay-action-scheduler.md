# Gameplay Action Scheduler

Status: implemented gameplay scheduling authority for Den task #5605.

`rule-scheduler` now contains two intentionally separate systems:

- `ChunkScheduler` remains the transient, version-checked queue for generation,
  meshing, collision rebuild, and upload work.
- `GameplayActionScheduler` is canonical Session authority for shared deferred
  gameplay proposals.

They share deterministic scheduling concerns, but not state, replay meaning, or
execution APIs.

## Explicit Action State

The gameplay scheduler stores either:

- `TickScheduledActionDraft`: propose an operation at or after one authority
  tick; or
- `EventConditionedActionDraft`: propose an operation after one exact gameplay
  event contract and bounded header selector matches, unless a tick timeout
  wins first.

Each accepted action receives an owner-assigned insertion sequence and retains
its stable action id, priority, proposal envelope, source, and causation. Ready
ordering is:

1. execution/timeout tick;
2. declared priority;
3. stable action id; and
4. insertion sequence.

Wall-clock time, arbitrary predicates, callbacks, coroutines, and async runtime
state are absent.

## Read Match, Then Mutate

`due_action_ids`, `matching_action_ids`, and `timed_out_action_ids` are read-only
queries over a frozen queue. Matching an event does not immediately mutate the
queue or apply its proposal.

The next explicit boundary routes an owner-gated `GameplaySchedulerCommand`:

- schedule tick/event action;
- execute a due tick action;
- trigger a matching event action;
- record timeout or cancellation; or
- record the later proposal owner's accepted/rejected routing outcome.

This preserves #5600 wave semantics: all Observe participants can collect
against one frozen wave, then the scheduler owner records the trigger and emits
one `GameplayScheduledDispatch` for normal proposal-owner routing. A scheduled
event cannot retroactively change the fact it observed.

Only the registered scheduler owner may apply commands. Schedule requests with
undeclared event or proposal contracts fail before queue mutation. Action ids
are retired after any terminal outcome and cannot execute or be reused.

## Typed Outcomes

The fact log distinguishes:

- scheduled;
- triggered;
- timed out;
- cancelled;
- rejected for a missing target;
- rejected for stale causation;
- accepted by the destination owner; and
- rejected by the destination owner with diagnostic codes.

Every command receipt includes the before/after scheduler state hashes and the
optional next-boundary dispatch. Fact hashes and diagnostic codes are sorted
before they enter authority evidence.

## Persistence and Replay

Snapshots contain the declared contract set, owner, next insertion sequence,
pending queue, complete outstanding dispatches, retired ids, fact log, and
state hash. Triggering persists the canonical proposal instead of leaving it
only in a transient command receipt. Decode rejects newer schemas, duplicates,
hash drift, and any snapshot whose queue does not equal replaying its facts.

`outstanding_dispatches` is the deterministic interruption/reload recovery
surface. Dispatches survive save/reload and fact replay and retire only when the
scheduler receives an opaque `GameplayRoutingReceipt` created by
`GameplayFabricCoordinator::route_proposal`. The scheduler checks its stored
proposal id, contract, and the fabric's public canonical proposal hash against
that receipt. Callers cannot independently assert the destination owner,
accepted flag, fact hashes, diagnostics, or routing hash.

Playback consumes the recorded facts; verification replay can rerun event
matching and owner routing against the same declared contracts.

The integration fixture models a factory `crafting-completed` event triggering
a progression-counter proposal, then routes that proposal through a closed
fabric registry and records the resulting routing receipt. This is the intended
expressive path: semantic condition, explicit deferred authority, normal
owner-routed mutation.

//! Tick data types and phase definitions for the ASHA headless authority core.
//!
//! # Lane
//!
//! `rust-state` — may depend on `core-ids`, `core-state`, `core-commands`,
//! `core-events`, `core-error`. Must not reference validator, applier, render,
//! protocol, or UI — those concerns belong in `sim-runner` and later phases.
//!
//! # Design
//!
//! A *tick* is one discrete step through the authority path:
//!
//! ```text
//! CollectInput → Validate → AccumulateEvents → ApplyEvents → Snapshot
//! ```
//!
//! [`TickInput`] carries the proposed commands for one tick. [`TickOutcome`]
//! records what was accepted, what was rejected, and how many events were
//! applied. The phases are named explicitly so policy, replay, snapshot, and
//! telemetry layers can attach at the right joints without restructuring the
//! kernel.
//!
//! # Snapshot/telemetry boundary
//!
//! `sim-kernel` does **not** compute state hashes, persist snapshot payloads, or
//! emit telemetry events. The kernel owns the tick's decision summary only. The
//! `Snapshot` phase marks the observation boundary after events have been
//! applied:
//!
//! - `sim-runner` derives state hashes from `core-snapshot`;
//! - `sim-runner` records replay checkpoints and `sim-replay` owns the artifact
//!   shape/encoding;
//! - telemetry remains a read-only projection layer downstream of the completed
//!   tick.
//!
//! This keeps the kernel free of replay/storage/projection dependencies while
//! making the handoff point explicit and testable.

#![forbid(unsafe_code)]

use core_commands::CommandEnvelope;
use core_events::EventBatch;

// ── Tick phases ───────────────────────────────────────────────────────────────

/// The ordered phases of one authority tick.
///
/// Explicit naming here means Phase 3/4/5 integration has named joints to
/// attach to rather than implicit position in a pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickPhase {
    /// Gather proposed [`CommandEnvelope`]s from inputs.
    CollectInput,
    /// Validate each command against the current [`core_state::StateStore`].
    Validate,
    /// Collect accepted [`EventBatch`]es from passing commands.
    AccumulateEvents,
    /// Apply each accepted batch to the store in order.
    ApplyEvents,
    /// Observation boundary after mutation. See [`SnapshotPhaseContract`] for
    /// the explicit ownership split: the kernel names this phase, while
    /// `sim-runner`/`core-snapshot`/`sim-replay` produce hash and checkpoint data.
    Snapshot,
}

/// Ownership contract for [`TickPhase::Snapshot`].
///
/// This is deliberately data-only: it documents and pins the lane boundary
/// without making `sim-kernel` depend on `core-snapshot`, `sim-replay`,
/// telemetry protocols, render, or tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotPhaseContract {
    /// Whether `TickOutcome` carries a state-hash field owned by this crate.
    pub kernel_emits_state_hash: bool,
    /// Whether `TickOutcome` carries a full snapshot payload owned by this crate.
    pub kernel_emits_snapshot_payload: bool,
    /// Crate/layer that computes deterministic state hashes after mutation.
    pub state_hash_owner: &'static str,
    /// Crate/layer that records replay checkpoint metadata.
    pub replay_checkpoint_owner: &'static str,
    /// Layer that may project observational telemetry from a completed tick.
    pub telemetry_owner: &'static str,
}

impl SnapshotPhaseContract {
    /// Current concrete boundary for the snapshot phase.
    pub const CURRENT: SnapshotPhaseContract = SnapshotPhaseContract {
        kernel_emits_state_hash: false,
        kernel_emits_snapshot_payload: false,
        state_hash_owner: "sim-runner via core-snapshot",
        replay_checkpoint_owner: "sim-runner records, sim-replay encodes",
        telemetry_owner: "downstream read-only telemetry projection",
    };
}

impl TickPhase {
    /// Returns the concrete snapshot ownership contract for the observation
    /// phase. Other phases do not own snapshot/hash/telemetry payloads.
    pub const fn snapshot_contract(self) -> Option<SnapshotPhaseContract> {
        match self {
            TickPhase::Snapshot => Some(SnapshotPhaseContract::CURRENT),
            TickPhase::CollectInput
            | TickPhase::Validate
            | TickPhase::AccumulateEvents
            | TickPhase::ApplyEvents => None,
        }
    }
}

// ── Tick I/O ──────────────────────────────────────────────────────────────────

/// Proposed commands submitted for one tick.
#[derive(Debug, Default)]
pub struct TickInput {
    pub tick: u64,
    pub commands: Vec<CommandEnvelope>,
}

impl TickInput {
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, env: CommandEnvelope) {
        self.commands.push(env);
    }
}

/// Per-command result for one accepted command.
#[derive(Debug)]
pub struct AcceptedEntry {
    pub envelope: CommandEnvelope,
    pub batch: EventBatch,
}

/// Per-command result for one rejected command.
#[derive(Debug)]
pub struct RejectedEntry {
    pub envelope: CommandEnvelope,
    /// Human-readable rejection reason (Debug repr of the validator error).
    pub reason: String,
}

/// Summary of one completed tick.
#[derive(Debug)]
pub struct TickOutcome {
    pub tick: u64,
    pub accepted: Vec<AcceptedEntry>,
    pub rejected: Vec<RejectedEntry>,
    /// Total [`core_events::DomainEvent`]s applied this tick.
    pub events_applied: usize,
}

impl TickOutcome {
    pub fn accepted_count(&self) -> usize {
        self.accepted.len()
    }

    pub fn rejected_count(&self) -> usize {
        self.rejected.len()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core_commands::{Command, CommandKind, EntityCommand};
    use core_ids::EntityId;

    #[test]
    fn tick_input_push_and_count() {
        let mut input = TickInput::new(1);
        assert!(input.commands.is_empty());
        input.push(CommandEnvelope::new(
            CommandKind::System,
            Command::Entity(EntityCommand::Create {
                id: EntityId::new(1),
            }),
        ));
        assert_eq!(input.commands.len(), 1);
    }

    #[test]
    fn tick_phases_are_distinct() {
        assert_ne!(TickPhase::CollectInput, TickPhase::Validate);
        assert_ne!(TickPhase::Validate, TickPhase::AccumulateEvents);
        assert_ne!(TickPhase::AccumulateEvents, TickPhase::ApplyEvents);
        assert_ne!(TickPhase::ApplyEvents, TickPhase::Snapshot);
    }

    #[test]
    fn snapshot_phase_contract_routes_payloads_outside_kernel() {
        let contract = TickPhase::Snapshot
            .snapshot_contract()
            .expect("snapshot phase has contract");
        assert!(!contract.kernel_emits_state_hash);
        assert!(!contract.kernel_emits_snapshot_payload);
        assert_eq!(contract.state_hash_owner, "sim-runner via core-snapshot");
        assert_eq!(
            contract.replay_checkpoint_owner,
            "sim-runner records, sim-replay encodes"
        );
        assert_eq!(
            contract.telemetry_owner,
            "downstream read-only telemetry projection"
        );
    }

    #[test]
    fn non_snapshot_phases_do_not_claim_snapshot_payloads() {
        for phase in [
            TickPhase::CollectInput,
            TickPhase::Validate,
            TickPhase::AccumulateEvents,
            TickPhase::ApplyEvents,
        ] {
            assert_eq!(phase.snapshot_contract(), None);
        }
    }
}

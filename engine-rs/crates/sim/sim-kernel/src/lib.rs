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
//! applied. The phases are named explicitly here so that Phase 3 (policy host)
//! and Phase 4 (replay) can attach at the right joints without restructuring
//! the kernel.

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
    /// Compute state hash / emit telemetry (Phase 4/5 placeholder).
    Snapshot,
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
}

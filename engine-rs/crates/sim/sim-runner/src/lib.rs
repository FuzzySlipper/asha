//! Headless tick execution for the ASHA authority core.
//!
//! # Lane
//!
//! `rust-state` — may depend on `core-ids`, `core-state`, `core-commands`,
//! `core-events`, `sim-kernel`, `sim-validator`, `sim-applier`. Must not
//! reference render, protocol, UI, or TypeScript packages.
//!
//! # Design
//!
//! [`run_tick`] wires the five kernel phases into a single function call:
//!
//! ```text
//! TickInput → validate each command → accumulate EventBatches
//!           → apply batches to StateStore → return TickOutcome
//! ```
//!
//! Rejected commands produce a [`RejectedEntry`] with the validator's
//! `Debug` reason; they do not touch the store. Accepted commands are applied
//! in submission order. The function returns a [`TickOutcome`] that callers
//! can inspect or forward to snapshot/telemetry layers (Phase 4/5).

#![forbid(unsafe_code)]

use core_state::StateStore;
use sim_applier::apply_batch;
use sim_kernel::{AcceptedEntry, RejectedEntry, TickInput, TickOutcome};
use sim_validator::validate;

/// Execute one authority tick: validate all proposed commands, apply accepted
/// event batches to `store` in order, and return the tick summary.
///
/// Rejected commands are recorded in [`TickOutcome::rejected`] and do not
/// mutate the store. Accepted commands are applied in submission order.
pub fn run_tick(store: &mut StateStore, input: TickInput) -> TickOutcome {
    let tick = input.tick;
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();

    // Phase: Validate + AccumulateEvents
    for envelope in input.commands {
        match validate(store, &envelope) {
            Ok(batch) => accepted.push(AcceptedEntry { envelope, batch }),
            Err(err) => rejected.push(RejectedEntry {
                envelope,
                reason: format!("{err:?}"),
            }),
        }
    }

    // Phase: ApplyEvents
    let mut events_applied = 0;
    for entry in &accepted {
        // apply_batch errors here would indicate a bug (validator already
        // checked the store); propagate as a panic to keep the path loud.
        apply_batch(store, &entry.batch)
            .expect("applier must not fail for validator-accepted events");
        events_applied += entry.batch.len();
    }

    TickOutcome {
        tick,
        accepted,
        rejected,
        events_applied,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core_commands::{
        Command, CommandKind, EntityCommand, ModeCommand, ProcessCommand, TagCommand,
    };
    use core_ids::{EntityId, ModeId, ProcessId, TagId};
    use sim_kernel::TickInput;

    fn sys(cmd: Command) -> core_commands::CommandEnvelope {
        core_commands::CommandEnvelope::new(CommandKind::System, cmd)
    }

    // ── Headless tick test ────────────────────────────────────────────────

    /// Phase 1 epic exit criterion: headless tick exercising the full
    /// authority path — propose → validate → apply → inspect state + hash.
    #[test]
    fn headless_tick_test_authority_core_flow() {
        use core_snapshot::{hash_store, snapshot};

        let mut store = StateStore::new();

        // -- Tick 1: define a tag, create an entity, start a process
        let mut input = TickInput::new(1);
        input.push(sys(Command::Tag(TagCommand::Define { id: TagId::new(1) })));
        input.push(sys(Command::Entity(EntityCommand::Create {
            id: EntityId::new(10),
        })));
        input.push(sys(Command::Mode(ModeCommand::Define {
            id: ModeId::new(1),
        })));
        input.push(sys(Command::Process(ProcessCommand::Start {
            id: ProcessId::new(1),
        })));

        let hash_before = hash_store(&store);
        let outcome = run_tick(&mut store, input);

        assert_eq!(outcome.tick, 1);
        assert_eq!(outcome.accepted_count(), 4);
        assert_eq!(outcome.rejected_count(), 0);
        assert_eq!(outcome.events_applied, 4);

        // State must have changed.
        let hash_after = hash_store(&store);
        assert_ne!(hash_before, hash_after, "tick must change state hash");

        assert!(store.tag(TagId::new(1)).is_some());
        assert!(store.entity(EntityId::new(10)).is_some());
        assert!(store.mode(ModeId::new(1)).is_some());
        assert!(store.process(ProcessId::new(1)).is_some());

        // -- Tick 2: add tag to entity, set process mode
        let mut input2 = TickInput::new(2);
        input2.push(sys(Command::Entity(EntityCommand::AddTag {
            id: EntityId::new(10),
            tag: TagId::new(1),
        })));
        input2.push(sys(Command::Process(ProcessCommand::SetMode {
            id: ProcessId::new(1),
            mode: ModeId::new(1),
        })));

        let outcome2 = run_tick(&mut store, input2);
        assert_eq!(outcome2.accepted_count(), 2);
        assert_eq!(outcome2.rejected_count(), 0);
        assert!(store
            .entity(EntityId::new(10))
            .unwrap()
            .tags
            .contains(&TagId::new(1)));
        assert_eq!(
            store.process(ProcessId::new(1)).unwrap().mode,
            Some(ModeId::new(1))
        );

        // -- Tick 3: delete entity
        let mut input3 = TickInput::new(3);
        input3.push(sys(Command::Entity(EntityCommand::Delete {
            id: EntityId::new(10),
        })));
        let outcome3 = run_tick(&mut store, input3);
        assert_eq!(outcome3.accepted_count(), 1);
        assert!(store.entity(EntityId::new(10)).is_none());

        // Snapshot the final state for inspectability.
        let snap = snapshot(&store);
        assert_eq!(snap.version, core_snapshot::SNAPSHOT_VERSION);
        assert_eq!(snap.hash, hash_store(&store));
    }

    // ── Rejected command does not mutate state ────────────────────────────

    #[test]
    fn rejected_command_does_not_mutate_store() {
        let mut store = StateStore::new();
        // Entity 99 does not exist — Delete should be rejected.
        let mut input = TickInput::new(1);
        input.push(sys(Command::Entity(EntityCommand::Delete {
            id: EntityId::new(99),
        })));

        let outcome = run_tick(&mut store, input);
        assert_eq!(outcome.accepted_count(), 0);
        assert_eq!(outcome.rejected_count(), 1);
        assert_eq!(outcome.events_applied, 0);
        assert!(store.entity(EntityId::new(99)).is_none()); // store unchanged
    }

    #[test]
    fn mixed_tick_accepted_and_rejected() {
        let mut store = StateStore::new();
        store.insert_entity(EntityId::new(1)); // already exists

        let mut input = TickInput::new(1);
        // This will be rejected (duplicate).
        input.push(sys(Command::Entity(EntityCommand::Create {
            id: EntityId::new(1),
        })));
        // This will be accepted (new entity).
        input.push(sys(Command::Entity(EntityCommand::Create {
            id: EntityId::new(2),
        })));

        let outcome = run_tick(&mut store, input);
        assert_eq!(outcome.accepted_count(), 1);
        assert_eq!(outcome.rejected_count(), 1);
        assert!(!outcome.rejected[0].reason.is_empty());
        // Entity 2 was created; entity 1 still exists unchanged.
        assert!(store.entity(EntityId::new(2)).is_some());
        assert_eq!(store.entity_count(), 2);
    }

    // ── Phase 1 epic exit criteria represented as tests ───────────────────

    /// create/update/delete entity fixture
    #[test]
    fn epic_exit_create_update_delete_entity() {
        let mut store = StateStore::new();

        // Create
        let mut i = TickInput::new(1);
        i.push(sys(Command::Tag(TagCommand::Define { id: TagId::new(1) })));
        i.push(sys(Command::Entity(EntityCommand::Create {
            id: EntityId::new(1),
        })));
        let o = run_tick(&mut store, i);
        assert_eq!(o.rejected_count(), 0);

        // Update (add tag)
        let mut i2 = TickInput::new(2);
        i2.push(sys(Command::Entity(EntityCommand::AddTag {
            id: EntityId::new(1),
            tag: TagId::new(1),
        })));
        let o2 = run_tick(&mut store, i2);
        assert_eq!(o2.rejected_count(), 0);
        assert!(store
            .entity(EntityId::new(1))
            .unwrap()
            .tags
            .contains(&TagId::new(1)));

        // Delete
        let mut i3 = TickInput::new(3);
        i3.push(sys(Command::Entity(EntityCommand::Delete {
            id: EntityId::new(1),
        })));
        let o3 = run_tick(&mut store, i3);
        assert_eq!(o3.rejected_count(), 0);
        assert!(store.entity(EntityId::new(1)).is_none());
    }

    /// command validation fixture
    #[test]
    fn epic_exit_command_validation_fixture() {
        let mut store = StateStore::new();
        store.insert_entity(EntityId::new(1));

        // Valid command.
        let mut i = TickInput::new(1);
        i.push(sys(Command::Entity(EntityCommand::Delete {
            id: EntityId::new(1),
        })));
        let o = run_tick(&mut store, i);
        assert_eq!(o.accepted_count(), 1);

        // Invalid command (already deleted).
        let mut i2 = TickInput::new(2);
        i2.push(sys(Command::Entity(EntityCommand::Delete {
            id: EntityId::new(1),
        })));
        let o2 = run_tick(&mut store, i2);
        assert_eq!(o2.rejected_count(), 1);
    }

    /// event application fixture (via full tick path)
    #[test]
    fn epic_exit_event_application_fixture() {
        let mut store = StateStore::new();
        let mut i = TickInput::new(1);
        i.push(sys(Command::Entity(EntityCommand::Create {
            id: EntityId::new(5),
        })));
        let o = run_tick(&mut store, i);
        assert_eq!(o.events_applied, 1);
        assert!(store.entity(EntityId::new(5)).is_some());
    }

    /// state hash fixture
    #[test]
    fn epic_exit_state_hash_fixture() {
        use core_snapshot::hash_store;

        let mut s1 = StateStore::new();
        let mut s2 = StateStore::new();

        let input_a = {
            let mut i = TickInput::new(1);
            i.push(sys(Command::Entity(EntityCommand::Create {
                id: EntityId::new(1),
            })));
            i
        };
        let input_b = {
            let mut i = TickInput::new(1);
            i.push(sys(Command::Entity(EntityCommand::Create {
                id: EntityId::new(1),
            })));
            i
        };

        run_tick(&mut s1, input_a);
        run_tick(&mut s2, input_b);

        assert_eq!(
            hash_store(&s1),
            hash_store(&s2),
            "same sequence → same hash"
        );

        // Different sequence → different hash.
        let mut i3 = TickInput::new(2);
        i3.push(sys(Command::Entity(EntityCommand::Create {
            id: EntityId::new(2),
        })));
        run_tick(&mut s1, i3);
        assert_ne!(hash_store(&s1), hash_store(&s2));
    }
}

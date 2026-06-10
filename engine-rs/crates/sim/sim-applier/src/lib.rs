//! Domain event application for the ASHA authority core.
//!
//! # Lane
//!
//! `rust-state` — may depend on `core-ids`, `core-state`, `core-events`,
//! `core-error`. Must not reference commands, render, protocol, or UI.
//!
//! # Design
//!
//! [`apply_batch`] and [`apply_event`] are the only mutation entry-points in
//! the authority path. They accept [`DomainEvent`]s produced by the validator
//! and write them through explicit [`StateStore`] methods — no shadow state,
//! no direct field access to store internals.
//!
//! An [`ApplyError`] signals a consistency violation (e.g. an event referencing
//! an ID that does not exist). In a well-ordered authority path this should
//! not occur; the error exists so misuse is loud rather than silent.

#![forbid(unsafe_code)]

use core_events::{DomainEvent, EventBatch};
use core_ids::{EntityId, ModeId, ProcessId, SignalId, SubjectId, TagId};
use core_state::StateStore;

// ── Error type ────────────────────────────────────────────────────────────────

/// Error returned when an event cannot be applied to the current store state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyError {
    EntityAlreadyExists { id: EntityId },
    EntityNotFound { id: EntityId },
    SubjectAlreadyExists { id: SubjectId },
    SubjectNotFound { id: SubjectId },
    ProcessAlreadyExists { id: ProcessId },
    ProcessNotFound { id: ProcessId },
    ModeAlreadyExists { id: ModeId },
    ModeNotFound { id: ModeId },
    SignalAlreadyExists { id: SignalId },
    SignalNotFound { id: SignalId },
    TagAlreadyExists { id: TagId },
    TagNotFound { id: TagId },
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Apply every event in `batch` to `store` in order.
///
/// Stops at the first error; partial application is intentional — the caller
/// is responsible for deciding whether to roll back or abort.
pub fn apply_batch(store: &mut StateStore, batch: &EventBatch) -> Result<(), ApplyError> {
    for event in batch.events() {
        apply_event(store, event)?;
    }
    Ok(())
}

/// Apply a single [`DomainEvent`] to `store`.
pub fn apply_event(store: &mut StateStore, event: &DomainEvent) -> Result<(), ApplyError> {
    match event {
        DomainEvent::EntityCreated { id } => {
            if !store.insert_entity(*id) {
                return Err(ApplyError::EntityAlreadyExists { id: *id });
            }
        }
        DomainEvent::EntityTagAdded { id, tag } => {
            let rec = store
                .entity_mut(*id)
                .ok_or(ApplyError::EntityNotFound { id: *id })?;
            rec.tags.insert(*tag);
        }
        DomainEvent::EntityTagRemoved { id, tag } => {
            let rec = store
                .entity_mut(*id)
                .ok_or(ApplyError::EntityNotFound { id: *id })?;
            rec.tags.remove(tag);
        }
        DomainEvent::EntityDeleted { id } => {
            if !store.remove_entity(*id) {
                return Err(ApplyError::EntityNotFound { id: *id });
            }
        }
        DomainEvent::SubjectCreated { id } => {
            if !store.insert_subject(*id) {
                return Err(ApplyError::SubjectAlreadyExists { id: *id });
            }
        }
        DomainEvent::SubjectDeleted { id } => {
            if !store.remove_subject(*id) {
                return Err(ApplyError::SubjectNotFound { id: *id });
            }
        }
        DomainEvent::ProcessStarted { id } => {
            if !store.insert_process(*id) {
                return Err(ApplyError::ProcessAlreadyExists { id: *id });
            }
        }
        DomainEvent::ProcessModeSet { id, mode } => {
            let rec = store
                .process_mut(*id)
                .ok_or(ApplyError::ProcessNotFound { id: *id })?;
            rec.mode = Some(*mode);
        }
        DomainEvent::ProcessStopped { id } => {
            if !store.remove_process(*id) {
                return Err(ApplyError::ProcessNotFound { id: *id });
            }
        }
        DomainEvent::ModeDefined { id } => {
            if !store.insert_mode(*id) {
                return Err(ApplyError::ModeAlreadyExists { id: *id });
            }
        }
        DomainEvent::ModeUndefined { id } => {
            if !store.remove_mode(*id) {
                return Err(ApplyError::ModeNotFound { id: *id });
            }
        }
        DomainEvent::SignalDefined { id } => {
            if !store.insert_signal(*id) {
                return Err(ApplyError::SignalAlreadyExists { id: *id });
            }
        }
        DomainEvent::SignalUndefined { id } => {
            if !store.remove_signal(*id) {
                return Err(ApplyError::SignalNotFound { id: *id });
            }
        }
        DomainEvent::TagDefined { id } => {
            if !store.insert_tag(*id) {
                return Err(ApplyError::TagAlreadyExists { id: *id });
            }
        }
        DomainEvent::TagUndefined { id } => {
            if !store.remove_tag(*id) {
                return Err(ApplyError::TagNotFound { id: *id });
            }
        }
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core_events::EventBatch;
    use core_ids::{EntityId, ModeId, ProcessId, TagId};

    // ── Event application fixture — create/update/delete lifecycle ────────

    #[test]
    fn event_application_fixture_entity_lifecycle() {
        let mut store = StateStore::new();
        let id = EntityId::new(1);

        // Create
        apply_event(&mut store, &DomainEvent::EntityCreated { id }).unwrap();
        assert!(store.entity(id).is_some());

        // Update (add tag — tag need not be pre-defined for applier; validator owns that check)
        let tid = TagId::new(5);
        apply_event(&mut store, &DomainEvent::EntityTagAdded { id, tag: tid }).unwrap();
        assert!(store.entity(id).unwrap().tags.contains(&tid));

        // Remove tag
        apply_event(&mut store, &DomainEvent::EntityTagRemoved { id, tag: tid }).unwrap();
        assert!(!store.entity(id).unwrap().tags.contains(&tid));

        // Delete
        apply_event(&mut store, &DomainEvent::EntityDeleted { id }).unwrap();
        assert!(store.entity(id).is_none());
    }

    #[test]
    fn event_application_fixture_process_lifecycle() {
        let mut store = StateStore::new();
        let pid = ProcessId::new(1);
        let mid = ModeId::new(7);

        apply_event(&mut store, &DomainEvent::ProcessStarted { id: pid }).unwrap();
        assert!(store.process(pid).unwrap().mode.is_none());

        apply_event(
            &mut store,
            &DomainEvent::ProcessModeSet { id: pid, mode: mid },
        )
        .unwrap();
        assert_eq!(store.process(pid).unwrap().mode, Some(mid));

        apply_event(&mut store, &DomainEvent::ProcessStopped { id: pid }).unwrap();
        assert!(store.process(pid).is_none());
    }

    // ── Ordered batch application ─────────────────────────────────────────

    #[test]
    fn event_batch_applied_in_order() {
        let mut store = StateStore::new();
        let id = EntityId::new(10);
        let tid = TagId::new(3);

        let mut batch = EventBatch::new();
        batch.push(DomainEvent::EntityCreated { id });
        batch.push(DomainEvent::EntityTagAdded { id, tag: tid });

        apply_batch(&mut store, &batch).unwrap();
        assert!(store.entity(id).unwrap().tags.contains(&tid));
    }

    #[test]
    fn event_batch_stops_at_first_error() {
        let mut store = StateStore::new();
        let id = EntityId::new(1);
        store.insert_entity(id); // already exists

        let mut batch = EventBatch::new();
        // First event will fail (duplicate).
        batch.push(DomainEvent::EntityCreated { id });
        // Second event would succeed if reached.
        batch.push(DomainEvent::EntityCreated {
            id: EntityId::new(2),
        });

        let err = apply_batch(&mut store, &batch).expect_err("duplicate must fail");
        assert_eq!(err, ApplyError::EntityAlreadyExists { id });
        // Second entity must NOT have been created.
        assert!(store.entity(EntityId::new(2)).is_none());
    }

    // ── Error cases ───────────────────────────────────────────────────────

    #[test]
    fn apply_entity_tag_on_missing_entity_errors() {
        let mut store = StateStore::new();
        let err = apply_event(
            &mut store,
            &DomainEvent::EntityTagAdded {
                id: EntityId::new(99),
                tag: TagId::new(1),
            },
        )
        .expect_err("missing entity must error");
        assert_eq!(
            err,
            ApplyError::EntityNotFound {
                id: EntityId::new(99)
            }
        );
    }

    #[test]
    fn apply_process_mode_on_missing_process_errors() {
        let mut store = StateStore::new();
        let err = apply_event(
            &mut store,
            &DomainEvent::ProcessModeSet {
                id: ProcessId::new(5),
                mode: ModeId::new(1),
            },
        )
        .expect_err("missing process must error");
        assert_eq!(
            err,
            ApplyError::ProcessNotFound {
                id: ProcessId::new(5)
            }
        );
    }
}

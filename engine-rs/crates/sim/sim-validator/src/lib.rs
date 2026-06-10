//! Command validation for the ASHA authority core.
//!
//! # Lane
//!
//! `rust-state` — may depend on `core-ids`, `core-state`, `core-commands`,
//! `core-events`, `core-error`. Must not reference render, protocol, or UI.
//!
//! # Design
//!
//! [`validate`] is a pure function: it reads the current [`StateStore`] and a
//! proposed [`CommandEnvelope`], then either returns an [`EventBatch`] of
//! accepted domain events or a structured [`ValidationError`] explaining why
//! the command was rejected.
//!
//! No state is mutated here. Applying the returned batch is the applier's job
//! (`sim-applier`). This separation means a failing validation test always
//! points at this crate, and a failing application test always points at the
//! applier.

#![forbid(unsafe_code)]

use core_commands::{
    Command, CommandEnvelope, EntityCommand, ModeCommand, ProcessCommand, SignalCommand,
    SubjectCommand, TagCommand,
};
use core_events::{DomainEvent, EventBatch};
use core_ids::{EntityId, ModeId, ProcessId, SignalId, SubjectId, TagId};
use core_state::StateStore;

// ── Error type ────────────────────────────────────────────────────────────────

/// Structured rejection reason returned when a command fails validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EntityAlreadyExists { id: EntityId },
    EntityNotFound { id: EntityId },
    TagNotFound { id: TagId },
    TagAlreadyOnEntity { id: EntityId, tag: TagId },
    TagNotOnEntity { id: EntityId, tag: TagId },
    SubjectAlreadyExists { id: SubjectId },
    SubjectNotFound { id: SubjectId },
    ProcessAlreadyExists { id: ProcessId },
    ProcessNotFound { id: ProcessId },
    ModeAlreadyExists { id: ModeId },
    ModeNotFound { id: ModeId },
    SignalAlreadyExists { id: SignalId },
    SignalNotFound { id: SignalId },
    TagAlreadyDefined { id: TagId },
    TagDefinitionNotFound { id: TagId },
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Validate a proposed command against the current store.
///
/// Returns a batch of [`DomainEvent`]s to apply if the command is accepted,
/// or a [`ValidationError`] if it is rejected. The store is not mutated.
pub fn validate(
    store: &StateStore,
    envelope: &CommandEnvelope,
) -> Result<EventBatch, ValidationError> {
    let mut batch = EventBatch::new();
    match &envelope.command {
        Command::Entity(cmd) => validate_entity(store, cmd, &mut batch)?,
        Command::Subject(cmd) => validate_subject(store, cmd, &mut batch)?,
        Command::Process(cmd) => validate_process(store, cmd, &mut batch)?,
        Command::Mode(cmd) => validate_mode(store, cmd, &mut batch)?,
        Command::Signal(cmd) => validate_signal(store, cmd, &mut batch)?,
        Command::Tag(cmd) => validate_tag(store, cmd, &mut batch)?,
    }
    Ok(batch)
}

// ── Per-noun validators ───────────────────────────────────────────────────────

fn validate_entity(
    store: &StateStore,
    cmd: &EntityCommand,
    batch: &mut EventBatch,
) -> Result<(), ValidationError> {
    match cmd {
        EntityCommand::Create { id } => {
            if store.entity(*id).is_some() {
                return Err(ValidationError::EntityAlreadyExists { id: *id });
            }
            batch.push(DomainEvent::EntityCreated { id: *id });
        }
        EntityCommand::AddTag { id, tag } => {
            let rec = store
                .entity(*id)
                .ok_or(ValidationError::EntityNotFound { id: *id })?;
            if store.tag(*tag).is_none() {
                return Err(ValidationError::TagNotFound { id: *tag });
            }
            if rec.tags.contains(tag) {
                return Err(ValidationError::TagAlreadyOnEntity { id: *id, tag: *tag });
            }
            batch.push(DomainEvent::EntityTagAdded { id: *id, tag: *tag });
        }
        EntityCommand::RemoveTag { id, tag } => {
            let rec = store
                .entity(*id)
                .ok_or(ValidationError::EntityNotFound { id: *id })?;
            if !rec.tags.contains(tag) {
                return Err(ValidationError::TagNotOnEntity { id: *id, tag: *tag });
            }
            batch.push(DomainEvent::EntityTagRemoved { id: *id, tag: *tag });
        }
        EntityCommand::Delete { id } => {
            if store.entity(*id).is_none() {
                return Err(ValidationError::EntityNotFound { id: *id });
            }
            batch.push(DomainEvent::EntityDeleted { id: *id });
        }
    }
    Ok(())
}

fn validate_subject(
    store: &StateStore,
    cmd: &SubjectCommand,
    batch: &mut EventBatch,
) -> Result<(), ValidationError> {
    match cmd {
        SubjectCommand::Create { id } => {
            if store.subject(*id).is_some() {
                return Err(ValidationError::SubjectAlreadyExists { id: *id });
            }
            batch.push(DomainEvent::SubjectCreated { id: *id });
        }
        SubjectCommand::Delete { id } => {
            if store.subject(*id).is_none() {
                return Err(ValidationError::SubjectNotFound { id: *id });
            }
            batch.push(DomainEvent::SubjectDeleted { id: *id });
        }
    }
    Ok(())
}

fn validate_process(
    store: &StateStore,
    cmd: &ProcessCommand,
    batch: &mut EventBatch,
) -> Result<(), ValidationError> {
    match cmd {
        ProcessCommand::Start { id } => {
            if store.process(*id).is_some() {
                return Err(ValidationError::ProcessAlreadyExists { id: *id });
            }
            batch.push(DomainEvent::ProcessStarted { id: *id });
        }
        ProcessCommand::SetMode { id, mode } => {
            if store.process(*id).is_none() {
                return Err(ValidationError::ProcessNotFound { id: *id });
            }
            if store.mode(*mode).is_none() {
                return Err(ValidationError::ModeNotFound { id: *mode });
            }
            batch.push(DomainEvent::ProcessModeSet {
                id: *id,
                mode: *mode,
            });
        }
        ProcessCommand::Stop { id } => {
            if store.process(*id).is_none() {
                return Err(ValidationError::ProcessNotFound { id: *id });
            }
            batch.push(DomainEvent::ProcessStopped { id: *id });
        }
    }
    Ok(())
}

fn validate_mode(
    store: &StateStore,
    cmd: &ModeCommand,
    batch: &mut EventBatch,
) -> Result<(), ValidationError> {
    match cmd {
        ModeCommand::Define { id } => {
            if store.mode(*id).is_some() {
                return Err(ValidationError::ModeAlreadyExists { id: *id });
            }
            batch.push(DomainEvent::ModeDefined { id: *id });
        }
        ModeCommand::Undefine { id } => {
            if store.mode(*id).is_none() {
                return Err(ValidationError::ModeNotFound { id: *id });
            }
            batch.push(DomainEvent::ModeUndefined { id: *id });
        }
    }
    Ok(())
}

fn validate_signal(
    store: &StateStore,
    cmd: &SignalCommand,
    batch: &mut EventBatch,
) -> Result<(), ValidationError> {
    match cmd {
        SignalCommand::Define { id } => {
            if store.signal(*id).is_some() {
                return Err(ValidationError::SignalAlreadyExists { id: *id });
            }
            batch.push(DomainEvent::SignalDefined { id: *id });
        }
        SignalCommand::Undefine { id } => {
            if store.signal(*id).is_none() {
                return Err(ValidationError::SignalNotFound { id: *id });
            }
            batch.push(DomainEvent::SignalUndefined { id: *id });
        }
    }
    Ok(())
}

fn validate_tag(
    store: &StateStore,
    cmd: &TagCommand,
    batch: &mut EventBatch,
) -> Result<(), ValidationError> {
    match cmd {
        TagCommand::Define { id } => {
            if store.tag(*id).is_some() {
                return Err(ValidationError::TagAlreadyDefined { id: *id });
            }
            batch.push(DomainEvent::TagDefined { id: *id });
        }
        TagCommand::Undefine { id } => {
            if store.tag(*id).is_none() {
                return Err(ValidationError::TagDefinitionNotFound { id: *id });
            }
            batch.push(DomainEvent::TagUndefined { id: *id });
        }
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core_commands::{CommandKind, EntityCommand, ProcessCommand, SubjectCommand, TagCommand};
    use core_ids::{EntityId, ModeId, ProcessId, TagId};

    fn input(cmd: Command) -> CommandEnvelope {
        CommandEnvelope::new(CommandKind::Input, cmd)
    }

    fn system(cmd: Command) -> CommandEnvelope {
        CommandEnvelope::new(CommandKind::System, cmd)
    }

    // ── Command validation fixture — accepted ─────────────────────────────

    #[test]
    fn command_validation_fixture_entity_create_accepted() {
        let store = StateStore::new();
        let env = input(Command::Entity(EntityCommand::Create {
            id: EntityId::new(1),
        }));
        let batch = validate(&store, &env).expect("fresh entity must be accepted");
        assert_eq!(batch.len(), 1);
        assert!(matches!(
            batch.events()[0],
            DomainEvent::EntityCreated { .. }
        ));
    }

    #[test]
    fn command_validation_fixture_tag_define_then_add_to_entity() {
        let mut store = StateStore::new();
        let eid = EntityId::new(10);
        let tid = TagId::new(5);

        // Seed state directly so we can test AddTag validation in isolation.
        store.insert_entity(eid);
        store.insert_tag(tid);

        let env = input(Command::Entity(EntityCommand::AddTag { id: eid, tag: tid }));
        let batch = validate(&store, &env).expect("add tag must be accepted");
        assert_eq!(batch.len(), 1);
        assert!(matches!(
            batch.events()[0],
            DomainEvent::EntityTagAdded { .. }
        ));
    }

    // ── Command validation fixture — rejected (stale reference) ───────────

    #[test]
    fn command_validation_fixture_entity_create_duplicate_rejected() {
        let mut store = StateStore::new();
        let id = EntityId::new(1);
        store.insert_entity(id);

        let env = input(Command::Entity(EntityCommand::Create { id }));
        let err = validate(&store, &env).expect_err("duplicate create must be rejected");
        assert_eq!(err, ValidationError::EntityAlreadyExists { id });
    }

    #[test]
    fn command_validation_fixture_stale_entity_id_rejected() {
        let store = StateStore::new(); // entity 99 never created
        let env = input(Command::Entity(EntityCommand::Delete {
            id: EntityId::new(99),
        }));
        let err = validate(&store, &env).expect_err("stale id must be rejected");
        assert_eq!(
            err,
            ValidationError::EntityNotFound {
                id: EntityId::new(99)
            }
        );
    }

    #[test]
    fn command_validation_add_tag_missing_tag_definition_rejected() {
        let mut store = StateStore::new();
        store.insert_entity(EntityId::new(1));
        // Tag 7 is not defined in the store.
        let env = input(Command::Entity(EntityCommand::AddTag {
            id: EntityId::new(1),
            tag: TagId::new(7),
        }));
        let err = validate(&store, &env).expect_err("undefined tag must be rejected");
        assert_eq!(err, ValidationError::TagNotFound { id: TagId::new(7) });
    }

    #[test]
    fn command_validation_add_tag_duplicate_rejected() {
        let mut store = StateStore::new();
        let eid = EntityId::new(1);
        let tid = TagId::new(3);
        store.insert_entity(eid);
        store.insert_tag(tid);
        store.entity_mut(eid).unwrap().tags.insert(tid);

        let env = input(Command::Entity(EntityCommand::AddTag { id: eid, tag: tid }));
        let err = validate(&store, &env).expect_err("duplicate tag must be rejected");
        assert_eq!(
            err,
            ValidationError::TagAlreadyOnEntity { id: eid, tag: tid }
        );
    }

    #[test]
    fn command_validation_process_set_mode_missing_mode_rejected() {
        let mut store = StateStore::new();
        store.insert_process(ProcessId::new(1));
        // Mode 5 is not defined.
        let env = system(Command::Process(ProcessCommand::SetMode {
            id: ProcessId::new(1),
            mode: ModeId::new(5),
        }));
        let err = validate(&store, &env).expect_err("undefined mode must be rejected");
        assert_eq!(err, ValidationError::ModeNotFound { id: ModeId::new(5) });
    }

    // ── Read-propose-validate separation ─────────────────────────────────

    #[test]
    fn validate_does_not_mutate_store() {
        let mut store = StateStore::new();
        store.insert_entity(EntityId::new(1));
        let count_before = store.entity_count();

        // Propose creating another entity — validate it but don't apply.
        let env = input(Command::Entity(EntityCommand::Create {
            id: EntityId::new(2),
        }));
        let _batch = validate(&store, &env).unwrap();

        // Store must be unchanged.
        assert_eq!(store.entity_count(), count_before);
        assert!(store.entity(EntityId::new(2)).is_none());
    }

    // ── Subject command validation ────────────────────────────────────────

    #[test]
    fn subject_create_accepted_and_duplicate_rejected() {
        let mut store = StateStore::new();
        let id = core_ids::SubjectId::new(1);
        let env = input(Command::Subject(SubjectCommand::Create { id }));
        assert!(validate(&store, &env).is_ok());

        store.insert_subject(id);
        let err = validate(&store, &env).expect_err("duplicate subject must be rejected");
        assert_eq!(err, ValidationError::SubjectAlreadyExists { id });
    }

    // ── Tag definition validation ─────────────────────────────────────────

    #[test]
    fn tag_define_accepted_and_duplicate_rejected() {
        let mut store = StateStore::new();
        let id = TagId::new(99);
        let env = system(Command::Tag(TagCommand::Define { id }));
        assert!(validate(&store, &env).is_ok());

        store.insert_tag(id);
        let err = validate(&store, &env).expect_err("duplicate tag define must be rejected");
        assert_eq!(err, ValidationError::TagAlreadyDefined { id });
    }
}

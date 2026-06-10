//! Script border shapes for the ASHA generated-contract boundary.
//!
//! # Lane
//!
//! `contract-steward` — owns the border between the Rust authority core and the
//! constrained TypeScript policy host. May depend on `core-ids`, `core-error`,
//! `core-state`, and `core-commands`; it currently uses only the first and last
//! because the border is pure data with no behavior.
//!
//! # Border ownership
//!
//! A policy script lives in TypeScript. It is handed a *read-only view* of the
//! world, and the only thing it may hand back is a *proposed command*. The
//! authority core then either accepts that command or returns a *rejection*.
//! Those three shapes — view, command, rejection — are this crate's entire
//! responsibility, and they are what Phase 2 codegen turns into TypeScript.
//!
//! - [`ScriptView`] is the read-only projection a policy sees.
//! - The proposed-command shapes are re-exported from `core-commands` so there
//!   is exactly one definition of a command in the workspace; this crate is the
//!   border surface codegen reads them through.
//! - [`ScriptRejection`] is the border form of a validation failure. The Phase 1
//!   validator (`sim-validator::ValidationError`) is the internal producer that
//!   maps onto this shape; the border owns the contract, the validator owns the
//!   policy of when to emit it.
//!
//! # Forbidden convenience logic
//!
//! No validation, no command execution, no policy evaluation, no rendering. The
//! view is built by the host (Phase 3), not here. These types are inert data so
//! that the TypeScript side and the Rust side cannot disagree about shape.

#![forbid(unsafe_code)]

use core_ids::{EntityId, ModeId, ProcessId, SignalId, SubjectId, TagId};

// The proposed-command vocabulary is defined once in `core-commands`. Re-export
// it as the border surface so codegen and TypeScript consumers see a single
// canonical command union rather than a parallel copy that could drift.
pub use core_commands::{
    Command, CommandEnvelope, CommandKind, EntityCommand, ModeCommand, ProcessCommand,
    SignalCommand, SubjectCommand, TagCommand,
};

// ── Read-only view ────────────────────────────────────────────────────────────

/// One entity as seen by a policy: its identity and the tags currently on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityView {
    pub id: EntityId,
    /// Tags in ascending order, mirroring the authoritative snapshot ordering.
    pub tags: Vec<TagId>,
}

/// One process as seen by a policy: its identity and current mode, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessView {
    pub id: ProcessId,
    pub mode: Option<ModeId>,
}

/// The complete read-only projection handed to a policy script for one tick.
///
/// This is deliberately a flat, owned snapshot rather than a borrow of live
/// state: a policy may not mutate the world, and the border may not leak
/// authority-core internals. Collections are expected to be ID-sorted by the
/// host so the view is comparable and deterministic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptView {
    pub entities: Vec<EntityView>,
    pub subjects: Vec<SubjectId>,
    pub processes: Vec<ProcessView>,
    pub modes: Vec<ModeId>,
    pub signals: Vec<SignalId>,
    pub tags: Vec<TagId>,
}

impl ScriptView {
    /// An empty view — the projection of an empty world.
    pub fn empty() -> Self {
        Self::default()
    }
}

// ── Rejection ─────────────────────────────────────────────────────────────────

/// The border form of a command rejection.
///
/// When a policy proposes a [`Command`] that the authority core refuses, the
/// reason is reported back to TypeScript as one of these variants. Each variant
/// carries the offending references so the policy can explain or recover.
///
/// This mirrors the Phase 1 internal `sim-validator::ValidationError`; that type
/// is the producer and this is the published contract. They are intentionally
/// separate so the validator can evolve its internal reasons while the border
/// stays a stable, generated promise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptRejection {
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

/// The outcome the authority core reports for a single proposed command:
/// either the command was accepted or it was rejected with a reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptOutcome {
    Accepted,
    Rejected(ScriptRejection),
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_construction_and_default_empty() {
        assert_eq!(ScriptView::empty(), ScriptView::default());

        let view = ScriptView {
            entities: vec![EntityView {
                id: EntityId::new(1),
                tags: vec![TagId::new(3), TagId::new(7)],
            }],
            processes: vec![ProcessView {
                id: ProcessId::new(5),
                mode: Some(ModeId::new(2)),
            }],
            subjects: vec![SubjectId::new(9)],
            ..ScriptView::default()
        };

        assert_eq!(view.entities.len(), 1);
        assert_eq!(view.entities[0].tags, vec![TagId::new(3), TagId::new(7)]);
        assert_eq!(view.processes[0].mode, Some(ModeId::new(2)));
        assert!(view.modes.is_empty());
    }

    #[test]
    fn command_union_is_reachable_through_border() {
        // A command authored "as TypeScript would" via the border surface.
        let env = CommandEnvelope::new(
            CommandKind::Policy,
            Command::Entity(EntityCommand::Create {
                id: EntityId::new(1),
            }),
        );
        assert_eq!(env.kind, CommandKind::Policy);
        assert!(matches!(
            env.command,
            Command::Entity(EntityCommand::Create { .. })
        ));
    }

    #[test]
    fn rejection_carries_offending_references() {
        let r = ScriptRejection::TagAlreadyOnEntity {
            id: EntityId::new(1),
            tag: TagId::new(4),
        };
        if let ScriptRejection::TagAlreadyOnEntity { id, tag } = r {
            assert_eq!(id, EntityId::new(1));
            assert_eq!(tag, TagId::new(4));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn outcome_distinguishes_accept_and_reject() {
        let ok = ScriptOutcome::Accepted;
        let no = ScriptOutcome::Rejected(ScriptRejection::EntityNotFound {
            id: EntityId::new(99),
        });
        assert_ne!(ok, no);
        assert!(matches!(no, ScriptOutcome::Rejected(_)));
    }
}

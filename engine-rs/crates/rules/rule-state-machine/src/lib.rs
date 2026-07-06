//! Generic finite-state transition authority.
//!
//! # Lane
//!
//! `rust-rule` — owns product-neutral finite-state validation. It does not know
//! renderer, UI, TypeScript, product nouns, or policy execution details.
//!
//! # ID vocabulary
//!
//! This crate uses existing ASHA ids instead of inventing string keys:
//!
//! - [`EntityId`](core_ids::EntityId) owns one machine instance.
//! - [`ProcessId`](core_ids::ProcessId) names the machine/spec.
//! - [`ModeId`](core_ids::ModeId) names states.
//!
//! Accepted transitions emit a local replay-shaped [`StateMachineEvent`] and can
//! also project to the existing [`DomainEvent::ProcessModeSet`] border when a
//! caller wants to mirror the machine's current state into the process/mode
//! vocabulary.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use core_error::ErrorCategory;
use core_events::DomainEvent;
use core_ids::{EntityId, ModeId, ProcessId};

/// A reusable finite-state machine specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachineSpec {
    pub machine: ProcessId,
    states: BTreeSet<ModeId>,
    transitions: BTreeSet<(ModeId, ModeId)>,
}

impl StateMachineSpec {
    pub fn new(machine: ProcessId, states: impl IntoIterator<Item = ModeId>) -> Self {
        Self {
            machine,
            states: states.into_iter().collect(),
            transitions: BTreeSet::new(),
        }
    }

    pub fn allow(mut self, from: ModeId, to: ModeId) -> Self {
        self.transitions.insert((from, to));
        self
    }

    pub fn contains_state(&self, state: ModeId) -> bool {
        self.states.contains(&state)
    }

    pub fn allows_transition(&self, from: ModeId, to: ModeId) -> bool {
        self.transitions.contains(&(from, to))
    }

    pub fn states(&self) -> impl Iterator<Item = ModeId> + '_ {
        self.states.iter().copied()
    }

    pub fn transitions(&self) -> impl Iterator<Item = (ModeId, ModeId)> + '_ {
        self.transitions.iter().copied()
    }
}

/// One entity-owned state-machine instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineInstance {
    pub entity: EntityId,
    pub machine: ProcessId,
    pub current: ModeId,
    pub revision: u64,
}

/// A proposed transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionRequest {
    pub entity: EntityId,
    pub machine: ProcessId,
    pub expected: ModeId,
    pub next: ModeId,
    pub expected_revision: Option<u64>,
}

impl TransitionRequest {
    pub const fn new(entity: EntityId, machine: ProcessId, expected: ModeId, next: ModeId) -> Self {
        Self {
            entity,
            machine,
            expected,
            next,
            expected_revision: None,
        }
    }

    pub const fn expecting_revision(mut self, revision: u64) -> Self {
        self.expected_revision = Some(revision);
        self
    }
}

/// Authoritative event emitted by this rule crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMachineEvent {
    MachineAttached {
        entity: EntityId,
        machine: ProcessId,
        state: ModeId,
        revision: u64,
    },
    StateTransitioned {
        entity: EntityId,
        machine: ProcessId,
        from: ModeId,
        to: ModeId,
        revision: u64,
    },
}

impl StateMachineEvent {
    pub fn kind(self) -> &'static str {
        match self {
            StateMachineEvent::MachineAttached { .. } => "state_machine.attached.v0",
            StateMachineEvent::StateTransitioned { .. } => "state_machine.transitioned.v0",
        }
    }

    /// Deterministic, compact replay line for fixtures/evidence.
    pub fn replay_line(self) -> String {
        match self {
            StateMachineEvent::MachineAttached {
                entity,
                machine,
                state,
                revision,
            } => format!(
                "{} entity={} machine={} state={} rev={}",
                self.kind(),
                entity.raw(),
                machine.raw(),
                state.raw(),
                revision
            ),
            StateMachineEvent::StateTransitioned {
                entity,
                machine,
                from,
                to,
                revision,
            } => format!(
                "{} entity={} machine={} from={} to={} rev={}",
                self.kind(),
                entity.raw(),
                machine.raw(),
                from.raw(),
                to.raw(),
                revision
            ),
        }
    }
}

/// Result of an accepted transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionApplied {
    pub instance: MachineInstance,
    pub previous: ModeId,
    pub event: StateMachineEvent,
}

impl TransitionApplied {
    /// Bridge to the existing process/mode event vocabulary.
    ///
    /// This does not include `entity` because the existing border models process
    /// mode by [`ProcessId`] only; callers that need the entity keep the local
    /// [`StateMachineEvent`] alongside it.
    pub fn process_mode_event(self) -> DomainEvent {
        DomainEvent::ProcessModeSet {
            id: self.instance.machine,
            mode: self.instance.current,
        }
    }
}

/// Typed rejection for state-machine operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMachineError {
    MachineAlreadyDefined {
        machine: ProcessId,
    },
    MachineMissing {
        machine: ProcessId,
    },
    EntityMissing {
        entity: EntityId,
    },
    InstanceAlreadyAttached {
        entity: EntityId,
        machine: ProcessId,
    },
    InstanceMissing {
        entity: EntityId,
        machine: ProcessId,
    },
    InvalidState {
        machine: ProcessId,
        state: ModeId,
    },
    InvalidTransition {
        machine: ProcessId,
        from: ModeId,
        to: ModeId,
    },
    StaleCurrentState {
        entity: EntityId,
        machine: ProcessId,
        expected: ModeId,
        actual: ModeId,
    },
    StaleRevision {
        entity: EntityId,
        machine: ProcessId,
        expected: u64,
        actual: u64,
    },
}

impl StateMachineError {
    pub fn category(self) -> ErrorCategory {
        match self {
            StateMachineError::MachineMissing { .. }
            | StateMachineError::EntityMissing { .. }
            | StateMachineError::InstanceMissing { .. } => ErrorCategory::NotFound,
            StateMachineError::MachineAlreadyDefined { .. }
            | StateMachineError::InstanceAlreadyAttached { .. }
            | StateMachineError::StaleCurrentState { .. }
            | StateMachineError::StaleRevision { .. } => ErrorCategory::Conflict,
            StateMachineError::InvalidState { .. }
            | StateMachineError::InvalidTransition { .. } => ErrorCategory::Invalid,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            StateMachineError::MachineAlreadyDefined { .. } => "machine_already_defined",
            StateMachineError::MachineMissing { .. } => "machine_missing",
            StateMachineError::EntityMissing { .. } => "entity_missing",
            StateMachineError::InstanceAlreadyAttached { .. } => "instance_already_attached",
            StateMachineError::InstanceMissing { .. } => "instance_missing",
            StateMachineError::InvalidState { .. } => "invalid_state",
            StateMachineError::InvalidTransition { .. } => "invalid_transition",
            StateMachineError::StaleCurrentState { .. } => "stale_current_state",
            StateMachineError::StaleRevision { .. } => "stale_revision",
        }
    }
}

impl core::fmt::Display for StateMachineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for StateMachineError {}

/// In-memory authority store for generic machine specs and entity-owned instances.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateMachineStore {
    machines: BTreeMap<ProcessId, StateMachineSpec>,
    entities: BTreeSet<EntityId>,
    instances: BTreeMap<(EntityId, ProcessId), MachineInstance>,
}

impl StateMachineStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an entity id as eligible to own machine instances.
    pub fn register_entity(&mut self, entity: EntityId) -> bool {
        self.entities.insert(entity)
    }

    pub fn define_machine(&mut self, spec: StateMachineSpec) -> Result<(), StateMachineError> {
        if self.machines.contains_key(&spec.machine) {
            return Err(StateMachineError::MachineAlreadyDefined {
                machine: spec.machine,
            });
        }
        self.machines.insert(spec.machine, spec);
        Ok(())
    }

    pub fn attach(
        &mut self,
        entity: EntityId,
        machine: ProcessId,
        initial: ModeId,
    ) -> Result<StateMachineEvent, StateMachineError> {
        if !self.entities.contains(&entity) {
            return Err(StateMachineError::EntityMissing { entity });
        }
        let spec = self
            .machines
            .get(&machine)
            .ok_or(StateMachineError::MachineMissing { machine })?;
        if !spec.contains_state(initial) {
            return Err(StateMachineError::InvalidState {
                machine,
                state: initial,
            });
        }
        let key = (entity, machine);
        if self.instances.contains_key(&key) {
            return Err(StateMachineError::InstanceAlreadyAttached { entity, machine });
        }
        let instance = MachineInstance {
            entity,
            machine,
            current: initial,
            revision: 0,
        };
        self.instances.insert(key, instance);
        Ok(StateMachineEvent::MachineAttached {
            entity,
            machine,
            state: initial,
            revision: 0,
        })
    }

    pub fn instance(&self, entity: EntityId, machine: ProcessId) -> Option<MachineInstance> {
        self.instances.get(&(entity, machine)).copied()
    }

    pub fn apply_transition(
        &mut self,
        request: TransitionRequest,
    ) -> Result<TransitionApplied, StateMachineError> {
        let spec =
            self.machines
                .get(&request.machine)
                .ok_or(StateMachineError::MachineMissing {
                    machine: request.machine,
                })?;
        if !self.entities.contains(&request.entity) {
            return Err(StateMachineError::EntityMissing {
                entity: request.entity,
            });
        }
        if !spec.contains_state(request.next) {
            return Err(StateMachineError::InvalidState {
                machine: request.machine,
                state: request.next,
            });
        }
        if !spec.allows_transition(request.expected, request.next) {
            return Err(StateMachineError::InvalidTransition {
                machine: request.machine,
                from: request.expected,
                to: request.next,
            });
        }

        let key = (request.entity, request.machine);
        let instance = self
            .instances
            .get_mut(&key)
            .ok_or(StateMachineError::InstanceMissing {
                entity: request.entity,
                machine: request.machine,
            })?;
        if instance.current != request.expected {
            return Err(StateMachineError::StaleCurrentState {
                entity: request.entity,
                machine: request.machine,
                expected: request.expected,
                actual: instance.current,
            });
        }
        if let Some(expected_revision) = request.expected_revision {
            if instance.revision != expected_revision {
                return Err(StateMachineError::StaleRevision {
                    entity: request.entity,
                    machine: request.machine,
                    expected: expected_revision,
                    actual: instance.revision,
                });
            }
        }

        let previous = instance.current;
        instance.current = request.next;
        instance.revision = instance.revision.saturating_add(1);
        let updated = *instance;
        let event = StateMachineEvent::StateTransitioned {
            entity: request.entity,
            machine: request.machine,
            from: previous,
            to: request.next,
            revision: updated.revision,
        };
        Ok(TransitionApplied {
            instance: updated,
            previous,
            event,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(id: u64) -> EntityId {
        EntityId::new(id)
    }

    fn machine(id: u64) -> ProcessId {
        ProcessId::new(id)
    }

    fn state(id: u64) -> ModeId {
        ModeId::new(id)
    }

    fn spec() -> StateMachineSpec {
        StateMachineSpec::new(machine(10), [state(1), state(2), state(3)])
            .allow(state(1), state(2))
            .allow(state(2), state(3))
    }

    fn seeded_store() -> StateMachineStore {
        let mut store = StateMachineStore::new();
        store.register_entity(entity(7));
        store.define_machine(spec()).unwrap();
        store.attach(entity(7), machine(10), state(1)).unwrap();
        store
    }

    #[test]
    fn valid_transition_updates_instance_and_emits_replay_event() {
        let mut store = seeded_store();

        let applied = store
            .apply_transition(TransitionRequest::new(
                entity(7),
                machine(10),
                state(1),
                state(2),
            ))
            .unwrap();

        assert_eq!(applied.previous, state(1));
        assert_eq!(applied.instance.current, state(2));
        assert_eq!(applied.instance.revision, 1);
        assert_eq!(
            applied.event,
            StateMachineEvent::StateTransitioned {
                entity: entity(7),
                machine: machine(10),
                from: state(1),
                to: state(2),
                revision: 1
            }
        );
        assert_eq!(
            applied.event.replay_line(),
            "state_machine.transitioned.v0 entity=7 machine=10 from=1 to=2 rev=1"
        );
    }

    #[test]
    fn process_mode_event_bridge_is_explicit() {
        let mut store = seeded_store();
        let applied = store
            .apply_transition(TransitionRequest::new(
                entity(7),
                machine(10),
                state(1),
                state(2),
            ))
            .unwrap();

        assert_eq!(
            applied.process_mode_event(),
            DomainEvent::ProcessModeSet {
                id: machine(10),
                mode: state(2)
            }
        );
    }

    #[test]
    fn invalid_transition_is_rejected_without_mutation() {
        let mut store = seeded_store();

        let err = store
            .apply_transition(TransitionRequest::new(
                entity(7),
                machine(10),
                state(1),
                state(3),
            ))
            .unwrap_err();

        assert_eq!(
            err,
            StateMachineError::InvalidTransition {
                machine: machine(10),
                from: state(1),
                to: state(3)
            }
        );
        assert_eq!(err.category(), ErrorCategory::Invalid);
        assert_eq!(
            store.instance(entity(7), machine(10)).unwrap().current,
            state(1)
        );
    }

    #[test]
    fn stale_current_state_is_rejected() {
        let mut store = seeded_store();
        store
            .apply_transition(TransitionRequest::new(
                entity(7),
                machine(10),
                state(1),
                state(2),
            ))
            .unwrap();

        let err = store
            .apply_transition(TransitionRequest::new(
                entity(7),
                machine(10),
                state(1),
                state(2),
            ))
            .unwrap_err();

        assert_eq!(
            err,
            StateMachineError::StaleCurrentState {
                entity: entity(7),
                machine: machine(10),
                expected: state(1),
                actual: state(2)
            }
        );
        assert_eq!(err.category(), ErrorCategory::Conflict);
        assert_eq!(err.code(), "stale_current_state");
    }

    #[test]
    fn stale_revision_is_rejected() {
        let mut store = seeded_store();

        let err = store
            .apply_transition(
                TransitionRequest::new(entity(7), machine(10), state(1), state(2))
                    .expecting_revision(99),
            )
            .unwrap_err();

        assert_eq!(
            err,
            StateMachineError::StaleRevision {
                entity: entity(7),
                machine: machine(10),
                expected: 99,
                actual: 0
            }
        );
    }

    #[test]
    fn missing_entity_machine_and_instance_are_classified() {
        let mut store = StateMachineStore::new();
        store.register_entity(entity(7));

        let missing_machine = store.attach(entity(7), machine(10), state(1)).unwrap_err();
        assert_eq!(
            missing_machine,
            StateMachineError::MachineMissing {
                machine: machine(10)
            }
        );

        store.define_machine(spec()).unwrap();
        let missing_entity = store.attach(entity(8), machine(10), state(1)).unwrap_err();
        assert_eq!(
            missing_entity,
            StateMachineError::EntityMissing { entity: entity(8) }
        );

        let missing_instance = store
            .apply_transition(TransitionRequest::new(
                entity(7),
                machine(10),
                state(1),
                state(2),
            ))
            .unwrap_err();
        assert_eq!(
            missing_instance,
            StateMachineError::InstanceMissing {
                entity: entity(7),
                machine: machine(10)
            }
        );
        assert_eq!(missing_instance.category(), ErrorCategory::NotFound);
    }

    #[test]
    fn invalid_initial_or_next_state_is_rejected() {
        let mut store = StateMachineStore::new();
        store.register_entity(entity(7));
        store.define_machine(spec()).unwrap();

        let invalid_initial = store.attach(entity(7), machine(10), state(99)).unwrap_err();
        assert_eq!(
            invalid_initial,
            StateMachineError::InvalidState {
                machine: machine(10),
                state: state(99)
            }
        );

        store.attach(entity(7), machine(10), state(1)).unwrap();
        let invalid_next = store
            .apply_transition(TransitionRequest::new(
                entity(7),
                machine(10),
                state(1),
                state(99),
            ))
            .unwrap_err();
        assert_eq!(invalid_next.code(), "invalid_state");
    }

    #[test]
    fn deterministic_iteration_shapes_are_stable() {
        let spec = StateMachineSpec::new(machine(10), [state(3), state(1), state(2)])
            .allow(state(2), state(3))
            .allow(state(1), state(2));

        assert_eq!(
            spec.states().collect::<Vec<_>>(),
            vec![state(1), state(2), state(3)]
        );
        assert_eq!(
            spec.transitions().collect::<Vec<_>>(),
            vec![(state(1), state(2)), (state(2), state(3))]
        );
    }
}

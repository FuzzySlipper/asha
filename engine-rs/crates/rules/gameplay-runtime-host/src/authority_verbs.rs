//! Direct typed authority operations shared by authored programs and Fabric adapters.
//!
//! This is deliberately not a string/JSON dispatcher. Admission compiles public
//! semantic references into these closed Rust variants before a Session exists.

use core_entity::{
    ActivatableCapabilityKind as CoreCapabilityKind,
    CapabilityActivationPresence as CoreCapabilityPresence, EntityLifecycle, EntityStore,
    EntityTransform, TransformCommand,
};
use core_ids::{EntityId, ModeId, ProcessId};
use core_math::Vec3;
use protocol_entity_authoring::{
    ActivatableCapabilityKind, CapabilityActivationAction, CapabilityActivationOutcome,
    CapabilityActivationPresence, CapabilityActivationRequest,
};
use protocol_game_extension::{
    GameplayCausationRef, GameplayEmitterRef, GameplayEntityRef, GameplayEventEnvelope,
    GameplayEventPhase,
};
use rule_gameplay_fabric::{
    adapt_state_machine_event, gameplay_module_payload_hash, gameplay_payload_hash,
    CapabilityActivationGameplayPayload, GameplayOwnerEventContext, StandardGameplayEventKind,
};
use rule_state_machine::{
    apply_transition_to_instance, MachineInstance, StateMachineSpec, TransitionRequest,
};
use serde::{Deserialize, Serialize};
use svc_entity_authoring::{apply_rule_owned_capability_activation, EcrpRuleOwner};

pub(crate) const DIRECT_AUTHORITY_OWNER_ID: &str = "authority.asha.direct-verbs";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorityCapability {
    Collision,
    Controller,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AuthorityVerb {
    TransitionState {
        machine_index: usize,
        expected: ModeId,
        next: ModeId,
        expected_revision: u64,
    },
    SetRelativeTranslation {
        entity: EntityId,
        base_translation: [f32; 3],
        offset: [f32; 3],
    },
    SetCapabilityActive {
        entity: EntityId,
        capability: AuthorityCapability,
        active: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthorityMachine {
    pub spec: StateMachineSpec,
    pub instance: MachineInstance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AuthorityOwnerFact {
    pub sequence: u32,
    pub verb_id: String,
    pub entity: u64,
    pub authority_revision: u64,
    pub detail: String,
    pub fact_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthorityVerbExecution {
    pub facts: Vec<AuthorityOwnerFact>,
    pub events: Vec<GameplayEventEnvelope>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorityVerbRejection {
    MachineMissing,
    StateTransitionRejected,
    TransformMissing,
    TransformRejected,
    CapabilityMissing,
    CapabilityRejected,
    CollisionOccupied,
    EventEncodingFailed,
}

impl AuthorityVerbRejection {
    pub const fn code(self) -> &'static str {
        match self {
            Self::MachineMissing => "authorityVerbMachineMissing",
            Self::StateTransitionRejected => "authorityVerbStateTransitionRejected",
            Self::TransformMissing => "authorityVerbTransformMissing",
            Self::TransformRejected => "authorityVerbTransformRejected",
            Self::CapabilityMissing => "authorityVerbCapabilityMissing",
            Self::CapabilityRejected => "authorityVerbCapabilityRejected",
            Self::CollisionOccupied => "authorityVerbCollisionOccupied",
            Self::EventEncodingFailed => "authorityVerbEventEncodingFailed",
        }
    }
}

pub(crate) struct AuthorityVerbExecutor<'a> {
    pub entities: &'a mut EntityStore,
    pub machines: &'a mut [AuthorityMachine],
}

impl AuthorityVerbExecutor<'_> {
    /// Apply one ordered operation group atomically across EntityStore and
    /// symbolic state-machine authority.
    pub fn execute_atomic(
        &mut self,
        verbs: &[AuthorityVerb],
        context: &GameplayOwnerEventContext,
    ) -> Result<AuthorityVerbExecution, AuthorityVerbRejection> {
        let entity_checkpoint = self.entities.clone();
        let machine_checkpoint = self.machines.to_vec();
        let mut facts = Vec::with_capacity(verbs.len());
        let mut events = Vec::new();

        for (index, verb) in verbs.iter().enumerate() {
            let result = self.execute_one(verb, u32::try_from(index).unwrap_or(u32::MAX), context);
            match result {
                Ok((fact, event)) => {
                    facts.push(fact);
                    events.extend(event);
                }
                Err(error) => {
                    *self.entities = entity_checkpoint;
                    self.machines.clone_from_slice(&machine_checkpoint);
                    return Err(error);
                }
            }
        }

        Ok(AuthorityVerbExecution { facts, events })
    }

    fn execute_one(
        &mut self,
        verb: &AuthorityVerb,
        sequence: u32,
        context: &GameplayOwnerEventContext,
    ) -> Result<(AuthorityOwnerFact, Vec<GameplayEventEnvelope>), AuthorityVerbRejection> {
        match verb {
            AuthorityVerb::TransitionState {
                machine_index,
                expected,
                next,
                expected_revision,
            } => {
                let machine = self
                    .machines
                    .get_mut(*machine_index)
                    .ok_or(AuthorityVerbRejection::MachineMissing)?;
                let applied = apply_transition_to_instance(
                    &machine.spec,
                    machine.instance,
                    TransitionRequest::new(
                        machine.instance.entity,
                        machine.instance.machine,
                        *expected,
                        *next,
                    )
                    .expecting_revision(*expected_revision),
                )
                .map_err(|_| AuthorityVerbRejection::StateTransitionRejected)?;
                machine.instance = applied.instance;
                let event = adapt_state_machine_event(context, applied.event)
                    .map_err(|_| AuthorityVerbRejection::EventEncodingFailed)?;
                let fact = owner_fact(
                    sequence,
                    "transition-state",
                    machine.instance.entity,
                    machine.instance.revision,
                    format!(
                        "machine={};from={};to={}",
                        machine.instance.machine.raw(),
                        applied.previous.raw(),
                        machine.instance.current.raw()
                    ),
                );
                Ok((fact, vec![event]))
            }
            AuthorityVerb::SetRelativeTranslation {
                entity,
                base_translation,
                offset,
            } => {
                let current = self
                    .entities
                    .transform(*entity)
                    .ok_or(AuthorityVerbRejection::TransformMissing)?
                    .transform;
                let translation = [
                    base_translation[0] + offset[0],
                    base_translation[1] + offset[1],
                    base_translation[2] + offset[2],
                ];
                self.entities
                    .apply_transform(TransformCommand::Set {
                        id: *entity,
                        transform: EntityTransform {
                            translation: Vec3::new(translation[0], translation[1], translation[2]),
                            ..current
                        },
                    })
                    .map_err(|_| AuthorityVerbRejection::TransformRejected)?;
                let fact = owner_fact(
                    sequence,
                    "set-relative-translation",
                    *entity,
                    0,
                    format!(
                        "translation={},{},{}",
                        translation[0], translation[1], translation[2]
                    ),
                );
                Ok((fact, Vec::new()))
            }
            AuthorityVerb::SetCapabilityActive {
                entity,
                capability,
                active,
            } => {
                let (protocol_capability, core_capability, owner, capability_name) =
                    match capability {
                        AuthorityCapability::Collision => (
                            ActivatableCapabilityKind::Collision,
                            CoreCapabilityKind::Collision,
                            EcrpRuleOwner::CollisionRule,
                            "collision",
                        ),
                        AuthorityCapability::Controller => (
                            ActivatableCapabilityKind::Controller,
                            CoreCapabilityKind::Controller,
                            EcrpRuleOwner::ControllerRule,
                            "controller",
                        ),
                    };
                let before = self
                    .entities
                    .capability_activation(*entity, core_capability)
                    .ok_or(AuthorityVerbRejection::CapabilityMissing)?;
                let desired = if *active {
                    CoreCapabilityPresence::Active
                } else {
                    CoreCapabilityPresence::Inactive
                };
                let mut emitted = Vec::new();
                if before.presence != desired {
                    if *capability == AuthorityCapability::Collision
                        && *active
                        && !collision_activation_is_clear(self.entities, *entity)?
                    {
                        return Err(AuthorityVerbRejection::CollisionOccupied);
                    }
                    let outcome = apply_rule_owned_capability_activation(
                        self.entities,
                        owner,
                        CapabilityActivationRequest {
                            entity: *entity,
                            capability: protocol_capability,
                            action: if *active {
                                CapabilityActivationAction::Activate
                            } else {
                                CapabilityActivationAction::Deactivate
                            },
                        },
                    );
                    let CapabilityActivationOutcome::Accepted { event, .. } = outcome else {
                        return Err(AuthorityVerbRejection::CapabilityRejected);
                    };
                    emitted.push(capability_event(context, event)?);
                }
                let fact = owner_fact(
                    sequence,
                    "set-capability-active",
                    *entity,
                    0,
                    format!("capability={capability_name};active={active}"),
                );
                Ok((fact, emitted))
            }
        }
    }
}

pub(crate) fn collision_activation_is_clear(
    entities: &EntityStore,
    target: EntityId,
) -> Result<bool, AuthorityVerbRejection> {
    let target_bounds = world_bounds(entities, target)?;
    for core in entities.entities() {
        if core.id == target
            || core.lifecycle != EntityLifecycle::Active
            || entities.active_collision(core.id).is_none()
        {
            continue;
        }
        let Ok(other_bounds) = world_bounds(entities, core.id) else {
            continue;
        };
        if aabb_overlaps(target_bounds, other_bounds) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn world_bounds(
    entities: &EntityStore,
    entity: EntityId,
) -> Result<core_entity::Aabb, AuthorityVerbRejection> {
    let transform = entities
        .transform(entity)
        .ok_or(AuthorityVerbRejection::TransformMissing)?
        .transform;
    let bounds = entities
        .bounds(entity)
        .ok_or(AuthorityVerbRejection::CapabilityMissing)?
        .bounds;
    Ok(core_entity::Aabb::new(
        bounds.min + transform.translation,
        bounds.max + transform.translation,
    ))
}

fn aabb_overlaps(left: core_entity::Aabb, right: core_entity::Aabb) -> bool {
    left.min.x < right.max.x
        && left.max.x > right.min.x
        && left.min.y < right.max.y
        && left.max.y > right.min.y
        && left.min.z < right.max.z
        && left.max.z > right.min.z
}

fn owner_fact(
    sequence: u32,
    verb_id: &str,
    entity: EntityId,
    authority_revision: u64,
    detail: String,
) -> AuthorityOwnerFact {
    let mut fact = AuthorityOwnerFact {
        sequence,
        verb_id: verb_id.to_owned(),
        entity: entity.raw(),
        authority_revision,
        detail,
        fact_hash: String::new(),
    };
    fact.fact_hash = gameplay_module_payload_hash(
        serde_json::to_string(&fact)
            .expect("authority owner fact serializes")
            .as_bytes(),
    );
    fact
}

fn capability_event(
    context: &GameplayOwnerEventContext,
    event: protocol_entity_authoring::CapabilityActivationEvent,
) -> Result<GameplayEventEnvelope, AuthorityVerbRejection> {
    let capability = match event.capability {
        ActivatableCapabilityKind::Collision => "collision",
        ActivatableCapabilityKind::Controller => "controller",
    };
    let presence = |value| match value {
        CapabilityActivationPresence::Absent => "absent",
        CapabilityActivationPresence::Inactive => "inactive",
        CapabilityActivationPresence::Active => "active",
    };
    let canonical_payload = serde_json::to_vec(&CapabilityActivationGameplayPayload {
        entity: event.entity.raw(),
        capability: capability.to_owned(),
        from: presence(event.from).to_owned(),
        to: presence(event.to).to_owned(),
    })
    .map_err(|_| AuthorityVerbRejection::EventEncodingFailed)?;
    Ok(GameplayEventEnvelope {
        event_id: format!("{}:capability", context.root_id),
        event: StandardGameplayEventKind::CapabilityActivationChanged.contract(),
        tick: context.tick,
        root_sequence: context.root_sequence,
        wave: 0,
        event_sequence: context.first_event_sequence,
        phase: GameplayEventPhase::PostCommit,
        emitter: GameplayEmitterRef::Owner {
            owner_id: context.owner_id.clone(),
        },
        causation: GameplayCausationRef {
            root_id: context.root_id.clone(),
            parent_event_id: context.parent_event_id.clone(),
            decision_id: None,
        },
        source: Some(GameplayEntityRef {
            entity: event.entity,
        }),
        subjects: Vec::new(),
        targets: vec![GameplayEntityRef {
            entity: event.entity,
        }],
        scope: None,
        tags: vec!["capability".to_owned(), capability.to_owned()],
        payload_hash: gameplay_payload_hash(&canonical_payload),
        canonical_payload,
    })
}

pub(crate) fn machine_spec(
    machine: u64,
    states: impl IntoIterator<Item = u64>,
    transitions: impl IntoIterator<Item = (u64, u64)>,
) -> StateMachineSpec {
    let machine = ProcessId::new(machine);
    let mut spec = StateMachineSpec::new(machine, states.into_iter().map(ModeId::new));
    for (from, to) in transitions {
        spec = spec.allow(ModeId::new(from), ModeId::new(to));
    }
    spec
}

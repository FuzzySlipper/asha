use core_entity::{
    ActivatableCapabilityKind as CoreCapabilityKind,
    CapabilityActivationAction as CoreActivationAction,
    CapabilityActivationCommand as CoreActivationCommand,
    CapabilityActivationError as CoreActivationError,
    CapabilityActivationPresence as CoreActivationPresence,
    CapabilityActivationReadout as CoreActivationReadout, CapabilityActivationState, EntityStore,
};
use protocol_entity_authoring::{
    ActivatableCapabilityKind, CapabilityActivationAction, CapabilityActivationDiagnostic,
    CapabilityActivationDiagnosticCode, CapabilityActivationEntityLifecycle,
    CapabilityActivationEvent, CapabilityActivationOutcome, CapabilityActivationPresence,
    CapabilityActivationReadout, CapabilityActivationRequest,
};

use crate::{rule_owner_allows, EcrpCapabilityMutation, EcrpRuleOwner};

/// Apply an owner-scoped typed activation proposal. Owner validation occurs
/// before the core store is called, so a forbidden attempt cannot mutate state.
pub fn apply_rule_owned_capability_activation(
    store: &mut EntityStore,
    owner: EcrpRuleOwner,
    request: CapabilityActivationRequest,
) -> CapabilityActivationOutcome {
    let required_owner = owner_for(request.capability);
    let mutation = activation_mutation(request.capability);
    if !rule_owner_allows(owner, mutation) {
        return CapabilityActivationOutcome::Forbidden {
            diagnostic: diagnostic(
                CapabilityActivationDiagnosticCode::ForbiddenOwner,
                request.entity,
                request.capability,
                format!(
                    "{owner:?} cannot apply {mutation:?} to {}; {required_owner:?} owns it",
                    capability_label(request.capability),
                ),
            ),
        };
    }

    let command = CoreActivationCommand {
        entity: request.entity,
        capability: to_core_kind(request.capability),
        action: match request.action {
            CapabilityActivationAction::Activate => CoreActivationAction::Activate,
            CapabilityActivationAction::Deactivate => CoreActivationAction::Deactivate,
        },
    };
    match store.apply_capability_activation(command) {
        Ok(event) => {
            let readout = store
                .capability_activation(request.entity, command.capability)
                .expect("accepted activation has a known entity");
            CapabilityActivationOutcome::Accepted {
                event: CapabilityActivationEvent {
                    entity: event.entity,
                    capability: request.capability,
                    from: state_presence(event.from),
                    to: state_presence(event.to),
                },
                readout: project_readout(readout),
            }
        }
        Err(error) => CapabilityActivationOutcome::Rejected {
            diagnostic: map_error(error, request.capability),
        },
    }
}

/// Generated read projection for tooling and downstream consumers. Unknown
/// entities return no readout; known entities report absent/inactive/active.
pub fn project_capability_activation(
    store: &EntityStore,
    entity: core_ids::EntityId,
    capability: ActivatableCapabilityKind,
) -> Option<CapabilityActivationReadout> {
    store
        .capability_activation(entity, to_core_kind(capability))
        .map(project_readout)
}

fn owner_for(capability: ActivatableCapabilityKind) -> EcrpRuleOwner {
    match capability {
        ActivatableCapabilityKind::Collision => EcrpRuleOwner::CollisionRule,
        ActivatableCapabilityKind::Controller => EcrpRuleOwner::ControllerRule,
    }
}

pub(crate) fn activation_mutation(capability: ActivatableCapabilityKind) -> EcrpCapabilityMutation {
    match capability {
        ActivatableCapabilityKind::Collision => EcrpCapabilityMutation::ActivateCollision,
        ActivatableCapabilityKind::Controller => EcrpCapabilityMutation::ActivateController,
    }
}

fn to_core_kind(capability: ActivatableCapabilityKind) -> CoreCapabilityKind {
    match capability {
        ActivatableCapabilityKind::Collision => CoreCapabilityKind::Collision,
        ActivatableCapabilityKind::Controller => CoreCapabilityKind::Controller,
    }
}

fn project_readout(readout: CoreActivationReadout) -> CapabilityActivationReadout {
    CapabilityActivationReadout {
        entity: readout.entity,
        capability: match readout.capability {
            CoreCapabilityKind::Collision => ActivatableCapabilityKind::Collision,
            CoreCapabilityKind::Controller => ActivatableCapabilityKind::Controller,
        },
        presence: match readout.presence {
            CoreActivationPresence::Absent => CapabilityActivationPresence::Absent,
            CoreActivationPresence::Inactive => CapabilityActivationPresence::Inactive,
            CoreActivationPresence::Active => CapabilityActivationPresence::Active,
        },
        entity_lifecycle: match readout.entity_lifecycle {
            core_entity::EntityLifecycle::Active => CapabilityActivationEntityLifecycle::Active,
            core_entity::EntityLifecycle::Disabled => CapabilityActivationEntityLifecycle::Disabled,
            core_entity::EntityLifecycle::Tombstoned => {
                CapabilityActivationEntityLifecycle::Tombstoned
            }
        },
        effective_active: readout.effective_active,
    }
}

fn state_presence(state: CapabilityActivationState) -> CapabilityActivationPresence {
    match state {
        CapabilityActivationState::Active => CapabilityActivationPresence::Active,
        CapabilityActivationState::Inactive => CapabilityActivationPresence::Inactive,
    }
}

fn map_error(
    error: CoreActivationError,
    requested_capability: ActivatableCapabilityKind,
) -> CapabilityActivationDiagnostic {
    match error {
        CoreActivationError::UnknownEntity { entity } => diagnostic(
            CapabilityActivationDiagnosticCode::UnknownEntity,
            entity,
            requested_capability,
            "unknown entity",
        ),
        CoreActivationError::Tombstoned { entity } => diagnostic(
            CapabilityActivationDiagnosticCode::Tombstoned,
            entity,
            requested_capability,
            "entity is tombstoned",
        ),
        CoreActivationError::CapabilityAbsent { entity, capability } => diagnostic(
            CapabilityActivationDiagnosticCode::CapabilityAbsent,
            entity,
            from_core_kind(capability),
            "capability is absent",
        ),
        CoreActivationError::AlreadyInState {
            entity,
            capability,
            state,
        } => diagnostic(
            CapabilityActivationDiagnosticCode::AlreadyInState,
            entity,
            from_core_kind(capability),
            format!("capability is already {}", state.label()),
        ),
    }
}

fn from_core_kind(capability: CoreCapabilityKind) -> ActivatableCapabilityKind {
    match capability {
        CoreCapabilityKind::Collision => ActivatableCapabilityKind::Collision,
        CoreCapabilityKind::Controller => ActivatableCapabilityKind::Controller,
    }
}

fn diagnostic(
    code: CapabilityActivationDiagnosticCode,
    entity: core_ids::EntityId,
    capability: ActivatableCapabilityKind,
    message: impl Into<String>,
) -> CapabilityActivationDiagnostic {
    CapabilityActivationDiagnostic {
        code,
        entity,
        capability,
        message: message.into(),
    }
}

fn capability_label(capability: ActivatableCapabilityKind) -> &'static str {
    match capability {
        ActivatableCapabilityKind::Collision => "collision",
        ActivatableCapabilityKind::Controller => "controller",
    }
}

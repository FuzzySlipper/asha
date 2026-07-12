use core_entity::{ControllerCapability, EntityLifecycleCommand, EntitySource, EntityStore};
use core_ids::{EntityId, ProcessId};
use protocol_entity_authoring::{
    ActivatableCapabilityKind, CapabilityActivationAction, CapabilityActivationOutcome,
    CapabilityActivationPresence, CapabilityActivationRequest,
};
use svc_entity_authoring::{
    apply_rule_owned_capability_activation, project_capability_activation, EcrpRuleOwner,
};

fn create(store: &mut EntityStore, entity: EntityId) {
    store
        .apply(EntityLifecycleCommand::Create {
            id: entity,
            source: EntitySource::RuntimeCreated { by: None },
            labels: Vec::new(),
        })
        .expect("create fixture");
}

fn request(entity: EntityId, capability: ActivatableCapabilityKind) -> CapabilityActivationRequest {
    CapabilityActivationRequest {
        entity,
        capability,
        action: CapabilityActivationAction::Deactivate,
    }
}

#[test]
fn collision_and_controller_owners_apply_only_their_typed_activation() {
    let entity = EntityId::new(1);
    let mut store = EntityStore::new();
    create(&mut store, entity);
    store.attach_collision(entity, false);
    store.attach_controller(entity, ControllerCapability::Process(ProcessId::new(3)));

    for (owner, capability) in [
        (
            EcrpRuleOwner::CollisionRule,
            ActivatableCapabilityKind::Collision,
        ),
        (
            EcrpRuleOwner::ControllerRule,
            ActivatableCapabilityKind::Controller,
        ),
    ] {
        let outcome =
            apply_rule_owned_capability_activation(&mut store, owner, request(entity, capability));
        assert!(matches!(
            outcome,
            CapabilityActivationOutcome::Accepted { .. }
        ));
        assert_eq!(
            project_capability_activation(&store, entity, capability)
                .expect("projection")
                .presence,
            CapabilityActivationPresence::Inactive
        );
    }
}

#[test]
fn wrong_owner_is_forbidden_before_state_mutation() {
    let entity = EntityId::new(1);
    let mut store = EntityStore::new();
    create(&mut store, entity);
    store.attach_collision(entity, false);
    let before = store.hash();

    let outcome = apply_rule_owned_capability_activation(
        &mut store,
        EcrpRuleOwner::ControllerRule,
        request(entity, ActivatableCapabilityKind::Collision),
    );
    assert!(matches!(
        outcome,
        CapabilityActivationOutcome::Forbidden { .. }
    ));
    assert_eq!(store.hash(), before);
    assert_eq!(
        project_capability_activation(&store, entity, ActivatableCapabilityKind::Collision)
            .expect("projection")
            .presence,
        CapabilityActivationPresence::Active
    );
}

use core_entity::{
    decode_snapshot, encode_snapshot, ActivatableCapabilityKind, CapabilityActivationAction,
    CapabilityActivationCommand, CapabilityActivationError, CapabilityActivationPresence,
    CapabilityActivationState, ControllerCapability, EntityLifecycleCommand, EntitySource,
    EntityStore, EntityTransform, MovementCommand, MovementError,
};
use core_ids::{EntityId, ProcessId};
use core_math::Vec3;

fn entity(id: u64) -> EntityId {
    EntityId::new(id)
}

fn create(store: &mut EntityStore, id: EntityId) {
    store
        .apply(EntityLifecycleCommand::Create {
            id,
            source: EntitySource::RuntimeCreated { by: None },
            labels: Vec::new(),
        })
        .expect("create fixture entity");
}

fn set_activation(
    store: &mut EntityStore,
    entity: EntityId,
    capability: ActivatableCapabilityKind,
    action: CapabilityActivationAction,
) {
    store
        .apply_capability_activation(CapabilityActivationCommand {
            entity,
            capability,
            action,
        })
        .expect("activation transition");
}

#[test]
fn absent_inactive_and_active_are_distinct_for_two_capability_families() {
    let id = entity(1);
    let mut store = EntityStore::new();
    create(&mut store, id);

    assert_eq!(
        store
            .capability_activation(id, ActivatableCapabilityKind::Collision)
            .expect("known entity")
            .presence,
        CapabilityActivationPresence::Absent
    );
    assert!(store.attach_collision(id, false));
    assert!(store.attach_controller(id, ControllerCapability::Process(ProcessId::new(9))));
    for capability in [
        ActivatableCapabilityKind::Collision,
        ActivatableCapabilityKind::Controller,
    ] {
        assert_eq!(
            store
                .capability_activation(id, capability)
                .expect("readout")
                .presence,
            CapabilityActivationPresence::Active
        );
        set_activation(
            &mut store,
            id,
            capability,
            CapabilityActivationAction::Deactivate,
        );
        assert_eq!(
            store
                .capability_activation(id, capability)
                .expect("readout")
                .presence,
            CapabilityActivationPresence::Inactive
        );
    }
    assert!(store.active_collision(id).is_none());
    assert!(store.active_controller(id).is_none());
    assert!(store.collision(id).is_some(), "inactive is still attached");
    assert!(store.controller(id).is_some(), "inactive is still attached");
}

#[test]
fn entity_disable_suppresses_effective_use_without_rewriting_activation() {
    let id = entity(1);
    let mut store = EntityStore::new();
    create(&mut store, id);
    store.attach_collision(id, false);

    store
        .apply(EntityLifecycleCommand::Disable { id })
        .expect("disable entity");
    let disabled = store
        .capability_activation(id, ActivatableCapabilityKind::Collision)
        .expect("readout");
    assert_eq!(disabled.presence, CapabilityActivationPresence::Active);
    assert_eq!(
        disabled.entity_lifecycle,
        core_entity::EntityLifecycle::Disabled
    );
    assert!(!disabled.effective_active);

    store
        .apply(EntityLifecycleCommand::Enable { id })
        .expect("enable entity");
    assert!(
        store
            .capability_activation(id, ActivatableCapabilityKind::Collision)
            .expect("readout")
            .effective_active
    );
}

#[test]
fn inactive_collision_is_ignored_by_owner_queries_and_movement() {
    let mover = entity(1);
    let obstacle = entity(2);
    let mut store = EntityStore::new();
    for id in [mover, obstacle] {
        create(&mut store, id);
        store.attach_transform(id, EntityTransform::at(Vec3::ZERO));
        store.attach_collision(id, id == obstacle);
    }
    set_activation(
        &mut store,
        obstacle,
        ActivatableCapabilityKind::Collision,
        CapabilityActivationAction::Deactivate,
    );
    store
        .apply_movement(MovementCommand {
            id: mover,
            delta: Vec3::new(1.0, 0.0, 0.0),
        })
        .expect("inactive obstacle does not participate");

    set_activation(
        &mut store,
        mover,
        ActivatableCapabilityKind::Collision,
        CapabilityActivationAction::Deactivate,
    );
    assert_eq!(
        store.apply_movement(MovementCommand {
            id: mover,
            delta: Vec3::new(1.0, 0.0, 0.0),
        }),
        Err(MovementError::NoCollider { id: mover })
    );
}

#[test]
fn activation_hash_snapshot_and_replay_are_stable() {
    fn run() -> EntityStore {
        let id = entity(4);
        let mut store = EntityStore::new();
        create(&mut store, id);
        store.attach_collision(id, false);
        store.attach_controller(id, ControllerCapability::Process(ProcessId::new(2)));
        set_activation(
            &mut store,
            id,
            ActivatableCapabilityKind::Collision,
            CapabilityActivationAction::Deactivate,
        );
        set_activation(
            &mut store,
            id,
            ActivatableCapabilityKind::Controller,
            CapabilityActivationAction::Deactivate,
        );
        store
    }

    let original = run();
    let replayed = run();
    assert_eq!(original.hash(), replayed.hash());
    let encoded = encode_snapshot(&original.snapshot());
    let restored = EntityStore::from_snapshot(decode_snapshot(&encoded).expect("decode snapshot"));
    assert_eq!(restored.hash(), original.hash());
    assert_eq!(encode_snapshot(&restored.snapshot()), encoded);
}

#[test]
fn schema_v1_snapshot_migrates_attached_capabilities_to_active() {
    let id = entity(4);
    let mut store = EntityStore::new();
    create(&mut store, id);
    store.attach_collision(id, false);
    let current = encode_snapshot(&store.snapshot());
    let legacy = current
        .replace("\"schemaVersion\": 2", "\"schemaVersion\": 1")
        .replace(", \"collisionActivation\": \"active\"", "")
        .replace(", \"controllerActivation\": null", "");
    let restored = EntityStore::from_snapshot(decode_snapshot(&legacy).expect("decode v1"));

    assert_eq!(
        restored
            .capability_activation(id, ActivatableCapabilityKind::Collision)
            .expect("readout")
            .presence,
        CapabilityActivationPresence::Active
    );
}

#[test]
fn invalid_transitions_fail_without_mutation_and_destroy_clears_state() {
    let id = entity(1);
    let mut store = EntityStore::new();
    create(&mut store, id);
    let before = store.hash();
    assert_eq!(
        store.apply_capability_activation(CapabilityActivationCommand {
            entity: id,
            capability: ActivatableCapabilityKind::Collision,
            action: CapabilityActivationAction::Deactivate,
        }),
        Err(CapabilityActivationError::CapabilityAbsent {
            entity: id,
            capability: ActivatableCapabilityKind::Collision,
        })
    );
    assert_eq!(store.hash(), before);

    store.attach_collision(id, false);
    set_activation(
        &mut store,
        id,
        ActivatableCapabilityKind::Collision,
        CapabilityActivationAction::Deactivate,
    );
    assert_eq!(
        store.apply_capability_activation(CapabilityActivationCommand {
            entity: id,
            capability: ActivatableCapabilityKind::Collision,
            action: CapabilityActivationAction::Deactivate,
        }),
        Err(CapabilityActivationError::AlreadyInState {
            entity: id,
            capability: ActivatableCapabilityKind::Collision,
            state: CapabilityActivationState::Inactive,
        })
    );
    store
        .apply(EntityLifecycleCommand::Destroy { id })
        .expect("destroy");
    assert_eq!(
        store
            .capability_activation(id, ActivatableCapabilityKind::Collision)
            .expect("tombstone readout")
            .presence,
        CapabilityActivationPresence::Absent
    );
}

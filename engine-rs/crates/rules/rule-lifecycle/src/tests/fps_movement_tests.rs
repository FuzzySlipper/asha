use super::*;

#[test]
fn autonomous_enemy_movement_is_role_gated_and_replay_recorded() {
    let mut session = load_custom_session();
    let enemy = EntityId::new(777);
    let before_entity_hash = session.entities.hash().0;
    let before_health_hash = session.combat.health_hash();

    let receipt = session
        .apply_autonomous_enemy_direct_nav_movement(enemy, [3.0, 1.62, 1.5], 8.0)
        .expect("registered enemy policy may propose direct-nav movement");

    assert_eq!(receipt.entity, enemy);
    assert_eq!(
        receipt.navigation.next_waypoint.to_array(),
        [3.0, 1.62, 1.5]
    );
    assert_ne!(receipt.entity_hash, before_entity_hash);
    assert_eq!(receipt.health_hash, before_health_hash);
    assert_eq!(session.replay_records.len(), 2);
    assert_eq!(
        session.replay_records[1].kind,
        "runtime_session.fps.autonomous_movement.v0"
    );
    assert_eq!(session.replay_records[1].entity_hash, receipt.entity_hash);
    assert_eq!(session.replay_records[1].record_hash, receipt.replay_hash);
}

#[test]
fn autonomous_movement_rejects_non_enemy_entities_without_mutation() {
    let mut input = custom_load_input();
    input.definitions.push(FpsStoredEntityDefinition {
        entity: EntityId::new(303),
        definition: definition(
            "actor/custom-neutral",
            "Custom Neutral",
            ([4.0, 1.0, 4.0], [4.5, 2.0, 4.5]),
        ),
        role: FpsRuntimeRole::Neutral,
        health: None,
        weapon: None,
        render_projection: None,
        policy_binding: None,
    });
    let mut session = load_fps_project_bundle(input).expect("load session with neutral entity");
    let before_entities = session.entities.clone();
    let before_replay = session.replay_records.clone();

    for entity in [EntityId::new(101), EntityId::new(303), EntityId::new(999)] {
        assert_eq!(
            session
                .apply_autonomous_enemy_direct_nav_movement(entity, [3.0, 1.62, 1.5], 8.0)
                .expect_err("only the registered autonomous enemy may move"),
            FpsRuntimeError::UnauthorizedAutonomousMovement { entity }
        );
        assert_eq!(session.entities, before_entities);
        assert_eq!(session.replay_records, before_replay);
    }

    let mut without_intent = custom_load_input();
    without_intent.definitions[1]
        .policy_binding
        .as_mut()
        .expect("enemy policy binding")
        .allowed_intents = vec!["runtime.intent.primary_fire.v0".into()];
    let mut session = load_fps_project_bundle(without_intent).expect("load policy binding");
    let before_entities = session.entities.clone();
    let before_replay = session.replay_records.clone();
    assert_eq!(
        session
            .apply_autonomous_enemy_direct_nav_movement(EntityId::new(777), [3.0, 1.62, 1.5], 8.0,)
            .expect_err("enemy policy must explicitly allow direct-nav movement"),
        FpsRuntimeError::UnauthorizedAutonomousMovement {
            entity: EntityId::new(777)
        }
    );
    assert_eq!(session.entities, before_entities);
    assert_eq!(session.replay_records, before_replay);
}

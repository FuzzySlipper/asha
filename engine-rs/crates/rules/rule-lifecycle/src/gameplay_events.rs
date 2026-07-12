use core_ids::EntityId;
use protocol_game_extension::GameplayEventEnvelope;
use rule_gameplay_fabric::{
    adapt_combat_readout, GameplayOwnerEventContext, GameplayOwnerEventError,
};
use svc_combat::{CombatReadout, HealthState};

#[derive(Debug, Clone, PartialEq)]
pub struct FpsPrimaryFireReceipt {
    pub shooter: EntityId,
    pub target: Option<EntityId>,
    pub target_health_before: Option<HealthState>,
    pub target_health_after: Option<HealthState>,
    pub combat: CombatReadout,
    pub gameplay_events: Vec<GameplayEventEnvelope>,
    pub lifecycle_status: crate::FpsLifecycleStatus,
    pub target_render_visible: Option<bool>,
    pub entity_hash: u64,
    pub health_hash: u64,
    pub replay_hash: u64,
}

pub(crate) fn adapt_primary_fire(
    tick: u64,
    root_sequence: u64,
    combat: &CombatReadout,
) -> Result<Vec<GameplayEventEnvelope>, GameplayOwnerEventError> {
    adapt_combat_readout(
        &GameplayOwnerEventContext {
            owner_id: "svc-combat".to_owned(),
            tick,
            root_id: format!("combat.primary-fire.{tick}.{}", combat.replay_hash),
            root_sequence,
            first_event_sequence: 0,
            parent_event_id: None,
        },
        combat,
    )
}

#[cfg(test)]
pub(crate) fn assert_primary_fire_events(receipt: &FpsPrimaryFireReceipt) {
    assert_eq!(receipt.gameplay_events.len(), 3);
    assert_eq!(
        receipt.gameplay_events[0].event,
        rule_gameplay_fabric::StandardGameplayEventKind::CombatFireHit.contract()
    );
    assert_eq!(
        receipt.gameplay_events[2].event,
        rule_gameplay_fabric::StandardGameplayEventKind::CombatEntityDefeated.contract()
    );
    assert!(receipt.gameplay_events.iter().all(|event| event.emitter
        == (protocol_game_extension::GameplayEmitterRef::Owner {
            owner_id: "svc-combat".to_owned(),
        })));
}

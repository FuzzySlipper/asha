use std::collections::{BTreeMap, BTreeSet};

use core_game_rules::{
    BoundedValue, EffectDuration, EffectOp, EffectOpId, EffectOpKind, ModifierDefinition,
    ModifierId, ReactionBehavior, ReactionDefinition, ReactionWindowId, ReactionWindowKind,
    StackPolicy, TickCadence, ValueChannelId, ValueDelta,
};
use core_ids::EntityId;
use core_time::Tick;
use protocol_game_rules::{
    GameRuleBoundedValue, GameRuleCatalog, GameRuleCatalogRef, GameRuleDuration,
    GameRuleEffectBundle, GameRuleEffectOp, GameRuleModifierDefinition, GameRuleResolutionRequest,
    GameRuleStackPolicy, GameRuleTickCadence, GameRuleValueChannelRef,
};
use rule_game_modifier::{GameModifierRuleState, ModifierCatalog, ValueFactKey};
use svc_game_rules::reaction::{resolve_reactions, ReactionResolutionInput};
use svc_game_rules::{resolve_protocol_request, validate_catalog};

const POISONED_IMPACT: &str =
    include_str!("../../../../../harness/fixtures/game-rules/poisoned-impact.snapshot.txt");
const RPG_ACTION: &str =
    include_str!("../../../../../harness/fixtures/game-rules/rpg-action.snapshot.txt");

#[test]
fn poisoned_impact_fixture_is_end_to_end_and_stable() {
    assert_eq!(poisoned_impact_snapshot(), POISONED_IMPACT);
}

#[test]
fn rpg_action_fixture_is_declarative_and_stable() {
    assert_eq!(rpg_action_snapshot(), RPG_ACTION);
}

fn poisoned_impact_snapshot() -> String {
    let catalog = poisoned_protocol_catalog();
    let validation = validate_catalog(&catalog);
    let request = GameRuleResolutionRequest {
        catalog: catalog.catalog.clone(),
        bundle_id: "bundle.poisoned-impact".to_string(),
        source: EntityId::new(101),
        target: EntityId::new(777),
        values: vec![GameRuleBoundedValue {
            channel_id: "value.health".to_string(),
            min: 0,
            current: 30,
            max: 30,
        }],
        tick: 4,
    };
    let receipt = resolve_protocol_request(&request, &catalog);

    let poison = mid("modifier.poison");
    let health = channel("value.health");
    let mut rule = GameModifierRuleState::new(ModifierCatalog::new(
        [ModifierDefinition::new(
            poison.clone(),
            StackPolicy::Refresh,
            EffectDuration::ticks(6).unwrap(),
        )
        .with_tick_cadence(TickCadence::every(2).unwrap())
        .with_effect_ops([oid("op.poison-tick")])
        .with_tags([core_game_rules::EffectTagId::parse("tag.poison").unwrap()])
        .with_source_hash("fnv1a64:poison")],
        [EffectOp::new(
            oid("op.poison-tick"),
            EffectOpKind::ApplyDelta {
                value: health.clone(),
                delta: ValueDelta::new(-2),
            },
        )],
    ));
    let applied = rule.apply_modifier(
        &poison,
        EntityId::new(101),
        EntityId::new(777),
        Tick::new(4),
    );
    let mut values = BTreeMap::from([(
        ValueFactKey {
            entity: EntityId::new(777),
            channel: health.clone(),
        },
        BoundedValue::new(0, 23, 30).unwrap(),
    )]);
    let tick6 = rule.tick(Tick::new(6), &mut values);
    let tick8 = rule.tick(Tick::new(8), &mut values);
    let tick10 = rule.tick(Tick::new(10), &mut values);

    let mut out = String::new();
    out.push_str("# game-rules fixture: poisoned-impact\n");
    out.push_str(&format!("catalogAccepted={}\n", validation.accepted()));
    out.push_str(&format!("resolutionAccepted={}\n", receipt.accepted));
    out.push_str(&format!(
        "pendingDeltas={:?}\n",
        receipt.pending_value_deltas
    ));
    out.push_str(&format!(
        "appliedModifiers={:?}\n",
        receipt.applied_modifiers
    ));
    out.push_str(&format!(
        "resolutionTrace={}\n",
        protocol_trace_codes(&receipt.trace)
    ));
    out.push_str(&format!("resolutionReplay={}\n", receipt.replay_hash));
    out.push_str(&format!("modifierApplyAccepted={}\n", applied.accepted));
    out.push_str(&format!("modifierApplyReplay={}\n", applied.replay_hash));
    out.push_str(&format!("tick6Events={:?}\n", tick6.events));
    out.push_str(&format!("tick6Replay={}\n", tick6.replay_hash));
    out.push_str(&format!("tick8Events={:?}\n", tick8.events));
    out.push_str(&format!("tick8Replay={}\n", tick8.replay_hash));
    out.push_str(&format!("tick10Events={:?}\n", tick10.events));
    out.push_str(&format!("tick10Replay={}\n", tick10.replay_hash));
    out.push_str(&format!(
        "finalHealth={}\n",
        values
            .get(&ValueFactKey {
                entity: EntityId::new(777),
                channel: health,
            })
            .unwrap()
            .current
    ));
    out.push_str(&format!("activeModifierCount={}\n", rule.active().len()));
    out.push_str(&format!("activeHash={}\n", rule.active_hash()));
    out
}

fn rpg_action_snapshot() -> String {
    let catalog = rpg_protocol_catalog();
    let validation = validate_catalog(&catalog);
    let request = GameRuleResolutionRequest {
        catalog: catalog.catalog.clone(),
        bundle_id: "bundle.rpg-action".to_string(),
        source: EntityId::new(501),
        target: EntityId::new(777),
        values: vec![
            GameRuleBoundedValue {
                channel_id: "value.health".to_string(),
                min: 0,
                current: 42,
                max: 42,
            },
            GameRuleBoundedValue {
                channel_id: "value.focus".to_string(),
                min: 0,
                current: 10,
                max: 10,
            },
        ],
        tick: 12,
    };
    let receipt = resolve_protocol_request(&request, &catalog);
    let reaction = resolve_reactions(
        &[ReactionDefinition::new(
            ReactionWindowId::parse("reaction.guard").unwrap(),
            ReactionWindowKind::PendingValueDelta,
            ReactionBehavior::ModifyPendingDelta {
                channel: channel("value.health"),
                amount: 3,
            },
        )
        .with_priority(10)
        .with_reads([channel("value.health")])],
        &ReactionResolutionInput {
            window: ReactionWindowKind::PendingValueDelta,
            channel: Some(channel("value.health")),
            pending_delta: -8,
            declared_reads: BTreeSet::from([channel("value.health")]),
            allowed_effect_ops: BTreeSet::from([oid("op.rpg-damage")]),
            allowed_modifiers: BTreeSet::from([mid("modifier.guard-broken")]),
        },
    );

    let mut out = String::new();
    out.push_str("# game-rules fixture: rpg-action\n");
    out.push_str(&format!("catalogAccepted={}\n", validation.accepted()));
    out.push_str(&format!("resolutionAccepted={}\n", receipt.accepted));
    out.push_str(&format!(
        "pendingDeltas={:?}\n",
        receipt.pending_value_deltas
    ));
    out.push_str(&format!(
        "appliedModifiers={:?}\n",
        receipt.applied_modifiers
    ));
    out.push_str(&format!(
        "resolutionTrace={}\n",
        protocol_trace_codes(&receipt.trace)
    ));
    out.push_str(&format!("resolutionReplay={}\n", receipt.replay_hash));
    out.push_str(&format!("reactionAccepted={}\n", reaction.accepted));
    out.push_str(&format!(
        "reactionPendingDelta={}\n",
        reaction.pending_delta
    ));
    out.push_str(&format!(
        "reactionTrace={}\n",
        core_trace_codes(&reaction.trace)
    ));
    out.push_str(&format!("reactionHash={}\n", reaction.reaction_hash));
    out
}

fn poisoned_protocol_catalog() -> GameRuleCatalog {
    GameRuleCatalog {
        catalog: GameRuleCatalogRef {
            catalog_id: "catalog.game-rules.poisoned-impact".to_string(),
            version: "0.1.0".to_string(),
            content_hash: "fnv1a64:poisoned-impact".to_string(),
        },
        value_channels: vec![GameRuleValueChannelRef {
            channel_id: "value.health".to_string(),
            display_name: Some("Health".to_string()),
        }],
        bundles: vec![GameRuleEffectBundle {
            bundle_id: "bundle.poisoned-impact".to_string(),
            effect_ops: vec![
                GameRuleEffectOp::ApplyDelta {
                    op_id: "op.impact-damage".to_string(),
                    channel_id: "value.health".to_string(),
                    amount: -7,
                    tags: vec!["tag.impact".to_string()],
                },
                GameRuleEffectOp::SchedulePeriodicEffect {
                    op_id: "op.schedule-poison".to_string(),
                    modifier_id: "modifier.poison".to_string(),
                    cadence: GameRuleTickCadence { period_ticks: 2 },
                    duration: GameRuleDuration::Ticks { ticks: 6 },
                    tags: vec!["tag.poison".to_string()],
                },
                GameRuleEffectOp::ApplyDelta {
                    op_id: "op.poison-tick".to_string(),
                    channel_id: "value.health".to_string(),
                    amount: -2,
                    tags: vec!["tag.poison".to_string()],
                },
            ],
            modifiers: vec![GameRuleModifierDefinition {
                modifier_id: "modifier.poison".to_string(),
                stack_policy: GameRuleStackPolicy::Refresh,
                duration: GameRuleDuration::Ticks { ticks: 6 },
                tick_cadence: Some(GameRuleTickCadence { period_ticks: 2 }),
                tags: vec!["tag.poison".to_string()],
                effect_op_ids: vec!["op.poison-tick".to_string()],
                source_hash: "fnv1a64:poison".to_string(),
            }],
            tags: vec!["tag.poison".to_string(), "tag.impact".to_string()],
            source_hash: "fnv1a64:poisoned-impact-bundle".to_string(),
        }],
    }
}

fn rpg_protocol_catalog() -> GameRuleCatalog {
    GameRuleCatalog {
        catalog: GameRuleCatalogRef {
            catalog_id: "catalog.game-rules.rpg-action".to_string(),
            version: "0.1.0".to_string(),
            content_hash: "fnv1a64:rpg-action".to_string(),
        },
        value_channels: vec![
            GameRuleValueChannelRef {
                channel_id: "value.health".to_string(),
                display_name: Some("Health".to_string()),
            },
            GameRuleValueChannelRef {
                channel_id: "value.focus".to_string(),
                display_name: Some("Focus".to_string()),
            },
        ],
        bundles: vec![GameRuleEffectBundle {
            bundle_id: "bundle.rpg-action".to_string(),
            effect_ops: vec![
                GameRuleEffectOp::Spend {
                    op_id: "op.spend-focus".to_string(),
                    channel_id: "value.focus".to_string(),
                    amount: 3,
                    tags: vec!["tag.resource".to_string()],
                },
                GameRuleEffectOp::ApplyDelta {
                    op_id: "op.rpg-damage".to_string(),
                    channel_id: "value.health".to_string(),
                    amount: -8,
                    tags: vec!["tag.damage".to_string()],
                },
                GameRuleEffectOp::ApplyModifier {
                    op_id: "op.apply-guard-break".to_string(),
                    modifier_id: "modifier.guard-broken".to_string(),
                    tags: vec!["tag.condition".to_string()],
                },
            ],
            modifiers: vec![GameRuleModifierDefinition {
                modifier_id: "modifier.guard-broken".to_string(),
                stack_policy: GameRuleStackPolicy::RejectDuplicate,
                duration: GameRuleDuration::Ticks { ticks: 2 },
                tick_cadence: None,
                tags: vec!["tag.condition".to_string()],
                effect_op_ids: Vec::new(),
                source_hash: "fnv1a64:guard-broken".to_string(),
            }],
            tags: vec!["tag.rpg-action".to_string()],
            source_hash: "fnv1a64:rpg-action-bundle".to_string(),
        }],
    }
}

fn mid(id: &str) -> ModifierId {
    ModifierId::parse(id).unwrap()
}

fn oid(id: &str) -> EffectOpId {
    EffectOpId::parse(id).unwrap()
}

fn channel(id: &str) -> ValueChannelId {
    ValueChannelId::parse(id).unwrap()
}

fn protocol_trace_codes(trace: &[protocol_game_rules::GameRuleTraceEntry]) -> String {
    trace
        .iter()
        .map(|entry| entry.code.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn core_trace_codes(trace: &[core_game_rules::GameRuleTraceEntry]) -> String {
    trace
        .iter()
        .map(|entry| entry.code.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

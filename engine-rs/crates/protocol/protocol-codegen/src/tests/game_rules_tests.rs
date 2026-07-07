use super::*;

/// Focused behavior test for the `gameRules` family: stable operation, stack,
/// diagnostic, and evidence vocabularies are sourced from `protocol-game-rules`,
/// while the catalog/receipt DTOs remain generated and publicly re-exported.
/// Guard for #4527.
#[test]
fn game_rules_family_emits_vocab_and_shapes() {
    let gr = file("gameRules.ts");
    for kind in protocol_game_rules::GAME_RULE_EFFECT_OP_KINDS {
        assert!(gr.contains(&format!("'{kind}'")), "missing op kind {kind}");
    }
    for policy in protocol_game_rules::GAME_RULE_STACK_POLICIES {
        assert!(
            gr.contains(&format!("'{policy}'")),
            "missing stack policy {policy}"
        );
    }
    for code in protocol_game_rules::GAME_RULE_DIAGNOSTIC_CODES {
        assert!(gr.contains(&format!("'{code}'")), "missing code {code}");
    }
    for kind in protocol_game_rules::GAME_RULE_EVIDENCE_KINDS {
        assert!(gr.contains(&format!("'{kind}'")), "missing evidence {kind}");
    }

    assert!(gr.contains("import type { EntityId } from './ids.js';"));
    assert!(gr.contains("import type { DiagnosticSeverity } from './diagnostics.js';"));
    assert!(gr.contains("export interface GameRuleCatalog {"));
    assert!(gr.contains("export type GameRuleEffectOp ="));
    assert!(gr.contains("readonly kind: 'schedulePeriodicEffect'"));
    assert!(gr.contains("export interface GameRuleResolutionReceipt {"));
    assert!(gr.contains("readonly tickCadence: GameRuleTickCadence | null;"));
}

#[test]
fn game_rules_rust_serialization_matches_ir_shape() {
    use core_ids::EntityId;
    use protocol_diagnostics::DiagnosticSeverity;
    use protocol_game_rules::{
        GameRuleBoundedValue, GameRuleCatalog, GameRuleCatalogRef, GameRuleDiagnostic,
        GameRuleDiagnosticCode, GameRuleDuration, GameRuleEffectBundle, GameRuleEffectOp,
        GameRuleEvidenceKind, GameRuleEvidenceRef, GameRuleModifierDefinition,
        GameRuleModifierState, GameRuleResolutionReceipt, GameRuleResolutionRequest,
        GameRuleStackPolicy, GameRuleTickCadence, GameRuleTraceEntry, GameRuleTraceRef,
        GameRuleValueChannelRef, GameRuleValueDelta,
    };

    let game_rules = module("gameRules");
    let catalog_ref = GameRuleCatalogRef {
        catalog_id: "catalog.game-rules.demo".to_string(),
        version: "0.1.0".to_string(),
        content_hash: "fnv1a64:catalog".to_string(),
    };
    let channel = GameRuleValueChannelRef {
        channel_id: "value.health".to_string(),
        display_name: Some("Health".to_string()),
    };
    let value = GameRuleBoundedValue {
        channel_id: "value.health".to_string(),
        min: 0,
        current: 8,
        max: 20,
    };
    let delta = GameRuleValueDelta {
        channel_id: "value.health".to_string(),
        amount: -4,
    };
    let cadence = GameRuleTickCadence { period_ticks: 3 };
    let modifier = GameRuleModifierDefinition {
        modifier_id: "modifier.poison".to_string(),
        stack_policy: GameRuleStackPolicy::Stack { max_stacks: 3 },
        duration: GameRuleDuration::Ticks { ticks: 9 },
        tick_cadence: Some(cadence.clone()),
        tags: vec!["tag.poison".to_string()],
        effect_op_ids: vec!["op.poison-tick".to_string()],
        source_hash: "fnv1a64:modifier".to_string(),
    };
    let bundle = GameRuleEffectBundle {
        bundle_id: "bundle.poisoned-impact".to_string(),
        effect_ops: vec![GameRuleEffectOp::SchedulePeriodicEffect {
            op_id: "op.schedule-poison".to_string(),
            modifier_id: "modifier.poison".to_string(),
            cadence: cadence.clone(),
            duration: GameRuleDuration::Ticks { ticks: 9 },
            tags: vec!["tag.poison".to_string()],
        }],
        modifiers: vec![modifier.clone()],
        tags: vec!["tag.poison".to_string()],
        source_hash: "fnv1a64:bundle".to_string(),
    };
    let catalog = GameRuleCatalog {
        catalog: catalog_ref.clone(),
        value_channels: vec![channel.clone()],
        bundles: vec![bundle.clone()],
    };
    let diagnostic = GameRuleDiagnostic {
        code: GameRuleDiagnosticCode::InvalidCadence,
        severity: DiagnosticSeverity::Error,
        path: "bundles[0].effectOps[0].cadence.periodTicks".to_string(),
        message: "cadence period must be greater than zero".to_string(),
    };
    let evidence = GameRuleEvidenceRef {
        kind: GameRuleEvidenceKind::ResolutionReceipt,
        uri: "asha://game-rules/receipt/demo".to_string(),
        content_hash: "fnv1a64:receipt".to_string(),
    };
    let trace_ref = GameRuleTraceRef {
        key: "bundle".to_string(),
        value: "bundle.poisoned-impact".to_string(),
    };
    let trace_entry = GameRuleTraceEntry {
        step: 1,
        code: "effect.accepted".to_string(),
        message: "bundle resolved".to_string(),
        refs: vec![trace_ref.clone()],
    };
    let state = GameRuleModifierState {
        modifier_id: "modifier.poison".to_string(),
        source: EntityId::new(1),
        target: EntityId::new(2),
        stacks: 1,
        applied_tick: 12,
        expires_tick: Some(21),
        next_tick: Some(15),
        source_hash: "fnv1a64:state".to_string(),
    };
    let request = GameRuleResolutionRequest {
        catalog: catalog_ref.clone(),
        bundle_id: "bundle.poisoned-impact".to_string(),
        source: EntityId::new(1),
        target: EntityId::new(2),
        values: vec![value.clone()],
        tick: 12,
    };
    let receipt = GameRuleResolutionReceipt {
        accepted: true,
        request_hash: "fnv1a64:request".to_string(),
        pending_value_deltas: vec![delta.clone()],
        applied_modifiers: vec![state.clone()],
        diagnostics: vec![diagnostic.clone()],
        trace: vec![trace_entry.clone()],
        evidence: vec![evidence.clone()],
        replay_hash: "fnv1a64:replay".to_string(),
    };

    compare_object_to_interface(
        &game_rules,
        "GameRuleCatalogRef",
        &serde_json::to_value(&catalog_ref).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleValueChannelRef",
        &serde_json::to_value(&channel).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleBoundedValue",
        &serde_json::to_value(&value).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleValueDelta",
        &serde_json::to_value(&delta).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_rules,
        "GameRuleDuration",
        "instant",
        &serde_json::to_value(GameRuleDuration::Instant).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_rules,
        "GameRuleDuration",
        "ticks",
        &serde_json::to_value(GameRuleDuration::Ticks { ticks: 9 }).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_rules,
        "GameRuleDuration",
        "infinite",
        &serde_json::to_value(GameRuleDuration::Infinite).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleTickCadence",
        &serde_json::to_value(&cadence).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_rules,
        "GameRuleStackPolicy",
        "refresh",
        &serde_json::to_value(GameRuleStackPolicy::Refresh).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_rules,
        "GameRuleStackPolicy",
        "stack",
        &serde_json::to_value(GameRuleStackPolicy::Stack { max_stacks: 3 }).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_rules,
        "GameRuleStackPolicy",
        "rejectDuplicate",
        &serde_json::to_value(GameRuleStackPolicy::RejectDuplicate).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_rules,
        "GameRuleStackPolicy",
        "replaceIfStronger",
        &serde_json::to_value(GameRuleStackPolicy::ReplaceIfStronger).unwrap(),
    )
    .unwrap();

    let op_tags = vec!["tag.sample".to_string()];
    let op_samples = [
        (
            "applyDelta",
            GameRuleEffectOp::ApplyDelta {
                op_id: "op.delta".to_string(),
                channel_id: "value.health".to_string(),
                amount: -2,
                tags: op_tags.clone(),
            },
        ),
        (
            "restore",
            GameRuleEffectOp::Restore {
                op_id: "op.restore".to_string(),
                channel_id: "value.health".to_string(),
                amount: 2,
                tags: op_tags.clone(),
            },
        ),
        (
            "spend",
            GameRuleEffectOp::Spend {
                op_id: "op.spend".to_string(),
                channel_id: "value.stamina".to_string(),
                amount: 1,
                tags: op_tags.clone(),
            },
        ),
        (
            "grant",
            GameRuleEffectOp::Grant {
                op_id: "op.grant".to_string(),
                channel_id: "value.stamina".to_string(),
                amount: 1,
                tags: op_tags.clone(),
            },
        ),
        (
            "applyModifier",
            GameRuleEffectOp::ApplyModifier {
                op_id: "op.apply-modifier".to_string(),
                modifier_id: "modifier.poison".to_string(),
                tags: op_tags.clone(),
            },
        ),
        (
            "removeModifier",
            GameRuleEffectOp::RemoveModifier {
                op_id: "op.remove-modifier".to_string(),
                modifier_id: "modifier.poison".to_string(),
                tags: op_tags.clone(),
            },
        ),
        (
            "schedulePeriodicEffect",
            GameRuleEffectOp::SchedulePeriodicEffect {
                op_id: "op.schedule".to_string(),
                modifier_id: "modifier.poison".to_string(),
                cadence: cadence.clone(),
                duration: GameRuleDuration::Ticks { ticks: 9 },
                tags: op_tags.clone(),
            },
        ),
        (
            "cancelResolution",
            GameRuleEffectOp::CancelResolution {
                op_id: "op.cancel".to_string(),
                reason: "immune".to_string(),
                tags: op_tags.clone(),
            },
        ),
        (
            "emitTrace",
            GameRuleEffectOp::EmitTrace {
                op_id: "op.trace".to_string(),
                code: "effect.trace".to_string(),
                message: "trace emitted".to_string(),
                tags: op_tags,
            },
        ),
    ];
    for (tag, op) in op_samples {
        compare_object_to_variant(
            &game_rules,
            "GameRuleEffectOp",
            tag,
            &serde_json::to_value(op).unwrap(),
        )
        .unwrap();
    }

    compare_object_to_interface(
        &game_rules,
        "GameRuleModifierDefinition",
        &serde_json::to_value(&modifier).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleEffectBundle",
        &serde_json::to_value(&bundle).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleCatalog",
        &serde_json::to_value(&catalog).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleDiagnostic",
        &serde_json::to_value(&diagnostic).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleEvidenceRef",
        &serde_json::to_value(&evidence).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleTraceRef",
        &serde_json::to_value(&trace_ref).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleTraceEntry",
        &serde_json::to_value(&trace_entry).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleModifierState",
        &serde_json::to_value(&state).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleResolutionRequest",
        &serde_json::to_value(&request).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_rules,
        "GameRuleResolutionReceipt",
        &serde_json::to_value(&receipt).unwrap(),
    )
    .unwrap();
}

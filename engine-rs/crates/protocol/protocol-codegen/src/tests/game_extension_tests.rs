use super::*;

/// Focused behavior test for the `gameExtension` family: stable hook/proposal/
/// receipt/diagnostic vocabularies are sourced from `protocol-game-extension`,
/// while manifests, hook requests, proposals, receipts, and replay evidence are
/// generated and publicly re-exported. Guard for #4516.
#[test]
fn game_extension_family_emits_vocab_and_shapes() {
    let ext = file("gameExtension.ts");
    for kind in protocol_game_extension::GAME_EXTENSION_HOOK_KINDS {
        assert!(
            ext.contains(&format!("'{kind}'")),
            "missing hook kind {kind}"
        );
    }
    for kind in protocol_game_extension::GAME_EXTENSION_PROPOSAL_KINDS {
        assert!(
            ext.contains(&format!("'{kind}'")),
            "missing proposal kind {kind}"
        );
    }
    for status in protocol_game_extension::GAME_EXTENSION_RECEIPT_STATUSES {
        assert!(
            ext.contains(&format!("'{status}'")),
            "missing receipt status {status}"
        );
    }
    for code in protocol_game_extension::GAME_EXTENSION_DIAGNOSTIC_CODES {
        assert!(
            ext.contains(&format!("'{code}'")),
            "missing diagnostic {code}"
        );
    }

    assert!(ext.contains("import type { EntityId } from './ids.js';"));
    assert!(ext.contains("import type { DiagnosticSeverity } from './diagnostics.js';"));
    assert!(ext.contains("export interface GameRuleModuleManifest {"));
    assert!(ext.contains("export interface WeaponEffectHookRequest {"));
    assert!(ext.contains("export type GameExtensionProposal ="));
    assert!(ext.contains("readonly kind: 'damageModifier'"));
    assert!(ext.contains("export interface GameExtensionReplayEvidence {"));
}

#[test]
fn game_extension_rust_serialization_matches_ir_shape() {
    use core_ids::EntityId;
    use protocol_diagnostics::DiagnosticSeverity;
    use protocol_game_extension::{
        GameExtensionDiagnostic, GameExtensionDiagnosticCode, GameExtensionHookKind,
        GameExtensionHookReceipt, GameExtensionProposal, GameExtensionReceiptStatus,
        GameExtensionReplayEvidence, GameExtensionTraceEntry, GameRuleHookDeclaration,
        GameRuleModuleManifest, GameRuleModuleRef, WeaponEffectHookRequest,
    };

    let game_extension = module("gameExtension");
    let module_ref = GameRuleModuleRef {
        module_id: "demo.primary_fire_effect".to_string(),
        version: "0.1.0".to_string(),
        contract_hash: "sha256:contract".to_string(),
    };
    let hook = GameRuleHookDeclaration {
        hook_id: "weapon.primary".to_string(),
        kind: GameExtensionHookKind::WeaponEffect,
        input_contract: "WeaponEffectHookRequest.v0".to_string(),
        output_contract: "GameExtensionProposal.v0".to_string(),
        required_capabilities: vec!["health".to_string(), "weaponMount".to_string()],
    };
    let manifest = GameRuleModuleManifest {
        module_ref: module_ref.clone(),
        declared_hooks: vec![hook.clone()],
        deterministic_requirements: vec![
            "no-wall-clock".to_string(),
            "no-ambient-random".to_string(),
            "no-ts-callback".to_string(),
        ],
        source_hash: "sha256:module-source".to_string(),
    };
    let diagnostic = GameExtensionDiagnostic {
        code: GameExtensionDiagnosticCode::InvalidProposal,
        severity: DiagnosticSeverity::Error,
        path: "proposal".to_string(),
        message: "proposal is invalid".to_string(),
    };
    let request = WeaponEffectHookRequest {
        module_ref: module_ref.clone(),
        hook_id: "weapon.primary".to_string(),
        request_id: "request-1".to_string(),
        tick: 42,
        source: EntityId::new(1),
        target: Some(EntityId::new(2)),
        base_damage: -8,
        range_millimeters: 400,
        tags: vec!["primary-fire".to_string()],
        input_hash: "fnv1a64:input".to_string(),
    };
    let damage = GameExtensionProposal::DamageModifier {
        proposal_id: "proposal.damage".to_string(),
        target: EntityId::new(2),
        channel_id: "value.health".to_string(),
        amount_delta: -2,
        tags: vec!["close-range".to_string()],
        proposal_hash: "fnv1a64:damage".to_string(),
    };
    let bundle = GameExtensionProposal::EffectBundle {
        proposal_id: "proposal.bundle".to_string(),
        bundle_id: "bundle.poisoned-impact".to_string(),
        tags: vec!["poison".to_string()],
        proposal_hash: "fnv1a64:bundle".to_string(),
    };
    let rejected = GameExtensionProposal::Reject {
        proposal_id: "proposal.reject".to_string(),
        code: GameExtensionDiagnosticCode::InvalidProposal,
        message: "module rejected".to_string(),
        proposal_hash: "fnv1a64:reject".to_string(),
    };
    let noop = GameExtensionProposal::Noop {
        proposal_id: "proposal.noop".to_string(),
        proposal_hash: "fnv1a64:noop".to_string(),
    };
    let trace = GameExtensionTraceEntry {
        step: 1,
        code: "module.proposed".to_string(),
        message: "module returned a typed proposal".to_string(),
        refs: vec!["proposal.damage".to_string()],
    };
    let receipt = GameExtensionHookReceipt {
        module_ref: module_ref.clone(),
        hook_id: "weapon.primary".to_string(),
        request_id: "request-1".to_string(),
        status: GameExtensionReceiptStatus::Proposed,
        input_hash: "fnv1a64:input".to_string(),
        proposal: Some(damage.clone()),
        diagnostics: vec![diagnostic.clone()],
        trace: vec![trace.clone()],
        proposal_hash: "fnv1a64:damage".to_string(),
    };
    let evidence = GameExtensionReplayEvidence {
        module_ref: module_ref.clone(),
        hook_id: "weapon.primary".to_string(),
        request_id: "request-1".to_string(),
        input_hash: "fnv1a64:input".to_string(),
        proposal_hash: "fnv1a64:damage".to_string(),
        validation_status: "accepted".to_string(),
        event_hashes: vec!["fnv1a64:event".to_string()],
        rejection_hashes: Vec::new(),
        replay_hash: "fnv1a64:replay".to_string(),
    };

    compare_object_to_interface(
        &game_extension,
        "GameRuleModuleRef",
        &serde_json::to_value(&module_ref).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_extension,
        "GameRuleHookDeclaration",
        &serde_json::to_value(&hook).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_extension,
        "GameRuleModuleManifest",
        &serde_json::to_value(&manifest).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_extension,
        "GameExtensionDiagnostic",
        &serde_json::to_value(&diagnostic).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_extension,
        "WeaponEffectHookRequest",
        &serde_json::to_value(&request).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_extension,
        "GameExtensionProposal",
        "damageModifier",
        &serde_json::to_value(&damage).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_extension,
        "GameExtensionProposal",
        "effectBundle",
        &serde_json::to_value(&bundle).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_extension,
        "GameExtensionProposal",
        "reject",
        &serde_json::to_value(&rejected).unwrap(),
    )
    .unwrap();
    compare_object_to_variant(
        &game_extension,
        "GameExtensionProposal",
        "noop",
        &serde_json::to_value(&noop).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_extension,
        "GameExtensionTraceEntry",
        &serde_json::to_value(&trace).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_extension,
        "GameExtensionHookReceipt",
        &serde_json::to_value(&receipt).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &game_extension,
        "GameExtensionReplayEvidence",
        &serde_json::to_value(&evidence).unwrap(),
    )
    .unwrap();
}

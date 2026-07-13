use core_ids::EntityId;
use game_rule_extension::{
    unsupported_hook_diagnostic, GameExtensionProposal, GameRuleExtensionResult,
    GameRuleHookDeclaration, GameRuleModule, GameRuleModuleManifest, GameRuleModuleRef,
    WeaponEffectHookRequest,
};
use protocol_game_extension::GameExtensionHookKind;
use rule_gameplay_fabric::{
    compatibility::{
        run_legacy_weapon_effect_transform, LegacyWeaponEffectTransformError,
        LEGACY_WEAPON_EFFECT_COMPATIBILITY_DIAGNOSTIC,
    },
    GameplayRuntimeDiagnosticCode,
};

struct FixtureModule {
    manifest: GameRuleModuleManifest,
    behavior: FixtureBehavior,
}

enum FixtureBehavior {
    Accept,
    WrongChannel,
    Reject,
}

impl GameRuleModule for FixtureModule {
    fn manifest(&self) -> &GameRuleModuleManifest {
        &self.manifest
    }

    fn evaluate_weapon_effect(
        &self,
        request: &WeaponEffectHookRequest,
    ) -> GameRuleExtensionResult<GameExtensionProposal> {
        match self.behavior {
            FixtureBehavior::Accept | FixtureBehavior::WrongChannel => {
                Ok(GameExtensionProposal::DamageModifier {
                    proposal_id: format!("{}.fixture", request.request_id),
                    target: request.target.unwrap(),
                    channel_id: match self.behavior {
                        FixtureBehavior::WrongChannel => "combat.foreign".to_owned(),
                        _ => "combat.primary_fire.damage".to_owned(),
                    },
                    amount_delta: 5,
                    tags: vec!["fixture".to_owned()],
                    proposal_hash: "fnv1a64:fixture-proposal".to_owned(),
                })
            }
            FixtureBehavior::Reject => Err(unsupported_hook_diagnostic(
                &request.hook_id,
                "fixture rejected the hook",
            )),
        }
    }
}

fn module(behavior: FixtureBehavior) -> FixtureModule {
    FixtureModule {
        manifest: GameRuleModuleManifest {
            module_ref: GameRuleModuleRef {
                module_id: "fixture.weapon_effect".to_owned(),
                version: "1.0.0".to_owned(),
                contract_hash: "fnv1a64:fixture-contract".to_owned(),
            },
            declared_hooks: vec![GameRuleHookDeclaration {
                hook_id: "weapon.primary".to_owned(),
                kind: GameExtensionHookKind::WeaponEffect,
                input_contract: "WeaponEffectHookRequest.v0".to_owned(),
                output_contract: "GameExtensionProposal.v0".to_owned(),
                required_capabilities: vec!["health".to_owned(), "weaponMount".to_owned()],
            }],
            deterministic_requirements: vec!["no-wall-clock".to_owned()],
            source_hash: "fnv1a64:fixture-artifact".to_owned(),
        },
        behavior,
    }
}

fn request() -> WeaponEffectHookRequest {
    WeaponEffectHookRequest {
        module_ref: module(FixtureBehavior::Accept).manifest.module_ref,
        hook_id: "weapon.primary".to_owned(),
        request_id: "request.compat-transform".to_owned(),
        tick: 19,
        source: EntityId::new(1),
        target: Some(EntityId::new(2)),
        base_damage: 8,
        range_millimeters: 900,
        tags: vec!["primary-fire".to_owned()],
        input_hash: "fnv1a64:fixture-input".to_owned(),
    }
}

#[test]
fn legacy_weapon_behavior_runs_inside_common_transform_and_owner_route() {
    assert_eq!(
        LEGACY_WEAPON_EFFECT_COMPATIBILITY_DIAGNOSTIC,
        "asha.compat.wave1.legacy-weapon-effect-hook"
    );
    let outcome =
        run_legacy_weapon_effect_transform(&module(FixtureBehavior::Accept), &request()).unwrap();
    assert_eq!(outcome.damage_delta, 5);
    assert!(outcome.decision_receipt.accepted());
    assert_eq!(outcome.decision_receipt.invocations.len(), 1);
    assert_eq!(
        outcome.decision_receipt.invocations[0].invocation_id,
        "compat.weapon-effect.transform"
    );
    assert!(outcome
        .decision_receipt
        .routing
        .as_ref()
        .is_some_and(|routing| routing.accepted));
}

#[test]
fn compatibility_owner_rejects_invalid_transformed_workspace() {
    let error =
        run_legacy_weapon_effect_transform(&module(FixtureBehavior::WrongChannel), &request())
            .unwrap_err();
    let LegacyWeaponEffectTransformError::DecisionRejected(receipt) = error else {
        panic!("expected common decision rejection");
    };
    assert!(receipt
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == GameplayRuntimeDiagnosticCode::OwnerRejected));
    assert!(receipt
        .routing
        .as_ref()
        .is_some_and(|routing| !routing.accepted));
}

#[test]
fn module_rejection_remains_typed_through_compatibility_wrapper() {
    let error = run_legacy_weapon_effect_transform(&module(FixtureBehavior::Reject), &request())
        .unwrap_err();
    let LegacyWeaponEffectTransformError::ModuleRejected(diagnostic) = error else {
        panic!("expected module diagnostic");
    };
    assert_eq!(diagnostic.path, "hooks.weapon.primary");
}

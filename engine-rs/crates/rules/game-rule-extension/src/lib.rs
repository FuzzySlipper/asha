//! Public Rust API for game-owned ASHA rule extension modules.
//!
//! Downstream games may compile crates against this boundary to contribute
//! authored rule decisions. The module is not a RuntimeSession replacement: it
//! receives deterministic hook requests and returns typed proposals that ASHA
//! validates and applies through normal authority paths.

#![forbid(unsafe_code)]

pub use protocol_game_extension::{
    GameExtensionDiagnostic, GameExtensionDiagnosticCode, GameExtensionHookKind,
    GameExtensionHookReceipt, GameExtensionProposal, GameExtensionReceiptStatus,
    GameExtensionReplayEvidence, GameExtensionTraceEntry, GameRuleHookDeclaration,
    GameRuleModuleManifest, GameRuleModuleRef, GameplayCausationRef, GameplayContractRef,
    GameplayEmitterRef, GameplayEntityRef, GameplayEventEnvelope, GameplayEventPhase,
    GameplayEventSchemaDeclaration, GameplayExecutionBudget, GameplayHeaderSelector,
    GameplayInvocationDescriptor, GameplayInvocationFamily, GameplayModuleManifest,
    GameplayModuleRef, GameplayOrderingConstraint, GameplayOwnedSchemaDeclaration,
    GameplayOwnerRef, GameplayProposalDeclaration, GameplayProposalEnvelope,
    GameplayReadViewRequirement, GameplayRegistryDiagnostic, GameplayRegistryDiagnosticCode,
    GameplayRegistryReadout, GameplayRegistryValidationOutcome, GameplaySubscriptionDeclaration,
    GameplayTopologyEdge, WeaponEffectHookRequest,
};

/// Result type used by game rule modules. Errors are typed diagnostics, never
/// raw dynamic payloads.
pub type GameRuleExtensionResult<T> = Result<T, GameExtensionDiagnostic>;

/// Public trait implemented by downstream game-owned Rust rule crates.
///
/// The default hook methods fail closed. ASHA RuntimeSession invocation code
/// chooses which declared hooks to call and remains responsible for validating
/// any returned proposal before it can mutate authority state.
pub trait GameRuleModule {
    /// Stable manifest compiled with the module and declared by the game
    /// project manifest. RuntimeSession compatibility checks use this metadata.
    fn manifest(&self) -> &GameRuleModuleManifest;

    /// Optional weapon-effect hook. The request carries deterministic facts
    /// supplied by ASHA; returned proposals are still pending authority
    /// validation.
    fn evaluate_weapon_effect(
        &self,
        request: &WeaponEffectHookRequest,
    ) -> GameRuleExtensionResult<GameExtensionProposal> {
        Err(unsupported_hook_diagnostic(
            &request.hook_id,
            "weapon effect hook is not implemented by this module",
        ))
    }
}

/// Build a typed fail-closed diagnostic for an unsupported hook.
pub fn unsupported_hook_diagnostic(hook_id: &str, message: &str) -> GameExtensionDiagnostic {
    GameExtensionDiagnostic {
        code: GameExtensionDiagnosticCode::UnsupportedHook,
        severity: protocol_diagnostics::DiagnosticSeverity::Error,
        path: format!("hooks.{hook_id}"),
        message: message.to_string(),
    }
}

/// Deterministically wrap a module proposal in a hook receipt. This helper does
/// not validate or apply the proposal; RuntimeSession owns that in #4517.
pub fn proposed_receipt(
    request: &WeaponEffectHookRequest,
    proposal: GameExtensionProposal,
    trace: Vec<GameExtensionTraceEntry>,
) -> GameExtensionHookReceipt {
    let proposal_hash = proposal.proposal_hash().to_string();
    GameExtensionHookReceipt {
        module_ref: request.module_ref.clone(),
        hook_id: request.hook_id.clone(),
        request_id: request.request_id.clone(),
        status: GameExtensionReceiptStatus::Proposed,
        input_hash: request.input_hash.clone(),
        proposal: Some(proposal),
        diagnostics: Vec::new(),
        trace,
        proposal_hash,
    }
}

/// Deterministically wrap a typed module diagnostic in a rejected receipt.
pub fn rejected_receipt(
    request: &WeaponEffectHookRequest,
    diagnostic: GameExtensionDiagnostic,
) -> GameExtensionHookReceipt {
    GameExtensionHookReceipt {
        module_ref: request.module_ref.clone(),
        hook_id: request.hook_id.clone(),
        request_id: request.request_id.clone(),
        status: GameExtensionReceiptStatus::RejectedByModule,
        input_hash: request.input_hash.clone(),
        proposal: None,
        diagnostics: vec![diagnostic],
        trace: Vec::new(),
        proposal_hash: "none".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_ids::EntityId;

    struct DemoModule {
        manifest: GameRuleModuleManifest,
    }

    impl GameRuleModule for DemoModule {
        fn manifest(&self) -> &GameRuleModuleManifest {
            &self.manifest
        }

        fn evaluate_weapon_effect(
            &self,
            request: &WeaponEffectHookRequest,
        ) -> GameRuleExtensionResult<GameExtensionProposal> {
            Ok(GameExtensionProposal::DamageModifier {
                proposal_id: format!("{}.close-range", request.request_id),
                target: request.target.expect("fixture target"),
                channel_id: "value.health".into(),
                amount_delta: -2,
                tags: vec!["close-range".into()],
                proposal_hash: "fnv1a64:proposal".into(),
            })
        }
    }

    fn manifest() -> GameRuleModuleManifest {
        GameRuleModuleManifest {
            module_ref: GameRuleModuleRef {
                module_id: "demo.primary_fire_effect".into(),
                version: "0.1.0".into(),
                contract_hash: "sha256:contract".into(),
            },
            declared_hooks: vec![GameRuleHookDeclaration {
                hook_id: "weapon.primary".into(),
                kind: GameExtensionHookKind::WeaponEffect,
                input_contract: "WeaponEffectHookRequest.v0".into(),
                output_contract: "GameExtensionProposal.v0".into(),
                required_capabilities: vec!["health".into(), "weaponMount".into()],
            }],
            deterministic_requirements: vec![
                "no-wall-clock".into(),
                "no-ambient-random".into(),
                "no-filesystem".into(),
                "no-network".into(),
                "no-ts-callback".into(),
            ],
            source_hash: "sha256:module-source".into(),
        }
    }

    fn request() -> WeaponEffectHookRequest {
        WeaponEffectHookRequest {
            module_ref: manifest().module_ref,
            hook_id: "weapon.primary".into(),
            request_id: "request-1".into(),
            tick: 42,
            source: EntityId::new(1),
            target: Some(EntityId::new(2)),
            base_damage: -8,
            range_millimeters: 400,
            tags: vec!["primary-fire".into()],
            input_hash: "fnv1a64:input".into(),
        }
    }

    #[test]
    fn implemented_module_returns_typed_pending_proposal() {
        let module = DemoModule {
            manifest: manifest(),
        };
        let request = request();
        let proposal = module
            .evaluate_weapon_effect(&request)
            .expect("demo module proposes");
        let receipt = proposed_receipt(&request, proposal, Vec::new());

        assert_eq!(
            module.manifest().module_ref.module_id,
            "demo.primary_fire_effect"
        );
        assert_eq!(receipt.status, GameExtensionReceiptStatus::Proposed);
        assert_eq!(receipt.proposal_hash, "fnv1a64:proposal");
    }

    #[test]
    fn default_hook_fails_closed_with_typed_diagnostic() {
        struct EmptyModule {
            manifest: GameRuleModuleManifest,
        }
        impl GameRuleModule for EmptyModule {
            fn manifest(&self) -> &GameRuleModuleManifest {
                &self.manifest
            }
        }

        let module = EmptyModule {
            manifest: manifest(),
        };
        let error = module
            .evaluate_weapon_effect(&request())
            .expect_err("default hook rejects");
        assert_eq!(error.code, GameExtensionDiagnosticCode::UnsupportedHook);
        assert_eq!(error.path, "hooks.weapon.primary");
    }
}

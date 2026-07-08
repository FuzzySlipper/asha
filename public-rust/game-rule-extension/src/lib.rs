//! Public facade crate for downstream ASHA game-owned rule modules.
//!
//! ASHA Game Projects should depend on this facade path rather than on private
//! `engine-rs/crates/*` implementation crates. The implementation source of
//! truth remains `engine-rs/crates/rules/game-rule-extension`.

#![forbid(unsafe_code)]

pub use game_rule_extension::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_module_trait<T: GameRuleModule>() {}

    #[test]
    fn facade_reexports_rule_module_trait_and_dtos() {
        struct EmptyModule {
            manifest: GameRuleModuleManifest,
        }

        impl GameRuleModule for EmptyModule {
            fn manifest(&self) -> &GameRuleModuleManifest {
                &self.manifest
            }
        }

        assert_module_trait::<EmptyModule>();
        let manifest = GameRuleModuleManifest {
            module_ref: GameRuleModuleRef {
                module_id: "demo.primary_fire_effect".to_string(),
                version: "0.1.0".to_string(),
                contract_hash: "sha256:contract".to_string(),
            },
            declared_hooks: vec![GameRuleHookDeclaration {
                hook_id: "weapon.primary".to_string(),
                kind: GameExtensionHookKind::WeaponEffect,
                input_contract: "WeaponEffectHookRequest.v0".to_string(),
                output_contract: "GameExtensionProposal.v0".to_string(),
                required_capabilities: vec!["health".to_string(), "weaponMount".to_string()],
            }],
            deterministic_requirements: vec!["no-wall-clock".to_string()],
            source_hash: "sha256:module-source".to_string(),
        };
        let module = EmptyModule { manifest };

        assert_eq!(
            module.manifest().module_ref.module_id,
            "demo.primary_fire_effect"
        );
    }
}

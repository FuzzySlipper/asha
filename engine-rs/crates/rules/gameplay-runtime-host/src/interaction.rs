//! Closed-registry gameplay reads and validated prefab interaction ingress.

use crate::{EntityId, GameplayRuntimeHost, GameplayRuntimeHostError};
use protocol_game_extension::{GameplayContractRef, GameplayEventEnvelope};
use rule_gameplay_fabric::{
    adapt_prefab_part_interaction, GameplayOwnerEventContext, PrefabPartInteractionGameplayPayload,
};
pub use rule_gameplay_fabric::{GameplayModuleNamedView, GameplayModuleStateScope};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayRuntimePrefabInteractionIntent {
    pub actor: EntityId,
    pub instance: u64,
    pub role: String,
    pub expected_target: EntityId,
    pub tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameplayRuntimePrefabInteractionReceipt {
    pub target: EntityId,
    pub event: GameplayEventEnvelope,
    pub reaction_frame_hash: String,
}

impl GameplayRuntimeHost {
    /// Project one closed-registry module view without revealing or copying its
    /// backing state. The returned canonical bytes remain identified by the
    /// registered contract, provider, revision, and view hash.
    pub fn module_named_view(
        &self,
        view: &GameplayContractRef,
        scope: &GameplayModuleStateScope,
    ) -> Result<GameplayModuleNamedView, GameplayRuntimeHostError> {
        self.session
            .module_state
            .named_view_by_contract(view, scope)
            .map_err(GameplayRuntimeHostError::State)
    }

    /// Resolve a stable role against the active prefab generation, validate
    /// the actor and caller's expected target, then publish the standard owner
    /// event. No caller-selected event contract or payload enters the Fabric.
    pub fn interact_with_prefab_part(
        &mut self,
        intent: GameplayRuntimePrefabInteractionIntent,
    ) -> Result<GameplayRuntimePrefabInteractionReceipt, GameplayRuntimeHostError> {
        let prefabs = self.prefab_readout();
        let instance = prefabs
            .instances
            .iter()
            .find(|candidate| candidate.instance == intent.instance)
            .ok_or_else(|| {
                GameplayRuntimeHostError::Prefab(format!(
                    "prefab instance {} is not active",
                    intent.instance
                ))
            })?;
        let target = instance
            .roles
            .iter()
            .find(|candidate| candidate.role == intent.role)
            .map(|role| EntityId::new(role.entity))
            .ok_or_else(|| {
                GameplayRuntimeHostError::Prefab(format!(
                    "prefab instance {} has no active role {}",
                    intent.instance, intent.role
                ))
            })?;
        if target != intent.expected_target {
            return Err(GameplayRuntimeHostError::Prefab(format!(
                "prefab role target mismatch: expected {}, resolved {}",
                intent.expected_target.raw(),
                target.raw()
            )));
        }
        let entities = self
            .session
            .bundle
            .runtime_entities
            .as_ref()
            .ok_or(GameplayRuntimeHostError::MissingEntityAuthority)?;
        if !entities.is_alive(intent.actor) {
            return Err(GameplayRuntimeHostError::Prefab(format!(
                "interaction actor {} is not active",
                intent.actor.raw()
            )));
        }
        if !entities.is_alive(target) {
            return Err(GameplayRuntimeHostError::Prefab(format!(
                "interaction target {} is not active",
                target.raw()
            )));
        }
        let payload = PrefabPartInteractionGameplayPayload {
            actor: intent.actor.raw(),
            instance: intent.instance,
            prefab: instance.prefab,
            role: intent.role,
            target: target.raw(),
            tick: intent.tick,
        };
        let context = GameplayOwnerEventContext {
            owner_id: "rule-project-bundle.prefab-interaction".to_owned(),
            tick: intent.tick,
            root_id: format!(
                "prefab-interaction:{}:{}:{}:{}",
                intent.tick,
                intent.actor.raw(),
                intent.instance,
                payload.role
            ),
            root_sequence: intent.tick,
            first_event_sequence: 0,
            parent_event_id: None,
        };
        let event = adapt_prefab_part_interaction(&context, &payload)
            .map_err(|error| GameplayRuntimeHostError::Codec(error.to_string()))?;
        let reaction = self.observe(event.clone())?;
        Ok(GameplayRuntimePrefabInteractionReceipt {
            target,
            event,
            reaction_frame_hash: reaction.frame.frame_hash,
        })
    }
}

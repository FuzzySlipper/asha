use super::*;

/// Opaque activation-state checkpoint used by a composed RuntimeSession
/// restart. It contains mutable authority state only; registry, bindings,
/// provider behavior, prefab authoring, and declared-read topology remain in
/// the live host.
#[derive(Clone)]
pub struct GameplayRuntimeResetCheckpoint {
    module_state: rule_gameplay_fabric::GameplayModuleStateCheckpoint,
    trigger_snapshot: rule_trigger_volume::TriggerVolumeSnapshot,
    reaction_frames: Vec<GameplayReactionFrame>,
    decision_continuations: GameplayDecisionContinuations,
    decision_receipts: Vec<GameplayDecisionReceipt>,
    scheduler: GameplayActionScheduler,
}

/// Opaque evidence checkpoint used by the enclosing RuntimeSession to make a
/// decision, owner commit, and resulting owner-event cascade one transaction.
/// It contains no module behavior, registry, EntityStore, or mutable authority
/// handle and is deliberately not a downstream persistence surface.
#[derive(Clone)]
pub struct GameplayRuntimeTransactionCheckpoint {
    reaction_frames: Vec<GameplayReactionFrame>,
    decision_continuations: GameplayDecisionContinuations,
    decision_receipts: Vec<GameplayDecisionReceipt>,
}

impl GameplayRuntimeHost {
    #[doc(hidden)]
    pub fn checkpoint_reset_state(&self) -> GameplayRuntimeResetCheckpoint {
        GameplayRuntimeResetCheckpoint {
            module_state: self.session.module_state.checkpoint(),
            trigger_snapshot: self.session.trigger_rule().snapshot(),
            reaction_frames: self.reaction_frames.clone(),
            decision_continuations: self.decision_continuations.clone(),
            decision_receipts: self.decision_receipts.clone(),
            scheduler: self.scheduler.clone(),
        }
    }

    #[doc(hidden)]
    pub fn restore_reset_state(
        &mut self,
        checkpoint: GameplayRuntimeResetCheckpoint,
    ) -> Result<(), GameplayRuntimeHostError> {
        self.session
            .restore_runtime_state(checkpoint.module_state, checkpoint.trigger_snapshot)?;
        self.reaction_frames = checkpoint.reaction_frames;
        self.decision_continuations = checkpoint.decision_continuations;
        self.decision_receipts = checkpoint.decision_receipts;
        self.scheduler = checkpoint.scheduler;
        Ok(())
    }

    #[doc(hidden)]
    pub fn checkpoint_transaction_evidence(&self) -> GameplayRuntimeTransactionCheckpoint {
        GameplayRuntimeTransactionCheckpoint {
            reaction_frames: self.reaction_frames.clone(),
            decision_continuations: self.decision_continuations.clone(),
            decision_receipts: self.decision_receipts.clone(),
        }
    }

    #[doc(hidden)]
    pub fn restore_transaction_evidence(
        &mut self,
        checkpoint: GameplayRuntimeTransactionCheckpoint,
    ) {
        self.reaction_frames = checkpoint.reaction_frames;
        self.decision_continuations = checkpoint.decision_continuations;
        self.decision_receipts = checkpoint.decision_receipts;
    }
}

pub(super) fn activation_hash(activation: &GameplayModuleBindingActivationReceipt) -> String {
    let bytes = serde_json::to_vec(activation).expect("activation receipt serializes");
    rule_gameplay_fabric::gameplay_module_payload_hash(&bytes)
}

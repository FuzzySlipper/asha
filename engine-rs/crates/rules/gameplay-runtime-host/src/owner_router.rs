//! Concrete RuntimeSession owner ports shared by decisions and scheduler routes.

use core_entity::EntityStore;
use protocol_entity_authoring::{
    ActivatableCapabilityKind, CapabilityActivationAction, CapabilityActivationOutcome,
    CapabilityActivationPresence, CapabilityActivationRequest,
};
use protocol_game_extension::{
    GameplayCausationRef, GameplayEmitterRef, GameplayEntityRef, GameplayEventEnvelope,
    GameplayEventPhase, GameplayOwnerRef,
};
use rule_gameplay_fabric::{
    gameplay_module_payload_hash, gameplay_payload_hash, CapabilityActivationGameplayPayload,
    CapabilityActivationGameplayProposal, GameplayDecisionOwner, GameplayDecisionRoutingOutput,
    GameplayOwnerRoutingCall, GameplayOwnerRoutingOutput, GameplayProposalRouter,
    StandardGameplayEventKind, StandardGameplayProposalKind,
    CAPABILITY_ACTIVATION_PROPOSAL_OWNER_ID,
};
use svc_entity_authoring::{apply_rule_owned_capability_activation, EcrpRuleOwner};

use crate::{EntityId, GameplayRuntimeDecisionOwner};

pub(crate) struct RuntimeSessionDecisionOwner<'a> {
    pub(crate) owner: &'a mut dyn GameplayRuntimeDecisionOwner,
}

impl GameplayDecisionOwner for RuntimeSessionDecisionOwner<'_> {
    fn revision_hash(&self, owner: &GameplayOwnerRef) -> String {
        self.owner.revision_hash(owner)
    }

    fn route_precommit(
        &mut self,
        call: &GameplayOwnerRoutingCall,
    ) -> GameplayDecisionRoutingOutput {
        let output = self.owner.route_precommit(&call.owner, &call.proposal);
        GameplayDecisionRoutingOutput {
            accepted: output.accepted,
            fact_hashes: output.fact_hashes,
            diagnostic_codes: output.diagnostic_codes,
        }
    }
}

pub(crate) struct RuntimeSessionOwnerRouter<'a> {
    pub(crate) entities: &'a mut EntityStore,
}

impl GameplayProposalRouter for RuntimeSessionOwnerRouter<'_> {
    fn route(&mut self, call: &GameplayOwnerRoutingCall) -> GameplayOwnerRoutingOutput {
        if call.proposal.proposal
            != StandardGameplayProposalKind::SetCapabilityActivation.contract()
            || call.owner.owner_id != CAPABILITY_ACTIVATION_PROPOSAL_OWNER_ID
        {
            return rejected_owner_output("unsupportedOwnerProposal");
        }
        let payload: CapabilityActivationGameplayProposal =
            match serde_json::from_slice(&call.proposal.canonical_payload) {
                Ok(payload) => payload,
                Err(_) => return rejected_owner_output("proposalDecodeFailed"),
            };
        if payload.entity == 0
            || payload.entity
                != call
                    .proposal
                    .targets
                    .first()
                    .map_or(0, |target| target.entity.raw())
        {
            return rejected_owner_output("proposalTargetMismatch");
        }
        let (capability, owner) = match payload.capability.as_str() {
            "collision" => (
                ActivatableCapabilityKind::Collision,
                EcrpRuleOwner::CollisionRule,
            ),
            "controller" => (
                ActivatableCapabilityKind::Controller,
                EcrpRuleOwner::ControllerRule,
            ),
            _ => return rejected_owner_output("unsupportedCapability"),
        };
        let action = match payload.action.as_str() {
            "activate" => CapabilityActivationAction::Activate,
            "deactivate" => CapabilityActivationAction::Deactivate,
            _ => return rejected_owner_output("unsupportedActivationAction"),
        };
        match apply_rule_owned_capability_activation(
            self.entities,
            owner,
            CapabilityActivationRequest {
                entity: EntityId::new(payload.entity),
                capability,
                action,
            },
        ) {
            CapabilityActivationOutcome::Accepted { event, .. } => {
                let event = match capability_activation_event(call, event) {
                    Ok(event) => event,
                    Err(_) => return rejected_owner_output("ownerEventEncodeFailed"),
                };
                GameplayOwnerRoutingOutput {
                    accepted: true,
                    fact_hashes: vec![gameplay_module_payload_hash(
                        &call.proposal.canonical_payload,
                    )],
                    events: vec![event],
                    ..GameplayOwnerRoutingOutput::default()
                }
            }
            CapabilityActivationOutcome::Rejected { diagnostic }
            | CapabilityActivationOutcome::Forbidden { diagnostic } => GameplayOwnerRoutingOutput {
                accepted: false,
                diagnostic_codes: vec![format!("{:?}", diagnostic.code)],
                ..GameplayOwnerRoutingOutput::default()
            },
        }
    }
}

fn capability_activation_event(
    call: &GameplayOwnerRoutingCall,
    event: protocol_entity_authoring::CapabilityActivationEvent,
) -> Result<GameplayEventEnvelope, serde_json::Error> {
    let capability = match event.capability {
        ActivatableCapabilityKind::Collision => "collision",
        ActivatableCapabilityKind::Controller => "controller",
    };
    let presence = |value| match value {
        CapabilityActivationPresence::Absent => "absent",
        CapabilityActivationPresence::Inactive => "inactive",
        CapabilityActivationPresence::Active => "active",
    };
    let payload = CapabilityActivationGameplayPayload {
        entity: event.entity.raw(),
        capability: capability.to_owned(),
        from: presence(event.from).to_owned(),
        to: presence(event.to).to_owned(),
    };
    let canonical_payload = serde_json::to_vec(&payload)?;
    Ok(GameplayEventEnvelope {
        event_id: "candidate-owner-event".to_owned(),
        event: StandardGameplayEventKind::CapabilityActivationChanged.contract(),
        tick: call.proposal.tick,
        root_sequence: call.proposal.root_sequence,
        wave: call.proposal.wave,
        event_sequence: 0,
        phase: GameplayEventPhase::PostCommit,
        emitter: GameplayEmitterRef::Owner {
            owner_id: call.owner.owner_id.clone(),
        },
        causation: GameplayCausationRef {
            root_id: call.proposal.causation.root_id.clone(),
            parent_event_id: call
                .proposal
                .originating_event_id
                .clone()
                .or_else(|| call.proposal.causation.parent_event_id.clone()),
            decision_id: call.proposal.causation.decision_id.clone(),
        },
        source: None,
        subjects: vec![GameplayEntityRef {
            entity: event.entity,
        }],
        targets: Vec::new(),
        scope: Some("capability-activation".to_owned()),
        tags: vec![capability.to_owned(), presence(event.to).to_owned()],
        payload_hash: gameplay_payload_hash(&canonical_payload),
        canonical_payload,
    })
}

fn rejected_owner_output(code: &str) -> GameplayOwnerRoutingOutput {
    GameplayOwnerRoutingOutput {
        accepted: false,
        diagnostic_codes: vec![code.to_owned()],
        ..GameplayOwnerRoutingOutput::default()
    }
}

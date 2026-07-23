//! Concrete RuntimeSession owner ports shared by Fabric and direct programs.

use core_entity::EntityStore;
use protocol_game_extension::GameplayOwnerRef;
use rule_gameplay_fabric::{
    CapabilityActivationGameplayProposal, GameplayDecisionOwner, GameplayDecisionRoutingOutput,
    GameplayOwnerEventContext, GameplayOwnerRoutingCall, GameplayOwnerRoutingOutput,
    GameplayProposalRouter, StandardGameplayProposalKind, CAPABILITY_ACTIVATION_PROPOSAL_OWNER_ID,
};

use crate::{
    authority_verbs::{
        AuthorityCapability, AuthorityVerb, AuthorityVerbExecutor, DIRECT_AUTHORITY_OWNER_ID,
    },
    EntityId, GameplayRuntimeDecisionOwner,
};

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
            return rejected("unsupportedOwnerProposal");
        }
        let payload: CapabilityActivationGameplayProposal =
            match serde_json::from_slice(&call.proposal.canonical_payload) {
                Ok(payload) => payload,
                Err(_) => return rejected("proposalDecodeFailed"),
            };
        if payload.entity == 0
            || payload.entity
                != call
                    .proposal
                    .targets
                    .first()
                    .map_or(0, |target| target.entity.raw())
        {
            return rejected("proposalTargetMismatch");
        }
        let capability = match payload.capability.as_str() {
            "collision" => AuthorityCapability::Collision,
            "controller" => AuthorityCapability::Controller,
            _ => return rejected("unsupportedCapability"),
        };
        let active = match payload.action.as_str() {
            "activate" => true,
            "deactivate" => false,
            _ => return rejected("unsupportedActivationAction"),
        };
        let mut no_machines = [];
        let execution = AuthorityVerbExecutor {
            entities: self.entities,
            machines: &mut no_machines,
        }
        .execute_atomic(
            &[AuthorityVerb::SetCapabilityActive {
                entity: EntityId::new(payload.entity),
                capability,
                active,
            }],
            &GameplayOwnerEventContext {
                owner_id: DIRECT_AUTHORITY_OWNER_ID.to_owned(),
                tick: call.proposal.tick,
                root_id: call.proposal.causation.root_id.clone(),
                root_sequence: call.proposal.root_sequence,
                first_event_sequence: 0,
                parent_event_id: call
                    .proposal
                    .originating_event_id
                    .clone()
                    .or_else(|| call.proposal.causation.parent_event_id.clone()),
            },
        );
        match execution {
            Ok(execution) => GameplayOwnerRoutingOutput {
                accepted: true,
                fact_hashes: execution
                    .facts
                    .into_iter()
                    .map(|fact| fact.fact_hash)
                    .collect(),
                events: execution.events,
                ..GameplayOwnerRoutingOutput::default()
            },
            Err(error) => rejected(error.code()),
        }
    }
}

fn rejected(code: &str) -> GameplayOwnerRoutingOutput {
    GameplayOwnerRoutingOutput {
        accepted: false,
        diagnostic_codes: vec![code.to_owned()],
        ..GameplayOwnerRoutingOutput::default()
    }
}

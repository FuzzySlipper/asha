//! Protocol border for game-owned Rust rule extension modules.
//!
//! # Lane
//!
//! `contract-steward` - schema-only DTOs and stable vocabularies. This crate
//! contains no RuntimeSession invocation, authority mutation, dynamic loading,
//! renderer/UI coupling, TypeScript callbacks, or arbitrary JSON command tunnel.

#![forbid(unsafe_code)]

use core_ids::EntityId;
use protocol_diagnostics::DiagnosticSeverity;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

fn serialize_entity_id<S>(id: &EntityId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(id.raw())
}

fn deserialize_entity_id<'de, D>(deserializer: D) -> Result<EntityId, D::Error>
where
    D: Deserializer<'de>,
{
    u64::deserialize(deserializer).map(EntityId::new)
}

fn serialize_optional_entity_id<S>(id: &Option<EntityId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    id.map(EntityId::raw).serialize(serializer)
}

fn deserialize_optional_entity_id<'de, D>(deserializer: D) -> Result<Option<EntityId>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<u64>::deserialize(deserializer).map(|raw| raw.map(EntityId::new))
}

pub const GAME_EXTENSION_HOOK_KINDS: &[&str] =
    &["weaponEffect", "interactionEffect", "spawnCondition"];

pub const GAME_EXTENSION_PROPOSAL_KINDS: &[&str] =
    &["damageModifier", "effectBundle", "reject", "noop"];

pub const GAME_EXTENSION_RECEIPT_STATUSES: &[&str] =
    &["proposed", "rejectedByModule", "unsupportedHook"];

pub const GAME_EXTENSION_DIAGNOSTIC_CODES: &[&str] = &[
    "unsupportedHook",
    "incompatibleContract",
    "invalidModuleManifest",
    "nondeterministicInput",
    "invalidProposal",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameExtensionHookKind {
    WeaponEffect,
    InteractionEffect,
    SpawnCondition,
}

impl GameExtensionHookKind {
    pub fn as_str(self) -> &'static str {
        match self {
            GameExtensionHookKind::WeaponEffect => "weaponEffect",
            GameExtensionHookKind::InteractionEffect => "interactionEffect",
            GameExtensionHookKind::SpawnCondition => "spawnCondition",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameExtensionReceiptStatus {
    Proposed,
    RejectedByModule,
    UnsupportedHook,
}

impl GameExtensionReceiptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            GameExtensionReceiptStatus::Proposed => "proposed",
            GameExtensionReceiptStatus::RejectedByModule => "rejectedByModule",
            GameExtensionReceiptStatus::UnsupportedHook => "unsupportedHook",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameExtensionDiagnosticCode {
    UnsupportedHook,
    IncompatibleContract,
    InvalidModuleManifest,
    NondeterministicInput,
    InvalidProposal,
}

impl GameExtensionDiagnosticCode {
    pub fn as_str(self) -> &'static str {
        match self {
            GameExtensionDiagnosticCode::UnsupportedHook => "unsupportedHook",
            GameExtensionDiagnosticCode::IncompatibleContract => "incompatibleContract",
            GameExtensionDiagnosticCode::InvalidModuleManifest => "invalidModuleManifest",
            GameExtensionDiagnosticCode::NondeterministicInput => "nondeterministicInput",
            GameExtensionDiagnosticCode::InvalidProposal => "invalidProposal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleModuleRef {
    pub module_id: String,
    pub version: String,
    pub contract_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleHookDeclaration {
    pub hook_id: String,
    pub kind: GameExtensionHookKind,
    pub input_contract: String,
    pub output_contract: String,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleModuleManifest {
    pub module_ref: GameRuleModuleRef,
    pub declared_hooks: Vec<GameRuleHookDeclaration>,
    pub deterministic_requirements: Vec<String>,
    pub source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExtensionDiagnostic {
    pub code: GameExtensionDiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeaponEffectHookRequest {
    pub module_ref: GameRuleModuleRef,
    pub hook_id: String,
    pub request_id: String,
    pub tick: u64,
    #[serde(
        serialize_with = "serialize_entity_id",
        deserialize_with = "deserialize_entity_id"
    )]
    pub source: EntityId,
    #[serde(
        serialize_with = "serialize_optional_entity_id",
        deserialize_with = "deserialize_optional_entity_id"
    )]
    pub target: Option<EntityId>,
    pub base_damage: i64,
    pub range_millimeters: u32,
    pub tags: Vec<String>,
    pub input_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GameExtensionProposal {
    DamageModifier {
        proposal_id: String,
        #[serde(
            serialize_with = "serialize_entity_id",
            deserialize_with = "deserialize_entity_id"
        )]
        target: EntityId,
        channel_id: String,
        amount_delta: i64,
        tags: Vec<String>,
        proposal_hash: String,
    },
    EffectBundle {
        proposal_id: String,
        bundle_id: String,
        tags: Vec<String>,
        proposal_hash: String,
    },
    Reject {
        proposal_id: String,
        code: GameExtensionDiagnosticCode,
        message: String,
        proposal_hash: String,
    },
    Noop {
        proposal_id: String,
        proposal_hash: String,
    },
}

impl GameExtensionProposal {
    pub fn proposal_hash(&self) -> &str {
        match self {
            GameExtensionProposal::DamageModifier { proposal_hash, .. }
            | GameExtensionProposal::EffectBundle { proposal_hash, .. }
            | GameExtensionProposal::Reject { proposal_hash, .. }
            | GameExtensionProposal::Noop { proposal_hash, .. } => proposal_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExtensionTraceEntry {
    pub step: u32,
    pub code: String,
    pub message: String,
    pub refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExtensionHookReceipt {
    pub module_ref: GameRuleModuleRef,
    pub hook_id: String,
    pub request_id: String,
    pub status: GameExtensionReceiptStatus,
    pub input_hash: String,
    pub proposal: Option<GameExtensionProposal>,
    pub diagnostics: Vec<GameExtensionDiagnostic>,
    pub trace: Vec<GameExtensionTraceEntry>,
    pub proposal_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExtensionReplayEvidence {
    pub module_ref: GameRuleModuleRef,
    pub hook_id: String,
    pub request_id: String,
    pub input_hash: String,
    pub proposal_hash: String,
    pub validation_status: String,
    pub event_hashes: Vec<String>,
    pub rejection_hashes: Vec<String>,
    pub replay_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol_diagnostics::DiagnosticSeverity;

    #[test]
    fn proposal_hash_is_variant_independent() {
        let proposal = GameExtensionProposal::DamageModifier {
            proposal_id: "proposal.demo".into(),
            target: EntityId::new(7),
            channel_id: "value.health".into(),
            amount_delta: 2,
            tags: vec!["close-range".into()],
            proposal_hash: "fnv1a64:proposal".into(),
        };
        assert_eq!(proposal.proposal_hash(), "fnv1a64:proposal");
    }

    #[test]
    fn weapon_effect_receipt_serializes_with_numeric_entity_ids() {
        let module_ref = GameRuleModuleRef {
            module_id: "demo.primary_fire_effect".into(),
            version: "0.1.0".into(),
            contract_hash: "sha256:contract".into(),
        };
        let receipt = GameExtensionHookReceipt {
            module_ref,
            hook_id: "weapon.primary".into(),
            request_id: "request-1".into(),
            status: GameExtensionReceiptStatus::Proposed,
            input_hash: "fnv1a64:input".into(),
            proposal: Some(GameExtensionProposal::Reject {
                proposal_id: "proposal.reject".into(),
                code: GameExtensionDiagnosticCode::InvalidProposal,
                message: "demo rejection".into(),
                proposal_hash: "fnv1a64:proposal".into(),
            }),
            diagnostics: vec![GameExtensionDiagnostic {
                code: GameExtensionDiagnosticCode::InvalidProposal,
                severity: DiagnosticSeverity::Warning,
                path: "proposal".into(),
                message: "classified".into(),
            }],
            trace: vec![GameExtensionTraceEntry {
                step: 1,
                code: "module.rejected".into(),
                message: "module returned a typed rejection".into(),
                refs: vec!["proposal.reject".into()],
            }],
            proposal_hash: "fnv1a64:proposal".into(),
        };

        let value = serde_json::to_value(&receipt).expect("receipt serializes");
        assert_eq!(value["proposal"]["kind"], "reject");
        assert_eq!(value["status"], "proposed");
    }
}

//! Protocol border for generic ASHA game-rules catalogs and readouts.
//!
//! # Lane
//!
//! `contract-steward` - schema-only DTOs and stable vocabularies. This crate
//! contains no authority validation, effect interpretation, service imports,
//! renderer types, bridge code, or TypeScript logic.

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

pub const GAME_RULE_EFFECT_OP_KINDS: &[&str] = &[
    "applyDelta",
    "restore",
    "spend",
    "grant",
    "applyModifier",
    "removeModifier",
    "schedulePeriodicEffect",
    "cancelResolution",
    "emitTrace",
];

pub const GAME_RULE_STACK_POLICIES: &[&str] =
    &["refresh", "stack", "rejectDuplicate", "replaceIfStronger"];

pub const GAME_RULE_DIAGNOSTIC_CODES: &[&str] = &[
    "unknownEffectOp",
    "invalidBoundedValue",
    "invalidAmount",
    "invalidDuration",
    "invalidCadence",
    "invalidStackPolicy",
    "undeclaredValueChannel",
    "unknownModifier",
    "cyclicPeriodicSchedule",
];

pub const GAME_RULE_EVIDENCE_KINDS: &[&str] = &[
    "catalogValidation",
    "resolutionReceipt",
    "trace",
    "replaySummary",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleCatalogRef {
    pub catalog_id: String,
    pub version: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleValueChannelRef {
    pub channel_id: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleBoundedValue {
    pub channel_id: String,
    pub min: i64,
    pub current: i64,
    pub max: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleValueDelta {
    pub channel_id: String,
    pub amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GameRuleDuration {
    Instant,
    Ticks { ticks: u64 },
    Infinite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleTickCadence {
    pub period_ticks: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GameRuleStackPolicy {
    Refresh,
    Stack { max_stacks: u32 },
    RejectDuplicate,
    ReplaceIfStronger,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GameRuleEffectOp {
    ApplyDelta {
        op_id: String,
        channel_id: String,
        amount: i64,
        tags: Vec<String>,
    },
    Restore {
        op_id: String,
        channel_id: String,
        amount: u32,
        tags: Vec<String>,
    },
    Spend {
        op_id: String,
        channel_id: String,
        amount: u32,
        tags: Vec<String>,
    },
    Grant {
        op_id: String,
        channel_id: String,
        amount: u32,
        tags: Vec<String>,
    },
    ApplyModifier {
        op_id: String,
        modifier_id: String,
        tags: Vec<String>,
    },
    RemoveModifier {
        op_id: String,
        modifier_id: String,
        tags: Vec<String>,
    },
    SchedulePeriodicEffect {
        op_id: String,
        modifier_id: String,
        cadence: GameRuleTickCadence,
        duration: GameRuleDuration,
        tags: Vec<String>,
    },
    CancelResolution {
        op_id: String,
        reason: String,
        tags: Vec<String>,
    },
    EmitTrace {
        op_id: String,
        code: String,
        message: String,
        tags: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleModifierDefinition {
    pub modifier_id: String,
    pub stack_policy: GameRuleStackPolicy,
    pub duration: GameRuleDuration,
    pub tick_cadence: Option<GameRuleTickCadence>,
    pub tags: Vec<String>,
    pub effect_op_ids: Vec<String>,
    pub source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleEffectBundle {
    pub bundle_id: String,
    pub effect_ops: Vec<GameRuleEffectOp>,
    pub modifiers: Vec<GameRuleModifierDefinition>,
    pub tags: Vec<String>,
    pub source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleCatalog {
    pub catalog: GameRuleCatalogRef,
    pub value_channels: Vec<GameRuleValueChannelRef>,
    pub bundles: Vec<GameRuleEffectBundle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameRuleDiagnosticCode {
    UnknownEffectOp,
    InvalidBoundedValue,
    InvalidAmount,
    InvalidDuration,
    InvalidCadence,
    InvalidStackPolicy,
    UndeclaredValueChannel,
    UnknownModifier,
    CyclicPeriodicSchedule,
}

impl GameRuleDiagnosticCode {
    pub fn as_str(self) -> &'static str {
        match self {
            GameRuleDiagnosticCode::UnknownEffectOp => "unknownEffectOp",
            GameRuleDiagnosticCode::InvalidBoundedValue => "invalidBoundedValue",
            GameRuleDiagnosticCode::InvalidAmount => "invalidAmount",
            GameRuleDiagnosticCode::InvalidDuration => "invalidDuration",
            GameRuleDiagnosticCode::InvalidCadence => "invalidCadence",
            GameRuleDiagnosticCode::InvalidStackPolicy => "invalidStackPolicy",
            GameRuleDiagnosticCode::UndeclaredValueChannel => "undeclaredValueChannel",
            GameRuleDiagnosticCode::UnknownModifier => "unknownModifier",
            GameRuleDiagnosticCode::CyclicPeriodicSchedule => "cyclicPeriodicSchedule",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleDiagnostic {
    pub code: GameRuleDiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleEvidenceRef {
    pub kind: GameRuleEvidenceKind,
    pub uri: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameRuleEvidenceKind {
    CatalogValidation,
    ResolutionReceipt,
    Trace,
    ReplaySummary,
}

impl GameRuleEvidenceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            GameRuleEvidenceKind::CatalogValidation => "catalogValidation",
            GameRuleEvidenceKind::ResolutionReceipt => "resolutionReceipt",
            GameRuleEvidenceKind::Trace => "trace",
            GameRuleEvidenceKind::ReplaySummary => "replaySummary",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleTraceRef {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleTraceEntry {
    pub step: u32,
    pub code: String,
    pub message: String,
    pub refs: Vec<GameRuleTraceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleModifierState {
    pub modifier_id: String,
    #[serde(
        serialize_with = "serialize_entity_id",
        deserialize_with = "deserialize_entity_id"
    )]
    pub source: EntityId,
    #[serde(
        serialize_with = "serialize_entity_id",
        deserialize_with = "deserialize_entity_id"
    )]
    pub target: EntityId,
    pub stacks: u32,
    pub applied_tick: u64,
    pub expires_tick: Option<u64>,
    pub next_tick: Option<u64>,
    pub source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleResolutionRequest {
    pub catalog: GameRuleCatalogRef,
    pub bundle_id: String,
    #[serde(
        serialize_with = "serialize_entity_id",
        deserialize_with = "deserialize_entity_id"
    )]
    pub source: EntityId,
    #[serde(
        serialize_with = "serialize_entity_id",
        deserialize_with = "deserialize_entity_id"
    )]
    pub target: EntityId,
    pub values: Vec<GameRuleBoundedValue>,
    pub tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRuleResolutionReceipt {
    pub accepted: bool,
    pub request_hash: String,
    pub pending_value_deltas: Vec<GameRuleValueDelta>,
    pub applied_modifiers: Vec<GameRuleModifierState>,
    pub diagnostics: Vec<GameRuleDiagnostic>,
    pub trace: Vec<GameRuleTraceEntry>,
    pub evidence: Vec<GameRuleEvidenceRef>,
    pub replay_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn catalog_ref() -> GameRuleCatalogRef {
        GameRuleCatalogRef {
            catalog_id: "catalog.game-rules.demo".to_string(),
            version: "0.1.0".to_string(),
            content_hash: "fnv1a64:catalog".to_string(),
        }
    }

    #[test]
    fn valid_poison_modifier_catalog_serializes_with_camel_case_border() {
        let catalog = GameRuleCatalog {
            catalog: catalog_ref(),
            value_channels: vec![GameRuleValueChannelRef {
                channel_id: "value.health".to_string(),
                display_name: Some("Health".to_string()),
            }],
            bundles: vec![GameRuleEffectBundle {
                bundle_id: "bundle.poisoned-impact".to_string(),
                effect_ops: vec![
                    GameRuleEffectOp::ApplyDelta {
                        op_id: "op.impact-damage".to_string(),
                        channel_id: "value.health".to_string(),
                        amount: -7,
                        tags: vec!["tag.impact".to_string()],
                    },
                    GameRuleEffectOp::SchedulePeriodicEffect {
                        op_id: "op.schedule-poison".to_string(),
                        modifier_id: "modifier.poison".to_string(),
                        cadence: GameRuleTickCadence { period_ticks: 3 },
                        duration: GameRuleDuration::Ticks { ticks: 9 },
                        tags: vec!["tag.poison".to_string()],
                    },
                ],
                modifiers: vec![GameRuleModifierDefinition {
                    modifier_id: "modifier.poison".to_string(),
                    stack_policy: GameRuleStackPolicy::Refresh,
                    duration: GameRuleDuration::Ticks { ticks: 9 },
                    tick_cadence: Some(GameRuleTickCadence { period_ticks: 3 }),
                    tags: vec!["tag.poison".to_string()],
                    effect_op_ids: vec!["op.poison-tick".to_string()],
                    source_hash: "fnv1a64:modifier".to_string(),
                }],
                tags: vec!["tag.poison".to_string()],
                source_hash: "fnv1a64:bundle".to_string(),
            }],
        };

        let serialized = serde_json::to_value(&catalog).unwrap();
        assert_eq!(
            serialized["catalog"]["catalogId"],
            json!("catalog.game-rules.demo")
        );
        assert_eq!(
            serialized["bundles"][0]["effectOps"][1]["kind"],
            json!("schedulePeriodicEffect")
        );
        assert_eq!(
            serialized["bundles"][0]["modifiers"][0]["tickCadence"]["periodTicks"],
            json!(3)
        );
    }

    #[test]
    fn rejected_bad_cadence_fixture_uses_classified_diagnostic_shape() {
        let diagnostic = GameRuleDiagnostic {
            code: GameRuleDiagnosticCode::InvalidCadence,
            severity: DiagnosticSeverity::Error,
            path: "bundles[0].effectOps[1].cadence.periodTicks".to_string(),
            message: "cadence period must be greater than zero".to_string(),
        };

        let serialized = serde_json::to_value(&diagnostic).unwrap();
        assert_eq!(serialized["code"], json!("invalidCadence"));
        assert_eq!(serialized["severity"], json!("error"));
        assert_eq!(
            GameRuleDiagnosticCode::InvalidCadence.as_str(),
            "invalidCadence"
        );
    }

    #[test]
    fn generic_action_bundle_and_resolution_receipt_are_schema_only() {
        let receipt = GameRuleResolutionReceipt {
            accepted: true,
            request_hash: "fnv1a64:request".to_string(),
            pending_value_deltas: vec![GameRuleValueDelta {
                channel_id: "value.health".to_string(),
                amount: -4,
            }],
            applied_modifiers: vec![GameRuleModifierState {
                modifier_id: "modifier.slow".to_string(),
                source: EntityId::new(1),
                target: EntityId::new(2),
                stacks: 1,
                applied_tick: 12,
                expires_tick: Some(18),
                next_tick: None,
                source_hash: "fnv1a64:slow".to_string(),
            }],
            diagnostics: vec![],
            trace: vec![GameRuleTraceEntry {
                step: 1,
                code: "effect.accepted".to_string(),
                message: "bundle resolved".to_string(),
                refs: vec![GameRuleTraceRef {
                    key: "bundle".to_string(),
                    value: "bundle.generic-action".to_string(),
                }],
            }],
            evidence: vec![GameRuleEvidenceRef {
                kind: GameRuleEvidenceKind::ResolutionReceipt,
                uri: "asha://game-rules/receipt/demo".to_string(),
                content_hash: "fnv1a64:receipt".to_string(),
            }],
            replay_hash: "fnv1a64:replay".to_string(),
        };

        let serialized = serde_json::to_value(&receipt).unwrap();
        assert_eq!(serialized["accepted"], json!(true));
        assert_eq!(serialized["appliedModifiers"][0]["source"], json!(1));
        assert_eq!(
            serialized["evidence"][0]["kind"],
            json!("resolutionReceipt")
        );
        assert_eq!(
            GameRuleEvidenceKind::ResolutionReceipt.as_str(),
            "resolutionReceipt"
        );
    }
}

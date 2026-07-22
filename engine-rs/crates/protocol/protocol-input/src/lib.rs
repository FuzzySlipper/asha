//! Generated-border contracts for named input actions and Session contexts.
//!
//! These DTOs describe normalized platform input, authored action catalogs,
//! Session-owned context state, and deterministic resolution evidence. They do
//! not listen to platform events and do not attach input state to entities.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const INPUT_BINDING_CATALOG_SCHEMA_VERSION: u32 = 1;
pub const PROJECT_INPUT_CATALOG_SCHEMA_VERSION: u32 = 1;
pub const INPUT_CONTEXT_STATE_SCHEMA_VERSION: u32 = 1;
pub const INPUT_ACTION_RECORD_SCHEMA_VERSION: u32 = 1;

pub type InputActionId = String;
pub type InputContextId = String;
pub type InputBindingId = String;

pub const INPUT_VALUE_KINDS: &[&str] = &["button", "axis1d", "axis2d"];
pub const INPUT_ACTION_PHASES: &[&str] = &["pressed", "held", "released", "changed"];
pub const PLATFORM_INPUT_KINDS: &[&str] =
    &["keyboardKey", "mouseButton", "mouseDelta", "mouseWheel"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InputValueKind {
    Button,
    Axis1d,
    Axis2d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InputActionPhase {
    Pressed,
    Held,
    Released,
    Changed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlatformInputKind {
    KeyboardKey,
    MouseButton,
    MouseDelta,
    MouseWheel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum InputValue {
    Button { pressed: bool },
    Axis1d { value: f64 },
    Axis2d { x: f64, y: f64 },
}

impl InputValue {
    pub fn value_kind(&self) -> InputValueKind {
        match self {
            Self::Button { .. } => InputValueKind::Button,
            Self::Axis1d { .. } => InputValueKind::Axis1d,
            Self::Axis2d { .. } => InputValueKind::Axis2d,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputActionDefinition {
    pub action_id: InputActionId,
    pub value_kind: InputValueKind,
    pub accepted_phases: Vec<InputActionPhase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputContextDefinition {
    pub context_id: InputContextId,
    pub priority: i32,
    pub consumes_lower_priority: bool,
}

/// Reserved versioned seam for modifiers/chords. Schema v1 catalogs must leave
/// this absent; a later schema can define execution without changing bindings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputBindingExtension {
    pub schema_version: u32,
    pub required_controls: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputBindingRecord {
    pub binding_id: InputBindingId,
    pub action_id: InputActionId,
    pub context_id: InputContextId,
    pub platform_kind: PlatformInputKind,
    pub control: String,
    pub scale: f64,
    pub extension: Option<InputBindingExtension>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputBindingCatalog {
    pub schema_version: u32,
    pub actions: Vec<InputActionDefinition>,
    pub contexts: Vec<InputContextDefinition>,
    pub bindings: Vec<InputBindingRecord>,
}

/// One immutable Game Project extension to the Engine input catalog.
///
/// Project actions, contexts, and bindings use a consumer-owned namespace.
/// Bindings may target a compatible Engine context such as `gameplay`, but
/// they cannot replace an Engine action, context, or normalized control.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectInputCatalog {
    pub schema_version: u32,
    pub namespace: String,
    pub actions: Vec<InputActionDefinition>,
    pub contexts: Vec<InputContextDefinition>,
    pub bindings: Vec<InputBindingRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSessionConfigureRequest {
    pub catalog: InputBindingCatalog,
    pub initial_contexts: Vec<InputContextId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveInputContext {
    pub context_id: InputContextId,
    pub stack_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputContextStackState {
    pub schema_version: u32,
    pub revision: u64,
    pub active_contexts: Vec<ActiveInputContext>,
    pub state_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum InputContextCommand {
    Push { context_id: InputContextId },
    Pop { expected_context_id: InputContextId },
    Replace { context_ids: Vec<InputContextId> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputContextChangeReceipt {
    pub accepted: bool,
    pub state: InputContextStackState,
    pub diagnostics: Vec<InputDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSessionSnapshot {
    pub catalog_hash: String,
    pub context_state: InputContextStackState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawInputSample {
    pub sequence: u64,
    pub platform_kind: PlatformInputKind,
    pub control: String,
    pub phase: InputActionPhase,
    pub value: InputValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedInputAction {
    pub sequence: u64,
    pub action_id: InputActionId,
    pub context_id: InputContextId,
    pub binding_id: InputBindingId,
    pub phase: InputActionPhase,
    pub value: InputValue,
}

/// Authority-issued semantic input evidence suitable for deterministic replay.
///
/// This deliberately contains the resolved action rather than a platform input
/// kind, control code, or browser event. The catalog and context hashes bind the
/// meaning of the action to the Session configuration that produced it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedInputAction {
    pub schema_version: u32,
    pub action: ResolvedInputAction,
    pub catalog_hash: String,
    pub context_hash: String,
    pub record_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InputDiagnosticCode {
    UnsupportedCatalogSchema,
    UnsupportedContextSchema,
    InvalidIdentifier,
    DuplicateAction,
    DuplicateContext,
    DuplicateBinding,
    CatalogLimitExceeded,
    DuplicateProjectCatalog,
    ReservedNamespace,
    ProtectedControl,
    InvalidControl,
    InvalidPriority,
    UnknownAction,
    UnknownContext,
    ConflictingBinding,
    ValueKindMismatch,
    UnsupportedBindingExtension,
    DuplicateActiveContext,
    NonCanonicalStackOrder,
    ContextStackMismatch,
    CatalogHashMismatch,
    ContextHashMismatch,
    NonFiniteInput,
    UnsupportedPhase,
    UnboundInput,
    ConsumedByContext,
    UnsupportedReplaySchema,
    ReplayRecordHashMismatch,
    ReplayAlreadyDelivered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputDiagnostic {
    pub code: InputDiagnosticCode,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputResolutionReceipt {
    pub sequence: u64,
    pub accepted: bool,
    pub consumed: bool,
    pub action: Option<ResolvedInputAction>,
    pub diagnostics: Vec<InputDiagnostic>,
    pub catalog_hash: String,
    pub context_hash: String,
    pub input_hash: String,
    pub resolution_hash: String,
    pub record: Option<RecordedInputAction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputActionReplayReceipt {
    pub accepted: bool,
    pub action: Option<ResolvedInputAction>,
    pub diagnostics: Vec<InputDiagnostic>,
    pub catalog_hash: String,
    pub context_hash: String,
    pub record_hash: String,
    pub replay_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_tables_match_serialized_variants() {
        assert_eq!(INPUT_VALUE_KINDS, ["button", "axis1d", "axis2d"]);
        assert_eq!(
            serde_json::to_value(InputActionPhase::Changed).unwrap(),
            serde_json::json!("changed")
        );
        assert_eq!(
            serde_json::to_value(PlatformInputKind::MouseDelta).unwrap(),
            serde_json::json!("mouseDelta")
        );
    }

    #[test]
    fn input_value_is_a_closed_tagged_union() {
        assert_eq!(
            serde_json::to_value(InputValue::Axis2d { x: 2.0, y: -1.0 }).unwrap(),
            serde_json::json!({"kind": "axis2d", "x": 2.0, "y": -1.0})
        );
    }
}

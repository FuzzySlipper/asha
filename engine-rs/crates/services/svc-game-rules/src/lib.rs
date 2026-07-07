//! Pure service layer for generic ASHA game-rule effect resolution.
//!
//! # Lane
//!
//! `rust-service` - validates game-rule catalogs and interprets generic effect
//! operations against explicit input facts. This crate returns pending receipts,
//! traces, diagnostics, and hashes only. It does not import or mutate
//! `SessionState`, combat state, renderer state, bridges, TypeScript, or policy
//! packages.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use core_game_rules::{
    BoundedValue, EffectOpId, EffectTagId, GameRuleCatalogId, ModifierId, ValueChannelId,
    ValueDelta,
};
use core_ids::EntityId;
use core_time::{Tick, TickDelta};
use protocol_diagnostics::DiagnosticSeverity;
use protocol_game_rules::{
    GameRuleBoundedValue, GameRuleCatalog, GameRuleCatalogRef, GameRuleDiagnostic,
    GameRuleDiagnosticCode, GameRuleDuration, GameRuleEffectOp, GameRuleEvidenceKind,
    GameRuleEvidenceRef, GameRuleModifierDefinition, GameRuleModifierState,
    GameRuleResolutionReceipt, GameRuleResolutionRequest, GameRuleStackPolicy, GameRuleTickCadence,
    GameRuleTraceEntry, GameRuleTraceRef, GameRuleValueDelta,
};

pub const AUTHORITY_VERSION: &str = "svc-game-rules.v0";

/// Service-local resolution request. It mirrors the protocol request and adds
/// incoming context tags supplied by the caller. Those tags are deterministic
/// inputs only; they do not grant mutation rights.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectResolutionRequest {
    pub catalog: GameRuleCatalogRef,
    pub bundle_id: String,
    pub source: EntityId,
    pub target: EntityId,
    pub values: Vec<GameRuleBoundedValue>,
    pub incoming_tags: Vec<String>,
    pub tick: u64,
}

impl From<GameRuleResolutionRequest> for EffectResolutionRequest {
    fn from(value: GameRuleResolutionRequest) -> Self {
        Self {
            catalog: value.catalog,
            bundle_id: value.bundle_id,
            source: value.source,
            target: value.target,
            values: value.values,
            incoming_tags: Vec::new(),
            tick: value.tick,
        }
    }
}

/// Result of validating an authored game-rule catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogValidationReport {
    pub diagnostics: Vec<GameRuleDiagnostic>,
    pub trace: Vec<GameRuleTraceEntry>,
    pub catalog_hash: String,
}

impl CatalogValidationReport {
    pub fn accepted(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error)
    }
}

pub fn validate_catalog(catalog: &GameRuleCatalog) -> CatalogValidationReport {
    let mut diagnostics = Vec::new();
    let mut trace = TraceBuilder::default();
    let mut channels = BTreeSet::new();
    let mut modifiers = BTreeMap::new();
    let mut ops = BTreeMap::new();

    validate_catalog_ref(&catalog.catalog, "catalog", &mut diagnostics);
    trace
        .push("catalog.validationStarted", "validating game-rule catalog")
        .with_ref("catalog", &catalog.catalog.catalog_id);

    for (index, channel) in catalog.value_channels.iter().enumerate() {
        validate_stable_id::<ValueChannelId>(
            &channel.channel_id,
            GameRuleDiagnosticCode::UndeclaredValueChannel,
            format!("valueChannels[{index}].channelId"),
            "value channel id is not a stable game-rule id",
            &mut diagnostics,
        );
        if !channels.insert(channel.channel_id.clone()) {
            diagnostics.push(diagnostic(
                GameRuleDiagnosticCode::UndeclaredValueChannel,
                DiagnosticSeverity::Error,
                format!("valueChannels[{index}].channelId"),
                "duplicate value channel id",
            ));
        }
    }

    for (bundle_index, bundle) in catalog.bundles.iter().enumerate() {
        validate_stable_id::<GameRuleCatalogId>(
            &bundle.bundle_id,
            GameRuleDiagnosticCode::UnknownEffectOp,
            format!("bundles[{bundle_index}].bundleId"),
            "bundle id is not a stable game-rule id",
            &mut diagnostics,
        );

        for (op_index, op) in bundle.effect_ops.iter().enumerate() {
            let path = format!("bundles[{bundle_index}].effectOps[{op_index}]");
            validate_effect_op(op, &channels, &path, &mut diagnostics);
            if let Some(previous) = ops.insert(op_id(op).to_string(), (bundle_index, op_index)) {
                diagnostics.push(diagnostic(
                    GameRuleDiagnosticCode::UnknownEffectOp,
                    DiagnosticSeverity::Error,
                    path,
                    format!(
                        "duplicate effect op id already declared at bundles[{}].effectOps[{}]",
                        previous.0, previous.1
                    ),
                ));
            }
        }

        for (modifier_index, modifier) in bundle.modifiers.iter().enumerate() {
            let path = format!("bundles[{bundle_index}].modifiers[{modifier_index}]");
            validate_modifier(modifier, &path, &mut diagnostics);
            if modifiers
                .insert(modifier.modifier_id.clone(), (bundle_index, modifier_index))
                .is_some()
            {
                diagnostics.push(diagnostic(
                    GameRuleDiagnosticCode::UnknownModifier,
                    DiagnosticSeverity::Error,
                    path,
                    "duplicate modifier id",
                ));
            }
        }
    }

    validate_references(catalog, &channels, &modifiers, &ops, &mut diagnostics);
    validate_schedule_cycles(catalog, &modifiers, &mut diagnostics);

    if diagnostics.is_empty() {
        trace
            .push("catalog.accepted", "catalog validation accepted")
            .with_ref("catalogHash", catalog_hash(catalog));
    } else {
        trace
            .push(
                "catalog.rejected",
                "catalog validation produced diagnostics",
            )
            .with_ref("diagnosticCount", diagnostics.len().to_string());
    }

    CatalogValidationReport {
        diagnostics,
        trace: trace.finish(),
        catalog_hash: catalog_hash(catalog),
    }
}

pub fn resolve_protocol_request(
    request: &GameRuleResolutionRequest,
    catalog: &GameRuleCatalog,
) -> GameRuleResolutionReceipt {
    resolve_effect_request(&request.clone().into(), catalog)
}

pub fn resolve_effect_request(
    request: &EffectResolutionRequest,
    catalog: &GameRuleCatalog,
) -> GameRuleResolutionReceipt {
    let validation = validate_catalog(catalog);
    let mut diagnostics = validation.diagnostics;
    let mut trace = TraceBuilder::from_entries(validation.trace);

    if request.catalog.catalog_id != catalog.catalog.catalog_id
        || request.catalog.version != catalog.catalog.version
        || request.catalog.content_hash != catalog.catalog.content_hash
    {
        diagnostics.push(diagnostic(
            GameRuleDiagnosticCode::UnknownEffectOp,
            DiagnosticSeverity::Error,
            "catalog",
            "request catalog reference does not match supplied catalog",
        ));
    }

    let Some(bundle) = catalog
        .bundles
        .iter()
        .find(|bundle| bundle.bundle_id == request.bundle_id)
    else {
        diagnostics.push(diagnostic(
            GameRuleDiagnosticCode::UnknownEffectOp,
            DiagnosticSeverity::Error,
            "bundleId",
            "requested effect bundle does not exist in catalog",
        ));
        return receipt(false, request, Vec::new(), Vec::new(), diagnostics, trace);
    };

    let declared_channels = catalog
        .value_channels
        .iter()
        .map(|channel| channel.channel_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut values = value_workspace(request, &declared_channels, &mut diagnostics);

    if !diagnostics.is_empty() {
        return receipt(false, request, Vec::new(), Vec::new(), diagnostics, trace);
    }

    let modifiers = catalog
        .bundles
        .iter()
        .flat_map(|bundle| bundle.modifiers.iter())
        .map(|modifier| (modifier.modifier_id.as_str(), modifier))
        .collect::<BTreeMap<_, _>>();
    let referenced_tick_ops = bundle
        .modifiers
        .iter()
        .flat_map(|modifier| modifier.effect_op_ids.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();

    let mut deltas = Vec::new();
    let mut pending_modifiers = Vec::new();
    let mut canceled = false;

    trace
        .push("resolution.started", "resolving effect bundle")
        .with_ref("bundle", &request.bundle_id)
        .with_ref("tick", request.tick.to_string());

    for op in &bundle.effect_ops {
        if referenced_tick_ops.contains(op_id(op)) {
            continue;
        }
        if canceled {
            break;
        }
        match op {
            GameRuleEffectOp::ApplyDelta {
                op_id,
                channel_id,
                amount,
                ..
            } => {
                apply_value_delta(
                    op_id,
                    channel_id,
                    *amount,
                    &mut values,
                    &mut deltas,
                    &mut trace,
                    &mut diagnostics,
                );
            }
            GameRuleEffectOp::Restore {
                op_id,
                channel_id,
                amount,
                ..
            }
            | GameRuleEffectOp::Grant {
                op_id,
                channel_id,
                amount,
                ..
            } => {
                apply_value_delta(
                    op_id,
                    channel_id,
                    i64::from(*amount),
                    &mut values,
                    &mut deltas,
                    &mut trace,
                    &mut diagnostics,
                );
            }
            GameRuleEffectOp::Spend {
                op_id,
                channel_id,
                amount,
                ..
            } => {
                apply_value_delta(
                    op_id,
                    channel_id,
                    -i64::from(*amount),
                    &mut values,
                    &mut deltas,
                    &mut trace,
                    &mut diagnostics,
                );
            }
            GameRuleEffectOp::ApplyModifier {
                op_id, modifier_id, ..
            } => {
                if let Some(modifier) = modifiers.get(modifier_id.as_str()) {
                    pending_modifiers.push(modifier_state_from_definition(
                        modifier,
                        request,
                        modifier.duration.clone(),
                        modifier.tick_cadence.clone(),
                    ));
                    trace
                        .push("modifier.pendingApply", "modifier application is pending")
                        .with_ref("op", op_id)
                        .with_ref("modifier", modifier_id);
                }
            }
            GameRuleEffectOp::RemoveModifier {
                op_id, modifier_id, ..
            } => {
                trace
                    .push("modifier.pendingRemove", "modifier removal is pending")
                    .with_ref("op", op_id)
                    .with_ref("modifier", modifier_id);
            }
            GameRuleEffectOp::SchedulePeriodicEffect {
                op_id,
                modifier_id,
                cadence,
                duration,
                ..
            } => {
                if let Some(modifier) = modifiers.get(modifier_id.as_str()) {
                    pending_modifiers.push(modifier_state_from_definition(
                        modifier,
                        request,
                        duration.clone(),
                        Some(cadence.clone()),
                    ));
                    trace
                        .push(
                            "modifier.periodicScheduled",
                            "periodic modifier tick is pending",
                        )
                        .with_ref("op", op_id)
                        .with_ref("modifier", modifier_id)
                        .with_ref("nextTick", next_tick(request.tick, cadence).to_string());
                }
            }
            GameRuleEffectOp::CancelResolution { op_id, reason, .. } => {
                canceled = true;
                trace
                    .push(
                        "resolution.canceled",
                        "effect operation canceled resolution",
                    )
                    .with_ref("op", op_id)
                    .with_ref("reason", reason);
            }
            GameRuleEffectOp::EmitTrace {
                op_id,
                code,
                message,
                ..
            } => {
                trace
                    .push(code.clone(), message.clone())
                    .with_ref("op", op_id);
            }
        }
    }

    let accepted = !canceled && diagnostics.is_empty();
    if accepted {
        trace
            .push(
                "resolution.accepted",
                "resolution produced pending outcomes",
            )
            .with_ref("valueDeltaCount", deltas.len().to_string())
            .with_ref("modifierCount", pending_modifiers.len().to_string());
    }

    receipt(
        accepted,
        request,
        deltas,
        pending_modifiers,
        diagnostics,
        trace,
    )
}

fn validate_catalog_ref(
    catalog: &GameRuleCatalogRef,
    path: &str,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    validate_stable_id::<GameRuleCatalogId>(
        &catalog.catalog_id,
        GameRuleDiagnosticCode::UnknownEffectOp,
        format!("{path}.catalogId"),
        "catalog id is not a stable game-rule id",
        diagnostics,
    );
    if catalog.version.is_empty() || catalog.content_hash.is_empty() {
        diagnostics.push(diagnostic(
            GameRuleDiagnosticCode::UnknownEffectOp,
            DiagnosticSeverity::Error,
            path,
            "catalog version and content hash are required",
        ));
    }
}

fn validate_effect_op(
    op: &GameRuleEffectOp,
    channels: &BTreeSet<String>,
    path: &str,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    validate_stable_id::<EffectOpId>(
        op_id(op),
        GameRuleDiagnosticCode::UnknownEffectOp,
        format!("{path}.opId"),
        "effect op id is not a stable game-rule id",
        diagnostics,
    );
    for (tag_index, tag) in op_tags(op).iter().enumerate() {
        validate_stable_id::<EffectTagId>(
            tag,
            GameRuleDiagnosticCode::UnknownEffectOp,
            format!("{path}.tags[{tag_index}]"),
            "effect tag is not a stable game-rule id",
            diagnostics,
        );
    }
    if let Some(channel) = op_channel(op) {
        if !channels.contains(channel) {
            diagnostics.push(diagnostic(
                GameRuleDiagnosticCode::UndeclaredValueChannel,
                DiagnosticSeverity::Error,
                format!("{path}.channelId"),
                format!("effect op references undeclared value channel `{channel}`"),
            ));
        }
    }
    match op {
        GameRuleEffectOp::Restore { amount, .. }
        | GameRuleEffectOp::Spend { amount, .. }
        | GameRuleEffectOp::Grant { amount, .. }
            if *amount == 0 =>
        {
            diagnostics.push(diagnostic(
                GameRuleDiagnosticCode::InvalidAmount,
                DiagnosticSeverity::Error,
                format!("{path}.amount"),
                "amount must be greater than zero",
            ));
        }
        GameRuleEffectOp::SchedulePeriodicEffect {
            cadence, duration, ..
        } => {
            validate_cadence(cadence, format!("{path}.cadence"), diagnostics);
            validate_duration(duration, format!("{path}.duration"), diagnostics);
        }
        _ => {}
    }
}

fn validate_modifier(
    modifier: &GameRuleModifierDefinition,
    path: &str,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    validate_stable_id::<ModifierId>(
        &modifier.modifier_id,
        GameRuleDiagnosticCode::UnknownModifier,
        format!("{path}.modifierId"),
        "modifier id is not a stable game-rule id",
        diagnostics,
    );
    validate_stack_policy(
        &modifier.stack_policy,
        format!("{path}.stackPolicy"),
        diagnostics,
    );
    validate_duration(&modifier.duration, format!("{path}.duration"), diagnostics);
    if let Some(cadence) = &modifier.tick_cadence {
        validate_cadence(cadence, format!("{path}.tickCadence"), diagnostics);
    }
    for (tag_index, tag) in modifier.tags.iter().enumerate() {
        validate_stable_id::<EffectTagId>(
            tag,
            GameRuleDiagnosticCode::UnknownModifier,
            format!("{path}.tags[{tag_index}]"),
            "modifier tag is not a stable game-rule id",
            diagnostics,
        );
    }
}

fn validate_references(
    catalog: &GameRuleCatalog,
    channels: &BTreeSet<String>,
    modifiers: &BTreeMap<String, (usize, usize)>,
    ops: &BTreeMap<String, (usize, usize)>,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    for (bundle_index, bundle) in catalog.bundles.iter().enumerate() {
        for (op_index, op) in bundle.effect_ops.iter().enumerate() {
            if let Some(modifier) = op_modifier(op) {
                if !modifiers.contains_key(modifier) {
                    diagnostics.push(diagnostic(
                        GameRuleDiagnosticCode::UnknownModifier,
                        DiagnosticSeverity::Error,
                        format!("bundles[{bundle_index}].effectOps[{op_index}].modifierId"),
                        format!("effect op references unknown modifier `{modifier}`"),
                    ));
                }
            }
            if let Some(channel) = op_channel(op) {
                if !channels.contains(channel) {
                    diagnostics.push(diagnostic(
                        GameRuleDiagnosticCode::UndeclaredValueChannel,
                        DiagnosticSeverity::Error,
                        format!("bundles[{bundle_index}].effectOps[{op_index}].channelId"),
                        format!("effect op references undeclared value channel `{channel}`"),
                    ));
                }
            }
        }

        for (modifier_index, modifier) in bundle.modifiers.iter().enumerate() {
            for (effect_index, effect_op_id) in modifier.effect_op_ids.iter().enumerate() {
                if !ops.contains_key(effect_op_id) {
                    diagnostics.push(diagnostic(
                        GameRuleDiagnosticCode::UnknownEffectOp,
                        DiagnosticSeverity::Error,
                        format!(
                            "bundles[{bundle_index}].modifiers[{modifier_index}].effectOpIds[{effect_index}]"
                        ),
                        format!("modifier references unknown effect op `{effect_op_id}`"),
                    ));
                }
            }
        }
    }
}

fn validate_schedule_cycles(
    catalog: &GameRuleCatalog,
    modifiers: &BTreeMap<String, (usize, usize)>,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for bundle in &catalog.bundles {
        for op in &bundle.effect_ops {
            if let GameRuleEffectOp::SchedulePeriodicEffect {
                op_id, modifier_id, ..
            } = op
            {
                if let Some((bundle_index, modifier_index)) = modifiers.get(modifier_id) {
                    let modifier = &catalog.bundles[*bundle_index].modifiers[*modifier_index];
                    edges
                        .entry(op_id.clone())
                        .or_default()
                        .extend(modifier.effect_op_ids.iter().cloned());
                }
            }
        }
    }

    for start in edges.keys() {
        let mut visiting = BTreeSet::new();
        if has_cycle(start, start, &edges, &mut visiting) {
            diagnostics.push(diagnostic(
                GameRuleDiagnosticCode::CyclicPeriodicSchedule,
                DiagnosticSeverity::Error,
                "bundles.effectOps",
                format!("periodic schedule cycle reaches `{start}`"),
            ));
        }
    }
}

fn has_cycle(
    start: &str,
    current: &str,
    edges: &BTreeMap<String, BTreeSet<String>>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    let Some(next) = edges.get(current) else {
        return false;
    };
    if !visiting.insert(current.to_string()) {
        return false;
    }
    for candidate in next {
        if candidate == start || has_cycle(start, candidate, edges, visiting) {
            return true;
        }
    }
    visiting.remove(current);
    false
}

fn value_workspace(
    request: &EffectResolutionRequest,
    declared_channels: &BTreeSet<&str>,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) -> BTreeMap<String, BoundedValue> {
    let mut values = BTreeMap::new();
    for (index, value) in request.values.iter().enumerate() {
        if !declared_channels.contains(value.channel_id.as_str()) {
            diagnostics.push(diagnostic(
                GameRuleDiagnosticCode::UndeclaredValueChannel,
                DiagnosticSeverity::Error,
                format!("values[{index}].channelId"),
                "resolution value fact uses undeclared channel",
            ));
            continue;
        }
        match BoundedValue::new(value.min, value.current, value.max) {
            Ok(bounded) => {
                if values.insert(value.channel_id.clone(), bounded).is_some() {
                    diagnostics.push(diagnostic(
                        GameRuleDiagnosticCode::InvalidBoundedValue,
                        DiagnosticSeverity::Error,
                        format!("values[{index}].channelId"),
                        "duplicate value fact for channel",
                    ));
                }
            }
            Err(_) => diagnostics.push(diagnostic(
                GameRuleDiagnosticCode::InvalidBoundedValue,
                DiagnosticSeverity::Error,
                format!("values[{index}]"),
                "bounded value requires min <= current <= max",
            )),
        }
    }
    values
}

fn apply_value_delta(
    op_id: &str,
    channel: &str,
    amount: i64,
    values: &mut BTreeMap<String, BoundedValue>,
    deltas: &mut Vec<GameRuleValueDelta>,
    trace: &mut TraceBuilder,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    let Some(value) = values.get(channel).copied() else {
        diagnostics.push(diagnostic(
            GameRuleDiagnosticCode::UndeclaredValueChannel,
            DiagnosticSeverity::Error,
            channel,
            "effect op has no supplied value fact",
        ));
        return;
    };
    let applied = value.apply_delta(ValueDelta::new(amount));
    values.insert(channel.to_string(), applied.after);
    let actual = applied.after.current - applied.before.current;
    deltas.push(GameRuleValueDelta {
        channel_id: channel.to_string(),
        amount: actual,
    });
    trace
        .push("value.deltaPending", "value delta is pending")
        .with_ref("op", op_id)
        .with_ref("channel", channel)
        .with_ref("amount", actual.to_string())
        .with_ref("after", applied.after.current.to_string());
}

fn modifier_state_from_definition(
    modifier: &GameRuleModifierDefinition,
    request: &EffectResolutionRequest,
    duration: GameRuleDuration,
    cadence: Option<GameRuleTickCadence>,
) -> GameRuleModifierState {
    GameRuleModifierState {
        modifier_id: modifier.modifier_id.clone(),
        source: request.source,
        target: request.target,
        stacks: 1,
        applied_tick: request.tick,
        expires_tick: duration_expires(request.tick, &duration),
        next_tick: cadence
            .as_ref()
            .map(|cadence| next_tick(request.tick, cadence)),
        source_hash: modifier.source_hash.clone(),
    }
}

fn receipt(
    accepted: bool,
    request: &EffectResolutionRequest,
    pending_value_deltas: Vec<GameRuleValueDelta>,
    applied_modifiers: Vec<GameRuleModifierState>,
    diagnostics: Vec<GameRuleDiagnostic>,
    trace: TraceBuilder,
) -> GameRuleResolutionReceipt {
    let trace = trace.finish();
    let request_hash = request_hash(request);
    let replay_hash = replay_hash(
        accepted,
        &request_hash,
        &pending_value_deltas,
        &applied_modifiers,
        &diagnostics,
        &trace,
    );
    let evidence = vec![GameRuleEvidenceRef {
        kind: GameRuleEvidenceKind::ResolutionReceipt,
        uri: format!("asha://game-rules/receipt/{request_hash}"),
        content_hash: replay_hash.clone(),
    }];

    GameRuleResolutionReceipt {
        accepted,
        request_hash,
        pending_value_deltas,
        applied_modifiers,
        diagnostics,
        trace,
        evidence,
        replay_hash,
    }
}

fn validate_stack_policy(
    policy: &GameRuleStackPolicy,
    path: String,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    if matches!(policy, GameRuleStackPolicy::Stack { max_stacks: 0 }) {
        diagnostics.push(diagnostic(
            GameRuleDiagnosticCode::InvalidStackPolicy,
            DiagnosticSeverity::Error,
            path,
            "stack policy maxStacks must be greater than zero",
        ));
    }
}

fn validate_duration(
    duration: &GameRuleDuration,
    path: String,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    if matches!(duration, GameRuleDuration::Ticks { ticks: 0 }) {
        diagnostics.push(diagnostic(
            GameRuleDiagnosticCode::InvalidDuration,
            DiagnosticSeverity::Error,
            path,
            "duration ticks must be greater than zero",
        ));
    }
}

fn validate_cadence(
    cadence: &GameRuleTickCadence,
    path: String,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    if cadence.period_ticks == 0 {
        diagnostics.push(diagnostic(
            GameRuleDiagnosticCode::InvalidCadence,
            DiagnosticSeverity::Error,
            path,
            "cadence periodTicks must be greater than zero",
        ));
    }
}

trait StableId: Sized {
    fn parse_id(value: &str) -> bool;
}

macro_rules! stable_id {
    ($ty:ty) => {
        impl StableId for $ty {
            fn parse_id(value: &str) -> bool {
                <$ty>::parse(value).is_ok()
            }
        }
    };
}

stable_id!(GameRuleCatalogId);
stable_id!(EffectOpId);
stable_id!(ModifierId);
stable_id!(ValueChannelId);
stable_id!(EffectTagId);

fn validate_stable_id<T: StableId>(
    value: &str,
    code: GameRuleDiagnosticCode,
    path: String,
    message: &str,
    diagnostics: &mut Vec<GameRuleDiagnostic>,
) {
    if !T::parse_id(value) {
        diagnostics.push(diagnostic(code, DiagnosticSeverity::Error, path, message));
    }
}

fn op_id(op: &GameRuleEffectOp) -> &str {
    match op {
        GameRuleEffectOp::ApplyDelta { op_id, .. }
        | GameRuleEffectOp::Restore { op_id, .. }
        | GameRuleEffectOp::Spend { op_id, .. }
        | GameRuleEffectOp::Grant { op_id, .. }
        | GameRuleEffectOp::ApplyModifier { op_id, .. }
        | GameRuleEffectOp::RemoveModifier { op_id, .. }
        | GameRuleEffectOp::SchedulePeriodicEffect { op_id, .. }
        | GameRuleEffectOp::CancelResolution { op_id, .. }
        | GameRuleEffectOp::EmitTrace { op_id, .. } => op_id,
    }
}

fn op_tags(op: &GameRuleEffectOp) -> &[String] {
    match op {
        GameRuleEffectOp::ApplyDelta { tags, .. }
        | GameRuleEffectOp::Restore { tags, .. }
        | GameRuleEffectOp::Spend { tags, .. }
        | GameRuleEffectOp::Grant { tags, .. }
        | GameRuleEffectOp::ApplyModifier { tags, .. }
        | GameRuleEffectOp::RemoveModifier { tags, .. }
        | GameRuleEffectOp::SchedulePeriodicEffect { tags, .. }
        | GameRuleEffectOp::CancelResolution { tags, .. }
        | GameRuleEffectOp::EmitTrace { tags, .. } => tags,
    }
}

fn op_channel(op: &GameRuleEffectOp) -> Option<&str> {
    match op {
        GameRuleEffectOp::ApplyDelta { channel_id, .. }
        | GameRuleEffectOp::Restore { channel_id, .. }
        | GameRuleEffectOp::Spend { channel_id, .. }
        | GameRuleEffectOp::Grant { channel_id, .. } => Some(channel_id),
        _ => None,
    }
}

fn op_modifier(op: &GameRuleEffectOp) -> Option<&str> {
    match op {
        GameRuleEffectOp::ApplyModifier { modifier_id, .. }
        | GameRuleEffectOp::RemoveModifier { modifier_id, .. }
        | GameRuleEffectOp::SchedulePeriodicEffect { modifier_id, .. } => Some(modifier_id),
        _ => None,
    }
}

fn duration_expires(start: u64, duration: &GameRuleDuration) -> Option<u64> {
    match duration {
        GameRuleDuration::Instant => Some(start),
        GameRuleDuration::Ticks { ticks } => {
            Some(Tick::new(start).advance(TickDelta::new(*ticks)).raw())
        }
        GameRuleDuration::Infinite => None,
    }
}

fn next_tick(start: u64, cadence: &GameRuleTickCadence) -> u64 {
    Tick::new(start)
        .advance(TickDelta::new(cadence.period_ticks))
        .raw()
}

fn diagnostic(
    code: GameRuleDiagnosticCode,
    severity: DiagnosticSeverity,
    path: impl Into<String>,
    message: impl Into<String>,
) -> GameRuleDiagnostic {
    GameRuleDiagnostic {
        code,
        severity,
        path: path.into(),
        message: message.into(),
    }
}

#[derive(Debug, Default)]
struct TraceBuilder {
    entries: Vec<GameRuleTraceEntry>,
}

impl TraceBuilder {
    fn from_entries(entries: Vec<GameRuleTraceEntry>) -> Self {
        Self { entries }
    }

    fn push(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> &mut GameRuleTraceEntry {
        let step = self.entries.len() as u32 + 1;
        self.entries.push(GameRuleTraceEntry {
            step,
            code: code.into(),
            message: message.into(),
            refs: Vec::new(),
        });
        self.entries.last_mut().expect("just pushed trace entry")
    }

    fn finish(mut self) -> Vec<GameRuleTraceEntry> {
        for entry in &mut self.entries {
            entry
                .refs
                .sort_by(|a, b| a.key.cmp(&b.key).then_with(|| a.value.cmp(&b.value)));
        }
        self.entries
    }
}

trait TraceEntryExt {
    fn with_ref(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self;
}

impl TraceEntryExt for GameRuleTraceEntry {
    fn with_ref(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.refs.push(GameRuleTraceRef {
            key: key.into(),
            value: value.into(),
        });
        self
    }
}

fn catalog_hash(catalog: &GameRuleCatalog) -> String {
    let mut parts = vec![
        "catalog".to_string(),
        catalog.catalog.catalog_id.clone(),
        catalog.catalog.version.clone(),
        catalog.catalog.content_hash.clone(),
    ];
    for channel in &catalog.value_channels {
        parts.push(format!(
            "channel:{}:{:?}",
            channel.channel_id, channel.display_name
        ));
    }
    for bundle in &catalog.bundles {
        parts.push(format!(
            "bundle:{}:{}",
            bundle.bundle_id, bundle.source_hash
        ));
        for op in &bundle.effect_ops {
            parts.push(format!("op:{}", op_fingerprint(op)));
        }
        for modifier in &bundle.modifiers {
            parts.push(format!("modifier:{}", modifier_fingerprint(modifier)));
        }
    }
    stable_hash(&parts)
}

fn request_hash(request: &EffectResolutionRequest) -> String {
    let mut parts = vec![
        "request".to_string(),
        request.catalog.catalog_id.clone(),
        request.catalog.version.clone(),
        request.catalog.content_hash.clone(),
        request.bundle_id.clone(),
        request.source.raw().to_string(),
        request.target.raw().to_string(),
        request.tick.to_string(),
    ];
    for tag in sorted(request.incoming_tags.iter().map(String::as_str)) {
        parts.push(format!("tag:{tag}"));
    }
    for value in sorted_values(&request.values) {
        parts.push(format!(
            "value:{}:{}:{}:{}",
            value.channel_id, value.min, value.current, value.max
        ));
    }
    stable_hash(&parts)
}

fn replay_hash(
    accepted: bool,
    request_hash: &str,
    deltas: &[GameRuleValueDelta],
    modifiers: &[GameRuleModifierState],
    diagnostics: &[GameRuleDiagnostic],
    trace: &[GameRuleTraceEntry],
) -> String {
    let mut parts = vec![
        "replay".to_string(),
        AUTHORITY_VERSION.to_string(),
        accepted.to_string(),
        request_hash.to_string(),
    ];
    for delta in deltas {
        parts.push(format!("delta:{}:{}", delta.channel_id, delta.amount));
    }
    for modifier in modifiers {
        parts.push(format!(
            "modifier:{}:{}:{}:{}:{:?}:{:?}:{}",
            modifier.modifier_id,
            modifier.source.raw(),
            modifier.target.raw(),
            modifier.applied_tick,
            modifier.expires_tick,
            modifier.next_tick,
            modifier.source_hash
        ));
    }
    for diagnostic in diagnostics {
        parts.push(format!(
            "diagnostic:{}:{:?}:{}:{}",
            diagnostic.code.as_str(),
            diagnostic.severity,
            diagnostic.path,
            diagnostic.message
        ));
    }
    for entry in trace {
        parts.push(format!(
            "trace:{}:{}:{}",
            entry.step, entry.code, entry.message
        ));
        for r in &entry.refs {
            parts.push(format!("trace-ref:{}:{}", r.key, r.value));
        }
    }
    stable_hash(&parts)
}

fn op_fingerprint(op: &GameRuleEffectOp) -> String {
    match op {
        GameRuleEffectOp::ApplyDelta {
            op_id,
            channel_id,
            amount,
            tags,
        } => format!(
            "applyDelta:{op_id}:{channel_id}:{amount}:{:?}",
            sorted(tags.iter().map(String::as_str))
        ),
        GameRuleEffectOp::Restore {
            op_id,
            channel_id,
            amount,
            tags,
        } => format!(
            "restore:{op_id}:{channel_id}:{amount}:{:?}",
            sorted(tags.iter().map(String::as_str))
        ),
        GameRuleEffectOp::Spend {
            op_id,
            channel_id,
            amount,
            tags,
        } => format!(
            "spend:{op_id}:{channel_id}:{amount}:{:?}",
            sorted(tags.iter().map(String::as_str))
        ),
        GameRuleEffectOp::Grant {
            op_id,
            channel_id,
            amount,
            tags,
        } => format!(
            "grant:{op_id}:{channel_id}:{amount}:{:?}",
            sorted(tags.iter().map(String::as_str))
        ),
        GameRuleEffectOp::ApplyModifier {
            op_id,
            modifier_id,
            tags,
        } => format!(
            "applyModifier:{op_id}:{modifier_id}:{:?}",
            sorted(tags.iter().map(String::as_str))
        ),
        GameRuleEffectOp::RemoveModifier {
            op_id,
            modifier_id,
            tags,
        } => format!(
            "removeModifier:{op_id}:{modifier_id}:{:?}",
            sorted(tags.iter().map(String::as_str))
        ),
        GameRuleEffectOp::SchedulePeriodicEffect {
            op_id,
            modifier_id,
            cadence,
            duration,
            tags,
        } => format!(
            "schedule:{op_id}:{modifier_id}:{}:{}:{:?}",
            cadence.period_ticks,
            duration_fingerprint(duration),
            sorted(tags.iter().map(String::as_str))
        ),
        GameRuleEffectOp::CancelResolution {
            op_id,
            reason,
            tags,
        } => format!(
            "cancel:{op_id}:{reason}:{:?}",
            sorted(tags.iter().map(String::as_str))
        ),
        GameRuleEffectOp::EmitTrace {
            op_id,
            code,
            message,
            tags,
        } => format!(
            "trace:{op_id}:{code}:{message}:{:?}",
            sorted(tags.iter().map(String::as_str))
        ),
    }
}

fn modifier_fingerprint(modifier: &GameRuleModifierDefinition) -> String {
    format!(
        "{}:{}:{}:{:?}:{:?}:{:?}:{}",
        modifier.modifier_id,
        stack_fingerprint(&modifier.stack_policy),
        duration_fingerprint(&modifier.duration),
        modifier.tick_cadence.as_ref().map(|c| c.period_ticks),
        sorted(modifier.tags.iter().map(String::as_str)),
        sorted(modifier.effect_op_ids.iter().map(String::as_str)),
        modifier.source_hash
    )
}

fn stack_fingerprint(policy: &GameRuleStackPolicy) -> String {
    match policy {
        GameRuleStackPolicy::Refresh => "refresh".to_string(),
        GameRuleStackPolicy::Stack { max_stacks } => format!("stack:{max_stacks}"),
        GameRuleStackPolicy::RejectDuplicate => "rejectDuplicate".to_string(),
        GameRuleStackPolicy::ReplaceIfStronger => "replaceIfStronger".to_string(),
    }
}

fn duration_fingerprint(duration: &GameRuleDuration) -> String {
    match duration {
        GameRuleDuration::Instant => "instant".to_string(),
        GameRuleDuration::Ticks { ticks } => format!("ticks:{ticks}"),
        GameRuleDuration::Infinite => "infinite".to_string(),
    }
}

fn sorted<'a>(values: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut values = values.collect::<Vec<_>>();
    values.sort_unstable();
    values
}

fn sorted_values(values: &[GameRuleBoundedValue]) -> Vec<&GameRuleBoundedValue> {
    let mut values = values.iter().collect::<Vec<_>>();
    values.sort_by(|a, b| a.channel_id.cmp(&b.channel_id));
    values
}

fn stable_hash(parts: &[String]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for part in parts {
        feed_u64(&mut hash, part.len() as u64);
        for byte in part.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("fnv1a64:{hash:016x}")
}

fn feed_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol_game_rules::{GameRuleEffectBundle, GameRuleValueChannelRef};

    fn catalog_ref() -> GameRuleCatalogRef {
        GameRuleCatalogRef {
            catalog_id: "catalog.game-rules.demo".to_string(),
            version: "0.1.0".to_string(),
            content_hash: "fnv1a64:catalog".to_string(),
        }
    }

    fn poisoned_catalog() -> GameRuleCatalog {
        GameRuleCatalog {
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
                    GameRuleEffectOp::ApplyDelta {
                        op_id: "op.poison-tick".to_string(),
                        channel_id: "value.health".to_string(),
                        amount: -2,
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
                    source_hash: "fnv1a64:modifier-poison".to_string(),
                }],
                tags: vec!["tag.poison".to_string()],
                source_hash: "fnv1a64:bundle".to_string(),
            }],
        }
    }

    fn request() -> EffectResolutionRequest {
        EffectResolutionRequest {
            catalog: catalog_ref(),
            bundle_id: "bundle.poisoned-impact".to_string(),
            source: EntityId::new(1),
            target: EntityId::new(2),
            values: vec![GameRuleBoundedValue {
                channel_id: "value.health".to_string(),
                min: 0,
                current: 20,
                max: 20,
            }],
            incoming_tags: vec!["tag.hit".to_string()],
            tick: 12,
        }
    }

    #[test]
    fn catalog_validation_accepts_fixture_and_rejects_bad_references() {
        let valid = validate_catalog(&poisoned_catalog());
        assert!(valid.accepted(), "{:?}", valid.diagnostics);

        let mut invalid = poisoned_catalog();
        invalid.bundles[0].effect_ops[0] = GameRuleEffectOp::Restore {
            op_id: "op.bad".to_string(),
            channel_id: "value.mana".to_string(),
            amount: 0,
            tags: vec![],
        };
        invalid.bundles[0].modifiers[0].effect_op_ids = vec!["op.missing".to_string()];
        let report = validate_catalog(&invalid);
        let codes = report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>();
        assert!(codes.contains(&GameRuleDiagnosticCode::InvalidAmount));
        assert!(codes.contains(&GameRuleDiagnosticCode::UndeclaredValueChannel));
        assert!(codes.contains(&GameRuleDiagnosticCode::UnknownEffectOp));
    }

    #[test]
    fn resolving_effects_returns_pending_receipt_without_mutating_input_facts() {
        let catalog = poisoned_catalog();
        let request = request();
        let original_values = request.values.clone();

        let receipt = resolve_effect_request(&request, &catalog);

        assert!(receipt.accepted, "{:?}", receipt.diagnostics);
        assert_eq!(request.values, original_values);
        assert_eq!(
            receipt.pending_value_deltas,
            vec![GameRuleValueDelta {
                channel_id: "value.health".to_string(),
                amount: -7,
            }]
        );
        assert_eq!(receipt.applied_modifiers.len(), 1);
        assert_eq!(receipt.applied_modifiers[0].next_tick, Some(15));
        assert_eq!(receipt.applied_modifiers[0].expires_tick, Some(21));
    }

    #[test]
    fn poisoned_impact_fixture_records_damage_modifier_and_scheduled_tick_trace() {
        let receipt = resolve_effect_request(&request(), &poisoned_catalog());
        let trace_codes = receipt
            .trace
            .iter()
            .map(|entry| entry.code.as_str())
            .collect::<Vec<_>>();

        assert!(trace_codes.contains(&"value.deltaPending"));
        assert!(trace_codes.contains(&"modifier.periodicScheduled"));
        assert!(trace_codes.contains(&"resolution.accepted"));
        assert_eq!(receipt.applied_modifiers[0].modifier_id, "modifier.poison");
        assert_eq!(receipt.applied_modifiers[0].applied_tick, 12);
    }

    #[test]
    fn replay_hash_is_stable_across_ordered_input_facts() {
        let catalog = poisoned_catalog();
        let mut a = request();
        a.values.push(GameRuleBoundedValue {
            channel_id: "value.health".to_string(),
            min: 0,
            current: 20,
            max: 20,
        });
        let mut b = request();
        b.incoming_tags = vec!["tag.z".to_string(), "tag.a".to_string()];
        let mut c = b.clone();
        c.incoming_tags = vec!["tag.a".to_string(), "tag.z".to_string()];

        let rejected = resolve_effect_request(&a, &catalog);
        assert!(!rejected.accepted);

        let left = resolve_effect_request(&b, &catalog);
        let right = resolve_effect_request(&c, &catalog);
        assert_eq!(left.request_hash, right.request_hash);
        assert_eq!(left.replay_hash, right.replay_hash);
    }

    #[test]
    fn cyclic_periodic_schedule_is_rejected() {
        let mut catalog = poisoned_catalog();
        catalog.bundles[0].modifiers[0].effect_op_ids = vec!["op.schedule-poison".to_string()];

        let report = validate_catalog(&catalog);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == GameRuleDiagnosticCode::CyclicPeriodicSchedule
        }));
    }
}

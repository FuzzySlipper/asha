//! Deterministic reaction-window validation and resolution.
//!
//! The service consumes explicit reaction definitions and one explicit pending
//! window input. Definitions are called directly by the resolver.

use std::collections::BTreeSet;

use core_game_rules::{
    EffectOpId, GameRuleTraceEntry, ModifierId, ReactionBehavior, ReactionDefinition,
    ReactionWindowKind, ValueChannelId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionValidationReport {
    pub accepted: bool,
    pub diagnostics: Vec<ReactionDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionDiagnostic {
    pub code: ReactionDiagnosticCode,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactionDiagnosticCode {
    UnsupportedWindow,
    UndeclaredRead,
    DisallowedEffectOp,
    DuplicateReaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionResolutionInput {
    pub window: ReactionWindowKind,
    pub channel: Option<ValueChannelId>,
    pub pending_delta: i64,
    pub declared_reads: BTreeSet<ValueChannelId>,
    pub allowed_effect_ops: BTreeSet<EffectOpId>,
    pub allowed_modifiers: BTreeSet<ModifierId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionResolution {
    pub accepted: bool,
    pub pending_delta: i64,
    pub canceled: bool,
    pub cancel_reason: Option<String>,
    pub follow_up_effect_ops: Vec<EffectOpId>,
    pub follow_up_modifiers: Vec<ModifierId>,
    pub trace: Vec<GameRuleTraceEntry>,
    pub reaction_hash: String,
}

pub fn validate_reactions(
    definitions: &[ReactionDefinition],
    supported_windows: &BTreeSet<ReactionWindowKind>,
    declared_reads: &BTreeSet<ValueChannelId>,
    allowed_effect_ops: &BTreeSet<EffectOpId>,
) -> ReactionValidationReport {
    let mut diagnostics = Vec::new();
    let mut seen = BTreeSet::new();

    for (index, definition) in definitions.iter().enumerate() {
        let path = format!("reactions[{index}]");
        if !seen.insert(definition.id.clone()) {
            diagnostics.push(diagnostic(
                ReactionDiagnosticCode::DuplicateReaction,
                &path,
                "duplicate reaction id",
            ));
        }
        if !supported_windows.contains(&definition.window) {
            diagnostics.push(diagnostic(
                ReactionDiagnosticCode::UnsupportedWindow,
                format!("{path}.window"),
                "reaction uses unsupported explicit window",
            ));
        }
        for read in &definition.reads {
            if !declared_reads.contains(read) {
                diagnostics.push(diagnostic(
                    ReactionDiagnosticCode::UndeclaredRead,
                    format!("{path}.reads"),
                    format!("reaction reads undeclared fact `{read}`"),
                ));
            }
        }
        if let ReactionBehavior::ApplyFollowUpEffect { effect_op } = &definition.behavior {
            if !allowed_effect_ops.contains(effect_op) {
                diagnostics.push(diagnostic(
                    ReactionDiagnosticCode::DisallowedEffectOp,
                    format!("{path}.behavior.effectOp"),
                    format!("reaction proposes disallowed effect op `{effect_op}`"),
                ));
            }
        }
    }

    ReactionValidationReport {
        accepted: diagnostics.is_empty(),
        diagnostics,
    }
}

pub fn resolve_reactions(
    definitions: &[ReactionDefinition],
    input: &ReactionResolutionInput,
) -> ReactionResolution {
    let supported_windows = BTreeSet::from([input.window]);
    let validation = validate_reactions(
        definitions,
        &supported_windows,
        &input.declared_reads,
        &input.allowed_effect_ops,
    );
    if !validation.accepted {
        return ReactionResolution {
            accepted: false,
            pending_delta: input.pending_delta,
            canceled: false,
            cancel_reason: None,
            follow_up_effect_ops: Vec::new(),
            follow_up_modifiers: Vec::new(),
            trace: vec![trace("reaction.rejected", "reaction validation failed")
                .with_ref("diagnostics", validation.diagnostics.len().to_string())],
            reaction_hash: stable_hash(&["rejected".to_string(), input.pending_delta.to_string()]),
        };
    }

    let mut ordered = definitions
        .iter()
        .filter(|definition| definition.window == input.window)
        .collect::<Vec<_>>();
    ordered.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.id.as_str().cmp(b.id.as_str()))
    });

    let mut pending_delta = input.pending_delta;
    let mut canceled = false;
    let mut cancel_reason = None;
    let mut follow_up_effect_ops = Vec::new();
    let mut follow_up_modifiers = Vec::new();
    let mut trace_entries = Vec::new();

    for definition in ordered {
        if canceled {
            break;
        }
        match &definition.behavior {
            ReactionBehavior::ModifyPendingDelta { channel, amount } => {
                if input
                    .channel
                    .as_ref()
                    .is_none_or(|current| current == channel)
                {
                    pending_delta = pending_delta.saturating_add(*amount);
                    trace_entries.push(
                        trace("reaction.deltaModified", "reaction modified pending delta")
                            .with_ref("reaction", definition.id.as_str())
                            .with_ref("amount", amount.to_string())
                            .with_ref("pendingDelta", pending_delta.to_string()),
                    );
                }
            }
            ReactionBehavior::CancelPendingEffect { reason } => {
                canceled = true;
                cancel_reason = Some(reason.clone());
                trace_entries.push(
                    trace("reaction.canceled", "reaction canceled pending effect")
                        .with_ref("reaction", definition.id.as_str())
                        .with_ref("reason", reason),
                );
            }
            ReactionBehavior::ApplyFollowUpEffect { effect_op } => {
                follow_up_effect_ops.push(effect_op.clone());
                trace_entries.push(
                    trace(
                        "reaction.followUpEffect",
                        "reaction proposed follow-up effect",
                    )
                    .with_ref("reaction", definition.id.as_str())
                    .with_ref("effectOp", effect_op.as_str()),
                );
            }
            ReactionBehavior::ApplyFollowUpModifier { modifier } => {
                if input.allowed_modifiers.contains(modifier) {
                    follow_up_modifiers.push(modifier.clone());
                    trace_entries.push(
                        trace(
                            "reaction.followUpModifier",
                            "reaction proposed follow-up modifier",
                        )
                        .with_ref("reaction", definition.id.as_str())
                        .with_ref("modifier", modifier.as_str()),
                    );
                }
            }
            ReactionBehavior::EmitTrace { code, message } => {
                trace_entries
                    .push(trace(code, message).with_ref("reaction", definition.id.as_str()));
            }
        }
    }

    let mut parts = vec![
        input.window.label().to_string(),
        input.pending_delta.to_string(),
        pending_delta.to_string(),
        canceled.to_string(),
    ];
    parts.extend(trace_entries.iter().map(|entry| entry.code.clone()));

    ReactionResolution {
        accepted: true,
        pending_delta,
        canceled,
        cancel_reason,
        follow_up_effect_ops,
        follow_up_modifiers,
        trace: trace_entries,
        reaction_hash: stable_hash(&parts),
    }
}

fn diagnostic(
    code: ReactionDiagnosticCode,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ReactionDiagnostic {
    ReactionDiagnostic {
        code,
        path: path.into(),
        message: message.into(),
    }
}

fn trace(code: impl Into<String>, message: impl Into<String>) -> GameRuleTraceEntry {
    GameRuleTraceEntry::new(0, code, message)
}

fn stable_hash(parts: &[String]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for part in parts {
        for byte in (part.len() as u64).to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        for byte in part.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_game_rules::ReactionWindowId;

    fn rid(value: &str) -> ReactionWindowId {
        ReactionWindowId::parse(value).unwrap()
    }

    fn channel(value: &str) -> ValueChannelId {
        ValueChannelId::parse(value).unwrap()
    }

    fn op(value: &str) -> EffectOpId {
        EffectOpId::parse(value).unwrap()
    }

    fn modifier(value: &str) -> ModifierId {
        ModifierId::parse(value).unwrap()
    }

    #[test]
    fn validation_rejects_unsupported_windows_and_disallowed_ops() {
        let definitions = vec![ReactionDefinition::new(
            rid("reaction.poison-proc"),
            ReactionWindowKind::PeriodicTick,
            ReactionBehavior::ApplyFollowUpEffect {
                effect_op: op("op.poison"),
            },
        )
        .with_reads([channel("value.health")])];
        let report = validate_reactions(
            &definitions,
            &BTreeSet::from([ReactionWindowKind::PendingValueDelta]),
            &BTreeSet::new(),
            &BTreeSet::new(),
        );

        assert!(!report.accepted);
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == ReactionDiagnosticCode::UnsupportedWindow));
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == ReactionDiagnosticCode::UndeclaredRead));
        assert!(report
            .diagnostics
            .iter()
            .any(|d| d.code == ReactionDiagnosticCode::DisallowedEffectOp));
    }

    #[test]
    fn deterministic_ordering_uses_priority_then_stable_id() {
        let health = channel("value.health");
        let definitions = vec![
            ReactionDefinition::new(
                rid("reaction.z"),
                ReactionWindowKind::PendingValueDelta,
                ReactionBehavior::ModifyPendingDelta {
                    channel: health.clone(),
                    amount: 1,
                },
            )
            .with_priority(1),
            ReactionDefinition::new(
                rid("reaction.a"),
                ReactionWindowKind::PendingValueDelta,
                ReactionBehavior::EmitTrace {
                    code: "reaction.trace".to_string(),
                    message: "tie breaker".to_string(),
                },
            )
            .with_priority(1),
            ReactionDefinition::new(
                rid("reaction.high"),
                ReactionWindowKind::PendingValueDelta,
                ReactionBehavior::ModifyPendingDelta {
                    channel: health.clone(),
                    amount: 3,
                },
            )
            .with_priority(10),
        ];
        let input = ReactionResolutionInput {
            window: ReactionWindowKind::PendingValueDelta,
            channel: Some(health.clone()),
            pending_delta: -10,
            declared_reads: BTreeSet::from([health]),
            allowed_effect_ops: BTreeSet::new(),
            allowed_modifiers: BTreeSet::new(),
        };
        let resolved = resolve_reactions(&definitions, &input);

        assert!(resolved.accepted);
        assert_eq!(resolved.pending_delta, -6);
        assert_eq!(resolved.trace[0].refs[0].1, "3");
        assert_eq!(resolved.trace[1].refs[0].1, "reaction.a");
    }

    #[test]
    fn shield_absorb_modifies_pending_damage_before_commit() {
        let health = channel("value.health");
        let definitions = vec![ReactionDefinition::new(
            rid("reaction.shield-absorb"),
            ReactionWindowKind::PendingValueDelta,
            ReactionBehavior::ModifyPendingDelta {
                channel: health.clone(),
                amount: 6,
            },
        )
        .with_priority(5)
        .with_reads([health.clone()])
        .with_source_hash("fnv1a64:shield")];
        let input = ReactionResolutionInput {
            window: ReactionWindowKind::PendingValueDelta,
            channel: Some(health.clone()),
            pending_delta: -10,
            declared_reads: BTreeSet::from([health]),
            allowed_effect_ops: BTreeSet::new(),
            allowed_modifiers: BTreeSet::new(),
        };

        let resolved = resolve_reactions(&definitions, &input);

        assert!(resolved.accepted);
        assert_eq!(resolved.pending_delta, -4);
        assert!(resolved
            .trace
            .iter()
            .any(|entry| entry.code == "reaction.deltaModified"));
    }

    #[test]
    fn cancel_and_follow_up_paths_are_explicit() {
        let poison = modifier("modifier.poison");
        let definitions = vec![
            ReactionDefinition::new(
                rid("reaction.parry"),
                ReactionWindowKind::AcceptedHit,
                ReactionBehavior::CancelPendingEffect {
                    reason: "blocked".to_string(),
                },
            )
            .with_priority(10),
            ReactionDefinition::new(
                rid("reaction.poison-on-hit"),
                ReactionWindowKind::AcceptedHit,
                ReactionBehavior::ApplyFollowUpModifier {
                    modifier: poison.clone(),
                },
            )
            .with_priority(1),
        ];
        let input = ReactionResolutionInput {
            window: ReactionWindowKind::AcceptedHit,
            channel: None,
            pending_delta: 0,
            declared_reads: BTreeSet::new(),
            allowed_effect_ops: BTreeSet::new(),
            allowed_modifiers: BTreeSet::from([poison]),
        };
        let resolved = resolve_reactions(&definitions, &input);

        assert!(resolved.canceled);
        assert_eq!(resolved.cancel_reason.as_deref(), Some("blocked"));
        assert!(resolved.follow_up_modifiers.is_empty());
    }

    #[test]
    fn public_reaction_terms_are_genre_neutral() {
        let public_terms = [
            ReactionWindowKind::AcceptedHit.label(),
            ReactionWindowKind::PendingValueDelta.label(),
            ReactionWindowKind::ModifierApplication.label(),
            ReactionWindowKind::ValueDepleted.label(),
            ReactionWindowKind::PeriodicTick.label(),
            ReactionWindowKind::ResolutionCancelled.label(),
            "ReactionResolutionInput",
            "ReactionDefinition",
            "ReactionBehavior",
        ]
        .join(" ")
        .to_lowercase();
        let forbidden = [
            concat!("tu", "rn"),
            concat!("ro", "und"),
            concat!("init", "iative"),
            concat!("action", " economy"),
            concat!("saving", " throw"),
        ];
        for forbidden in forbidden {
            assert!(
                !public_terms.contains(forbidden),
                "{forbidden} leaked into reaction service API"
            );
        }
    }
}

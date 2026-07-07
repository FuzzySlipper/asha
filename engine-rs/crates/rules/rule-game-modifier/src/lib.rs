//! Authority rule for accepted game-rule modifier lifecycle.
//!
//! # Lane
//!
//! `rust-rule` - owns modifier apply/refresh/stack/tick/expire state for the
//! generic game-rules substrate. It consumes core game-rule definitions and
//! mutates only its own modifier/value authority tables. It has no renderer,
//! bridge, TypeScript, policy, or wall-clock dependency.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use core_game_rules::{
    BoundedValue, EffectDuration, EffectOp, EffectOpId, EffectOpKind, GameRuleTraceEntry,
    ModifierDefinition, ModifierId, StackPolicy, ValueChannelId, ValueDelta,
};
use core_ids::EntityId;
use core_time::Tick;

pub const AUTHORITY_VERSION: &str = "rule-game-modifier.v0";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModifierInstanceKey {
    pub target: EntityId,
    pub modifier: ModifierId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueFactKey {
    pub entity: EntityId,
    pub channel: ValueChannelId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierCatalog {
    modifiers: BTreeMap<ModifierId, ModifierDefinition>,
    effect_ops: BTreeMap<EffectOpId, EffectOp>,
}

impl ModifierCatalog {
    pub fn new(
        modifiers: impl IntoIterator<Item = ModifierDefinition>,
        effect_ops: impl IntoIterator<Item = EffectOp>,
    ) -> Self {
        Self {
            modifiers: modifiers
                .into_iter()
                .map(|modifier| (modifier.id.clone(), modifier))
                .collect(),
            effect_ops: effect_ops
                .into_iter()
                .map(|op| (op.id.clone(), op))
                .collect(),
        }
    }

    pub fn modifier(&self, id: &ModifierId) -> Option<&ModifierDefinition> {
        self.modifiers.get(id)
    }

    pub fn effect_op(&self, id: &EffectOpId) -> Option<&EffectOp> {
        self.effect_ops.get(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveModifier {
    pub modifier: ModifierId,
    pub source: EntityId,
    pub target: EntityId,
    pub stacks: u32,
    pub applied_at: Tick,
    pub expires_at: Option<Tick>,
    pub next_tick: Option<Tick>,
    pub source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierRuleEvent {
    ModifierApplied {
        modifier: ModifierId,
        source: EntityId,
        target: EntityId,
        stacks: u32,
        next_tick: Option<Tick>,
        expires_at: Option<Tick>,
    },
    ModifierRefreshed {
        modifier: ModifierId,
        target: EntityId,
        next_tick: Option<Tick>,
        expires_at: Option<Tick>,
    },
    ModifierStacked {
        modifier: ModifierId,
        target: EntityId,
        stacks: u32,
    },
    ModifierRejected {
        modifier: ModifierId,
        target: EntityId,
        reason: ModifierRuleRejection,
    },
    PeriodicEffectTicked {
        modifier: ModifierId,
        target: EntityId,
        tick: Tick,
    },
    ValueDeltaApplied {
        modifier: ModifierId,
        target: EntityId,
        channel: ValueChannelId,
        amount: i64,
        before: i64,
        after: i64,
    },
    ModifierExpired {
        modifier: ModifierId,
        target: EntityId,
        tick: Tick,
    },
    ModifierRemoved {
        modifier: ModifierId,
        target: EntityId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierRuleRejection {
    UnknownModifier,
    DuplicateRejected,
    StackLimitReached,
    MissingValueFact,
    UnsupportedTickOp,
    InvalidDefinition,
}

impl ModifierRuleRejection {
    pub const fn label(self) -> &'static str {
        match self {
            ModifierRuleRejection::UnknownModifier => "unknownModifier",
            ModifierRuleRejection::DuplicateRejected => "duplicateRejected",
            ModifierRuleRejection::StackLimitReached => "stackLimitReached",
            ModifierRuleRejection::MissingValueFact => "missingValueFact",
            ModifierRuleRejection::UnsupportedTickOp => "unsupportedTickOp",
            ModifierRuleRejection::InvalidDefinition => "invalidDefinition",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierRuleReceipt {
    pub accepted: bool,
    pub rejection: Option<ModifierRuleRejection>,
    pub events: Vec<ModifierRuleEvent>,
    pub trace: Vec<GameRuleTraceEntry>,
    pub active_hash: String,
    pub replay_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameModifierRuleState {
    catalog: ModifierCatalog,
    active: BTreeMap<ModifierInstanceKey, ActiveModifier>,
}

impl GameModifierRuleState {
    pub fn new(catalog: ModifierCatalog) -> Self {
        Self {
            catalog,
            active: BTreeMap::new(),
        }
    }

    pub fn active(&self) -> &BTreeMap<ModifierInstanceKey, ActiveModifier> {
        &self.active
    }

    pub fn apply_modifier(
        &mut self,
        modifier: &ModifierId,
        source: EntityId,
        target: EntityId,
        tick: Tick,
    ) -> ModifierRuleReceipt {
        let Some(definition) = self.catalog.modifier(modifier).cloned() else {
            return rejected(
                ModifierRuleRejection::UnknownModifier,
                Vec::new(),
                trace("modifier.rejected", "unknown modifier")
                    .with_ref("modifier", modifier.as_str()),
                self.active_hash(),
            );
        };
        if definition.tick_cadence.is_some() && definition.effect_ops.is_empty() {
            return rejected(
                ModifierRuleRejection::InvalidDefinition,
                Vec::new(),
                trace(
                    "modifier.rejected",
                    "periodic modifier has no tick effect ops",
                )
                .with_ref("modifier", modifier.as_str()),
                self.active_hash(),
            );
        }

        let key = ModifierInstanceKey {
            target,
            modifier: modifier.clone(),
        };
        let mut events = Vec::new();
        let mut traces = Vec::new();

        if let Some(existing) = self.active.get_mut(&key) {
            match definition.stack_policy {
                StackPolicy::Refresh => {
                    refresh(existing, &definition, source, tick);
                    events.push(ModifierRuleEvent::ModifierRefreshed {
                        modifier: modifier.clone(),
                        target,
                        next_tick: existing.next_tick,
                        expires_at: existing.expires_at,
                    });
                    traces.push(
                        trace("modifier.refreshed", "modifier refreshed")
                            .with_ref("modifier", modifier.as_str()),
                    );
                }
                StackPolicy::Stack { max_stacks } => {
                    if existing.stacks >= max_stacks {
                        events.push(ModifierRuleEvent::ModifierRejected {
                            modifier: modifier.clone(),
                            target,
                            reason: ModifierRuleRejection::StackLimitReached,
                        });
                        return rejected(
                            ModifierRuleRejection::StackLimitReached,
                            events,
                            trace("modifier.rejected", "stack limit reached")
                                .with_ref("modifier", modifier.as_str()),
                            self.active_hash(),
                        );
                    }
                    existing.stacks += 1;
                    refresh(existing, &definition, source, tick);
                    events.push(ModifierRuleEvent::ModifierStacked {
                        modifier: modifier.clone(),
                        target,
                        stacks: existing.stacks,
                    });
                    traces.push(
                        trace("modifier.stacked", "modifier stack increased")
                            .with_ref("stacks", existing.stacks.to_string()),
                    );
                }
                StackPolicy::RejectDuplicate => {
                    events.push(ModifierRuleEvent::ModifierRejected {
                        modifier: modifier.clone(),
                        target,
                        reason: ModifierRuleRejection::DuplicateRejected,
                    });
                    return rejected(
                        ModifierRuleRejection::DuplicateRejected,
                        events,
                        trace("modifier.rejected", "duplicate modifier rejected")
                            .with_ref("modifier", modifier.as_str()),
                        self.active_hash(),
                    );
                }
                StackPolicy::ReplaceIfStronger => {
                    if definition.source_hash >= existing.source_hash {
                        *existing = active_from_definition(&definition, source, target, tick);
                        events.push(ModifierRuleEvent::ModifierRefreshed {
                            modifier: modifier.clone(),
                            target,
                            next_tick: existing.next_tick,
                            expires_at: existing.expires_at,
                        });
                        traces.push(
                            trace(
                                "modifier.replaced",
                                "modifier replaced by stronger definition",
                            )
                            .with_ref("modifier", modifier.as_str()),
                        );
                    } else {
                        events.push(ModifierRuleEvent::ModifierRejected {
                            modifier: modifier.clone(),
                            target,
                            reason: ModifierRuleRejection::DuplicateRejected,
                        });
                        return rejected(
                            ModifierRuleRejection::DuplicateRejected,
                            events,
                            trace("modifier.rejected", "weaker replacement rejected")
                                .with_ref("modifier", modifier.as_str()),
                            self.active_hash(),
                        );
                    }
                }
            }
        } else {
            let active = active_from_definition(&definition, source, target, tick);
            events.push(ModifierRuleEvent::ModifierApplied {
                modifier: modifier.clone(),
                source,
                target,
                stacks: active.stacks,
                next_tick: active.next_tick,
                expires_at: active.expires_at,
            });
            traces.push(
                trace("modifier.applied", "modifier applied")
                    .with_ref("modifier", modifier.as_str())
                    .with_ref("target", target.raw().to_string()),
            );
            self.active.insert(key, active);
        }

        self.accepted(events, traces)
    }

    pub fn tick(
        &mut self,
        tick: Tick,
        values: &mut BTreeMap<ValueFactKey, BoundedValue>,
    ) -> ModifierRuleReceipt {
        let mut next_active = self.active.clone();
        let mut next_values = values.clone();
        let mut events = Vec::new();
        let mut traces = Vec::new();

        for (key, active) in &self.active {
            if active
                .expires_at
                .is_some_and(|expires_at| tick >= expires_at)
            {
                next_active.remove(key);
                events.push(ModifierRuleEvent::ModifierExpired {
                    modifier: key.modifier.clone(),
                    target: key.target,
                    tick,
                });
                traces.push(
                    trace("modifier.expired", "modifier expired")
                        .with_ref("modifier", key.modifier.as_str())
                        .with_ref("tick", tick.raw().to_string()),
                );
                continue;
            }

            let Some(next_tick) = active.next_tick else {
                continue;
            };
            if tick < next_tick {
                continue;
            }

            let Some(definition) = self.catalog.modifier(&key.modifier) else {
                return rejected(
                    ModifierRuleRejection::UnknownModifier,
                    events,
                    trace("modifier.rejected", "active modifier missing definition")
                        .with_ref("modifier", key.modifier.as_str()),
                    self.active_hash(),
                );
            };

            for op_id in &definition.effect_ops {
                let Some(op) = self.catalog.effect_op(op_id) else {
                    return rejected(
                        ModifierRuleRejection::UnsupportedTickOp,
                        events,
                        trace("modifier.rejected", "missing modifier tick effect op")
                            .with_ref("op", op_id.as_str()),
                        self.active_hash(),
                    );
                };
                if let Err(receipt) =
                    apply_tick_op(active, op, &mut next_values, &mut events, &mut traces)
                {
                    return receipt;
                }
            }

            events.push(ModifierRuleEvent::PeriodicEffectTicked {
                modifier: key.modifier.clone(),
                target: key.target,
                tick,
            });
            traces.push(
                trace("modifier.ticked", "periodic modifier tick accepted")
                    .with_ref("modifier", key.modifier.as_str())
                    .with_ref("tick", tick.raw().to_string()),
            );

            if let Some(updated) = next_active.get_mut(key) {
                updated.next_tick = definition
                    .tick_cadence
                    .map(|cadence| tick.advance(cadence.period()));
            }
        }

        self.active = next_active;
        *values = next_values;
        self.accepted(events, traces)
    }

    pub fn remove_modifier(
        &mut self,
        modifier: &ModifierId,
        target: EntityId,
    ) -> ModifierRuleReceipt {
        let key = ModifierInstanceKey {
            target,
            modifier: modifier.clone(),
        };
        let removed = self.active.remove(&key);
        if removed.is_none() {
            return rejected(
                ModifierRuleRejection::UnknownModifier,
                Vec::new(),
                trace("modifier.rejected", "active modifier not found")
                    .with_ref("modifier", modifier.as_str()),
                self.active_hash(),
            );
        }
        self.accepted(
            vec![ModifierRuleEvent::ModifierRemoved {
                modifier: modifier.clone(),
                target,
            }],
            vec![trace("modifier.removed", "modifier removed")
                .with_ref("modifier", modifier.as_str())],
        )
    }

    pub fn active_hash(&self) -> String {
        let mut parts = vec!["active".to_string(), AUTHORITY_VERSION.to_string()];
        for (key, active) in &self.active {
            parts.push(format!(
                "{}:{}:{}:{}:{:?}:{:?}:{}",
                key.target.raw(),
                key.modifier,
                active.source.raw(),
                active.stacks,
                active.expires_at.map(Tick::raw),
                active.next_tick.map(Tick::raw),
                active.source_hash
            ));
        }
        stable_hash(&parts)
    }

    fn accepted(
        &self,
        events: Vec<ModifierRuleEvent>,
        trace: Vec<GameRuleTraceEntry>,
    ) -> ModifierRuleReceipt {
        let active_hash = self.active_hash();
        ModifierRuleReceipt {
            accepted: true,
            rejection: None,
            replay_hash: replay_hash(true, None, &events, &trace, &active_hash),
            events,
            trace,
            active_hash,
        }
    }
}

fn apply_tick_op(
    active: &ActiveModifier,
    op: &EffectOp,
    values: &mut BTreeMap<ValueFactKey, BoundedValue>,
    events: &mut Vec<ModifierRuleEvent>,
    trace: &mut Vec<GameRuleTraceEntry>,
) -> Result<(), ModifierRuleReceipt> {
    let (channel, amount) = match &op.kind {
        EffectOpKind::ApplyDelta { value, delta } => (value, delta.amount),
        EffectOpKind::Restore { value, amount } | EffectOpKind::Grant { value, amount } => {
            (value, i64::from(amount.raw()))
        }
        EffectOpKind::Spend { value, amount } => (value, -i64::from(amount.raw())),
        EffectOpKind::EmitTrace { code, message } => {
            trace.push(
                new_trace(code.clone(), message.clone())
                    .with_ref("modifier", active.modifier.as_str())
                    .with_ref("target", active.target.raw().to_string()),
            );
            return Ok(());
        }
        _ => {
            return Err(rejected(
                ModifierRuleRejection::UnsupportedTickOp,
                events.clone(),
                new_trace("modifier.rejected", "unsupported modifier tick operation")
                    .with_ref("op", op.id.as_str()),
                String::new(),
            ));
        }
    };

    let key = ValueFactKey {
        entity: active.target,
        channel: channel.clone(),
    };
    let Some(current) = values.get(&key).copied() else {
        return Err(rejected(
            ModifierRuleRejection::MissingValueFact,
            events.clone(),
            new_trace("modifier.rejected", "missing value fact for modifier tick")
                .with_ref("channel", channel.as_str()),
            String::new(),
        ));
    };
    let applied = current.apply_delta(ValueDelta::new(amount * i64::from(active.stacks)));
    values.insert(key, applied.after);
    let actual = applied.after.current - applied.before.current;
    events.push(ModifierRuleEvent::ValueDeltaApplied {
        modifier: active.modifier.clone(),
        target: active.target,
        channel: channel.clone(),
        amount: actual,
        before: applied.before.current,
        after: applied.after.current,
    });
    trace.push(
        new_trace("value.deltaApplied", "modifier tick applied value delta")
            .with_ref("modifier", active.modifier.as_str())
            .with_ref("channel", channel.as_str())
            .with_ref("amount", actual.to_string()),
    );
    Ok(())
}

fn active_from_definition(
    definition: &ModifierDefinition,
    source: EntityId,
    target: EntityId,
    tick: Tick,
) -> ActiveModifier {
    ActiveModifier {
        modifier: definition.id.clone(),
        source,
        target,
        stacks: 1,
        applied_at: tick,
        expires_at: expires_at(definition.duration, tick),
        next_tick: definition
            .tick_cadence
            .map(|cadence| cadence.next_after(tick)),
        source_hash: definition.source_hash.clone(),
    }
}

fn refresh(
    active: &mut ActiveModifier,
    definition: &ModifierDefinition,
    source: EntityId,
    tick: Tick,
) {
    active.source = source;
    active.applied_at = tick;
    active.expires_at = expires_at(definition.duration, tick);
    active.next_tick = definition
        .tick_cadence
        .map(|cadence| cadence.next_after(tick));
    active.source_hash = definition.source_hash.clone();
}

fn expires_at(duration: EffectDuration, tick: Tick) -> Option<Tick> {
    match duration {
        EffectDuration::Instant => Some(tick),
        EffectDuration::Ticks(delta) => Some(tick.advance(delta)),
        EffectDuration::Infinite => None,
    }
}

fn rejected(
    rejection: ModifierRuleRejection,
    events: Vec<ModifierRuleEvent>,
    trace: GameRuleTraceEntry,
    active_hash: String,
) -> ModifierRuleReceipt {
    let trace = vec![trace];
    ModifierRuleReceipt {
        accepted: false,
        rejection: Some(rejection),
        replay_hash: replay_hash(false, Some(rejection), &events, &trace, &active_hash),
        events,
        trace,
        active_hash,
    }
}

fn trace(code: &str, message: &str) -> GameRuleTraceEntry {
    new_trace(code, message)
}

fn new_trace(code: impl Into<String>, message: impl Into<String>) -> GameRuleTraceEntry {
    GameRuleTraceEntry::new(0, code, message)
}

fn replay_hash(
    accepted: bool,
    rejection: Option<ModifierRuleRejection>,
    events: &[ModifierRuleEvent],
    trace: &[GameRuleTraceEntry],
    active_hash: &str,
) -> String {
    let mut parts = vec![
        "replay".to_string(),
        AUTHORITY_VERSION.to_string(),
        accepted.to_string(),
        active_hash.to_string(),
        rejection.map_or("none".to_string(), |r| r.label().to_string()),
    ];
    for event in events {
        parts.push(event_fingerprint(event));
    }
    for entry in trace {
        parts.push(format!("trace:{}:{}", entry.code, entry.message));
        for (key, value) in &entry.refs {
            parts.push(format!("ref:{key}:{value}"));
        }
    }
    stable_hash(&parts)
}

fn event_fingerprint(event: &ModifierRuleEvent) -> String {
    match event {
        ModifierRuleEvent::ModifierApplied {
            modifier,
            source,
            target,
            stacks,
            next_tick,
            expires_at,
        } => format!(
            "applied:{modifier}:{}:{}:{stacks}:{:?}:{:?}",
            source.raw(),
            target.raw(),
            next_tick.map(Tick::raw),
            expires_at.map(Tick::raw)
        ),
        ModifierRuleEvent::ModifierRefreshed {
            modifier,
            target,
            next_tick,
            expires_at,
        } => format!(
            "refreshed:{modifier}:{}:{:?}:{:?}",
            target.raw(),
            next_tick.map(Tick::raw),
            expires_at.map(Tick::raw)
        ),
        ModifierRuleEvent::ModifierStacked {
            modifier,
            target,
            stacks,
        } => format!("stacked:{modifier}:{}:{stacks}", target.raw()),
        ModifierRuleEvent::ModifierRejected {
            modifier,
            target,
            reason,
        } => format!("rejected:{modifier}:{}:{}", target.raw(), reason.label()),
        ModifierRuleEvent::PeriodicEffectTicked {
            modifier,
            target,
            tick,
        } => format!("ticked:{modifier}:{}:{}", target.raw(), tick.raw()),
        ModifierRuleEvent::ValueDeltaApplied {
            modifier,
            target,
            channel,
            amount,
            before,
            after,
        } => format!(
            "delta:{modifier}:{}:{channel}:{amount}:{before}:{after}",
            target.raw()
        ),
        ModifierRuleEvent::ModifierExpired {
            modifier,
            target,
            tick,
        } => format!("expired:{modifier}:{}:{}", target.raw(), tick.raw()),
        ModifierRuleEvent::ModifierRemoved { modifier, target } => {
            format!("removed:{modifier}:{}", target.raw())
        }
    }
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
    use core_game_rules::{EffectTagId, TickCadence, ValueChannelId};

    fn mid(id: &str) -> ModifierId {
        ModifierId::parse(id).unwrap()
    }

    fn oid(id: &str) -> EffectOpId {
        EffectOpId::parse(id).unwrap()
    }

    fn channel(id: &str) -> ValueChannelId {
        ValueChannelId::parse(id).unwrap()
    }

    fn poison_catalog(policy: StackPolicy, duration: u64) -> ModifierCatalog {
        let poison = mid("modifier.poison");
        let health = channel("value.health");
        ModifierCatalog::new(
            [
                ModifierDefinition::new(poison, policy, EffectDuration::ticks(duration).unwrap())
                    .with_tick_cadence(TickCadence::every(1).unwrap())
                    .with_effect_ops([oid("op.poison-tick")])
                    .with_tags([EffectTagId::parse("tag.poison").unwrap()])
                    .with_source_hash("fnv1a64:poison"),
            ],
            [EffectOp::new(
                oid("op.poison-tick"),
                EffectOpKind::ApplyDelta {
                    value: health,
                    delta: ValueDelta::new(-2),
                },
            )],
        )
    }

    fn values(current: i64) -> BTreeMap<ValueFactKey, BoundedValue> {
        BTreeMap::from([(
            ValueFactKey {
                entity: EntityId::new(2),
                channel: channel("value.health"),
            },
            BoundedValue::new(0, current, 20).unwrap(),
        )])
    }

    #[test]
    fn stack_policies_refresh_stack_and_reject_duplicates() {
        let source = EntityId::new(1);
        let target = EntityId::new(2);
        let modifier = mid("modifier.poison");

        let mut refresh_rule = GameModifierRuleState::new(poison_catalog(StackPolicy::Refresh, 5));
        assert!(
            refresh_rule
                .apply_modifier(&modifier, source, target, Tick::new(0))
                .accepted
        );
        let refreshed = refresh_rule.apply_modifier(&modifier, source, target, Tick::new(2));
        assert!(matches!(
            refreshed.events[0],
            ModifierRuleEvent::ModifierRefreshed { .. }
        ));
        assert_eq!(
            refresh_rule
                .active()
                .values()
                .next()
                .unwrap()
                .expires_at
                .unwrap(),
            Tick::new(7)
        );

        let mut stack_rule =
            GameModifierRuleState::new(poison_catalog(StackPolicy::Stack { max_stacks: 2 }, 5));
        assert!(
            stack_rule
                .apply_modifier(&modifier, source, target, Tick::new(0))
                .accepted
        );
        assert!(
            stack_rule
                .apply_modifier(&modifier, source, target, Tick::new(1))
                .accepted
        );
        let capped = stack_rule.apply_modifier(&modifier, source, target, Tick::new(2));
        assert_eq!(
            capped.rejection,
            Some(ModifierRuleRejection::StackLimitReached)
        );

        let mut reject_rule =
            GameModifierRuleState::new(poison_catalog(StackPolicy::RejectDuplicate, 5));
        assert!(
            reject_rule
                .apply_modifier(&modifier, source, target, Tick::new(0))
                .accepted
        );
        let duplicate = reject_rule.apply_modifier(&modifier, source, target, Tick::new(1));
        assert_eq!(
            duplicate.rejection,
            Some(ModifierRuleRejection::DuplicateRejected)
        );
    }

    #[test]
    fn poison_modifier_ticks_in_order_and_stops_after_expiration() {
        let source = EntityId::new(1);
        let target = EntityId::new(2);
        let modifier = mid("modifier.poison");
        let mut rule = GameModifierRuleState::new(poison_catalog(StackPolicy::Refresh, 3));
        let mut values = values(10);

        assert!(
            rule.apply_modifier(&modifier, source, target, Tick::new(0))
                .accepted
        );

        let tick1 = rule.tick(Tick::new(1), &mut values);
        assert!(tick1.accepted);
        assert!(tick1.events.iter().any(|event| matches!(
            event,
            ModifierRuleEvent::ValueDeltaApplied { amount: -2, .. }
        )));
        assert_eq!(
            values[&ValueFactKey {
                entity: target,
                channel: channel("value.health")
            }]
                .current,
            8
        );

        assert!(rule.tick(Tick::new(2), &mut values).accepted);
        let tick3 = rule.tick(Tick::new(3), &mut values);
        assert!(tick3
            .events
            .iter()
            .any(|event| matches!(event, ModifierRuleEvent::ModifierExpired { .. })));
        assert!(rule.active().is_empty());
        assert_eq!(
            values[&ValueFactKey {
                entity: target,
                channel: channel("value.health")
            }]
                .current,
            6
        );
    }

    #[test]
    fn malformed_tick_operation_fails_closed_without_mutating_values() {
        let source = EntityId::new(1);
        let target = EntityId::new(2);
        let modifier = mid("modifier.poison");
        let catalog = ModifierCatalog::new(
            [ModifierDefinition::new(
                modifier.clone(),
                StackPolicy::Refresh,
                EffectDuration::ticks(5).unwrap(),
            )
            .with_tick_cadence(TickCadence::every(1).unwrap())
            .with_effect_ops([oid("op.bad")])
            .with_source_hash("fnv1a64:bad")],
            [EffectOp::new(
                oid("op.bad"),
                EffectOpKind::ApplyModifier {
                    modifier: mid("modifier.other"),
                },
            )],
        );
        let mut rule = GameModifierRuleState::new(catalog);
        let mut values = values(10);
        let before = values.clone();

        assert!(
            rule.apply_modifier(&modifier, source, target, Tick::new(0))
                .accepted
        );
        let receipt = rule.tick(Tick::new(1), &mut values);

        assert!(!receipt.accepted);
        assert_eq!(
            receipt.rejection,
            Some(ModifierRuleRejection::UnsupportedTickOp)
        );
        assert_eq!(values, before);
    }

    #[test]
    fn replay_hash_is_stable_for_identical_modifier_lifecycle() {
        let modifier = mid("modifier.poison");
        let run = || {
            let mut rule = GameModifierRuleState::new(poison_catalog(StackPolicy::Refresh, 3));
            let mut values = values(10);
            let mut hashes = Vec::new();
            hashes.push(
                rule.apply_modifier(&modifier, EntityId::new(1), EntityId::new(2), Tick::new(0))
                    .replay_hash,
            );
            hashes.push(rule.tick(Tick::new(1), &mut values).replay_hash);
            hashes.push(rule.tick(Tick::new(2), &mut values).replay_hash);
            hashes.push(rule.tick(Tick::new(3), &mut values).replay_hash);
            (hashes, rule.active_hash(), values)
        };

        let left = run();
        let right = run();
        assert_eq!(left, right);
    }

    #[test]
    fn replace_if_stronger_uses_source_hash_ordering() {
        let modifier = mid("modifier.poison");
        let make = |hash: &str| {
            ModifierDefinition::new(
                modifier.clone(),
                StackPolicy::ReplaceIfStronger,
                EffectDuration::ticks(5).unwrap(),
            )
            .with_tick_cadence(TickCadence::every(1).unwrap())
            .with_effect_ops([oid("op.poison-tick")])
            .with_source_hash(hash)
        };
        let health = channel("value.health");
        let mut rule = GameModifierRuleState::new(ModifierCatalog::new(
            [make("fnv1a64:b")],
            [EffectOp::new(
                oid("op.poison-tick"),
                EffectOpKind::ApplyDelta {
                    value: health,
                    delta: ValueDelta::new(-1),
                },
            )],
        ));
        assert!(
            rule.apply_modifier(&modifier, EntityId::new(1), EntityId::new(2), Tick::new(0))
                .accepted
        );
        assert!(
            rule.apply_modifier(&modifier, EntityId::new(1), EntityId::new(2), Tick::new(1))
                .accepted
        );
    }
}

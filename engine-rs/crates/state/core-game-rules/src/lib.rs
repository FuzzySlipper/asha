//! Core vocabulary for generic ASHA game-rule effects.
//!
//! # Lane
//!
//! `rust-state` - shared ids, bounded values, effect/modifier/timing shapes,
//! and deterministic trace helpers. This crate owns no RuntimeSession,
//! renderer, bridge, TypeScript, or authority mutation path.
//!
//! # Design
//!
//! The types here are inert data vocabulary. Services validate catalogs and
//! resolve pending outcomes; rules commit accepted facts. Keeping this crate
//! small lets action-game and authored-action content share effect mechanics
//! without importing genre-specific rule systems.

#![forbid(unsafe_code)]

pub mod reaction;
pub use reaction::{ReactionBehavior, ReactionDefinition, ReactionWindow, ReactionWindowKind};

use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

use core_error::{AshaError, ErrorCategory};
use core_ids::EntityId;
use core_time::{Tick, TickDelta};

macro_rules! string_id {
    (
        $(#[$attr:meta])*
        $name:ident
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, GameRuleCoreError> {
                let value = value.into();
                validate_stable_id(&value)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

string_id!(
    /// Stable id for a game-rules catalog.
    GameRuleCatalogId
);

string_id!(
    /// Stable id for one effect operation.
    EffectOpId
);

string_id!(
    /// Stable id for a modifier definition or live modifier instance.
    ModifierId
);

string_id!(
    /// Stable id for a bounded value channel such as health or shields.
    ValueChannelId
);

string_id!(
    /// Stable id for an effect tag/category.
    EffectTagId
);

string_id!(
    /// Stable id for an explicit reaction window.
    ReactionWindowId
);

/// A validation error in the core game-rules vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameRuleCoreError {
    EmptyId,
    InvalidIdChar { value: String },
    InvalidBounds { min: i64, current: i64, max: i64 },
    InvalidAmount { amount: u32 },
    InvalidDuration { ticks: u64 },
    InvalidCadence { period_ticks: u64 },
}

impl GameRuleCoreError {
    pub fn label(&self) -> &'static str {
        match self {
            GameRuleCoreError::EmptyId => "emptyId",
            GameRuleCoreError::InvalidIdChar { .. } => "invalidIdChar",
            GameRuleCoreError::InvalidBounds { .. } => "invalidBounds",
            GameRuleCoreError::InvalidAmount { .. } => "invalidAmount",
            GameRuleCoreError::InvalidDuration { .. } => "invalidDuration",
            GameRuleCoreError::InvalidCadence { .. } => "invalidCadence",
        }
    }
}

impl Display for GameRuleCoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameRuleCoreError::EmptyId => f.write_str("game-rule id must be non-empty"),
            GameRuleCoreError::InvalidIdChar { value } => {
                write!(f, "game-rule id `{value}` contains unsupported characters")
            }
            GameRuleCoreError::InvalidBounds { min, current, max } => {
                write!(
                    f,
                    "bounded value requires min <= current <= max, got {min} <= {current} <= {max}"
                )
            }
            GameRuleCoreError::InvalidAmount { amount } => {
                write!(f, "amount must be greater than zero, got {amount}")
            }
            GameRuleCoreError::InvalidDuration { ticks } => {
                write!(f, "duration must be greater than zero ticks, got {ticks}")
            }
            GameRuleCoreError::InvalidCadence { period_ticks } => {
                write!(
                    f,
                    "cadence period must be greater than zero ticks, got {period_ticks}"
                )
            }
        }
    }
}

impl std::error::Error for GameRuleCoreError {}

impl From<GameRuleCoreError> for AshaError {
    fn from(value: GameRuleCoreError) -> Self {
        AshaError::new(ErrorCategory::Invalid, value.to_string())
    }
}

/// A bounded signed value. The channel id lives on operations/facts so this type
/// can represent any numeric channel without carrying a fixed noun.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedValue {
    pub min: i64,
    pub current: i64,
    pub max: i64,
}

impl BoundedValue {
    pub fn new(min: i64, current: i64, max: i64) -> Result<Self, GameRuleCoreError> {
        if min > current || current > max {
            return Err(GameRuleCoreError::InvalidBounds { min, current, max });
        }
        Ok(Self { min, current, max })
    }

    pub fn apply_delta(self, delta: ValueDelta) -> AppliedValueDelta {
        let next = self
            .current
            .saturating_add(delta.amount)
            .clamp(self.min, self.max);
        AppliedValueDelta {
            before: self,
            delta,
            after: Self {
                min: self.min,
                current: next,
                max: self.max,
            },
        }
    }
}

/// A signed value change. Positive grants/restores; negative spends/damages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueDelta {
    pub amount: i64,
}

impl ValueDelta {
    pub const fn new(amount: i64) -> Self {
        Self { amount }
    }
}

/// Result of applying a delta to a bounded value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppliedValueDelta {
    pub before: BoundedValue,
    pub delta: ValueDelta,
    pub after: BoundedValue,
}

/// A positive unsigned amount used by restore/spend/grant operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PositiveAmount(u32);

impl PositiveAmount {
    pub fn new(amount: u32) -> Result<Self, GameRuleCoreError> {
        if amount == 0 {
            return Err(GameRuleCoreError::InvalidAmount { amount });
        }
        Ok(Self(amount))
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Duration for an effect or modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EffectDuration {
    Instant,
    Ticks(TickDelta),
    Infinite,
}

impl EffectDuration {
    pub fn ticks(ticks: u64) -> Result<Self, GameRuleCoreError> {
        if ticks == 0 {
            return Err(GameRuleCoreError::InvalidDuration { ticks });
        }
        Ok(Self::Ticks(TickDelta::new(ticks)))
    }

    pub fn expires_at(self, start: Tick) -> Option<Tick> {
        match self {
            EffectDuration::Instant => Some(start),
            EffectDuration::Ticks(delta) => Some(start.advance(delta)),
            EffectDuration::Infinite => None,
        }
    }
}

/// Periodic tick cadence for a modifier or scheduled effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TickCadence {
    period: TickDelta,
}

impl TickCadence {
    pub fn every(period_ticks: u64) -> Result<Self, GameRuleCoreError> {
        if period_ticks == 0 {
            return Err(GameRuleCoreError::InvalidCadence { period_ticks });
        }
        Ok(Self {
            period: TickDelta::new(period_ticks),
        })
    }

    pub const fn period(self) -> TickDelta {
        self.period
    }

    pub fn next_after(self, tick: Tick) -> Tick {
        tick.advance(self.period)
    }
}

/// Stack policy for repeated applications of the same modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StackPolicy {
    Refresh,
    Stack { max_stacks: u32 },
    RejectDuplicate,
    ReplaceIfStronger,
}

impl StackPolicy {
    pub fn stacking(max_stacks: u32) -> Self {
        Self::Stack {
            max_stacks: max_stacks.max(1),
        }
    }
}

/// A generic effect operation. It is declarative data; services/rules decide
/// whether an operation is valid or accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectOp {
    pub id: EffectOpId,
    pub kind: EffectOpKind,
    pub tags: BTreeSet<EffectTagId>,
}

impl EffectOp {
    pub fn new(id: EffectOpId, kind: EffectOpKind) -> Self {
        Self {
            id,
            kind,
            tags: BTreeSet::new(),
        }
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = EffectTagId>) -> Self {
        self.tags = tags.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectOpKind {
    ApplyDelta {
        value: ValueChannelId,
        delta: ValueDelta,
    },
    Restore {
        value: ValueChannelId,
        amount: PositiveAmount,
    },
    Spend {
        value: ValueChannelId,
        amount: PositiveAmount,
    },
    Grant {
        value: ValueChannelId,
        amount: PositiveAmount,
    },
    ApplyModifier {
        modifier: ModifierId,
    },
    RemoveModifier {
        modifier: ModifierId,
    },
    SchedulePeriodicEffect {
        modifier: ModifierId,
        cadence: TickCadence,
        duration: EffectDuration,
    },
    CancelResolution {
        reason: String,
    },
    EmitTrace {
        code: String,
        message: String,
    },
}

/// Authored modifier definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierDefinition {
    pub id: ModifierId,
    pub stack_policy: StackPolicy,
    pub duration: EffectDuration,
    pub tick_cadence: Option<TickCadence>,
    pub tags: BTreeSet<EffectTagId>,
    pub effect_ops: Vec<EffectOpId>,
    pub source_hash: String,
}

impl ModifierDefinition {
    pub fn new(id: ModifierId, stack_policy: StackPolicy, duration: EffectDuration) -> Self {
        Self {
            id,
            stack_policy,
            duration,
            tick_cadence: None,
            tags: BTreeSet::new(),
            effect_ops: Vec::new(),
            source_hash: String::new(),
        }
    }

    pub fn with_tick_cadence(mut self, cadence: TickCadence) -> Self {
        self.tick_cadence = Some(cadence);
        self
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = EffectTagId>) -> Self {
        self.tags = tags.into_iter().collect();
        self
    }

    pub fn with_effect_ops(mut self, ops: impl IntoIterator<Item = EffectOpId>) -> Self {
        self.effect_ops = ops.into_iter().collect();
        self.effect_ops.sort();
        self
    }

    pub fn with_source_hash(mut self, source_hash: impl Into<String>) -> Self {
        self.source_hash = source_hash.into();
        self
    }

    pub fn stable_hash(&self) -> u64 {
        let mut h = Fnv1a::new();
        h.feed_str(self.id.as_str());
        h.feed_u64(stack_policy_code(self.stack_policy));
        h.feed_u64(duration_code(self.duration));
        h.feed_u64(
            self.tick_cadence
                .map_or(0, |cadence| cadence.period().raw()),
        );
        h.feed_u64(self.tags.len() as u64);
        for tag in &self.tags {
            h.feed_str(tag.as_str());
        }
        h.feed_u64(self.effect_ops.len() as u64);
        for op in &self.effect_ops {
            h.feed_str(op.as_str());
        }
        h.feed_str(&self.source_hash);
        h.finish()
    }
}

/// Runtime state for an accepted modifier instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierState {
    pub modifier: ModifierId,
    pub source: EntityId,
    pub target: EntityId,
    pub stacks: u32,
    pub applied_at: Tick,
    pub expires_at: Option<Tick>,
    pub next_tick: Option<Tick>,
    pub source_hash: String,
}

impl ModifierState {
    pub fn from_definition(
        definition: &ModifierDefinition,
        source: EntityId,
        target: EntityId,
        applied_at: Tick,
    ) -> Self {
        Self {
            modifier: definition.id.clone(),
            source,
            target,
            stacks: 1,
            applied_at,
            expires_at: definition.duration.expires_at(applied_at),
            next_tick: definition
                .tick_cadence
                .map(|cadence| cadence.next_after(applied_at)),
            source_hash: definition.source_hash.clone(),
        }
    }
}

/// One deterministic trace entry emitted by validation/resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuleTraceEntry {
    pub step: u32,
    pub code: String,
    pub message: String,
    pub refs: Vec<(String, String)>,
}

impl GameRuleTraceEntry {
    pub fn new(step: u32, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            step,
            code: code.into(),
            message: message.into(),
            refs: Vec::new(),
        }
    }

    pub fn with_ref(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.refs.push((key.into(), value.into()));
        self.refs.sort();
        self
    }
}

/// Render a trace as stable line-oriented text for goldens and review packets.
pub fn render_trace(entries: &[GameRuleTraceEntry]) -> String {
    let mut ordered = entries.to_vec();
    ordered.sort_by(|a, b| a.step.cmp(&b.step).then_with(|| a.code.cmp(&b.code)));

    let mut out = String::new();
    for entry in ordered {
        out.push_str(&format!(
            "step={} code={} message={}\n",
            entry.step, entry.code, entry.message
        ));
        for (key, value) in entry.refs {
            out.push_str(&format!("  ref.{key}={value}\n"));
        }
    }
    out
}

fn validate_stable_id(value: &str) -> Result<(), GameRuleCoreError> {
    if value.is_empty() {
        return Err(GameRuleCoreError::EmptyId);
    }
    if !value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'/' | b':'))
    {
        return Err(GameRuleCoreError::InvalidIdChar {
            value: value.to_string(),
        });
    }
    Ok(())
}

fn stack_policy_code(policy: StackPolicy) -> u64 {
    match policy {
        StackPolicy::Refresh => 1,
        StackPolicy::Stack { max_stacks } => 10_000 + u64::from(max_stacks),
        StackPolicy::RejectDuplicate => 2,
        StackPolicy::ReplaceIfStronger => 3,
    }
}

fn duration_code(duration: EffectDuration) -> u64 {
    match duration {
        EffectDuration::Instant => 1,
        EffectDuration::Ticks(delta) => 10_000 + delta.raw(),
        EffectDuration::Infinite => 2,
    }
}

struct Fnv1a(u64);

impl Fnv1a {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn feed_u64(&mut self, value: u64) {
        for b in value.to_le_bytes() {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn feed_str(&mut self, value: &str) {
        self.feed_u64(value.len() as u64);
        for b in value.bytes() {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> EffectOpId {
        EffectOpId::parse(value).unwrap()
    }

    fn tag(value: &str) -> EffectTagId {
        EffectTagId::parse(value).unwrap()
    }

    #[test]
    fn stable_ids_reject_empty_or_unsupported_characters() {
        assert_eq!(
            ModifierId::parse("").unwrap_err(),
            GameRuleCoreError::EmptyId
        );
        assert!(matches!(
            ValueChannelId::parse("health space").unwrap_err(),
            GameRuleCoreError::InvalidIdChar { .. }
        ));
        assert_eq!(
            ReactionWindowId::parse("reaction.hit:pre-delta")
                .unwrap()
                .as_str(),
            "reaction.hit:pre-delta"
        );
    }

    #[test]
    fn bounded_value_rejects_invalid_ranges_and_clamps_deltas() {
        assert!(matches!(
            BoundedValue::new(0, 12, 10),
            Err(GameRuleCoreError::InvalidBounds { .. })
        ));

        let value = BoundedValue::new(0, 5, 10).unwrap();
        assert_eq!(value.apply_delta(ValueDelta::new(20)).after.current, 10);
        assert_eq!(value.apply_delta(ValueDelta::new(-20)).after.current, 0);
        assert_eq!(value.apply_delta(ValueDelta::new(3)).after.current, 8);
    }

    #[test]
    fn amounts_duration_and_cadence_fail_closed() {
        assert_eq!(
            PositiveAmount::new(0).unwrap_err(),
            GameRuleCoreError::InvalidAmount { amount: 0 }
        );
        assert_eq!(
            EffectDuration::ticks(0).unwrap_err(),
            GameRuleCoreError::InvalidDuration { ticks: 0 }
        );
        assert_eq!(
            TickCadence::every(0).unwrap_err(),
            GameRuleCoreError::InvalidCadence { period_ticks: 0 }
        );
    }

    #[test]
    fn stack_policy_normalizes_zero_cap_to_one() {
        assert_eq!(
            StackPolicy::stacking(0),
            StackPolicy::Stack { max_stacks: 1 }
        );
        assert_eq!(
            StackPolicy::stacking(4),
            StackPolicy::Stack { max_stacks: 4 }
        );
    }

    #[test]
    fn modifier_state_derives_expiration_and_next_tick() {
        let definition = ModifierDefinition::new(
            ModifierId::parse("modifier.poison").unwrap(),
            StackPolicy::Refresh,
            EffectDuration::ticks(9).unwrap(),
        )
        .with_tick_cadence(TickCadence::every(3).unwrap())
        .with_source_hash("fnv1a64:abc");

        let state = ModifierState::from_definition(
            &definition,
            EntityId::new(1),
            EntityId::new(2),
            Tick::new(10),
        );

        assert_eq!(state.expires_at, Some(Tick::new(19)));
        assert_eq!(state.next_tick, Some(Tick::new(13)));
        assert_eq!(state.stacks, 1);
    }

    #[test]
    fn effect_ops_preserve_typed_kinds_and_sorted_tags() {
        let value = ValueChannelId::parse("value.health").unwrap();
        let op = EffectOp::new(
            id("op.damage"),
            EffectOpKind::ApplyDelta {
                value,
                delta: ValueDelta::new(-5),
            },
        )
        .with_tags([tag("tag.poison"), tag("tag.damage")]);

        let tags: Vec<&str> = op.tags.iter().map(EffectTagId::as_str).collect();
        assert_eq!(tags, vec!["tag.damage", "tag.poison"]);
        assert!(matches!(op.kind, EffectOpKind::ApplyDelta { .. }));
    }

    #[test]
    fn modifier_hash_is_stable_across_input_order() {
        let a = ModifierDefinition::new(
            ModifierId::parse("modifier.poison").unwrap(),
            StackPolicy::stacking(3),
            EffectDuration::ticks(12).unwrap(),
        )
        .with_tick_cadence(TickCadence::every(4).unwrap())
        .with_tags([tag("tag.poison"), tag("tag.periodic")])
        .with_effect_ops([id("op.tick"), id("op.apply")])
        .with_source_hash("fnv1a64:1234");

        let b = ModifierDefinition::new(
            ModifierId::parse("modifier.poison").unwrap(),
            StackPolicy::stacking(3),
            EffectDuration::ticks(12).unwrap(),
        )
        .with_tick_cadence(TickCadence::every(4).unwrap())
        .with_tags([tag("tag.periodic"), tag("tag.poison")])
        .with_effect_ops([id("op.apply"), id("op.tick")])
        .with_source_hash("fnv1a64:1234");

        assert_eq!(a.stable_hash(), b.stable_hash());
        assert_eq!(a.stable_hash(), 0x555b_b0a4_d950_2cf1);
    }

    #[test]
    fn trace_rendering_is_deterministic() {
        let rendered = render_trace(&[
            GameRuleTraceEntry::new(2, "modifier.applied", "accepted")
                .with_ref("target", "2")
                .with_ref("source", "1"),
            GameRuleTraceEntry::new(1, "effect.validated", "catalog clean"),
        ]);

        assert_eq!(
            rendered,
            "step=1 code=effect.validated message=catalog clean\nstep=2 code=modifier.applied message=accepted\n  ref.source=1\n  ref.target=2\n"
        );
    }
}

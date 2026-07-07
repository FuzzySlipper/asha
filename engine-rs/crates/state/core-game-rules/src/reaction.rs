//! Generic reaction-window vocabulary for pending game-rule effects.
//!
//! Reaction windows are explicit typed phases over already-known pending facts.

use std::collections::BTreeSet;

use crate::{EffectOpId, EffectTagId, ModifierId, ReactionWindowId, ValueChannelId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReactionWindowKind {
    AcceptedHit,
    PendingValueDelta,
    ModifierApplication,
    ValueDepleted,
    PeriodicTick,
    ResolutionCancelled,
}

impl ReactionWindowKind {
    pub const fn label(self) -> &'static str {
        match self {
            ReactionWindowKind::AcceptedHit => "acceptedHit",
            ReactionWindowKind::PendingValueDelta => "pendingValueDelta",
            ReactionWindowKind::ModifierApplication => "modifierApplication",
            ReactionWindowKind::ValueDepleted => "valueDepleted",
            ReactionWindowKind::PeriodicTick => "periodicTick",
            ReactionWindowKind::ResolutionCancelled => "resolutionCancelled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReactionWindow {
    pub id: ReactionWindowId,
    pub kind: ReactionWindowKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactionBehavior {
    ModifyPendingDelta {
        channel: ValueChannelId,
        amount: i64,
    },
    CancelPendingEffect {
        reason: String,
    },
    ApplyFollowUpEffect {
        effect_op: EffectOpId,
    },
    ApplyFollowUpModifier {
        modifier: ModifierId,
    },
    EmitTrace {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionDefinition {
    pub id: ReactionWindowId,
    pub window: ReactionWindowKind,
    pub priority: i32,
    pub reads: BTreeSet<ValueChannelId>,
    pub allowed_effect_ops: BTreeSet<EffectOpId>,
    pub behavior: ReactionBehavior,
    pub tags: BTreeSet<EffectTagId>,
    pub source_hash: String,
}

impl ReactionDefinition {
    pub fn new(
        id: ReactionWindowId,
        window: ReactionWindowKind,
        behavior: ReactionBehavior,
    ) -> Self {
        Self {
            id,
            window,
            priority: 0,
            reads: BTreeSet::new(),
            allowed_effect_ops: BTreeSet::new(),
            behavior,
            tags: BTreeSet::new(),
            source_hash: String::new(),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_reads(mut self, reads: impl IntoIterator<Item = ValueChannelId>) -> Self {
        self.reads = reads.into_iter().collect();
        self
    }

    pub fn with_allowed_effect_ops(mut self, ops: impl IntoIterator<Item = EffectOpId>) -> Self {
        self.allowed_effect_ops = ops.into_iter().collect();
        self
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = EffectTagId>) -> Self {
        self.tags = tags.into_iter().collect();
        self
    }

    pub fn with_source_hash(mut self, source_hash: impl Into<String>) -> Self {
        self.source_hash = source_hash.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_generic_and_genre_neutral() {
        let labels = [
            ReactionWindowKind::AcceptedHit.label(),
            ReactionWindowKind::PendingValueDelta.label(),
            ReactionWindowKind::ModifierApplication.label(),
            ReactionWindowKind::ValueDepleted.label(),
            ReactionWindowKind::PeriodicTick.label(),
            ReactionWindowKind::ResolutionCancelled.label(),
        ]
        .join(" ");

        let forbidden = [
            concat!("tu", "rn"),
            concat!("ro", "und"),
            concat!("init", "iative"),
            concat!("action", "Economy"),
            concat!("saving", "Throw"),
        ];
        for forbidden in forbidden {
            assert!(
                !labels.contains(forbidden),
                "{forbidden} leaked into labels"
            );
        }
    }
}

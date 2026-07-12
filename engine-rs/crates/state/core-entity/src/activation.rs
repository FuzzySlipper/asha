//! Typed activation state for the capability families that genuinely support
//! temporary inactivity without detachment.

use core_ids::EntityId;

use crate::core::EntityLifecycle;

/// Closed inventory of capability families with activation semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActivatableCapabilityKind {
    Collision,
    Controller,
}

impl ActivatableCapabilityKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Collision => "collision",
            Self::Controller => "controller",
        }
    }
}

/// Stored state for an attached activatable capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityActivationState {
    Active,
    Inactive,
}

impl CapabilityActivationState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

/// Query result that preserves the difference between no capability and an
/// attached capability whose owner has made it inactive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityActivationPresence {
    Absent,
    Inactive,
    Active,
}

impl CapabilityActivationPresence {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Inactive => "inactive",
            Self::Active => "active",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityActivationAction {
    Activate,
    Deactivate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityActivationCommand {
    pub entity: EntityId,
    pub capability: ActivatableCapabilityKind,
    pub action: CapabilityActivationAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityActivationEvent {
    pub entity: EntityId,
    pub capability: ActivatableCapabilityKind,
    pub from: CapabilityActivationState,
    pub to: CapabilityActivationState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityActivationReadout {
    pub entity: EntityId,
    pub capability: ActivatableCapabilityKind,
    pub presence: CapabilityActivationPresence,
    /// Entity lifecycle is a separate suppression axis. An active capability on
    /// a disabled entity remains active but is not effective until re-enabled.
    pub entity_lifecycle: EntityLifecycle,
    pub effective_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityActivationError {
    UnknownEntity {
        entity: EntityId,
    },
    Tombstoned {
        entity: EntityId,
    },
    CapabilityAbsent {
        entity: EntityId,
        capability: ActivatableCapabilityKind,
    },
    AlreadyInState {
        entity: EntityId,
        capability: ActivatableCapabilityKind,
        state: CapabilityActivationState,
    },
}

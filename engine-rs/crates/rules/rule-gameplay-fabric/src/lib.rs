//! Bounded runtime coordination over an immutable gameplay-fabric registry.
//!
//! This crate owns invocation moments and owner routing. It does not expose
//! dynamic registration, raw Session state, bridge operations, or TypeScript
//! callbacks.

#![forbid(unsafe_code)]

mod decision;
mod legacy_weapon_transform;
mod observe;
mod owner_events;
mod reaction_adapter;
mod reads;
mod state;
mod types;

/// Quarantined Wave 1 adapters retained only for named downstream migrations.
/// New gameplay modules must use the ordinary fabric families and static
/// RuntimeSession composition instead of importing this namespace.
pub mod compatibility {
    pub use crate::legacy_weapon_transform::{
        run_legacy_weapon_effect_transform, LegacyWeaponEffectTransformError,
        LegacyWeaponEffectTransformOutcome, LEGACY_WEAPON_EFFECT_COMPATIBILITY_DIAGNOSTIC,
    };
}

pub use decision::gameplay_payload_hash;
pub use observe::{
    direct_authority_routing_receipt, gameplay_proposal_hash, verify_gameplay_routing_evidence,
    GameplayFabricCoordinator,
};
pub use owner_events::*;
pub use reaction_adapter::{
    resolve_declared_reactions, ReactionBehavior, ReactionDefinition, ReactionResolution,
    ReactionResolutionInput, ReactionWindowKind,
};
pub use reads::*;
pub use state::*;
pub use types::*;

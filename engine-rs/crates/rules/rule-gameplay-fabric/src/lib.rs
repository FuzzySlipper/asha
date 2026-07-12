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

pub use decision::gameplay_payload_hash;
pub use legacy_weapon_transform::*;
pub use observe::{gameplay_proposal_hash, GameplayFabricCoordinator};
pub use owner_events::*;
pub use reaction_adapter::{
    resolve_declared_reactions, ReactionBehavior, ReactionDefinition, ReactionResolution,
    ReactionResolutionInput, ReactionWindowKind,
};
pub use reads::*;
pub use state::*;
pub use types::*;

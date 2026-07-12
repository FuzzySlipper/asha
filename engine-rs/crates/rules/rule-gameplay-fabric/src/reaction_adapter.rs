//! Adapter for the existing generic reaction-window resolver.
//!
//! Gameplay modules can use this from a `React` invocation without rebuilding
//! priority/stable-id ordering or declared-read/effect validation.

pub use core_game_rules::{ReactionBehavior, ReactionDefinition, ReactionWindowKind};
pub use svc_game_rules::reaction::{ReactionResolution, ReactionResolutionInput};

pub fn resolve_declared_reactions(
    definitions: &[ReactionDefinition],
    input: &ReactionResolutionInput,
) -> ReactionResolution {
    svc_game_rules::reaction::resolve_reactions(definitions, input)
}

//! Immutable gameplay-fabric contracts, typed codecs, and Session topology.
//!
//! # Lane
//!
//! `rust-service` — validates a statically linked module/provider/owner graph
//! before RuntimeSession bootstrap commits. This crate does not dispatch
//! handlers, apply proposals, own persistent module state, or mutate a live
//! registry.

#![forbid(unsafe_code)]

mod codec;
mod registry;
mod topology;
mod validation;

pub use codec::{GameplayCodecError, GameplayEventCodecRegistration, TypedGameplayEventCodec};
pub use registry::{
    GameplayFabricRegistry, GameplayFabricRegistryBuilder, GameplayLinkedProvider,
    GameplayProposalOwnerRegistration, GameplayReadViewProviderRegistration,
    GameplayRegistryBuildError, GameplayStateOwnerRegistration,
};

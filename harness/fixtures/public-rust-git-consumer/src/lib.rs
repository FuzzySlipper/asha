//! Clean-checkout consumer proof for the governed public Rust distribution.

#![forbid(unsafe_code)]

use asha_gameplay_module_sdk::GameplayStaticCompositionBuilder;
use asha_runtime_session_composition::DeferredRuntimeSessionBuilder;

pub fn gameplay_composition_builder() -> GameplayStaticCompositionBuilder {
    GameplayStaticCompositionBuilder::new()
}

pub fn runtime_session_builder_type_name() -> &'static str {
    core::any::type_name::<DeferredRuntimeSessionBuilder>()
}

pub fn authored_behavior_composition() -> asha_gameplay_module_sdk::GameplayStaticComposition {
    let mut builder = GameplayStaticCompositionBuilder::new();
    builder.include_standard_owner_events();
    builder
        .build()
        .expect("the public Engine standard owner events compose")
}

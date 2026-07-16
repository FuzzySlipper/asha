//! Provider regression that links concrete gameplay modules
//! into ASHA's generated N-API transport without copying its operation table.

#![forbid(unsafe_code)]

use asha_native_runtime_provider::install_native_engine_bridge_factory;
use asha_runtime_session_composition::{
    EngineBridge, RuntimeBridgeError, RuntimeBridgeErrorKind, StaticRuntimeSessionBuilder,
};

fn build_composed_bridge() -> Result<EngineBridge, RuntimeBridgeError> {
    StaticRuntimeSessionBuilder::activate_project_with_prefabs(
        asha_gameplay_module_fixture::composed_runtime_host_project_input(4),
        asha_gameplay_module_fixture::composed_runtime_prefab_bootstrap(),
    )
    .and_then(StaticRuntimeSessionBuilder::build)
    .map_err(|error| {
        RuntimeBridgeError::new(
            RuntimeBridgeErrorKind::Internal,
            format!("composed native provider activation failed: {error}"),
        )
    })
}

#[asha_native_runtime_provider::native_provider_module_init]
fn install_composed_provider() {
    install_native_engine_bridge_factory(build_composed_bridge)
        .expect("the fixture installs exactly one native provider factory");
}

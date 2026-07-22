use super::*;

pub(super) fn configure(
    bridge: &mut EngineBridge,
    request: InputSessionConfigureRequest,
) -> BridgeResult<InputSessionSnapshot> {
    bridge.require_initialized("configure_input_session")?;
    let project_catalogs = bridge
        .gameplay
        .static_gameplay_host
        .as_ref()
        .and_then(|host| host.activated_project_content_readout())
        .map(|content| {
            content
                .documents
                .iter()
                .filter_map(|document| match document {
                    protocol_project_content::ProjectContentDocumentDto::InputCatalog {
                        catalog,
                        ..
                    } => Some(catalog.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let base_catalog = if project_catalogs.is_empty() {
        request.catalog
    } else {
        rule_input::default_browser_input_catalog()
    };
    let catalog = rule_input::compose_project_input_catalog(base_catalog, &project_catalogs)
        .map_err(input_activation_error)?;
    let resolver = InputSessionResolver::activate(catalog, request.initial_contexts)
        .map_err(input_activation_error)?;
    let snapshot = resolver.snapshot();
    bridge.input.input_session = Some(resolver);
    Ok(snapshot)
}

fn input_activation_error(error: rule_input::InputCatalogValidationError) -> RuntimeBridgeError {
    let details = error
        .diagnostics()
        .iter()
        .map(|item| format!("{:?}@{}: {}", item.code, item.path, item.message))
        .collect::<Vec<_>>()
        .join("; ");
    RuntimeBridgeError::new(
        RuntimeBridgeErrorKind::InvalidInput,
        format!("input Session activation rejected: {details}"),
    )
}

pub(super) fn apply_context_command(
    bridge: &mut EngineBridge,
    command: InputContextCommand,
) -> BridgeResult<InputContextChangeReceipt> {
    let resolver = bridge.input.input_session.as_mut().ok_or_else(|| {
        RuntimeBridgeError::new(
            RuntimeBridgeErrorKind::NotInitialized,
            "apply_input_context_command called before configure_input_session",
        )
    })?;
    Ok(resolver.apply_context_command(command))
}

pub(super) fn submit(
    bridge: &EngineBridge,
    sample: RawInputSample,
) -> BridgeResult<InputResolutionReceipt> {
    let resolver = bridge.input.input_session.as_ref().ok_or_else(|| {
        RuntimeBridgeError::new(
            RuntimeBridgeErrorKind::NotInitialized,
            "submit_raw_input called before configure_input_session",
        )
    })?;
    Ok(resolver.resolve(sample))
}

pub(super) fn replay(
    bridge: &mut EngineBridge,
    record: RecordedInputAction,
) -> BridgeResult<InputActionReplayReceipt> {
    let resolver = bridge.input.input_session.as_mut().ok_or_else(|| {
        RuntimeBridgeError::new(
            RuntimeBridgeErrorKind::NotInitialized,
            "replay_resolved_input_action called before configure_input_session",
        )
    })?;
    Ok(resolver.replay(record))
}

pub(super) fn read_context_state(bridge: &EngineBridge) -> BridgeResult<InputContextStackState> {
    let resolver = bridge.input.input_session.as_ref().ok_or_else(|| {
        RuntimeBridgeError::new(
            RuntimeBridgeErrorKind::NotInitialized,
            "read_input_context_state called before configure_input_session",
        )
    })?;
    Ok(resolver.context_state().clone())
}

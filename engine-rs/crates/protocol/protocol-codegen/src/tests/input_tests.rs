use super::*;

pub(super) fn extend_round_trip_coverage(coverage: &mut BTreeSet<String>) {
    coverage.extend([
        interface_coverage_key("input", "InputActionDefinition"),
        interface_coverage_key("input", "InputContextDefinition"),
        interface_coverage_key("input", "InputBindingExtension"),
        interface_coverage_key("input", "InputBindingRecord"),
        interface_coverage_key("input", "InputBindingCatalog"),
        interface_coverage_key("input", "InputSessionConfigureRequest"),
        interface_coverage_key("input", "ActiveInputContext"),
        interface_coverage_key("input", "InputContextStackState"),
        variant_coverage_key("input", "InputContextCommand", "push"),
        variant_coverage_key("input", "InputContextCommand", "pop"),
        variant_coverage_key("input", "InputContextCommand", "replace"),
        interface_coverage_key("input", "InputContextChangeReceipt"),
        interface_coverage_key("input", "InputSessionSnapshot"),
        variant_coverage_key("input", "InputValue", "button"),
        variant_coverage_key("input", "InputValue", "axis1d"),
        variant_coverage_key("input", "InputValue", "axis2d"),
        interface_coverage_key("input", "RawInputSample"),
        interface_coverage_key("input", "ResolvedInputAction"),
        interface_coverage_key("input", "RecordedInputAction"),
        interface_coverage_key("input", "InputDiagnostic"),
        interface_coverage_key("input", "InputResolutionReceipt"),
        interface_coverage_key("input", "InputActionReplayReceipt"),
    ]);
}

#[test]
fn input_rust_serialization_matches_ir_shape() {
    use protocol_input::{
        ActiveInputContext, InputActionDefinition, InputActionPhase, InputActionReplayReceipt,
        InputBindingCatalog, InputBindingExtension, InputBindingRecord, InputContextChangeReceipt,
        InputContextCommand, InputContextDefinition, InputContextStackState, InputDiagnostic,
        InputDiagnosticCode, InputResolutionReceipt, InputSessionConfigureRequest,
        InputSessionSnapshot, InputValue, InputValueKind, PlatformInputKind, RawInputSample,
        RecordedInputAction, ResolvedInputAction, INPUT_ACTION_PHASES,
        INPUT_ACTION_RECORD_SCHEMA_VERSION, INPUT_BINDING_CATALOG_SCHEMA_VERSION,
        INPUT_CONTEXT_STATE_SCHEMA_VERSION, INPUT_VALUE_KINDS, PLATFORM_INPUT_KINDS,
    };

    let input = module("input");
    assert_eq!(
        string_enum_values(&input, "InputValueKind"),
        INPUT_VALUE_KINDS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    );
    assert_eq!(
        string_enum_values(&input, "InputActionPhase"),
        INPUT_ACTION_PHASES
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    );
    assert_eq!(
        string_enum_values(&input, "PlatformInputKind"),
        PLATFORM_INPUT_KINDS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    );

    let action = InputActionDefinition {
        action_id: "camera.look".into(),
        value_kind: InputValueKind::Axis2d,
        accepted_phases: vec![InputActionPhase::Changed],
    };
    let context = InputContextDefinition {
        context_id: "gameplay".into(),
        priority: 10,
        consumes_lower_priority: false,
    };
    let extension = InputBindingExtension {
        schema_version: 2,
        required_controls: vec!["ShiftLeft".into()],
    };
    let binding = InputBindingRecord {
        binding_id: "game.look.mouse".into(),
        action_id: action.action_id.clone(),
        context_id: context.context_id.clone(),
        platform_kind: PlatformInputKind::MouseDelta,
        control: "PointerDelta".into(),
        scale: 0.5,
        extension: Some(extension.clone()),
    };
    let catalog = InputBindingCatalog {
        schema_version: INPUT_BINDING_CATALOG_SCHEMA_VERSION,
        actions: vec![action.clone()],
        contexts: vec![context.clone()],
        bindings: vec![binding.clone()],
    };
    let configure = InputSessionConfigureRequest {
        catalog: catalog.clone(),
        initial_contexts: vec!["gameplay".into()],
    };
    let active = ActiveInputContext {
        context_id: context.context_id.clone(),
        stack_order: 0,
    };
    let state = InputContextStackState {
        schema_version: INPUT_CONTEXT_STATE_SCHEMA_VERSION,
        revision: 4,
        active_contexts: vec![active.clone()],
        state_hash: "fnv1a64:1111111111111111".into(),
    };
    let diagnostic = InputDiagnostic {
        code: InputDiagnosticCode::ConsumedByContext,
        path: "contextState.activeContexts".into(),
        message: "consumed".into(),
    };

    for (name, value) in [
        (
            "InputActionDefinition",
            serde_json::to_value(action).unwrap(),
        ),
        (
            "InputContextDefinition",
            serde_json::to_value(context).unwrap(),
        ),
        (
            "InputBindingExtension",
            serde_json::to_value(extension).unwrap(),
        ),
        ("InputBindingRecord", serde_json::to_value(binding).unwrap()),
        (
            "InputBindingCatalog",
            serde_json::to_value(catalog).unwrap(),
        ),
        (
            "InputSessionConfigureRequest",
            serde_json::to_value(configure).unwrap(),
        ),
        ("ActiveInputContext", serde_json::to_value(active).unwrap()),
        (
            "InputContextStackState",
            serde_json::to_value(&state).unwrap(),
        ),
        (
            "InputDiagnostic",
            serde_json::to_value(&diagnostic).unwrap(),
        ),
    ] {
        compare_object_to_interface(&input, name, &value).unwrap();
    }

    for (tag, command) in [
        (
            "push",
            InputContextCommand::Push {
                context_id: "menu".into(),
            },
        ),
        (
            "pop",
            InputContextCommand::Pop {
                expected_context_id: "menu".into(),
            },
        ),
        (
            "replace",
            InputContextCommand::Replace {
                context_ids: vec!["gameplay".into()],
            },
        ),
    ] {
        let serialized = serde_json::to_value(command).unwrap();
        compare_object_to_variant(&input, "InputContextCommand", tag, &serialized).unwrap();
    }

    for (tag, value) in [
        ("button", InputValue::Button { pressed: true }),
        ("axis1d", InputValue::Axis1d { value: 1.0 }),
        ("axis2d", InputValue::Axis2d { x: 2.0, y: -1.0 }),
    ] {
        let serialized = serde_json::to_value(value).unwrap();
        compare_object_to_variant(&input, "InputValue", tag, &serialized).unwrap();
    }

    let raw = RawInputSample {
        sequence: 7,
        platform_kind: PlatformInputKind::MouseDelta,
        control: "PointerDelta".into(),
        phase: InputActionPhase::Changed,
        value: InputValue::Axis2d { x: 2.0, y: -1.0 },
    };
    let resolved = ResolvedInputAction {
        sequence: raw.sequence,
        action_id: "camera.look".into(),
        context_id: "gameplay".into(),
        binding_id: "game.look.mouse".into(),
        phase: raw.phase,
        value: raw.value.clone(),
    };
    let record = RecordedInputAction {
        schema_version: INPUT_ACTION_RECORD_SCHEMA_VERSION,
        action: resolved.clone(),
        catalog_hash: "fnv1a64:2222222222222222".into(),
        context_hash: state.state_hash.clone(),
        record_hash: "fnv1a64:5555555555555555".into(),
    };
    let receipt = InputResolutionReceipt {
        sequence: raw.sequence,
        accepted: true,
        consumed: true,
        action: Some(resolved.clone()),
        diagnostics: Vec::new(),
        catalog_hash: "fnv1a64:2222222222222222".into(),
        context_hash: state.state_hash.clone(),
        input_hash: "fnv1a64:3333333333333333".into(),
        resolution_hash: "fnv1a64:4444444444444444".into(),
        record: Some(record.clone()),
    };
    let replay = InputActionReplayReceipt {
        accepted: true,
        action: Some(resolved.clone()),
        diagnostics: Vec::new(),
        catalog_hash: record.catalog_hash.clone(),
        context_hash: record.context_hash.clone(),
        record_hash: record.record_hash.clone(),
        replay_hash: "fnv1a64:6666666666666666".into(),
    };
    let change = InputContextChangeReceipt {
        accepted: false,
        state: state.clone(),
        diagnostics: vec![diagnostic],
    };
    let snapshot = InputSessionSnapshot {
        catalog_hash: receipt.catalog_hash.clone(),
        context_state: state,
    };
    for (name, value) in [
        ("RawInputSample", serde_json::to_value(raw).unwrap()),
        (
            "ResolvedInputAction",
            serde_json::to_value(resolved).unwrap(),
        ),
        ("RecordedInputAction", serde_json::to_value(record).unwrap()),
        (
            "InputResolutionReceipt",
            serde_json::to_value(receipt).unwrap(),
        ),
        (
            "InputActionReplayReceipt",
            serde_json::to_value(replay).unwrap(),
        ),
        (
            "InputContextChangeReceipt",
            serde_json::to_value(change).unwrap(),
        ),
        (
            "InputSessionSnapshot",
            serde_json::to_value(snapshot).unwrap(),
        ),
    ] {
        compare_object_to_interface(&input, name, &value).unwrap();
    }
}

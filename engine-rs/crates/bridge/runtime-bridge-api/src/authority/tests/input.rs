use super::*;
use protocol_input::{
    InputActionDefinition, InputActionPhase, InputBindingRecord, InputContextDefinition,
    InputDiagnosticCode, InputValue, InputValueKind, PlatformInputKind,
    INPUT_BINDING_CATALOG_SCHEMA_VERSION,
};

fn catalog() -> InputBindingCatalog {
    InputBindingCatalog {
        schema_version: INPUT_BINDING_CATALOG_SCHEMA_VERSION,
        actions: vec![InputActionDefinition {
            action_id: "game.move.forward".into(),
            value_kind: InputValueKind::Button,
            accepted_phases: vec![InputActionPhase::Held],
        }],
        contexts: vec![
            InputContextDefinition {
                context_id: "gameplay".into(),
                priority: 10,
                consumes_lower_priority: false,
            },
            InputContextDefinition {
                context_id: "menu".into(),
                priority: 100,
                consumes_lower_priority: true,
            },
        ],
        bindings: vec![InputBindingRecord {
            binding_id: "game.forward.w".into(),
            action_id: "game.move.forward".into(),
            context_id: "gameplay".into(),
            platform_kind: PlatformInputKind::KeyboardKey,
            control: "KeyW".into(),
            scale: 1.0,
            extension: None,
        }],
    }
}

#[test]
fn runtime_bridge_routes_normalized_input_through_session_rule_authority() {
    let mut bridge = init_bridge();
    let snapshot = bridge
        .configure_input_session(InputSessionConfigureRequest {
            catalog: catalog(),
            initial_contexts: vec!["gameplay".into()],
        })
        .unwrap();
    assert_eq!(
        snapshot.context_state.active_contexts[0].context_id,
        "gameplay"
    );

    let resolved = bridge
        .submit_raw_input(RawInputSample {
            sequence: 1,
            platform_kind: PlatformInputKind::KeyboardKey,
            control: "KeyW".into(),
            phase: InputActionPhase::Held,
            value: InputValue::Button { pressed: true },
        })
        .unwrap();
    assert_eq!(resolved.action.unwrap().action_id, "game.move.forward");

    let pushed = bridge
        .apply_input_context_command(InputContextCommand::Push {
            context_id: "menu".into(),
        })
        .unwrap();
    assert!(pushed.accepted);
    let blocked = bridge
        .submit_raw_input(RawInputSample {
            sequence: 2,
            platform_kind: PlatformInputKind::KeyboardKey,
            control: "KeyW".into(),
            phase: InputActionPhase::Held,
            value: InputValue::Button { pressed: true },
        })
        .unwrap();
    assert!(blocked.consumed);
    assert_eq!(
        blocked.diagnostics[0].code,
        InputDiagnosticCode::ConsumedByContext
    );
    assert_eq!(bridge.read_input_context_state().unwrap(), pushed.state);
}

#[test]
fn invalid_input_catalog_never_replaces_an_active_session() {
    let mut bridge = init_bridge();
    let before = bridge
        .configure_input_session(InputSessionConfigureRequest {
            catalog: catalog(),
            initial_contexts: vec!["gameplay".into()],
        })
        .unwrap();
    let mut invalid = catalog();
    invalid.schema_version = 99;
    let error = bridge
        .configure_input_session(InputSessionConfigureRequest {
            catalog: invalid,
            initial_contexts: vec!["menu".into()],
        })
        .unwrap_err();
    assert_eq!(error.kind, RuntimeBridgeErrorKind::InvalidInput);
    assert_eq!(
        bridge.read_input_context_state().unwrap(),
        before.context_state
    );
}

#[test]
fn runtime_bridge_replays_authority_issued_actions_exactly_once_without_raw_input() {
    let mut source = init_bridge();
    source
        .configure_input_session(InputSessionConfigureRequest {
            catalog: catalog(),
            initial_contexts: vec!["gameplay".into()],
        })
        .unwrap();
    let record = source
        .submit_raw_input(RawInputSample {
            sequence: 3,
            platform_kind: PlatformInputKind::KeyboardKey,
            control: "KeyW".into(),
            phase: InputActionPhase::Held,
            value: InputValue::Button { pressed: true },
        })
        .unwrap()
        .record
        .unwrap();

    let mut replay = init_bridge();
    replay
        .configure_input_session(InputSessionConfigureRequest {
            catalog: catalog(),
            initial_contexts: vec!["gameplay".into()],
        })
        .unwrap();
    let delivered = replay.replay_resolved_input_action(record.clone()).unwrap();
    assert!(delivered.accepted);
    assert_eq!(delivered.action.unwrap(), record.action);

    let duplicate = replay.replay_resolved_input_action(record).unwrap();
    assert!(!duplicate.accepted);
    assert_eq!(
        duplicate.diagnostics[0].code,
        InputDiagnosticCode::ReplayAlreadyDelivered
    );
}

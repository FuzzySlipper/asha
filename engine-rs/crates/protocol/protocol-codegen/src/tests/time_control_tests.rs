use super::*;

pub(super) fn extend_round_trip_coverage(coverage: &mut BTreeSet<String>) {
    coverage.extend([
        variant_coverage_key("timeControl", "TimeControlCommand", "pause"),
        variant_coverage_key("timeControl", "TimeControlCommand", "resume"),
        variant_coverage_key("timeControl", "TimeControlCommand", "setSpeedMultiplier"),
        variant_coverage_key("timeControl", "TimeControlCommand", "stepTicks"),
        interface_coverage_key("timeControl", "TimeControlState"),
        interface_coverage_key("timeControl", "TimeControlReceipt"),
    ]);
}

#[test]
fn time_control_rust_serialization_matches_ir_shape() {
    use protocol_time_control::{
        TimeControlCommand, TimeControlMode, TimeControlReceipt, TimeControlState,
        TIME_CONTROL_MODES, TIME_CONTROL_REJECTIONS, TIME_CONTROL_STATE_SCHEMA_VERSION,
    };

    let time_control = module("timeControl");
    assert_eq!(
        string_enum_values(&time_control, "TimeControlMode"),
        TIME_CONTROL_MODES
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    );
    assert_eq!(
        string_enum_values(&time_control, "TimeControlRejection"),
        TIME_CONTROL_REJECTIONS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    );

    for (tag, command) in [
        ("pause", TimeControlCommand::Pause),
        ("resume", TimeControlCommand::Resume),
        (
            "setSpeedMultiplier",
            TimeControlCommand::SetSpeedMultiplier { multiplier: 2 },
        ),
        ("stepTicks", TimeControlCommand::StepTicks { ticks: 3 }),
    ] {
        let serialized = serde_json::to_value(command).unwrap();
        compare_object_to_variant(&time_control, "TimeControlCommand", tag, &serialized).unwrap();
    }

    let before = TimeControlState {
        schema_version: TIME_CONTROL_STATE_SCHEMA_VERSION,
        mode: TimeControlMode::Paused,
        speed_multiplier: 1,
        revision: 4,
        authority_tick: 10,
        state_hash: "fnv1a64:1111111111111111".into(),
    };
    let after = TimeControlState {
        revision: 5,
        authority_tick: 13,
        state_hash: "fnv1a64:2222222222222222".into(),
        ..before.clone()
    };
    let receipt = TimeControlReceipt {
        accepted: true,
        before: before.clone(),
        after,
        exact_ticks_advanced: 3,
        rejection: None,
        receipt_hash: "fnv1a64:3333333333333333".into(),
    };
    compare_object_to_interface(
        &time_control,
        "TimeControlState",
        &serde_json::to_value(before).unwrap(),
    )
    .unwrap();
    compare_object_to_interface(
        &time_control,
        "TimeControlReceipt",
        &serde_json::to_value(receipt).unwrap(),
    )
    .unwrap();
}

use super::*;

#[test]
fn time_control_requires_an_initialized_session() {
    let mut bridge = EngineBridge::new();
    assert_eq!(
        bridge.read_time_control_state().unwrap_err().kind,
        RuntimeBridgeErrorKind::NotInitialized
    );
    assert_eq!(
        bridge
            .apply_time_control_command(TimeControlCommand::Pause)
            .unwrap_err()
            .kind,
        RuntimeBridgeErrorKind::NotInitialized
    );
}

#[test]
fn pause_blocks_cadence_ticks_while_projection_reads_remain_live() {
    let mut bridge = init_bridge();
    let initial = bridge.read_time_control_state().unwrap();
    assert_eq!(initial.mode, TimeControlMode::Running);
    assert_eq!(initial.speed_multiplier, 1);
    assert_eq!(initial.authority_tick, 0);

    let pause = bridge
        .apply_time_control_command(TimeControlCommand::Pause)
        .unwrap();
    assert!(pause.accepted);
    assert_eq!(pause.after.mode, TimeControlMode::Paused);
    assert_eq!(pause.exact_ticks_advanced, 0);

    assert_eq!(
        bridge
            .step_simulation(StepInputEnvelope { tick: 9 })
            .unwrap(),
        StepResult {
            tick: 0,
            diff_count: 0,
        }
    );
    assert_eq!(bridge.read_projection_frame(0).unwrap().authority_tick, 0);
    assert_eq!(bridge.read_time_control_state().unwrap(), pause.after);
}

#[test]
fn exact_steps_advance_the_requested_count_and_remain_paused() {
    let mut bridge = init_bridge();
    bridge
        .apply_time_control_command(TimeControlCommand::Pause)
        .unwrap();

    let receipt = bridge
        .apply_time_control_command(TimeControlCommand::StepTicks { ticks: 3 })
        .unwrap();
    assert!(receipt.accepted);
    assert_eq!(receipt.before.authority_tick, 0);
    assert_eq!(receipt.after.authority_tick, 3);
    assert_eq!(receipt.after.mode, TimeControlMode::Paused);
    assert_eq!(receipt.exact_ticks_advanced, 3);
    assert_ne!(receipt.before.state_hash, receipt.after.state_hash);

    assert_eq!(
        bridge
            .step_simulation(StepInputEnvelope { tick: 20 })
            .unwrap()
            .tick,
        3
    );
    bridge
        .apply_time_control_command(TimeControlCommand::Resume)
        .unwrap();
    assert_eq!(
        bridge
            .step_simulation(StepInputEnvelope { tick: 4 })
            .unwrap()
            .tick,
        4
    );
}

#[test]
fn invalid_time_commands_are_atomic_and_classified() {
    let mut bridge = init_bridge();
    let running = bridge.read_time_control_state().unwrap();
    let running_step = bridge
        .apply_time_control_command(TimeControlCommand::StepTicks { ticks: 1 })
        .unwrap();
    assert!(!running_step.accepted);
    assert_eq!(
        running_step.rejection,
        Some(TimeControlRejection::NotPausedForExactStep)
    );
    assert_eq!(running_step.before, running);
    assert_eq!(running_step.after, running);

    let speed = bridge
        .apply_time_control_command(TimeControlCommand::SetSpeedMultiplier { multiplier: 0 })
        .unwrap();
    assert!(!speed.accepted);
    assert_eq!(
        speed.rejection,
        Some(TimeControlRejection::InvalidSpeedMultiplier)
    );
    assert_eq!(speed.before, speed.after);

    bridge
        .apply_time_control_command(TimeControlCommand::Pause)
        .unwrap();
    for ticks in [0, sim_runner::MAX_EXACT_STEP_TICKS + 1] {
        let rejected = bridge
            .apply_time_control_command(TimeControlCommand::StepTicks { ticks })
            .unwrap();
        assert!(!rejected.accepted);
        assert_eq!(
            rejected.rejection,
            Some(TimeControlRejection::InvalidStepCount)
        );
        assert_eq!(rejected.before, rejected.after);
    }
}

#[test]
fn speed_changes_cadence_metadata_not_explicit_tick_results() {
    let mut normal = init_bridge();
    let mut faster = init_bridge();
    let speed = faster
        .apply_time_control_command(TimeControlCommand::SetSpeedMultiplier { multiplier: 4 })
        .unwrap();
    assert!(speed.accepted);
    assert_eq!(speed.after.speed_multiplier, 4);

    let normal_step = normal
        .step_simulation(StepInputEnvelope { tick: 7 })
        .unwrap();
    let faster_step = faster
        .step_simulation(StepInputEnvelope { tick: 7 })
        .unwrap();
    assert_eq!(normal_step, faster_step);
    assert_eq!(normal_step.diff_count, 3);
}

#[test]
fn equivalent_time_commands_produce_deterministic_receipt_hashes() {
    let mut first = init_bridge();
    let mut second = init_bridge();
    let first_receipt = first
        .apply_time_control_command(TimeControlCommand::Pause)
        .unwrap();
    let second_receipt = second
        .apply_time_control_command(TimeControlCommand::Pause)
        .unwrap();
    assert_eq!(first_receipt, second_receipt);
}

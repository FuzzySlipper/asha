use super::*;

fn mode(mode: sim_runner::TimeControlMode) -> TimeControlMode {
    match mode {
        sim_runner::TimeControlMode::Paused => TimeControlMode::Paused,
        sim_runner::TimeControlMode::Running => TimeControlMode::Running,
    }
}

fn command(command: TimeControlCommand) -> sim_runner::TimeControlCommand {
    match command {
        TimeControlCommand::Pause => sim_runner::TimeControlCommand::Pause,
        TimeControlCommand::Resume => sim_runner::TimeControlCommand::Resume,
        TimeControlCommand::SetSpeedMultiplier { multiplier } => {
            sim_runner::TimeControlCommand::SetSpeedMultiplier { multiplier }
        }
        TimeControlCommand::StepTicks { ticks } => {
            sim_runner::TimeControlCommand::StepTicks { ticks }
        }
    }
}

fn rejection(rejection: sim_runner::TimeControlRejection) -> TimeControlRejection {
    match rejection {
        sim_runner::TimeControlRejection::AlreadyPaused => TimeControlRejection::AlreadyPaused,
        sim_runner::TimeControlRejection::AlreadyRunning => TimeControlRejection::AlreadyRunning,
        sim_runner::TimeControlRejection::NotPausedForExactStep => {
            TimeControlRejection::NotPausedForExactStep
        }
        sim_runner::TimeControlRejection::InvalidSpeedMultiplier => {
            TimeControlRejection::InvalidSpeedMultiplier
        }
        sim_runner::TimeControlRejection::InvalidStepCount => {
            TimeControlRejection::InvalidStepCount
        }
    }
}

fn state(bridge: &EngineBridge, authority_tick: u64) -> TimeControlState {
    let controller = bridge.time_controller.state();
    let mode = mode(controller.mode);
    let mode_label = match mode {
        TimeControlMode::Paused => "paused",
        TimeControlMode::Running => "running",
    };
    let state_hash = format!(
        "fnv1a64:{}",
        EngineBridge::fnv1a64(&format!(
            "{}|{}|{}|{}|{}",
            TIME_CONTROL_STATE_SCHEMA_VERSION,
            mode_label,
            controller.speed_multiplier,
            controller.revision,
            authority_tick
        ))
    );
    TimeControlState {
        schema_version: TIME_CONTROL_STATE_SCHEMA_VERSION,
        mode,
        speed_multiplier: controller.speed_multiplier,
        revision: controller.revision,
        authority_tick,
        state_hash,
    }
}

fn receipt_hash(
    accepted: bool,
    before: &TimeControlState,
    after: &TimeControlState,
    exact_ticks_advanced: u32,
    rejection: Option<TimeControlRejection>,
) -> String {
    format!(
        "fnv1a64:{}",
        EngineBridge::fnv1a64(&format!(
            "{}|{}|{}|{}|{:?}",
            accepted, before.state_hash, after.state_hash, exact_ticks_advanced, rejection
        ))
    )
}

pub(super) fn apply(
    bridge: &mut EngineBridge,
    requested: TimeControlCommand,
) -> BridgeResult<TimeControlReceipt> {
    bridge.require_initialized("apply_time_control_command")?;
    if let TimeControlCommand::StepTicks { ticks } = requested {
        if ticks > 0 && ticks <= sim_runner::MAX_EXACT_STEP_TICKS {
            bridge
                .authority_tick
                .checked_add(u64::from(ticks))
                .ok_or_else(|| {
                    RuntimeBridgeError::new(
                        RuntimeBridgeErrorKind::InvalidInput,
                        "exact time-control step would overflow the authority tick",
                    )
                })?;
        }
    }
    let before = state(bridge, bridge.authority_tick);
    let authority_receipt = bridge.time_controller.apply(command(requested));
    if authority_receipt.accepted {
        bridge.authority_tick = bridge
            .authority_tick
            .checked_add(u64::from(authority_receipt.exact_ticks_to_advance))
            .expect("valid exact step was overflow-checked before authority mutation");
    }
    let after = state(bridge, bridge.authority_tick);
    let rejection = authority_receipt.rejection.map(rejection);
    let exact_ticks_advanced = authority_receipt.exact_ticks_to_advance;
    Ok(TimeControlReceipt {
        accepted: authority_receipt.accepted,
        receipt_hash: receipt_hash(
            authority_receipt.accepted,
            &before,
            &after,
            exact_ticks_advanced,
            rejection,
        ),
        before,
        after,
        exact_ticks_advanced,
        rejection,
    })
}

pub(super) fn read(bridge: &EngineBridge) -> BridgeResult<TimeControlState> {
    bridge.require_initialized("read_time_control_state")?;
    Ok(state(bridge, bridge.authority_tick))
}

pub(super) fn step(
    bridge: &mut EngineBridge,
    input: StepInputEnvelope,
) -> BridgeResult<StepResult> {
    bridge.require_initialized("step_simulation")?;
    if bridge.time_controller.cadence_tick_budget() == 0 {
        return Ok(StepResult {
            tick: bridge.authority_tick,
            diff_count: 0,
        });
    }
    bridge.authority_tick = input.tick;
    Ok(StepResult {
        tick: bridge.authority_tick,
        diff_count: (bridge.authority_tick % 4) as u32,
    })
}

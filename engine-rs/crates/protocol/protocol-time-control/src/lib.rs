//! Generated-border contracts for fixed-tick simulation time control.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const TIME_CONTROL_STATE_SCHEMA_VERSION: u32 = 1;
pub const TIME_CONTROL_MODES: &[&str] = &["paused", "running"];
pub const TIME_CONTROL_REJECTIONS: &[&str] = &[
    "alreadyPaused",
    "alreadyRunning",
    "invalidSpeedMultiplier",
    "invalidStepCount",
    "notPausedForExactStep",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimeControlMode {
    Paused,
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "operation")]
pub enum TimeControlCommand {
    Pause,
    Resume,
    SetSpeedMultiplier { multiplier: u8 },
    StepTicks { ticks: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimeControlRejection {
    AlreadyPaused,
    AlreadyRunning,
    InvalidSpeedMultiplier,
    InvalidStepCount,
    NotPausedForExactStep,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimeControlState {
    pub schema_version: u32,
    pub mode: TimeControlMode,
    pub speed_multiplier: u8,
    pub revision: u64,
    pub authority_tick: u64,
    pub state_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimeControlReceipt {
    pub accepted: bool,
    pub before: TimeControlState,
    pub after: TimeControlState,
    pub exact_ticks_advanced: u32,
    pub rejection: Option<TimeControlRejection>,
    pub receipt_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_and_state_use_closed_camel_case_wire_shapes() {
        assert_eq!(
            serde_json::to_value(TimeControlCommand::StepTicks { ticks: 3 }).unwrap(),
            serde_json::json!({"operation": "stepTicks", "ticks": 3})
        );
        assert_eq!(
            serde_json::to_value(TimeControlMode::Paused).unwrap(),
            serde_json::json!("paused")
        );
    }
}

//! Generic process lifecycle rule.
//!
//! # Lane
//!
//! `rust-rule` — validates abstract process lifecycle operations and emits
//! authoritative [`DomainEvent`](core_events::DomainEvent) batches. The rule is
//! product-neutral: it knows process ids, mode ids, and generic terminal reasons,
//! not game nouns or renderer behavior.
//!
//! # Design
//!
//! `core-commands` currently exposes `Start`, `SetMode`, and `Stop`; `core-events`
//! records `ProcessStarted`, `ProcessModeSet`, and `ProcessStopped`. This crate
//! is the rule-owned validation surface around that vocabulary. It also models
//! higher-level terminal intent (`Interrupt`, `Cancel`, `Complete`, `Fail`) as
//! [`ProcessTerminalReason`] so callers do not encode those decisions in strings.
//! Until the state/event border carries terminal reasons, every accepted terminal
//! action emits the existing `ProcessStopped` event.

#![forbid(unsafe_code)]

use core_commands::ProcessCommand;
use core_error::ErrorCategory;
use core_events::{DomainEvent, EventBatch};
use core_ids::{ModeId, ProcessId};
use core_state::StateStore;

/// The observed authoritative state for a process id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedProcessState {
    /// No active process with this id exists.
    Absent,
    /// The process is active. `mode` is optional because `Start` creates a
    /// process before a mode is necessarily assigned.
    Running { mode: Option<ModeId> },
}

impl ObservedProcessState {
    pub fn from_store(store: &StateStore, id: ProcessId) -> Self {
        match store.process(id) {
            Some(record) => Self::Running { mode: record.mode },
            None => Self::Absent,
        }
    }
}

/// Generic terminal intent for stopping an active process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessTerminalReason {
    Stop,
    Interrupt,
    Cancel,
    Complete,
    Fail,
}

impl ProcessTerminalReason {
    pub fn label(self) -> &'static str {
        match self {
            ProcessTerminalReason::Stop => "stop",
            ProcessTerminalReason::Interrupt => "interrupt",
            ProcessTerminalReason::Cancel => "cancel",
            ProcessTerminalReason::Complete => "complete",
            ProcessTerminalReason::Fail => "fail",
        }
    }
}

/// A proposed lifecycle/mode transition for a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessAction {
    Start,
    SetMode { mode: ModeId },
    Terminate { reason: ProcessTerminalReason },
}

/// A typed process request before validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessRequest {
    pub id: ProcessId,
    pub action: ProcessAction,
}

impl ProcessRequest {
    pub const fn start(id: ProcessId) -> Self {
        Self {
            id,
            action: ProcessAction::Start,
        }
    }

    pub const fn set_mode(id: ProcessId, mode: ModeId) -> Self {
        Self {
            id,
            action: ProcessAction::SetMode { mode },
        }
    }

    pub const fn terminate(id: ProcessId, reason: ProcessTerminalReason) -> Self {
        Self {
            id,
            action: ProcessAction::Terminate { reason },
        }
    }
}

impl From<ProcessCommand> for ProcessRequest {
    fn from(command: ProcessCommand) -> Self {
        match command {
            ProcessCommand::Start { id } => ProcessRequest::start(id),
            ProcessCommand::SetMode { id, mode } => ProcessRequest::set_mode(id, mode),
            ProcessCommand::Stop { id } => {
                ProcessRequest::terminate(id, ProcessTerminalReason::Stop)
            }
        }
    }
}

/// Typed rejection from the process rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessRuleError {
    AlreadyRunning { id: ProcessId },
    NotRunning { id: ProcessId },
    ModeNotDefined { id: ModeId },
    ModeAlreadySet { id: ProcessId, mode: ModeId },
}

impl ProcessRuleError {
    pub fn category(self) -> ErrorCategory {
        match self {
            ProcessRuleError::AlreadyRunning { .. } | ProcessRuleError::ModeAlreadySet { .. } => {
                ErrorCategory::Conflict
            }
            ProcessRuleError::NotRunning { .. } | ProcessRuleError::ModeNotDefined { .. } => {
                ErrorCategory::NotFound
            }
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            ProcessRuleError::AlreadyRunning { .. } => "process_already_running",
            ProcessRuleError::NotRunning { .. } => "process_not_running",
            ProcessRuleError::ModeNotDefined { .. } => "mode_not_defined",
            ProcessRuleError::ModeAlreadySet { .. } => "mode_already_set",
        }
    }
}

impl core::fmt::Display for ProcessRuleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProcessRuleError::AlreadyRunning { id } => {
                write!(f, "process {} is already running", id.raw())
            }
            ProcessRuleError::NotRunning { id } => {
                write!(f, "process {} is not running", id.raw())
            }
            ProcessRuleError::ModeNotDefined { id } => {
                write!(f, "mode {} is not defined", id.raw())
            }
            ProcessRuleError::ModeAlreadySet { id, mode } => {
                write!(f, "process {} is already in mode {}", id.raw(), mode.raw())
            }
        }
    }
}

impl std::error::Error for ProcessRuleError {}

/// Validate a process command from the existing command border.
pub fn validate_process_command(
    store: &StateStore,
    command: &ProcessCommand,
) -> Result<EventBatch, ProcessRuleError> {
    validate_process_request(store, ProcessRequest::from(command.clone()))
}

/// Validate a typed process request and return the authoritative event batch.
pub fn validate_process_request(
    store: &StateStore,
    request: ProcessRequest,
) -> Result<EventBatch, ProcessRuleError> {
    let mut batch = EventBatch::new();
    match request.action {
        ProcessAction::Start => {
            if store.process(request.id).is_some() {
                return Err(ProcessRuleError::AlreadyRunning { id: request.id });
            }
            batch.push(DomainEvent::ProcessStarted { id: request.id });
        }
        ProcessAction::SetMode { mode } => {
            let observed = ObservedProcessState::from_store(store, request.id);
            match observed {
                ObservedProcessState::Absent => {
                    return Err(ProcessRuleError::NotRunning { id: request.id });
                }
                ObservedProcessState::Running {
                    mode: Some(current),
                } if current == mode => {
                    return Err(ProcessRuleError::ModeAlreadySet {
                        id: request.id,
                        mode,
                    });
                }
                ObservedProcessState::Running { .. } => {}
            }
            if store.mode(mode).is_none() {
                return Err(ProcessRuleError::ModeNotDefined { id: mode });
            }
            batch.push(DomainEvent::ProcessModeSet {
                id: request.id,
                mode,
            });
        }
        ProcessAction::Terminate { .. } => {
            if store.process(request.id).is_none() {
                return Err(ProcessRuleError::NotRunning { id: request.id });
            }
            batch.push(DomainEvent::ProcessStopped { id: request.id });
        }
    }
    Ok(batch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(id: u64) -> ProcessId {
        ProcessId::new(id)
    }

    fn mode(id: u64) -> ModeId {
        ModeId::new(id)
    }

    #[test]
    fn start_accepts_absent_process() {
        let store = StateStore::new();

        let batch = validate_process_request(&store, ProcessRequest::start(process(7))).unwrap();

        assert_eq!(
            batch.events(),
            &[DomainEvent::ProcessStarted { id: process(7) }]
        );
    }

    #[test]
    fn duplicate_start_is_rejected_without_events() {
        let mut store = StateStore::new();
        store.insert_process(process(7));

        let err = validate_process_request(&store, ProcessRequest::start(process(7))).unwrap_err();

        assert_eq!(err, ProcessRuleError::AlreadyRunning { id: process(7) });
        assert_eq!(err.category(), ErrorCategory::Conflict);
        assert_eq!(err.code(), "process_already_running");
    }

    #[test]
    fn set_mode_requires_running_process_and_defined_mode() {
        let mut store = StateStore::new();
        store.insert_process(process(7));

        let missing_mode =
            validate_process_request(&store, ProcessRequest::set_mode(process(7), mode(3)))
                .unwrap_err();
        assert_eq!(
            missing_mode,
            ProcessRuleError::ModeNotDefined { id: mode(3) }
        );

        store.insert_mode(mode(3));
        let batch = validate_process_request(&store, ProcessRequest::set_mode(process(7), mode(3)))
            .unwrap();

        assert_eq!(
            batch.events(),
            &[DomainEvent::ProcessModeSet {
                id: process(7),
                mode: mode(3)
            }]
        );
    }

    #[test]
    fn set_mode_rejects_missing_process_and_duplicate_mode() {
        let mut store = StateStore::new();
        store.insert_mode(mode(3));

        let missing =
            validate_process_request(&store, ProcessRequest::set_mode(process(7), mode(3)))
                .unwrap_err();
        assert_eq!(missing, ProcessRuleError::NotRunning { id: process(7) });

        store.insert_process(process(7));
        store.process_mut(process(7)).unwrap().mode = Some(mode(3));
        let duplicate =
            validate_process_request(&store, ProcessRequest::set_mode(process(7), mode(3)))
                .unwrap_err();
        assert_eq!(
            duplicate,
            ProcessRuleError::ModeAlreadySet {
                id: process(7),
                mode: mode(3)
            }
        );
    }

    #[test]
    fn terminal_reasons_share_existing_stopped_event() {
        let reasons = [
            ProcessTerminalReason::Stop,
            ProcessTerminalReason::Interrupt,
            ProcessTerminalReason::Cancel,
            ProcessTerminalReason::Complete,
            ProcessTerminalReason::Fail,
        ];

        for reason in reasons {
            let mut store = StateStore::new();
            store.insert_process(process(9));
            let request = ProcessRequest::terminate(process(9), reason);

            let batch = validate_process_request(&store, request).unwrap();

            assert_eq!(
                batch.events(),
                &[DomainEvent::ProcessStopped { id: process(9) }],
                "terminal reason {} should use the current process-stopped event border",
                reason.label()
            );
        }
    }

    #[test]
    fn terminal_action_rejects_absent_process() {
        let store = StateStore::new();

        let err = validate_process_request(
            &store,
            ProcessRequest::terminate(process(9), ProcessTerminalReason::Interrupt),
        )
        .unwrap_err();

        assert_eq!(err, ProcessRuleError::NotRunning { id: process(9) });
        assert_eq!(err.category(), ErrorCategory::NotFound);
    }

    #[test]
    fn process_command_adapter_preserves_existing_border() {
        let store = StateStore::new();
        let command = ProcessCommand::Start { id: process(2) };

        let batch = validate_process_command(&store, &command).unwrap();

        assert_eq!(
            batch.events(),
            &[DomainEvent::ProcessStarted { id: process(2) }]
        );
    }

    #[test]
    fn accepted_batches_are_deterministic_and_replay_shaped() {
        let mut store = StateStore::new();
        store.insert_process(process(1));
        store.insert_mode(mode(2));

        let a = validate_process_request(&store, ProcessRequest::set_mode(process(1), mode(2)))
            .unwrap();
        let b = validate_process_request(&store, ProcessRequest::set_mode(process(1), mode(2)))
            .unwrap();

        assert_eq!(a, b);
        assert_eq!(a.len(), 1);
    }
}

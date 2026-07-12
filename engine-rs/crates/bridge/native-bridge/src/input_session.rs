use napi_derive::napi;
use runtime_bridge_api::{
    InputContextCommand, InputSessionConfigureRequest, RawInputSample, RecordedInputAction,
    RuntimeBridge, RuntimeBridgeError, RuntimeBridgeErrorKind,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{to_napi, with_bridge};

fn parse_request<T: DeserializeOwned>(request_json: &str, operation: &str) -> napi::Result<T> {
    serde_json::from_str(request_json).map_err(|err| {
        to_napi(RuntimeBridgeError::new(
            RuntimeBridgeErrorKind::InvalidInput,
            format!("{operation} request is not valid JSON: {err}"),
        ))
    })
}

fn serialize_result<T: Serialize>(value: &T, operation: &str) -> napi::Result<String> {
    serde_json::to_string(value).map_err(|err| {
        to_napi(RuntimeBridgeError::new(
            RuntimeBridgeErrorKind::Internal,
            format!("{operation} result could not be serialized: {err}"),
        ))
    })
}

#[napi]
pub fn configure_input_session(handle: i64, request_json: String) -> napi::Result<String> {
    let request =
        parse_request::<InputSessionConfigureRequest>(&request_json, "configure input session")?;
    with_bridge(handle, |bridge| {
        let snapshot = bridge.configure_input_session(request).map_err(to_napi)?;
        serialize_result(&snapshot, "configure input session")
    })
}

#[napi]
pub fn apply_input_context_command(handle: i64, command_json: String) -> napi::Result<String> {
    let command =
        parse_request::<InputContextCommand>(&command_json, "apply input context command")?;
    with_bridge(handle, |bridge| {
        let receipt = bridge
            .apply_input_context_command(command)
            .map_err(to_napi)?;
        serialize_result(&receipt, "apply input context command")
    })
}

#[napi]
pub fn submit_raw_input(handle: i64, sample_json: String) -> napi::Result<String> {
    let sample = parse_request::<RawInputSample>(&sample_json, "submit raw input")?;
    with_bridge(handle, |bridge| {
        let receipt = bridge.submit_raw_input(sample).map_err(to_napi)?;
        serialize_result(&receipt, "submit raw input")
    })
}

#[napi]
pub fn replay_resolved_input_action(handle: i64, record_json: String) -> napi::Result<String> {
    let record =
        parse_request::<RecordedInputAction>(&record_json, "replay resolved input action")?;
    with_bridge(handle, |bridge| {
        let receipt = bridge
            .replay_resolved_input_action(record)
            .map_err(to_napi)?;
        serialize_result(&receipt, "replay resolved input action")
    })
}

#[napi]
pub fn read_input_context_state(handle: i64) -> napi::Result<String> {
    with_bridge(handle, |bridge| {
        let state = bridge.read_input_context_state().map_err(to_napi)?;
        serialize_result(&state, "read input context state")
    })
}

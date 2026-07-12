use super::*;

pub(super) fn extend_round_trip_coverage(coverage: &mut BTreeSet<String>) {
    coverage.extend([
        interface_coverage_key("telemetry", "LiveTelemetryMetric"),
        interface_coverage_key("telemetry", "LiveTelemetryDiagnostic"),
        interface_coverage_key("telemetry", "LiveTelemetrySnapshot"),
    ]);
}

#[test]
fn telemetry_rust_serialization_matches_ir_shape() {
    use protocol_telemetry::{
        LiveTelemetryCounter, LiveTelemetryDiagnostic, LiveTelemetryDiagnosticCode,
        LiveTelemetryMetric, LiveTelemetrySnapshot, TelemetryEnvelope, TelemetryEvent,
        TelemetryLevel, TelemetryMetric, TelemetryMetricKind, TelemetrySource, TELEMETRY_LEVELS,
        TELEMETRY_METRIC_KINDS, TELEMETRY_SOURCES,
    };

    let telemetry = module("telemetry");
    assert_eq!(
        string_enum_values(&telemetry, "TelemetrySource"),
        TELEMETRY_SOURCES
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    );
    assert_eq!(
        string_enum_values(&telemetry, "TelemetryLevel"),
        TELEMETRY_LEVELS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    );
    assert_eq!(
        string_enum_values(&telemetry, "TelemetryMetricKind"),
        TELEMETRY_METRIC_KINDS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    );

    let envelope = TelemetryEnvelope {
        protocol_version: 1,
        emitted_at_tick: 99,
        events: vec![TelemetryEvent::Metric {
            source: TelemetrySource::Runtime,
            level: TelemetryLevel::Info,
            sequence: 4,
            metric: TelemetryMetric {
                name: "frame.projection".to_string(),
                kind: TelemetryMetricKind::DurationMs,
                value: 2.5,
                unit: Some("ms".to_string()),
            },
        }],
    };
    let serialized = serde_json::to_value(&envelope).unwrap();
    compare_object_to_interface(&telemetry, "TelemetryEnvelope", &serialized).unwrap();
    compare_object_to_variant(
        &telemetry,
        "TelemetryEvent",
        "metric",
        &serialized["events"][0],
    )
    .unwrap();
    compare_object_to_interface(
        &telemetry,
        "TelemetryMetric",
        &serialized["events"][0]["metric"],
    )
    .unwrap();
    assert_eq!(serialized["protocolVersion"], json!(1));
    assert_eq!(serialized["emittedAtTick"], json!(99));
    assert_eq!(serialized["events"][0]["source"], json!("runtime"));
    assert_eq!(serialized["events"][0]["level"], json!("info"));
    assert_eq!(
        serialized["events"][0]["metric"]["kind"],
        json!("durationMs")
    );

    let trace = serde_json::to_value(TelemetryEvent::Trace {
        source: TelemetrySource::Policy,
        level: TelemetryLevel::Debug,
        sequence: 5,
        span: "tick".to_string(),
        message: "policy pass complete".to_string(),
    })
    .unwrap();
    compare_object_to_variant(&telemetry, "TelemetryEvent", "trace", &trace).unwrap();
    assert_eq!(trace["source"], json!("policy"));

    let snapshot = serde_json::to_value(LiveTelemetrySnapshot {
        schema_version: 1,
        authority_tick: 99,
        sample_sequence: 7,
        metrics: vec![LiveTelemetryMetric {
            counter: LiveTelemetryCounter::FrameTimeMs,
            kind: TelemetryMetricKind::DurationMs,
            value: 16.5,
            unit: "ms".into(),
        }],
        frame_time_history_ms: vec![15.0, 16.5],
        diagnostics: vec![LiveTelemetryDiagnostic {
            code: LiveTelemetryDiagnosticCode::CounterUnavailable,
            counter: Some(LiveTelemetryCounter::DrawCallCount),
            message: "draw call counter is unavailable".into(),
        }],
    })
    .unwrap();
    compare_object_to_interface(&telemetry, "LiveTelemetrySnapshot", &snapshot).unwrap();
    compare_object_to_interface(&telemetry, "LiveTelemetryMetric", &snapshot["metrics"][0])
        .unwrap();
    compare_object_to_interface(
        &telemetry,
        "LiveTelemetryDiagnostic",
        &snapshot["diagnostics"][0],
    )
    .unwrap();
}

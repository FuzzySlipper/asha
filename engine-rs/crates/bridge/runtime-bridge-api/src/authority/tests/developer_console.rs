use super::*;

fn detail(code: &str) -> DeveloperConsoleDetail {
    DeveloperConsoleDetail {
        code: code.to_owned(),
        operation: Some("test_operation".to_owned()),
        resource_kind: None,
        resource_id: None,
        reason: None,
    }
}

#[test]
fn initialization_exposes_a_typed_capability_record() {
    let bridge = init_bridge();
    let snapshot = bridge.read_developer_console().unwrap();
    assert_eq!(snapshot.schema_version, DEVELOPER_CONSOLE_SCHEMA_VERSION);
    assert_eq!(snapshot.records.len(), 1);
    assert_eq!(
        snapshot.records[0].category,
        DeveloperConsoleCategory::Capability
    );
    assert_eq!(snapshot.records[0].detail.code, "capability_attached");
    assert_eq!(snapshot.records[0].session.as_deref(), Some("engine:1"));
    assert!(snapshot.snapshot_hash.starts_with("fnv1a64:"));
    assert_eq!(
        serde_json::to_value(&snapshot).unwrap()["records"][0]["detail"]["code"],
        "capability_attached"
    );
}

#[test]
fn rejection_and_projection_degradation_reach_the_console() {
    let mut bridge = init_bridge();
    let result = bridge
        .submit_commands(CommandBatch {
            commands: vec![VoxelCommand::SetVoxel {
                grid: GridId::new(1),
                coord: VoxelCoord::new(0, 0, 0),
                value: VoxelValue::solid_raw(99),
            }],
        })
        .unwrap();
    assert_eq!(result.rejected, 1);

    bridge.projection.projection_frame = None;
    assert!(bridge.read_projection_frame(0).is_err());
    let snapshot = bridge.read_developer_console().unwrap();
    assert!(snapshot
        .records
        .iter()
        .any(|record| record.detail.code == "operation_rejected"));
    assert!(snapshot.records.iter().any(|record| {
        record.detail.code == "resource_degraded"
            && record.source == DeveloperConsoleSource::Projection
    }));
    assert!(snapshot
        .records
        .iter()
        .any(|record| record.detail.code == "capability_unavailable"));
}

#[test]
fn retention_rate_limit_order_and_hash_are_deterministic() {
    let bridge = init_bridge();
    bridge.reset_developer_console();
    for index in 0..(DEVELOPER_CONSOLE_MAX_RECORDS_PER_TICK + 4) {
        bridge.record_developer_console(DeveloperConsoleEmission {
            severity: DiagnosticSeverity::Info,
            category: DeveloperConsoleCategory::Gameplay,
            source: DeveloperConsoleSource::Authority,
            message: format!("same tick {index}"),
            correlation: Some(format!("correlation:{index}")),
            authority_tick: Some(7),
            detail: detail("rate_limited_test"),
        });
    }
    let limited = bridge.developer_console_snapshot();
    assert_eq!(
        limited.records.len(),
        DEVELOPER_CONSOLE_MAX_RECORDS_PER_TICK
    );
    assert_eq!(limited.dropped_record_count, 4);
    assert_eq!(limited.records[0].sequence, 0);
    assert_eq!(limited.records.last().unwrap().sequence, 15);
    assert_eq!(limited, bridge.developer_console_snapshot());

    for index in 0..(DEVELOPER_CONSOLE_MAX_RECORDS + 5) {
        bridge.record_developer_console(DeveloperConsoleEmission {
            severity: DiagnosticSeverity::Warning,
            category: DeveloperConsoleCategory::Runtime,
            source: DeveloperConsoleSource::RuntimeHost,
            message: format!("retained {index}"),
            correlation: None,
            authority_tick: Some(100 + index as u64),
            detail: detail("retention_test"),
        });
    }
    let retained = bridge.developer_console_snapshot();
    assert_eq!(retained.records.len(), DEVELOPER_CONSOLE_MAX_RECORDS);
    assert!(retained.dropped_record_count > 4);
    assert_eq!(
        retained.first_sequence,
        retained.records.first().map(|record| record.sequence)
    );
    assert!(retained
        .records
        .windows(2)
        .all(|records| records[0].sequence < records[1].sequence));
}

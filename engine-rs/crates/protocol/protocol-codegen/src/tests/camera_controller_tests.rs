use super::*;

pub(super) fn extend_round_trip_coverage(coverage: &mut BTreeSet<String>) {
    coverage.extend([
        interface_coverage_key("view", "CameraTransitionSpec"),
        variant_coverage_key("view", "CameraModeTarget", "firstPerson"),
        variant_coverage_key("view", "CameraModeTarget", "orbit"),
        variant_coverage_key("view", "CameraModeTarget", "topDown"),
        interface_coverage_key("view", "CameraModeCommand"),
        interface_coverage_key("view", "CameraControllerState"),
        interface_coverage_key("view", "CameraTransitionReadout"),
        interface_coverage_key("view", "CameraModeChangeReceipt"),
        interface_coverage_key("view", "CameraNavigationInput"),
        interface_coverage_key("view", "CameraNavigationInputEnvelope"),
        interface_coverage_key("view", "CameraNavigationReceipt"),
        interface_coverage_key("view", "CameraControllerReadRequest"),
    ]);
}

#[test]
fn camera_controller_rust_serialization_matches_ir_shape() {
    use protocol_view::{
        CameraBasis, CameraControllerReadRequest, CameraControllerRejection, CameraControllerState,
        CameraHandle, CameraMode, CameraModeChangeReceipt, CameraModeCommand, CameraModeTarget,
        CameraNavigationInput, CameraNavigationInputEnvelope, CameraNavigationReceipt, CameraPose,
        CameraSnapshot, CameraTransitionEasing, CameraTransitionReadout, CameraTransitionSpec,
        PerspectiveProjection, ViewportSize, CAMERA_CONTROLLER_STATE_SCHEMA_VERSION,
    };

    let view = module("view");
    assert_eq!(
        string_enum_values(&view, "CameraMode"),
        BTreeSet::from([
            "firstPerson".to_string(),
            "orbit".to_string(),
            "topDown".to_string(),
        ])
    );
    assert_eq!(
        string_enum_values(&view, "CameraTransitionEasing"),
        BTreeSet::from(["linear".to_string(), "smoothStep".to_string()])
    );
    assert_eq!(
        string_enum_values(&view, "CameraControllerRejection"),
        BTreeSet::from([
            "staleRevision".to_string(),
            "invalidTarget".to_string(),
            "incompatibleMode".to_string(),
            "invalidInput".to_string(),
            "terrainBlocked".to_string(),
        ])
    );

    let camera = CameraHandle::new(4);
    let pose = CameraPose {
        position: [3.0, 4.0, 5.0],
        yaw_degrees: 10.0,
        pitch_degrees: -20.0,
    };
    let snapshot = CameraSnapshot {
        camera,
        tick: 9,
        pose,
        basis: CameraBasis {
            forward: [0.0, 0.0, -1.0],
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
        },
        projection: PerspectiveProjection {
            fov_y_degrees: 60.0,
            near: 0.1,
            far: 500.0,
        },
        viewport: ViewportSize {
            width: 1280,
            height: 720,
        },
    };
    let transition_spec = CameraTransitionSpec {
        duration_milliseconds: 300,
        easing: CameraTransitionEasing::SmoothStep,
    };
    let input = CameraNavigationInput {
        pan_right: 1.0,
        pan_forward: -0.5,
        yaw_delta_degrees: 4.0,
        pitch_delta_degrees: -2.0,
        zoom_delta: 1.5,
        dt_seconds: 1.0 / 60.0,
        pan_speed_units_per_second: 8.0,
    };
    let state = CameraControllerState {
        schema_version: CAMERA_CONTROLLER_STATE_SCHEMA_VERSION,
        revision: 2,
        camera,
        mode: CameraMode::Orbit,
        pivot: Some([0.0, 1.0, 2.0]),
        distance: Some(6.0),
        min_distance: Some(2.0),
        max_distance: Some(20.0),
        snapshot,
        state_hash: "fnv1a64:1111111111111111".into(),
    };
    let transition = CameraTransitionReadout {
        from: snapshot,
        to: snapshot,
        duration_milliseconds: 300,
        easing: CameraTransitionEasing::SmoothStep,
        transition_hash: "fnv1a64:2222222222222222".into(),
    };

    for (tag, target) in [
        ("firstPerson", CameraModeTarget::FirstPerson { pose }),
        (
            "orbit",
            CameraModeTarget::Orbit {
                pivot: [0.0, 1.0, 2.0],
                distance: 6.0,
                min_distance: 2.0,
                max_distance: 20.0,
                yaw_degrees: 10.0,
                pitch_degrees: -20.0,
            },
        ),
        (
            "topDown",
            CameraModeTarget::TopDown {
                pivot: [0.0, 1.0, 2.0],
                height: 10.0,
                min_height: 3.0,
                max_height: 30.0,
                yaw_degrees: 0.0,
                pitch_degrees: -80.0,
            },
        ),
    ] {
        compare_object_to_variant(
            &view,
            "CameraModeTarget",
            tag,
            &serde_json::to_value(target).unwrap(),
        )
        .unwrap();
    }

    let mode_command = CameraModeCommand {
        camera,
        expected_revision: 1,
        target: CameraModeTarget::Orbit {
            pivot: [0.0, 1.0, 2.0],
            distance: 6.0,
            min_distance: 2.0,
            max_distance: 20.0,
            yaw_degrees: 10.0,
            pitch_degrees: -20.0,
        },
        transition: Some(transition_spec),
        tick: 9,
    };
    let mode_receipt = CameraModeChangeReceipt {
        accepted: true,
        before: state.clone(),
        after: state.clone(),
        transition: Some(transition),
        terrain_constrained: false,
        rejection: None,
        receipt_hash: "fnv1a64:3333333333333333".into(),
    };
    let navigation_envelope = CameraNavigationInputEnvelope {
        camera,
        expected_revision: 2,
        input,
        tick: 10,
    };
    let navigation_receipt = CameraNavigationReceipt {
        accepted: false,
        before: state.clone(),
        after: state.clone(),
        terrain_constrained: false,
        rejection: Some(CameraControllerRejection::StaleRevision),
        receipt_hash: "fnv1a64:4444444444444444".into(),
    };

    for (name, value) in [
        (
            "CameraTransitionSpec",
            serde_json::to_value(transition_spec).unwrap(),
        ),
        (
            "CameraModeCommand",
            serde_json::to_value(mode_command).unwrap(),
        ),
        (
            "CameraControllerState",
            serde_json::to_value(&state).unwrap(),
        ),
        (
            "CameraTransitionReadout",
            serde_json::to_value(mode_receipt.transition.as_ref().unwrap()).unwrap(),
        ),
        (
            "CameraModeChangeReceipt",
            serde_json::to_value(mode_receipt).unwrap(),
        ),
        (
            "CameraNavigationInput",
            serde_json::to_value(input).unwrap(),
        ),
        (
            "CameraNavigationInputEnvelope",
            serde_json::to_value(navigation_envelope).unwrap(),
        ),
        (
            "CameraNavigationReceipt",
            serde_json::to_value(navigation_receipt).unwrap(),
        ),
        (
            "CameraControllerReadRequest",
            serde_json::to_value(CameraControllerReadRequest { camera }).unwrap(),
        ),
    ] {
        compare_object_to_interface(&view, name, &value).unwrap();
    }
}

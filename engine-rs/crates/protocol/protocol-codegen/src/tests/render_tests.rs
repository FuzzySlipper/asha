use super::*;

pub(super) fn extend_round_trip_coverage(coverage: &mut BTreeSet<String>) {
    coverage.extend([
        interface_coverage_key("render", "SpatialGridSpec"),
        interface_coverage_key("render", "EditorGridStyle"),
        interface_coverage_key("render", "EditorGridDescriptor"),
        interface_coverage_key("render", "EditorGridBounds"),
        interface_coverage_key("render", "EditorGridProjectionReadout"),
        interface_coverage_key("render", "MaterialInstanceParameters"),
        variant_coverage_key("render", "RenderDiff", "setMaterialInstanceParameters"),
        interface_coverage_key("render", "LightShadowIntent"),
        variant_coverage_key("render", "LightDescriptor", "ambient"),
        variant_coverage_key("render", "LightDescriptor", "directional"),
        variant_coverage_key("render", "LightDescriptor", "point"),
        variant_coverage_key("render", "LightDescriptor", "spot"),
        variant_coverage_key("render", "RenderDiff", "createLight"),
        variant_coverage_key("render", "RenderDiff", "updateLight"),
    ]);
}

#[test]
fn editor_grid_descriptor_matches_public_y_up_shape() {
    let render = module("render");
    assert_eq!(
        string_enum_values(&render, "SpatialGridCoordinateSystem"),
        BTreeSet::from(["rightHandedYUp".to_owned()])
    );
    assert_eq!(
        string_enum_values(&render, "EditorGridPlane"),
        BTreeSet::from(["xy".to_owned(), "xz".to_owned(), "yz".to_owned()])
    );
    assert_eq!(
        string_enum_values(&render, "SpatialGridSnapAnchor"),
        BTreeSet::from(["boundary".to_owned(), "cellCenter".to_owned()])
    );
    let descriptor = json!({
        "visible": true,
        "grid": {
            "coordinateSystem": "rightHandedYUp",
            "origin": [0.25, 0.0, -0.5],
            "spacing": [0.5, 1.0, 0.25]
        },
        "plane": "xz",
        "snapAnchor": "cellCenter",
        "style": {
            "minorColor": [0.1, 0.2, 0.3, 0.45],
            "majorColor": [0.2, 0.4, 0.6, 0.8],
            "xAxisColor": [0.9, 0.2, 0.2, 1.0],
            "yAxisColor": [0.2, 0.9, 0.2, 1.0],
            "zAxisColor": [0.2, 0.4, 1.0, 1.0],
            "majorLineEvery": 4,
            "opacity": 0.85,
            "fadeStart": 12.0,
            "fadeEnd": 48.0
        }
    });
    compare_object_to_interface(&render, "EditorGridDescriptor", &descriptor).unwrap();
    compare_object_to_interface(&render, "SpatialGridSpec", &descriptor["grid"]).unwrap();
    compare_object_to_interface(&render, "EditorGridStyle", &descriptor["style"]).unwrap();
    let readout = json!({
        "descriptor": descriptor,
        "bounds": { "min": [-8.0, 0.0, -8.0], "max": [8.0, 0.0, 8.0] },
        "minorLineStep": 1,
        "renderedLineCount": 130
    });
    compare_object_to_interface(&render, "EditorGridProjectionReadout", &readout).unwrap();
    compare_object_to_interface(&render, "EditorGridBounds", &readout["bounds"]).unwrap();
}

#[test]
fn renderer_neutral_light_samples_match_render_ir_shape() {
    let render = module("render");
    assert_eq!(
        string_enum_values(&render, "LightShadowIntent"),
        BTreeSet::from(["disabled".to_owned(), "requested".to_owned()])
    );
    let samples = [
        (
            "ambient",
            json!({
                "kind": "ambient", "color": [0.2, 0.3, 0.4],
                "intensity": 0.5, "enabled": true, "shadowIntent": "disabled"
            }),
        ),
        (
            "directional",
            json!({
                "kind": "directional", "color": [1.0, 0.9, 0.8],
                "intensity": 2.0, "enabled": true, "direction": [-1.0, -2.0, -1.0],
                "shadowIntent": "requested"
            }),
        ),
        (
            "point",
            json!({
                "kind": "point", "color": [1.0, 0.4, 0.2],
                "intensity": 4.0, "enabled": true, "position": [2.0, 3.0, 4.0],
                "range": 12.0, "decay": 2.0, "shadowIntent": "disabled"
            }),
        ),
        (
            "spot",
            json!({
                "kind": "spot", "color": [0.4, 0.6, 1.0],
                "intensity": 6.0, "enabled": true, "position": [0.0, 8.0, 0.0],
                "direction": [0.0, -1.0, 0.0], "range": 20.0, "decay": 2.0,
                "outerAngleRadians": 0.7, "penumbra": 0.25,
                "shadowIntent": "requested"
            }),
        ),
    ];
    for (kind, sample) in &samples {
        compare_object_to_variant(&render, "LightDescriptor", kind, sample).unwrap();
    }
    compare_object_to_variant(
        &render,
        "RenderDiff",
        "createLight",
        &json!({
            "op": "createLight", "handle": 90, "parent": null,
            "light": samples[0].1
        }),
    )
    .unwrap();
    compare_object_to_variant(
        &render,
        "RenderDiff",
        "updateLight",
        &json!({
            "op": "updateLight", "handle": 90, "light": samples[1].1
        }),
    )
    .unwrap();
}

#[test]
fn material_feedback_fixture_matches_render_ir_shape() {
    let render = module("render");
    let fixture_path = repo_root().join("harness/fixtures/render-diffs/material-feedback.json");
    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(&fixture_path).unwrap_or_else(|err| {
            panic!(
                "failed to read material feedback render-diff fixture {}: {err}",
                fixture_path.display()
            )
        }),
    )
    .unwrap();
    let op = fixture["ops"]
        .as_array()
        .and_then(|ops| ops.last())
        .expect("material feedback fixture should end with an operation");
    compare_object_to_variant(&render, "RenderDiff", "setMaterialInstanceParameters", op).unwrap();
    compare_object_to_interface(&render, "MaterialInstanceParameters", &op["parameters"]).unwrap();
}

#[test]
fn animated_mesh_fixture_matches_render_ir_shape() {
    use protocol_render::{AnimatedMeshRuntimeFormat, AnimationLoopMode};

    let render = module("render");
    assert_eq!(
        string_enum_values(&render, "AnimatedMeshRuntimeFormat"),
        BTreeSet::from([AnimatedMeshRuntimeFormat::Glb.label().to_string()])
    );
    assert_eq!(
        string_enum_values(&render, "AnimationLoopMode"),
        BTreeSet::from([
            AnimationLoopMode::Once.label().to_string(),
            AnimationLoopMode::Repeat.label().to_string(),
            AnimationLoopMode::PingPong.label().to_string(),
        ])
    );

    let fixture_path = repo_root().join("harness/fixtures/render-diffs/animated-mesh.json");
    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(&fixture_path).unwrap_or_else(|err| {
            panic!(
                "failed to read animated mesh render-diff fixture {}: {err}",
                fixture_path.display()
            )
        }),
    )
    .unwrap();
    let ops = fixture["ops"]
        .as_array()
        .expect("animated mesh fixture should contain ops array");
    assert_eq!(ops.len(), 3);

    compare_object_to_variant(&render, "RenderDiff", "defineAnimatedMesh", &ops[0]).unwrap();
    compare_object_to_interface(&render, "AnimatedMeshAsset", &ops[0]["asset"]).unwrap();
    compare_object_to_interface(
        &render,
        "AnimationClipDescriptor",
        &ops[0]["asset"]["clips"][0],
    )
    .unwrap();
    assert_eq!(
        ops[0]["asset"]["runtimeFormat"],
        json!(AnimatedMeshRuntimeFormat::Glb.label())
    );
    assert_eq!(ops[0]["asset"]["defaultClip"], json!("idle"));

    compare_object_to_variant(&render, "RenderDiff", "createAnimatedMeshInstance", &ops[1])
        .unwrap();
    compare_object_to_interface(
        &render,
        "AnimatedMeshInstanceDescriptor",
        &ops[1]["instance"],
    )
    .unwrap();
    assert_eq!(ops[1]["instance"]["playback"], Value::Null);

    compare_object_to_variant(&render, "RenderDiff", "setAnimatedMeshPlayback", &ops[2]).unwrap();
    compare_object_to_variant(
        &render,
        "AnimatedMeshPlaybackCommand",
        "play",
        &ops[2]["playback"],
    )
    .unwrap();
    assert_eq!(
        ops[2]["playback"]["loop"],
        json!(AnimationLoopMode::Repeat.label())
    );

    let stop = json!({ "action": "stop", "fadeSeconds": 0.125 });
    compare_object_to_variant(&render, "AnimatedMeshPlaybackCommand", "stop", &stop).unwrap();
    let pause = json!({ "action": "pause" });
    compare_object_to_variant(&render, "AnimatedMeshPlaybackCommand", "pause", &pause).unwrap();
    let resume = json!({ "action": "resume" });
    compare_object_to_variant(&render, "AnimatedMeshPlaybackCommand", "resume", &resume).unwrap();
}

use super::*;

pub(super) fn extend_round_trip_coverage(coverage: &mut BTreeSet<String>) {
    coverage.extend([
        variant_coverage_key("scene", "SceneLight", "ambient"),
        variant_coverage_key("scene", "SceneLight", "directional"),
        variant_coverage_key("scene", "SceneLight", "point"),
        variant_coverage_key("scene", "SceneLight", "spot"),
    ]);
}

/// The scene vocabulary and durable document shapes must come through the
/// generated border without renderer-specific types.
#[test]
fn scene_family_emits_tags_codes_and_shapes() {
    let source = file("scene.ts");
    for tag in protocol_scene::SCENE_NODE_KIND_TAGS {
        assert!(
            source.contains(&format!("'{tag}'")),
            "missing node-kind tag {tag}"
        );
    }
    for code in protocol_scene::SCENE_VALIDATION_CODES {
        assert!(
            source.contains(&format!("'{code}'")),
            "missing validation code {code}"
        );
    }
    for code in protocol_scene::SCENE_DOCUMENT_CODEC_DIAGNOSTIC_CODES {
        assert!(
            source.contains(&format!("'{code}'")),
            "missing scene-document codec diagnostic code {code}"
        );
    }
    for shape in [
        "export type ProjectId =",
        "export type SceneId =",
        "export type RuntimeSessionId =",
        "export type SceneNodeId =",
        "export interface FlatSceneDocument {",
        "export interface SceneDocumentDecodeRequest {",
        "export interface SceneDocumentEncodeRequest {",
        "export interface SceneDocumentCodecResult {",
        "export interface SceneNodeRecord {",
        "export interface SceneValidationReport {",
        "export interface SceneSourceTrace {",
        "export interface BootstrapRecord {",
        "import type { EntityId } from './ids.js';",
    ] {
        assert!(
            source.contains(shape),
            "missing generated scene shape {shape}"
        );
    }
}

#[test]
fn stored_scene_light_samples_match_generated_ir_shape() {
    let scene = module("scene");
    assert_eq!(
        string_enum_values(&scene, "SceneLightShadowIntent"),
        BTreeSet::from(["disabled".to_owned(), "requested".to_owned()])
    );
    let samples = [
        (
            "ambient",
            json!({ "kind": "ambient", "color": [0.1, 0.2, 0.3], "intensity": 0.5, "enabled": true, "shadowIntent": "disabled" }),
        ),
        (
            "directional",
            json!({ "kind": "directional", "color": [1.0, 0.9, 0.8], "intensity": 2.0, "enabled": true, "shadowIntent": "requested" }),
        ),
        (
            "point",
            json!({ "kind": "point", "color": [1.0, 0.4, 0.2], "intensity": 4.0, "enabled": true, "range": 12.0, "decay": 2.0, "shadowIntent": "disabled" }),
        ),
        (
            "spot",
            json!({ "kind": "spot", "color": [0.4, 0.6, 1.0], "intensity": 6.0, "enabled": true, "range": null, "decay": 2.0, "outerAngleRadians": 0.7, "penumbra": 0.25, "shadowIntent": "requested" }),
        ),
    ];
    for (kind, sample) in samples {
        compare_object_to_variant(&scene, "SceneLight", kind, &sample).unwrap();
    }
}

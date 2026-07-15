use core_ids::{SceneId, SceneNodeId};
use core_math::Vec3;
use core_scene::{
    apply_scene_object_command, decode, encode, scene_object_snapshot, validate, FlatSceneDocument,
    NodeMetadata, Quat, SceneLight, SceneLightInvalid, SceneLightShadowIntent, SceneMetadata,
    SceneNodeKind, SceneNodeRecord, SceneObjectCommand, SceneTransform, SceneValidationError,
};

fn record(id: u64, light: SceneLight) -> SceneNodeRecord {
    SceneNodeRecord {
        id: SceneNodeId::new(id),
        parent: None,
        child_order: id as u32,
        transform: SceneTransform::IDENTITY,
        kind: SceneNodeKind::Light(light),
        metadata: NodeMetadata {
            label: Some(format!("light-{id}")),
            tags: vec![],
        },
    }
}

fn document() -> FlatSceneDocument {
    let disabled = SceneLightShadowIntent::Disabled;
    FlatSceneDocument {
        id: SceneId::new(7),
        schema_version: 2,
        metadata: SceneMetadata {
            name: Some("lights".into()),
            authoring_format_version: 2,
        },
        dependencies: vec![],
        nodes: vec![
            record(
                1,
                SceneLight::Ambient {
                    color: [0.1, 0.2, 0.3],
                    intensity: 0.4,
                    enabled: true,
                    shadow_intent: disabled,
                },
            ),
            record(
                2,
                SceneLight::Directional {
                    color: [1.0, 0.9, 0.8],
                    intensity: 2.0,
                    enabled: true,
                    shadow_intent: SceneLightShadowIntent::Requested,
                },
            ),
            record(
                3,
                SceneLight::Point {
                    color: [1.0, 0.3, 0.1],
                    intensity: 5.0,
                    enabled: true,
                    range: Some(9.0),
                    decay: 2.0,
                    shadow_intent: disabled,
                },
            ),
            record(
                4,
                SceneLight::Spot {
                    color: [0.2, 0.4, 1.0],
                    intensity: 7.0,
                    enabled: false,
                    range: None,
                    decay: 1.0,
                    outer_angle_radians: 0.7,
                    penumbra: 0.25,
                    shadow_intent: disabled,
                },
            ),
        ],
    }
}

#[test]
fn all_stored_light_kinds_round_trip_canonically() {
    let source = document();
    assert!(validate(&source).is_ok());
    let encoded = encode(&source);
    let decoded = decode(&encoded).expect("canonical light document decodes");
    assert_eq!(decoded, source.canonical());
    assert_eq!(encode(&decoded), encoded);
    assert_eq!(
        encoded,
        include_str!("../../../../../harness/fixtures/scenes/lights-v2.json")
    );
}

#[test]
fn stored_light_validation_classifies_values_and_scale() {
    let mut invalid = document();
    invalid.nodes[0].kind = SceneNodeKind::Light(SceneLight::Ambient {
        color: [2.0, 0.0, 0.0],
        intensity: 1.0,
        enabled: true,
        shadow_intent: SceneLightShadowIntent::Disabled,
    });
    invalid.nodes[1].transform =
        SceneTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::new(2.0, 1.0, 1.0));
    let errors = validate(&invalid).errors;
    assert!(errors.contains(&SceneValidationError::InvalidLight {
        node: SceneNodeId::new(1),
        reason: SceneLightInvalid::InvalidColor
    }));
    assert!(errors.contains(&SceneValidationError::InvalidLight {
        node: SceneNodeId::new(2),
        reason: SceneLightInvalid::NonUnitScale
    }));
}

#[test]
fn light_nodes_require_an_explicit_v2_document_without_migrating_v1() {
    let mut legacy = document();
    legacy.schema_version = 1;
    legacy.metadata.authoring_format_version = 1;
    assert!(validate(&legacy).errors.iter().any(|error| matches!(
        error,
        SceneValidationError::InvalidLight {
            reason: SceneLightInvalid::RequiresSchema2,
            ..
        }
    )));
    assert_eq!(legacy.schema_version, 1);
    assert_eq!(legacy.metadata.authoring_format_version, 1);
}

#[test]
fn scene_decode_rejects_unknown_nested_light_fields() {
    let encoded = encode(&document()).replacen(
        "\"shadowIntent\": \"disabled\"",
        "\"shadowIntent\": \"disabled\", \"rendererObject\": {}",
        1,
    );
    let error = decode(&encoded).expect_err("unknown light property fails closed");
    assert!(format!("{error:?}").contains("unknown field `rendererObject`"));
}

#[test]
fn typed_light_edit_changes_hash_and_preserves_node_identity() {
    let source = document();
    let before = scene_object_snapshot(&source).document_hash;
    let replacement = SceneLight::Point {
        color: [0.8, 0.7, 0.6],
        intensity: 9.0,
        enabled: true,
        range: Some(20.0),
        decay: 2.0,
        shadow_intent: SceneLightShadowIntent::Requested,
    };
    let outcome = apply_scene_object_command(
        &source,
        before,
        SceneObjectCommand::UpdateLight {
            id: SceneNodeId::new(3),
            light: replacement.clone(),
        },
    )
    .expect("valid typed light update");
    assert_ne!(outcome.snapshot.document_hash, before);
    assert_eq!(
        outcome.document.nodes[2].kind,
        SceneNodeKind::Light(replacement)
    );
}

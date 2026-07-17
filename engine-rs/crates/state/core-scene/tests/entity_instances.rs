use std::collections::BTreeSet;
use std::path::PathBuf;

use core_ids::{RuntimeSessionId, SceneId, SceneNodeId};
use core_math::Vec3;
use core_scene::{
    decode, encode, validate, BootstrapError, BootstrapPlan, BootstrapReferenceError,
    BootstrapResolutionContext, FlatSceneDocument, NodeMetadata, SceneBootstrapBindings,
    SceneCatalogBinding, SceneDecodeError, SceneEntityInstance, SceneEntityReference,
    SceneGeneratorBinding, SceneMetadata, SceneNodeKind, SceneNodeRecord, SceneTransform,
    SceneValidationError,
};

fn node(id: u64, parent: Option<u64>, kind: SceneNodeKind) -> SceneNodeRecord {
    SceneNodeRecord {
        id: SceneNodeId::new(id),
        parent: parent.map(SceneNodeId::new),
        child_order: 0,
        transform: SceneTransform::IDENTITY,
        kind,
        metadata: NodeMetadata::default(),
    }
}

fn canonical_instance_scene() -> FlatSceneDocument {
    let mut root = node(10, None, SceneNodeKind::EmptyGroup);
    root.transform.translation = Vec3::new(10.0, 0.0, 0.0);
    let mut player = node(
        20,
        Some(10),
        SceneNodeKind::EntityInstance(SceneEntityInstance {
            instance_id: "instance.player".into(),
            reference: SceneEntityReference::EntityDefinition {
                stable_id: "actor/demo-player".into(),
            },
            spawn_marker_id: Some("spawn.player.start".into()),
        }),
    );
    player.transform.translation = Vec3::new(1.0, 2.0, 3.0);
    FlatSceneDocument {
        id: SceneId::new(4103),
        schema_version: 3,
        metadata: SceneMetadata {
            name: Some("Generated tunnel room".into()),
            authoring_format_version: 3,
        },
        dependencies: vec![],
        nodes: vec![
            player,
            node(
                1,
                None,
                SceneNodeKind::Bootstrap(SceneBootstrapBindings {
                    generator: Some(SceneGeneratorBinding {
                        provider_id: "asha.generated-tunnel".into(),
                        preset_id: "tiny-enclosed".into(),
                        seed: 17,
                    }),
                    catalogs: vec![
                        SceneCatalogBinding {
                            binding_id: "spawns".into(),
                            catalog_id: "asha.generated-tunnel.spawns.v1".into(),
                            source_path: "catalogs/spawns/generated-tunnel.spawns.json".into(),
                        },
                        SceneCatalogBinding {
                            binding_id: "materials".into(),
                            catalog_id: "asha.generated-tunnel.materials.v1".into(),
                            source_path: "catalogs/materials/generated-tunnel.materials.json"
                                .into(),
                        },
                    ],
                }),
            ),
            root,
        ],
    }
}

fn resolution_context() -> BootstrapResolutionContext {
    BootstrapResolutionContext {
        entity_definition_ids: BTreeSet::from(["actor/demo-player".into()]),
        prefab_ids: BTreeSet::new(),
        spawn_marker_ids: BTreeSet::from(["spawn.player.start".into()]),
        generator_presets: BTreeSet::from([(
            "asha.generated-tunnel".into(),
            "tiny-enclosed".into(),
        )]),
        catalog_ids: BTreeSet::from([
            "asha.generated-tunnel.materials.v1".into(),
            "asha.generated-tunnel.spawns.v1".into(),
        ]),
    }
}

fn committed_fixture() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .ancestors()
        .find(|ancestor| ancestor.join("engine-rs").is_dir() && ancestor.join("harness").is_dir())
        .expect("repo root");
    std::fs::read_to_string(repo_root.join("harness/fixtures/scenes/entity-instance-v3.json"))
        .expect("read entity-instance-v3.json")
}

#[test]
fn schema_three_entity_instances_round_trip_canonically() {
    let source = encode(&canonical_instance_scene());
    assert_eq!(
        source,
        committed_fixture(),
        "canonical entity-instance fixture drifted"
    );
    let decoded = decode(&source).expect("canonical entity-instance scene decodes");
    assert!(validate(&decoded).is_ok());
    assert_eq!(encode(&decoded), source);
    assert!(source.contains("\"kind\": \"entityInstance\""));
    assert!(source.contains("\"kind\": \"bootstrap\""));
    assert!(source.find("\"materials\"").unwrap() < source.find("\"spawns\"").unwrap());
}

#[test]
fn legacy_demo_shape_has_a_distinct_migration_error() {
    let source = r#"{"kind":"SceneDocument","sceneId":"old","placements":[]}"#;
    assert_eq!(decode(source), Err(SceneDecodeError::LegacyDemoScene));
}

#[test]
fn resolved_bootstrap_is_atomic_and_retains_local_and_world_placement() {
    let doc = canonical_instance_scene();
    assert_eq!(
        BootstrapPlan::prepare(&doc, RuntimeSessionId::new(7)),
        Err(BootstrapError::ResolutionContextRequired)
    );

    let mut incomplete = resolution_context();
    incomplete.spawn_marker_ids.clear();
    let error = BootstrapPlan::prepare_resolved(&doc, RuntimeSessionId::new(7), &incomplete)
        .expect_err("unresolved marker rejects before state creation");
    assert!(matches!(
        error,
        BootstrapError::UnresolvedReferences { errors }
            if errors == vec![BootstrapReferenceError::UnknownSpawnMarker {
                node: SceneNodeId::new(20),
                marker_id: "spawn.player.start".into(),
            }]
    ));

    let plan =
        BootstrapPlan::prepare_resolved(&doc, RuntimeSessionId::new(7), &resolution_context())
            .expect("all stored references resolve");
    let instance = &plan.resolved_instances()[0];
    assert_eq!(
        instance.local_transform.translation,
        Vec3::new(1.0, 2.0, 3.0)
    );
    assert_eq!(
        instance.world_transform.translation,
        Vec3::new(11.0, 2.0, 3.0)
    );

    let (world, record) = plan.apply();
    assert_eq!(record.resolved_instances, plan.resolved_instances());
    assert_eq!(
        world.transform(instance.entity).unwrap().translation,
        Vec3::new(11.0, 2.0, 3.0)
    );
    assert_ne!(record.scene_content_hash.0, 0);
    assert_eq!(
        record.bootstrap_bindings.unwrap().catalogs[0].binding_id,
        "materials"
    );

    let mut reordered = doc.clone();
    reordered.nodes.reverse();
    let (_, reordered_record) = BootstrapPlan::prepare_resolved(
        &reordered,
        RuntimeSessionId::new(7),
        &resolution_context(),
    )
    .expect("canonical allocation ignores input node order")
    .apply();
    assert_eq!(
        reordered_record.scene_content_hash,
        record.scene_content_hash
    );
    assert_eq!(
        reordered_record.resolved_instances,
        record.resolved_instances
    );
}

#[test]
fn duplicate_instance_and_catalog_bindings_are_classified() {
    let mut doc = canonical_instance_scene();
    let mut duplicate = node(
        30,
        None,
        SceneNodeKind::EntityInstance(SceneEntityInstance {
            instance_id: "instance.player".into(),
            reference: SceneEntityReference::Prefab {
                prefab_id: 70,
                variant_id: Some("red".into()),
            },
            spawn_marker_id: None,
        }),
    );
    duplicate.child_order = 1;
    doc.nodes.push(duplicate);
    let SceneNodeKind::Bootstrap(bindings) = &mut doc.nodes[1].kind else {
        panic!("fixture bootstrap node");
    };
    bindings.catalogs.push(bindings.catalogs[0].clone());

    let errors = validate(&doc).errors;
    assert!(errors.iter().any(|error| matches!(
        error,
        SceneValidationError::DuplicateEntityInstanceId { instance_id, .. }
            if instance_id == "instance.player"
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        SceneValidationError::DuplicateCatalogBinding { binding_id, .. }
            if binding_id == "spawns"
    )));
}

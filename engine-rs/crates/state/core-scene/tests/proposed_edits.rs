//! Validation tests for the proposal-only scene authoring flow (#2380).
//!
//! These mirror the TS `@asha/editor-tools` scene-authoring proposals on the Rust
//! authority side: each test starts from a valid base document, applies the kind of
//! mutation a proposal would carry (add a node, reparent/group, set a transform,
//! point at the wrong-kind asset), and asserts that `validate` accepts good edits
//! and rejects bad ones with the classified error a UI would surface.
//!
//! Authority owns validation — TS proposals never decide acceptance; they reflect
//! the report produced here.

use core_assets::{markers, AssetId, AssetRef, AssetReference, AssetVersionReq};
use core_ids::{SceneId, SceneNodeId};
use core_scene::SceneTransform;
use core_scene::{
    validate, FlatSceneDocument, NodeMetadata, SceneMetadata, SceneNode, SceneNodeKind,
    SceneNodeRecord, SceneTree, SceneValidationError,
};

fn mesh_ref(id: &str) -> AssetReference {
    AssetRef::<markers::StaticMesh>::parse(id, AssetVersionReq::Any, None)
        .unwrap()
        .erase()
}

/// A valid base: root group (1) with a static-mesh child (2).
fn base_doc() -> FlatSceneDocument {
    let child = SceneNode::leaf(
        SceneNodeId::new(2),
        SceneNodeKind::StaticMesh(mesh_ref("mesh/static-mesh-fixture-a")),
    );
    let root =
        SceneNode::leaf(SceneNodeId::new(1), SceneNodeKind::EmptyGroup).with_children(vec![child]);
    SceneTree {
        id: SceneId::new(1001),
        schema_version: 1,
        metadata: SceneMetadata {
            name: Some("base".into()),
            authoring_format_version: 1,
        },
        dependencies: vec![mesh_ref("mesh/static-mesh-fixture-a")],
        roots: vec![root],
    }
    .to_flat()
}

fn record_index(doc: &FlatSceneDocument, id: u64) -> usize {
    doc.nodes.iter().position(|n| n.id.raw() == id).unwrap()
}

#[test]
fn base_document_is_valid() {
    assert!(validate(&base_doc()).is_ok());
}

#[test]
fn proposed_add_group_node_is_accepted() {
    // Proposal: add an empty group (3) under the root (1).
    let mut doc = base_doc();
    doc.nodes.push(SceneNodeRecord {
        id: SceneNodeId::new(3),
        parent: Some(SceneNodeId::new(1)),
        child_order: 1,
        transform: SceneTransform::IDENTITY,
        kind: SceneNodeKind::EmptyGroup,
        metadata: NodeMetadata::default(),
    });
    assert!(validate(&doc).is_ok());
}

#[test]
fn proposed_add_static_mesh_node_is_accepted() {
    // Proposal: add a static-mesh node (3) bound to a mesh asset.
    let mut doc = base_doc();
    doc.nodes.push(SceneNodeRecord {
        id: SceneNodeId::new(3),
        parent: Some(SceneNodeId::new(1)),
        child_order: 1,
        transform: SceneTransform::IDENTITY,
        kind: SceneNodeKind::StaticMesh(mesh_ref("mesh/static-mesh-fixture-b")),
        metadata: NodeMetadata::default(),
    });
    assert!(validate(&doc).is_ok());
}

#[test]
fn proposed_duplicate_id_add_is_rejected() {
    // Proposal: add a node reusing an existing id (2).
    let mut doc = base_doc();
    doc.nodes.push(SceneNodeRecord {
        id: SceneNodeId::new(2),
        parent: Some(SceneNodeId::new(1)),
        child_order: 1,
        transform: SceneTransform::IDENTITY,
        kind: SceneNodeKind::EmptyGroup,
        metadata: NodeMetadata::default(),
    });
    let report = validate(&doc);
    assert!(report
        .errors
        .iter()
        .any(|e| matches!(e, SceneValidationError::DuplicateNodeId { id } if id.raw() == 2)));
}

#[test]
fn proposed_reparent_to_absent_parent_is_rejected() {
    // Proposal: reparent node 2 under an absent parent (99).
    let mut doc = base_doc();
    let i = record_index(&doc, 2);
    doc.nodes[i].parent = Some(SceneNodeId::new(99));
    let report = validate(&doc);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        SceneValidationError::UnknownParent { node, parent }
            if node.raw() == 2 && parent.raw() == 99
    )));
}

#[test]
fn proposed_reparent_forming_a_cycle_is_rejected() {
    // Proposal: reparent the root (1) under its descendant (2): 1 -> 2 -> 1.
    let mut doc = base_doc();
    let i = record_index(&doc, 1);
    doc.nodes[i].parent = Some(SceneNodeId::new(2));
    let report = validate(&doc);
    let cycle = report
        .errors
        .iter()
        .find_map(|e| match e {
            SceneValidationError::Cycle { path } => Some(path.clone()),
            _ => None,
        })
        .expect("cycle reported");
    assert_eq!(cycle.len(), 2);
}

#[test]
fn proposed_wrong_kind_asset_is_rejected() {
    // Proposal: add a static-mesh node pointing at a material asset id.
    let bad = AssetReference::new(
        AssetId::parse("material/concrete-wet").unwrap(),
        AssetVersionReq::Any,
        None,
    );
    let mut doc = base_doc();
    doc.nodes.push(SceneNodeRecord {
        id: SceneNodeId::new(3),
        parent: Some(SceneNodeId::new(1)),
        child_order: 1,
        transform: SceneTransform::IDENTITY,
        kind: SceneNodeKind::StaticMesh(bad),
        metadata: NodeMetadata::default(),
    });
    let report = validate(&doc);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        SceneValidationError::AssetKindMismatch {
            expected: core_assets::AssetKind::StaticMesh,
            actual: core_assets::AssetKind::Material,
            ..
        }
    )));
}

#[test]
fn proposed_set_invalid_transform_is_rejected() {
    // Proposal: set a transform with a zero-scale axis on node 2.
    let mut doc = base_doc();
    let i = record_index(&doc, 2);
    doc.nodes[i].transform = SceneTransform {
        scale: core_math::Vec3::new(0.0, 1.0, 1.0),
        ..SceneTransform::IDENTITY
    };
    let report = validate(&doc);
    assert!(report.errors.iter().any(
        |e| matches!(e, SceneValidationError::InvalidTransform { node, .. } if node.raw() == 2)
    ));
}

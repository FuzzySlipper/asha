//! Regenerator for the committed canonical scene golden fixture.
//!
//! Run with `cargo run -p core-scene --example dump_canonical_scene` and redirect
//! into `harness/fixtures/scenes/sample-flat.json`. The `golden` integration test
//! pins the committed bytes against this same builder so drift fails CI.

use core_assets::{markers, AssetRef, AssetReference, AssetVersionReq};
use core_ids::{SceneId, SceneNodeId};
use core_math::Vec3;
use core_scene::{
    encode, NodeMetadata, SceneMetadata, SceneNode, SceneNodeKind, SceneTransform, SceneTree,
};

fn mesh_ref(id: &str) -> AssetReference {
    AssetRef::<markers::StaticMesh>::parse(id, AssetVersionReq::Any, None)
        .unwrap()
        .erase()
}

pub fn sample_tree() -> SceneTree {
    let child_a = SceneNode {
        id: SceneNodeId::new(2),
        transform: SceneTransform {
            translation: Vec3::new(1.0, 0.0, 0.0),
            ..SceneTransform::IDENTITY
        },
        kind: SceneNodeKind::StaticMesh(mesh_ref("mesh/static-mesh-fixture-a")),
        metadata: NodeMetadata {
            label: Some("mesh-a".into()),
            tags: vec!["b-tag".into(), "a-tag".into()],
        },
        children: vec![],
    };
    let grandchild = SceneNode::leaf(SceneNodeId::new(4), SceneNodeKind::EmptyGroup);
    let child_b = SceneNode::leaf(SceneNodeId::new(3), SceneNodeKind::EmptyGroup)
        .with_children(vec![grandchild]);
    let root = SceneNode::leaf(SceneNodeId::new(1), SceneNodeKind::EmptyGroup)
        .with_children(vec![child_a, child_b]);

    SceneTree {
        id: SceneId::new(100),
        schema_version: 1,
        metadata: SceneMetadata {
            name: Some("sample".into()),
            authoring_format_version: 1,
        },
        dependencies: vec![mesh_ref("mesh/static-mesh-fixture-a")],
        roots: vec![root],
    }
}

fn main() {
    print!("{}", encode(&sample_tree().to_flat()));
}

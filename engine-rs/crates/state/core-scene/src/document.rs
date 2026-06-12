//! Scene document representations: the ergonomic authoring **tree** and the
//! canonical **flat** form, plus deterministic conversions between them.
//!
//! Per scene-capability-01: the tree is for authoring/visualization, the flat
//! `parent_id` form is the canonical serialized/validated truth. Conversions are
//! deterministic and preserve authoring order via [`SceneNodeRecord::child_order`].

use core_assets::{AssetKind, AssetReference};
use core_ids::{SceneId, SceneNodeId};

use crate::transform::SceneTransform;

/// What a scene node *is*. Asset-backed variants carry a kind-erased
/// [`AssetReference`]; [`crate::validate`] checks the reference's kind matches
/// the variant, which is how a wrong-kind asset reference is rejected.
///
/// Kept intentionally small. Further variants (spawn templates, lights, …) are
/// gated on their own capability docs and validation plans (scene-capability-01,
/// recommendation 7) rather than being added ad hoc to satisfy a local task.
#[derive(Debug, Clone, PartialEq)]
pub enum SceneNodeKind {
    /// A pure grouping/transform node with no asset.
    EmptyGroup,
    /// References an authored static mesh asset.
    StaticMesh(AssetReference),
    /// References a sprite asset.
    Sprite(AssetReference),
    /// References an authored voxel volume asset.
    VoxelVolume(AssetReference),
}

impl SceneNodeKind {
    /// The asset kind this node variant must reference, if any.
    pub fn expected_asset_kind(&self) -> Option<AssetKind> {
        match self {
            SceneNodeKind::EmptyGroup => None,
            SceneNodeKind::StaticMesh(_) => Some(AssetKind::StaticMesh),
            SceneNodeKind::Sprite(_) => Some(AssetKind::Sprite),
            SceneNodeKind::VoxelVolume(_) => Some(AssetKind::VoxelVolume),
        }
    }

    /// The asset reference carried by this node, if any.
    pub fn asset(&self) -> Option<&AssetReference> {
        match self {
            SceneNodeKind::EmptyGroup => None,
            SceneNodeKind::StaticMesh(a)
            | SceneNodeKind::Sprite(a)
            | SceneNodeKind::VoxelVolume(a) => Some(a),
        }
    }

    /// Stable tag used in canonical serialization and diagnostics.
    pub fn tag(&self) -> &'static str {
        match self {
            SceneNodeKind::EmptyGroup => "emptyGroup",
            SceneNodeKind::StaticMesh(_) => "staticMesh",
            SceneNodeKind::Sprite(_) => "sprite",
            SceneNodeKind::VoxelVolume(_) => "voxelVolume",
        }
    }
}

/// Debug/authoring metadata that never affects authority semantics.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeMetadata {
    /// Human-readable debug label.
    pub label: Option<String>,
    /// Free-form debug tags (sorted in canonical form for determinism).
    pub tags: Vec<String>,
}

/// Document-level metadata.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SceneMetadata {
    /// Human-readable scene name.
    pub name: Option<String>,
    /// Version of the authoring format the document was written in.
    pub authoring_format_version: u32,
}

// ── Authoring tree form ───────────────────────────────────────────────────────

/// One node in the ergonomic authoring tree. Children are an ordered list; that
/// order is the authoring intent the flat form preserves.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneNode {
    pub id: SceneNodeId,
    pub transform: SceneTransform,
    pub kind: SceneNodeKind,
    pub metadata: NodeMetadata,
    pub children: Vec<SceneNode>,
}

impl SceneNode {
    /// A leaf node with identity transform and no metadata.
    pub fn leaf(id: SceneNodeId, kind: SceneNodeKind) -> Self {
        Self {
            id,
            transform: SceneTransform::IDENTITY,
            kind,
            metadata: NodeMetadata::default(),
            children: Vec::new(),
        }
    }

    /// Replace this node's children (builder-style).
    pub fn with_children(mut self, children: Vec<SceneNode>) -> Self {
        self.children = children;
        self
    }
}

/// The authoring/visualization tree document. Not authority/runtime truth — it
/// flattens to a [`FlatSceneDocument`] for validation and serialization.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneTree {
    pub id: SceneId,
    pub schema_version: u32,
    pub metadata: SceneMetadata,
    pub dependencies: Vec<AssetReference>,
    pub roots: Vec<SceneNode>,
}

impl SceneTree {
    /// Flatten to the canonical form, assigning each node its `parent_id` and the
    /// `child_order` index it held among its siblings. Deterministic: a pre-order
    /// walk yields records in a stable order; the flat document re-sorts by
    /// stable id for its canonical layout.
    pub fn to_flat(&self) -> FlatSceneDocument {
        let mut nodes = Vec::new();
        for (order, root) in self.roots.iter().enumerate() {
            flatten_into(&mut nodes, root, None, order as u32);
        }
        FlatSceneDocument {
            id: self.id,
            schema_version: self.schema_version,
            metadata: self.metadata.clone(),
            dependencies: self.dependencies.clone(),
            nodes,
        }
    }
}

/// Total nodes in a forest, counting children recursively.
fn count_nodes(nodes: &[SceneNode]) -> usize {
    nodes.iter().map(|n| 1 + count_nodes(&n.children)).sum()
}

fn flatten_into(
    out: &mut Vec<SceneNodeRecord>,
    node: &SceneNode,
    parent: Option<SceneNodeId>,
    child_order: u32,
) {
    out.push(SceneNodeRecord {
        id: node.id,
        parent,
        child_order,
        transform: node.transform,
        kind: node.kind.clone(),
        metadata: node.metadata.clone(),
    });
    for (order, child) in node.children.iter().enumerate() {
        flatten_into(out, child, Some(node.id), order as u32);
    }
}

// ── Flat canonical form ───────────────────────────────────────────────────────

/// One node as stored in the canonical flat document.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneNodeRecord {
    pub id: SceneNodeId,
    /// Parent node id, or `None` for a root.
    pub parent: Option<SceneNodeId>,
    /// Authoring order among siblings, preserved for tree reconstruction.
    pub child_order: u32,
    pub transform: SceneTransform,
    pub kind: SceneNodeKind,
    pub metadata: NodeMetadata,
}

/// The canonical, flat, validation/serialization form of a scene document.
#[derive(Debug, Clone, PartialEq)]
pub struct FlatSceneDocument {
    pub id: SceneId,
    pub schema_version: u32,
    pub metadata: SceneMetadata,
    pub dependencies: Vec<AssetReference>,
    /// Node records. Canonical order is ascending stable id (see
    /// [`FlatSceneDocument::canonicalize`]).
    pub nodes: Vec<SceneNodeRecord>,
}

impl FlatSceneDocument {
    /// Sort nodes into canonical order (ascending stable id) and sort metadata
    /// tags/dependencies so two documents with the same content serialize
    /// byte-identically regardless of authoring order.
    pub fn canonicalize(&mut self) {
        self.nodes.sort_by_key(|n| n.id.raw());
        self.dependencies
            .sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
        for node in &mut self.nodes {
            node.metadata.tags.sort();
        }
    }

    /// A canonicalized clone.
    pub fn canonical(&self) -> FlatSceneDocument {
        let mut doc = self.clone();
        doc.canonicalize();
        doc
    }

    /// Reconstruct the authoring tree/view. Siblings are ordered by
    /// `(child_order, stable_id)` so the result is deterministic even if two
    /// siblings share a `child_order`. Returns `None` if the records do not form
    /// a single coherent forest (unknown parent or a cycle) — callers should run
    /// [`crate::validate`] first to get a classified reason.
    pub fn to_tree(&self) -> Option<SceneTree> {
        use std::collections::BTreeMap;

        let known: std::collections::HashSet<u64> = self.nodes.iter().map(|n| n.id.raw()).collect();
        // Bucket child indices by parent (None = roots), keyed for stable order.
        let mut children_of: BTreeMap<Option<u64>, Vec<usize>> = BTreeMap::new();
        for (i, rec) in self.nodes.iter().enumerate() {
            if let Some(p) = rec.parent {
                if !known.contains(&p.raw()) {
                    return None; // unknown parent
                }
            }
            children_of
                .entry(rec.parent.map(|p| p.raw()))
                .or_default()
                .push(i);
        }

        // Build each node recursively, guarding against cycles via a visited set.
        let mut visiting = std::collections::HashSet::new();
        let roots = self.build_children(None, &children_of, &mut visiting)?;

        // A node trapped in a cycle (or under an unknown parent) is never reached
        // from a root; if any record went unplaced the records are not a coherent
        // forest. Compare against the unique id count so duplicate ids don't mask
        // an unplaced node.
        let unique_ids: std::collections::HashSet<u64> =
            self.nodes.iter().map(|n| n.id.raw()).collect();
        if count_nodes(&roots) != unique_ids.len() {
            return None;
        }

        Some(SceneTree {
            id: self.id,
            schema_version: self.schema_version,
            metadata: self.metadata.clone(),
            dependencies: self.dependencies.clone(),
            roots,
        })
    }

    fn build_children(
        &self,
        parent: Option<u64>,
        children_of: &std::collections::BTreeMap<Option<u64>, Vec<usize>>,
        visiting: &mut std::collections::HashSet<u64>,
    ) -> Option<Vec<SceneNode>> {
        let Some(idxs) = children_of.get(&parent) else {
            return Some(Vec::new());
        };
        let mut ordered: Vec<usize> = idxs.clone();
        ordered.sort_by_key(|&i| (self.nodes[i].child_order, self.nodes[i].id.raw()));

        let mut out = Vec::with_capacity(ordered.len());
        for i in ordered {
            let rec = &self.nodes[i];
            if !visiting.insert(rec.id.raw()) {
                return None; // cycle
            }
            let children = self.build_children(Some(rec.id.raw()), children_of, visiting)?;
            visiting.remove(&rec.id.raw());
            out.push(SceneNode {
                id: rec.id,
                transform: rec.transform,
                kind: rec.kind.clone(),
                metadata: rec.metadata.clone(),
                children,
            });
        }
        Some(out)
    }
}

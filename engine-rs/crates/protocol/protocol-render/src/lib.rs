//! Retained-mode render diff shapes for the ASHA generated-contract boundary.
//!
//! # Lane
//!
//! `contract-steward` — owns the border shape the authority core uses to drive a
//! retained-mode renderer. May depend on `core-ids` and `core-error`; it must
//! **not** depend on `core-state` or `sim-kernel`, because the border describes
//! *what changed on screen*, never *why the world changed*.
//!
//! # Border ownership
//!
//! A retained-mode renderer keeps a long-lived scene of nodes addressed by
//! stable [`RenderHandle`]s. Each tick the authority core emits a
//! [`RenderFrameDiff`]: a list of create / update / destroy operations against
//! those handles. The renderer applies the diff; it never reconstructs the
//! scene from scratch.
//!
//! These are the shapes Phase 2 codegen turns into TypeScript so a renderer
//! bridge can consume diffs in a type-safe way.
//!
//! # Abstract renderables
//!
//! Phase 5 fixes the vocabulary to *abstract* renderables only: a node is a
//! [`Geometry`] primitive (cube, sphere, quad, point, line) with a placeholder
//! [`Material`], a [`Transform`], a visibility flag, a [`RenderLayer`]
//! (scene vs. debug overlay), and [`RenderMetadata`] (source entity, tags,
//! label). [`Material`] is deliberately a placeholder (flat colour + wireframe);
//! there is no texture/shader system here, and no product-domain geometry.
//!
//! # Forbidden convenience logic
//!
//! No renderer behavior: no scene application, no interpolation, no transform
//! math, no diffing of two scenes. This crate defines the wire shape of a diff
//! and nothing that acts on one.

#![forbid(unsafe_code)]

use core_ids::{EntityId, TagId};

// ── Handles ───────────────────────────────────────────────────────────────────

/// Stable identifier for a node in the retained render scene.
///
/// A handle is allocated when a node is created and stays valid until the node
/// is destroyed. It is distinct from an [`EntityId`]: many render nodes may
/// project a single sim entity, and some nodes (overlays, gizmos) project none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderHandle(pub u64);

impl RenderHandle {
    #[inline]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

// ── Transform ─────────────────────────────────────────────────────────────────

/// Minimal affine transform for a render node.
///
/// Translation, a quaternion rotation, and a non-uniform scale. Enough to place
/// a node; deliberately not a full transform hierarchy or matrix type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: [f32; 3],
    /// Rotation quaternion in `[x, y, z, w]` order.
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Transform {
    /// The identity transform: origin, no rotation, unit scale.
    pub const IDENTITY: Transform = Transform {
        translation: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    };
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

// ── Geometry ──────────────────────────────────────────────────────────────────

/// An abstract primitive shape. Concrete extents come from the node's
/// [`Transform`] scale; primitives are unit-sized in local space.
///
/// This is intentionally a tiny, product-agnostic vocabulary — enough to draw
/// boxes, markers, and debug lines, not a mesh/asset system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Geometry {
    /// A unit cube.
    Cube,
    /// A unit sphere.
    Sphere,
    /// A flat unit quad (e.g. a ground tile or billboard backing).
    Quad,
    /// A single point marker.
    Point,
    /// A line segment between two local-space endpoints (debug overlays).
    Line { a: [f32; 3], b: [f32; 3] },
}

// ── Material ──────────────────────────────────────────────────────────────────

/// Placeholder visual appearance for a node: a flat linear-RGBA colour and an
/// optional wireframe flag. No textures, shaders, or PBR — that is out of scope
/// for the abstract border.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Material {
    /// Linear RGBA, each component in `0.0..=1.0`.
    pub color: [f32; 4],
    /// Draw as wireframe (common for debug overlays).
    pub wireframe: bool,
}

impl Material {
    /// Opaque white, filled.
    pub const DEFAULT: Material = Material {
        color: [1.0, 1.0, 1.0, 1.0],
        wireframe: false,
    };
}

impl Default for Material {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ── Layer ─────────────────────────────────────────────────────────────────────

/// Which retained layer a node belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderLayer {
    /// The main projected scene.
    #[default]
    Scene,
    /// A debug overlay drawn on top of the scene (gizmos, labels, lines).
    Debug,
}

// ── Metadata ──────────────────────────────────────────────────────────────────

/// Descriptive metadata carried on a render node.
///
/// Links a node back to the abstract sim vocabulary (an optional source entity
/// and any descriptive tags) plus a human label for inspection/overlay text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderMetadata {
    /// The sim entity this node visualizes, if any.
    pub source: Option<EntityId>,
    /// Descriptive tags, in ascending order.
    pub tags: Vec<TagId>,
    /// Optional human-readable label (also used as overlay text).
    pub label: Option<String>,
}

// ── Node ──────────────────────────────────────────────────────────────────────

/// The full description of a node at creation time.
///
/// Geometry is fixed for a node's lifetime — changing the primitive means
/// destroy + create. Everything else (transform, material, visibility,
/// metadata) is independently mutable via [`RenderDiff::Update`].
#[derive(Debug, Clone, PartialEq)]
pub struct RenderNode {
    pub geometry: Geometry,
    pub material: Material,
    pub transform: Transform,
    pub visible: bool,
    pub layer: RenderLayer,
    pub metadata: RenderMetadata,
}

impl RenderNode {
    /// A visible scene node with the given geometry and otherwise default
    /// transform/material/metadata.
    pub fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            material: Material::DEFAULT,
            transform: Transform::IDENTITY,
            visible: true,
            layer: RenderLayer::Scene,
            metadata: RenderMetadata::default(),
        }
    }
}

// ── Diff operations ───────────────────────────────────────────────────────────

/// A single retained-mode change against the render scene.
///
/// `Update` carries optional fields so a tick can change only a transform, only
/// visibility, only material, or only metadata, without re-sending the node.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderDiff {
    /// Introduce a new node, optionally parented under an existing one.
    Create {
        handle: RenderHandle,
        parent: Option<RenderHandle>,
        node: RenderNode,
    },
    /// Mutate an existing node's mutable facets.
    Update {
        handle: RenderHandle,
        transform: Option<Transform>,
        material: Option<Material>,
        visible: Option<bool>,
        metadata: Option<RenderMetadata>,
    },
    /// Remove a node (and, by renderer convention, its descendants).
    Destroy { handle: RenderHandle },
}

/// All retained-mode changes emitted for a single tick, in apply order.
///
/// Order is significant: a `Create` of a parent must precede a `Create` of its
/// child, and a `Destroy` is the last word on a handle within the frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderFrameDiff {
    pub ops: Vec<RenderDiff>,
}

impl RenderFrameDiff {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, op: RenderDiff) {
        self.ops.push(op);
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_roundtrip_and_distinct_from_entity() {
        let h = RenderHandle::new(7);
        assert_eq!(h.raw(), 7);
        let meta = RenderMetadata {
            source: Some(EntityId::new(7)),
            ..RenderMetadata::default()
        };
        assert_eq!(meta.source, Some(EntityId::new(7)));
    }

    #[test]
    fn defaults_are_sensible() {
        assert_eq!(Transform::default(), Transform::IDENTITY);
        assert_eq!(Material::default(), Material::DEFAULT);
        assert_eq!(RenderLayer::default(), RenderLayer::Scene);

        let node = RenderNode::new(Geometry::Cube);
        assert!(node.visible);
        assert_eq!(node.layer, RenderLayer::Scene);
        assert_eq!(node.material, Material::DEFAULT);
        assert_eq!(node.geometry, Geometry::Cube);
    }

    #[test]
    fn create_update_destroy_frame_in_order() {
        let mut frame = RenderFrameDiff::new();
        assert!(frame.is_empty());

        frame.push(RenderDiff::Create {
            handle: RenderHandle::new(1),
            parent: None,
            node: RenderNode {
                metadata: RenderMetadata {
                    source: Some(EntityId::new(42)),
                    tags: vec![TagId::new(1)],
                    label: Some("root".to_string()),
                },
                ..RenderNode::new(Geometry::Cube)
            },
        });
        frame.push(RenderDiff::Create {
            handle: RenderHandle::new(2),
            parent: Some(RenderHandle::new(1)),
            node: RenderNode::new(Geometry::Sphere),
        });
        frame.push(RenderDiff::Update {
            handle: RenderHandle::new(2),
            transform: Some(Transform {
                translation: [1.0, 0.0, 0.0],
                ..Transform::IDENTITY
            }),
            material: None,
            visible: Some(false),
            metadata: None,
        });
        frame.push(RenderDiff::Destroy {
            handle: RenderHandle::new(2),
        });

        assert_eq!(frame.len(), 4);
        assert!(matches!(
            frame.ops[1],
            RenderDiff::Create {
                parent: Some(RenderHandle(1)),
                ..
            }
        ));
        assert!(matches!(
            frame.ops.last(),
            Some(RenderDiff::Destroy {
                handle: RenderHandle(2)
            })
        ));
    }

    #[test]
    fn update_can_change_only_one_facet() {
        let only_visibility = RenderDiff::Update {
            handle: RenderHandle::new(3),
            transform: None,
            material: None,
            visible: Some(false),
            metadata: None,
        };
        if let RenderDiff::Update {
            transform,
            material,
            visible,
            metadata,
            ..
        } = only_visibility
        {
            assert!(transform.is_none());
            assert!(material.is_none());
            assert!(metadata.is_none());
            assert_eq!(visible, Some(false));
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn debug_overlay_line_node() {
        let node = RenderNode {
            geometry: Geometry::Line {
                a: [0.0, 0.0, 0.0],
                b: [1.0, 1.0, 0.0],
            },
            layer: RenderLayer::Debug,
            material: Material {
                color: [1.0, 0.0, 0.0, 1.0],
                wireframe: true,
            },
            ..RenderNode::new(Geometry::Point)
        };
        assert_eq!(node.layer, RenderLayer::Debug);
        assert!(matches!(node.geometry, Geometry::Line { .. }));
        assert!(node.material.wireframe);
    }
}

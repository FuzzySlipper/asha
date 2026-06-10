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
//! # Placeholders
//!
//! [`Transform`] and [`RenderMetadata`] are intentionally minimal placeholders.
//! Phase 2's job is to prove the *border* — handle lifecycle and diff framing —
//! not to design a material system or scene-graph schema. Their fields are
//! concrete enough to generate meaningful TypeScript and to be extended
//! additively later.
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

// ── Placeholders ──────────────────────────────────────────────────────────────

/// Minimal affine transform placeholder for a render node.
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

/// Minimal metadata placeholder carried on a render node.
///
/// Links a node back to the abstract sim vocabulary (an optional source entity
/// and any descriptive tags) plus a human label. Real material/visual data is
/// out of scope for the Phase 2 border.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderMetadata {
    /// The sim entity this node visualizes, if any.
    pub source: Option<EntityId>,
    /// Descriptive tags, in ascending order.
    pub tags: Vec<TagId>,
    /// Optional human-readable label for debugging/inspection.
    pub label: Option<String>,
}

// ── Diff operations ───────────────────────────────────────────────────────────

/// A single retained-mode change against the render scene.
///
/// `Update` carries optional fields so a tick can change only a transform, only
/// metadata, or both, without re-sending the whole node.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderDiff {
    /// Introduce a new node, optionally parented under an existing one.
    Create {
        handle: RenderHandle,
        parent: Option<RenderHandle>,
        transform: Transform,
        metadata: RenderMetadata,
    },
    /// Mutate an existing node's transform and/or metadata.
    Update {
        handle: RenderHandle,
        transform: Option<Transform>,
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
        // A handle and an entity with the same raw value are different types;
        // metadata links them explicitly rather than by collision.
        let meta = RenderMetadata {
            source: Some(EntityId::new(7)),
            ..RenderMetadata::default()
        };
        assert_eq!(meta.source, Some(EntityId::new(7)));
    }

    #[test]
    fn identity_transform_is_default() {
        assert_eq!(Transform::default(), Transform::IDENTITY);
        assert_eq!(Transform::IDENTITY.scale, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn create_update_destroy_frame_in_order() {
        let mut frame = RenderFrameDiff::new();
        assert!(frame.is_empty());

        frame.push(RenderDiff::Create {
            handle: RenderHandle::new(1),
            parent: None,
            transform: Transform::IDENTITY,
            metadata: RenderMetadata {
                source: Some(EntityId::new(42)),
                tags: vec![TagId::new(1)],
                label: Some("root".to_string()),
            },
        });
        frame.push(RenderDiff::Create {
            handle: RenderHandle::new(2),
            parent: Some(RenderHandle::new(1)),
            transform: Transform::IDENTITY,
            metadata: RenderMetadata::default(),
        });
        frame.push(RenderDiff::Update {
            handle: RenderHandle::new(2),
            transform: Some(Transform {
                translation: [1.0, 0.0, 0.0],
                ..Transform::IDENTITY
            }),
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
        let only_meta = RenderDiff::Update {
            handle: RenderHandle::new(3),
            transform: None,
            metadata: Some(RenderMetadata {
                label: Some("renamed".to_string()),
                ..RenderMetadata::default()
            }),
        };
        if let RenderDiff::Update {
            transform,
            metadata,
            ..
        } = only_meta
        {
            assert!(transform.is_none());
            assert!(metadata.is_some());
        } else {
            panic!("wrong variant");
        }
    }
}

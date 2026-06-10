//! Debug-overlay render projection.
//!
//! # Lane
//!
//! `rust-render` — may depend on `core-state`, `protocol-render`, and
//! `render-bridge`. Like `render-bridge` it only emits diffs; it never draws.
//!
//! # Design
//!
//! A debug overlay is just another retained projection, so this crate reuses
//! `render-bridge`'s [`RenderProjector`] with a [`DebugLabelProjection`]: one
//! `Debug`-layer point marker per entity, wireframed and labelled with the
//! entity id. Because it goes through the same projector, it gets stable
//! handles and correct create/update/destroy behavior for free.
//!
//! # Non-goals
//!
//! No product-domain overlays, no gizmo interaction, no rendering. The overlay
//! vocabulary stays abstract (points, lines, labels on the `Debug` layer).

#![forbid(unsafe_code)]

use core_state::EntityRecord;
use protocol_render::{Geometry, Material, RenderLayer, RenderMetadata, RenderNode, Transform};
use render_bridge::{NodeProjection, RenderProjector};

/// Projects each entity to a `Debug`-layer point marker labelled with its id.
pub struct DebugLabelProjection;

impl NodeProjection for DebugLabelProjection {
    fn project_entity(&self, record: &EntityRecord) -> RenderNode {
        let id = record.id.raw();
        RenderNode {
            geometry: Geometry::Point,
            material: Material {
                color: [1.0, 1.0, 0.0, 1.0],
                wireframe: true,
            },
            // Float the marker above the scene node for the same entity.
            transform: Transform {
                translation: [id as f32, 1.0, 0.0],
                ..Transform::IDENTITY
            },
            visible: true,
            layer: RenderLayer::Debug,
            metadata: RenderMetadata {
                source: Some(record.id),
                tags: record.tags.iter().copied().collect(),
                label: Some(format!("#{id}")),
            },
        }
    }
}

/// A projector that emits the debug-label overlay (a retained `Debug`-layer
/// marker per entity).
pub fn debug_overlay_projector() -> RenderProjector<DebugLabelProjection> {
    RenderProjector::new(DebugLabelProjection)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core_ids::EntityId;
    use core_state::StateStore;
    use protocol_render::{Geometry, RenderDiff, RenderLayer};

    #[test]
    fn emits_debug_layer_label_nodes_for_entities() {
        let mut store = StateStore::new();
        store.insert_entity(EntityId::new(1));
        store.insert_entity(EntityId::new(2));

        let mut p = debug_overlay_projector();
        let frame = p.project(&store);

        assert_eq!(frame.len(), 2);
        match &frame.ops[0] {
            RenderDiff::Create { node, .. } => {
                assert_eq!(node.layer, RenderLayer::Debug);
                assert!(matches!(node.geometry, Geometry::Point));
                assert!(node.material.wireframe);
                assert_eq!(node.metadata.label.as_deref(), Some("#1"));
            }
            other => panic!("expected create, got {other:?}"),
        }
    }

    #[test]
    fn overlay_reuses_retained_destroy_behavior() {
        let mut store = StateStore::new();
        store.insert_entity(EntityId::new(1));
        let mut p = debug_overlay_projector();
        let _ = p.project(&store);

        store.remove_entity(EntityId::new(1));
        let frame = p.project(&store);
        assert_eq!(frame.len(), 1);
        assert!(matches!(frame.ops[0], RenderDiff::Destroy { .. }));
    }
}

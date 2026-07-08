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
//! Richer overlay callers can use [`DebugOverlayProjector`] with explicit
//! [`DebugOverlayDescriptor`] values. That path emits retained `Debug`-layer
//! point markers, line segments, and label anchors without introducing renderer
//! or product-domain authority concepts.
//!
//! # Non-goals
//!
//! No product-domain overlays, no gizmo interaction, no rendering. The overlay
//! vocabulary stays abstract (points, lines, labels on the `Debug` layer).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use core_ids::{EntityId, TagId};
use core_state::EntityRecord;
use protocol_render::{
    Geometry, Material, RenderDiff, RenderFrameDiff, RenderHandle, RenderLayer, RenderMetadata,
    RenderNode, Transform,
};
use render_bridge::{NodeProjection, RenderProjector};

/// Stable retained identity for a debug overlay primitive.
///
/// It is deliberately separate from [`EntityId`]: overlays may describe
/// measurements, labels, traces, or helper geometry that are not themselves
/// authoritative entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DebugOverlayId(u64);

impl DebugOverlayId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// The product-neutral primitive vocabulary that `render-debug` can project.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugOverlayPrimitive {
    /// A point marker at a world/debug-space position.
    Point { position: [f32; 3] },
    /// A line segment between two world/debug-space endpoints.
    Line { a: [f32; 3], b: [f32; 3] },
    /// A text label anchored at a world/debug-space position.
    Label { position: [f32; 3], text: String },
}

/// A typed, projection-only debug overlay descriptor.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugOverlayDescriptor {
    pub id: DebugOverlayId,
    pub primitive: DebugOverlayPrimitive,
    pub color: [f32; 4],
    pub wireframe: bool,
    pub visible: bool,
    pub source: Option<EntityId>,
    pub tags: Vec<TagId>,
    pub label: Option<String>,
}

impl DebugOverlayDescriptor {
    pub fn point(id: DebugOverlayId, position: [f32; 3]) -> Self {
        Self {
            id,
            primitive: DebugOverlayPrimitive::Point { position },
            color: [1.0, 1.0, 0.0, 1.0],
            wireframe: true,
            visible: true,
            source: None,
            tags: Vec::new(),
            label: None,
        }
    }

    pub fn line(id: DebugOverlayId, a: [f32; 3], b: [f32; 3]) -> Self {
        Self {
            id,
            primitive: DebugOverlayPrimitive::Line { a, b },
            color: [0.0, 1.0, 0.0, 1.0],
            wireframe: false,
            visible: true,
            source: None,
            tags: Vec::new(),
            label: None,
        }
    }

    pub fn label(id: DebugOverlayId, position: [f32; 3], text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            id,
            primitive: DebugOverlayPrimitive::Label {
                position,
                text: text.clone(),
            },
            color: [1.0, 1.0, 1.0, 1.0],
            wireframe: false,
            visible: true,
            source: None,
            tags: Vec::new(),
            label: Some(text),
        }
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_wireframe(mut self, wireframe: bool) -> Self {
        self.wireframe = wireframe;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_source(mut self, source: EntityId) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = TagId>) -> Self {
        self.tags = tags.into_iter().collect();
        self.tags.sort();
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn to_node(&self) -> RenderNode {
        let (geometry, translation, label) = match &self.primitive {
            DebugOverlayPrimitive::Point { position } => {
                (Geometry::Point, *position, self.label.clone())
            }
            DebugOverlayPrimitive::Line { a, b } => (
                Geometry::Line { a: *a, b: *b },
                [0.0, 0.0, 0.0],
                self.label.clone(),
            ),
            DebugOverlayPrimitive::Label { position, text } => (
                Geometry::Point,
                *position,
                self.label.clone().or_else(|| Some(text.clone())),
            ),
        };

        RenderNode {
            geometry,
            material: Material {
                color: self.color,
                wireframe: self.wireframe,
            },
            transform: Transform {
                translation,
                ..Transform::IDENTITY
            },
            visible: self.visible,
            layer: RenderLayer::Debug,
            metadata: RenderMetadata {
                source: self.source,
                tags: self.tags.clone(),
                label,
            },
        }
    }
}

/// Retained projector for typed debug overlay descriptors.
#[derive(Debug)]
pub struct DebugOverlayProjector {
    handles: BTreeMap<DebugOverlayId, RenderHandle>,
    last: BTreeMap<DebugOverlayId, RenderNode>,
    next_handle: u64,
}

impl Default for DebugOverlayProjector {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugOverlayProjector {
    pub fn new() -> Self {
        Self {
            handles: BTreeMap::new(),
            last: BTreeMap::new(),
            next_handle: 1,
        }
    }

    /// Project the descriptor set into retained render diffs.
    ///
    /// Descriptors are sorted by [`DebugOverlayId`] before diffing. If a caller
    /// supplies duplicate IDs, the last descriptor for that ID wins; callers
    /// should treat IDs as stable unique overlay identities.
    pub fn project(&mut self, descriptors: &[DebugOverlayDescriptor]) -> RenderFrameDiff {
        let current: BTreeMap<DebugOverlayId, RenderNode> = descriptors
            .iter()
            .map(|descriptor| (descriptor.id, descriptor.to_node()))
            .collect();

        let mut frame = RenderFrameDiff::new();

        for (id, node) in &current {
            if !self.last.contains_key(id) {
                let handle = self.allocate(*id);
                frame.push(RenderDiff::Create {
                    handle,
                    parent: None,
                    node: node.clone(),
                });
            }
        }

        for (id, node) in &current {
            if let Some(prev) = self.last.get(id) {
                if prev != node {
                    let handle = self.handles[id];
                    frame.push(update_diff(handle, prev, node));
                }
            }
        }

        let removed: Vec<DebugOverlayId> = self
            .last
            .keys()
            .filter(|id| !current.contains_key(id))
            .copied()
            .collect();
        for id in removed {
            let handle = self
                .handles
                .remove(&id)
                .expect("a projected debug overlay must have a handle");
            frame.push(RenderDiff::Destroy { handle });
        }

        self.last = current;
        frame
    }

    pub fn handle_of(&self, id: DebugOverlayId) -> Option<RenderHandle> {
        self.handles.get(&id).copied()
    }

    fn allocate(&mut self, id: DebugOverlayId) -> RenderHandle {
        if let Some(handle) = self.handles.get(&id) {
            return *handle;
        }
        let handle = RenderHandle::new(self.next_handle);
        self.next_handle += 1;
        self.handles.insert(id, handle);
        handle
    }
}

fn update_diff(handle: RenderHandle, prev: &RenderNode, node: &RenderNode) -> RenderDiff {
    RenderDiff::Update {
        handle,
        transform: (prev.transform != node.transform).then_some(node.transform),
        material: (prev.material != node.material).then_some(node.material),
        visible: (prev.visible != node.visible).then_some(node.visible),
        metadata: (prev.metadata != node.metadata).then(|| node.metadata.clone()),
    }
}

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
    use protocol_render::{Geometry, RenderDiff, RenderLayer, RenderMetadata, Transform};
    use render_bridge::json;
    use std::path::Path;

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

    #[test]
    fn descriptor_projector_emits_debug_points_lines_and_labels() {
        let mut projector = DebugOverlayProjector::new();
        let frame = projector.project(&[
            DebugOverlayDescriptor::point(DebugOverlayId::new(1), [1.0, 2.0, 3.0])
                .with_color([0.25, 0.5, 1.0, 1.0])
                .with_label("point-a")
                .with_source(EntityId::new(7)),
            DebugOverlayDescriptor::line(DebugOverlayId::new(2), [0.0, 0.0, 0.0], [0.0, 2.0, 0.0])
                .with_label("line-a"),
            DebugOverlayDescriptor::label(DebugOverlayId::new(3), [2.0, 2.0, 0.0], "label-a"),
        ]);

        assert_eq!(frame.len(), 3);
        match &frame.ops[0] {
            RenderDiff::Create { handle, node, .. } => {
                assert_eq!(*handle, RenderHandle::new(1));
                assert_eq!(node.layer, RenderLayer::Debug);
                assert_eq!(node.geometry, Geometry::Point);
                assert_eq!(node.transform.translation, [1.0, 2.0, 3.0]);
                assert_eq!(node.material.color, [0.25, 0.5, 1.0, 1.0]);
                assert_eq!(node.metadata.source, Some(EntityId::new(7)));
                assert_eq!(node.metadata.label.as_deref(), Some("point-a"));
            }
            other => panic!("expected point create, got {other:?}"),
        }
        match &frame.ops[1] {
            RenderDiff::Create { node, .. } => {
                assert_eq!(
                    node.geometry,
                    Geometry::Line {
                        a: [0.0, 0.0, 0.0],
                        b: [0.0, 2.0, 0.0],
                    }
                );
                assert_eq!(node.layer, RenderLayer::Debug);
                assert_eq!(node.metadata.label.as_deref(), Some("line-a"));
            }
            other => panic!("expected line create, got {other:?}"),
        }
        match &frame.ops[2] {
            RenderDiff::Create { node, .. } => {
                assert_eq!(node.geometry, Geometry::Point);
                assert_eq!(node.transform.translation, [2.0, 2.0, 0.0]);
                assert_eq!(node.metadata.label.as_deref(), Some("label-a"));
            }
            other => panic!("expected label create, got {other:?}"),
        }
    }

    #[test]
    fn descriptor_projector_updates_mutable_facets_and_destroys_removed_overlays() {
        let mut projector = DebugOverlayProjector::new();
        let point_id = DebugOverlayId::new(1);
        let line_id = DebugOverlayId::new(2);

        let _ = projector.project(&[
            DebugOverlayDescriptor::point(point_id, [0.0, 0.0, 0.0]).with_label("old"),
            DebugOverlayDescriptor::line(line_id, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
        ]);
        let point_handle = projector.handle_of(point_id).unwrap();
        let line_handle = projector.handle_of(line_id).unwrap();

        let frame = projector.project(&[DebugOverlayDescriptor::point(point_id, [1.0, 0.0, 0.0])
            .with_color([1.0, 0.0, 0.0, 1.0])
            .with_label("new")]);

        assert_eq!(frame.len(), 2);
        match &frame.ops[0] {
            RenderDiff::Update {
                handle,
                transform,
                material,
                visible,
                metadata,
            } => {
                assert_eq!(*handle, point_handle);
                assert_eq!(
                    *transform,
                    Some(Transform {
                        translation: [1.0, 0.0, 0.0],
                        ..Transform::IDENTITY
                    })
                );
                assert_eq!(material.unwrap().color, [1.0, 0.0, 0.0, 1.0]);
                assert_eq!(*visible, None);
                assert_eq!(
                    *metadata,
                    Some(RenderMetadata {
                        source: None,
                        tags: Vec::new(),
                        label: Some("new".to_string()),
                    })
                );
            }
            other => panic!("expected update, got {other:?}"),
        }
        assert!(matches!(
            frame.ops[1],
            RenderDiff::Destroy { handle } if handle == line_handle
        ));
        assert_eq!(projector.handle_of(line_id), None);
    }

    #[test]
    fn debug_overlay_primitives_fixture_matches_committed_render_diff() {
        let mut projector = DebugOverlayProjector::new();
        let frame = projector.project(&[
            DebugOverlayDescriptor::point(DebugOverlayId::new(1), [1.0, 2.0, 3.0])
                .with_color([0.25, 0.5, 1.0, 1.0])
                .with_label("point-a")
                .with_source(EntityId::new(7)),
            DebugOverlayDescriptor::line(DebugOverlayId::new(2), [0.0, 0.0, 0.0], [0.0, 2.0, 0.0])
                .with_label("line-a"),
            DebugOverlayDescriptor::label(DebugOverlayId::new(3), [2.0, 2.0, 0.0], "label-a"),
        ]);
        let actual = json::encode_frame(&frame);
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .expect("repo root")
            .join("harness/fixtures/render-diffs/debug-overlay-primitives.json");

        if std::env::var_os("BLESS").is_some() {
            std::fs::write(&path, actual).unwrap();
            return;
        }

        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert_eq!(actual, expected);
    }
}

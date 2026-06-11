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
    /// Replace a node's geometry with an uploaded voxel mesh payload (ADR 0007).
    /// Identity/material/transform stay on the node, so a chunk remesh is just
    /// another `ReplaceMeshPayload` rather than a destroy+create.
    ReplaceMeshPayload {
        handle: RenderHandle,
        payload: MeshPayloadDescriptor,
    },
}

// ── Mesh payload descriptors (voxel-capability-07 / ADR 0007) ──────────────────

/// A vertex attribute stream's element type. Only `f32` today; the enum leaves
/// room for future attribute encodings without a shape break.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshAttributeKind {
    F32,
}

/// Which vertex attribute a stream carries. `Uv`/`Color` are reserved for the
/// terrain-atlas and per-vertex-colour material strategies (unused initially).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshAttributeName {
    Position,
    Normal,
    Uv,
    Color,
}

/// One declared vertex attribute stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshAttribute {
    pub name: MeshAttributeName,
    /// Components per vertex (e.g. 3 for position/normal).
    pub components: u8,
    pub kind: MeshAttributeKind,
}

/// Index buffer element width. `u32` everywhere today (u16 optimisation deferred).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshIndexWidth {
    U32,
}

/// The buffer layout a renderer needs to wrap bytes as typed arrays without
/// transcoding (separate attribute streams; `BufferGeometry`-compatible).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshBufferLayout {
    pub vertex_count: u32,
    pub index_count: u32,
    pub index_width: MeshIndexWidth,
    pub attributes: Vec<MeshAttribute>,
}

/// One material-slot draw group over a contiguous index range (→ `addGroup`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshGroupDescriptor {
    pub material_slot: u16,
    pub start: u32,
    pub count: u32,
}

/// Axis-aligned mesh bounds (chunk-local).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshBoundsDescriptor {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Where the bulk vertex/index bytes live: `Inline` for small golden fixtures,
/// `Handle` for runtime (bridge-owned buffer referenced by handle + byte offsets,
/// per ADR 0006 — the renderer wraps the bytes as typed-array views).
#[derive(Debug, Clone, PartialEq)]
pub enum MeshPayloadSource {
    Inline {
        positions: Vec<f32>,
        normals: Vec<f32>,
        indices: Vec<u32>,
    },
    Handle {
        buffer: u64,
        positions_byte_offset: u32,
        normals_byte_offset: u32,
        indices_byte_offset: u32,
    },
}

/// The full mesh-payload border: layout + material groups + bounds + data source.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshPayloadDescriptor {
    pub layout: MeshBufferLayout,
    pub groups: Vec<MeshGroupDescriptor>,
    pub bounds: MeshBoundsDescriptor,
    pub source: MeshPayloadSource,
}

/// A malformed mesh payload descriptor, classified for agent routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshDescriptorError {
    /// An inline attribute stream's length disagrees with the layout.
    AttributeLengthMismatch {
        name: MeshAttributeName,
        expected: usize,
        actual: usize,
    },
    /// An inline index references a vertex outside `vertex_count`.
    IndexOutOfRange { index: u32, vertex_count: u32 },
    /// Material group ranges do not exactly tile `index_count`.
    GroupsDoNotTile { covered: u64, index_count: u32 },
    /// A group's `[start, start+count)` range falls outside the index buffer.
    GroupOutOfRange {
        start: u32,
        count: u32,
        index_count: u32,
    },
}

impl core::fmt::Display for MeshDescriptorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MeshDescriptorError::AttributeLengthMismatch {
                name,
                expected,
                actual,
            } => {
                write!(f, "attribute {name:?} length {actual}, expected {expected}")
            }
            MeshDescriptorError::IndexOutOfRange {
                index,
                vertex_count,
            } => {
                write!(f, "index {index} out of range for {vertex_count} vertices")
            }
            MeshDescriptorError::GroupsDoNotTile {
                covered,
                index_count,
            } => {
                write!(f, "groups cover {covered} indices, expected {index_count}")
            }
            MeshDescriptorError::GroupOutOfRange {
                start,
                count,
                index_count,
            } => write!(
                f,
                "group [{start}, {}) outside {index_count} indices",
                *start as u64 + *count as u64
            ),
        }
    }
}

impl std::error::Error for MeshDescriptorError {}

impl MeshPayloadDescriptor {
    /// Validate self-consistency: inline stream lengths, index ranges, and that
    /// the material groups exactly tile the index buffer.
    pub fn validate(&self) -> Result<(), MeshDescriptorError> {
        let vc = self.layout.vertex_count;
        let ic = self.layout.index_count;

        if let MeshPayloadSource::Inline {
            positions,
            normals,
            indices,
        } = &self.source
        {
            let expect_v = vc as usize * 3;
            if positions.len() != expect_v {
                return Err(MeshDescriptorError::AttributeLengthMismatch {
                    name: MeshAttributeName::Position,
                    expected: expect_v,
                    actual: positions.len(),
                });
            }
            if normals.len() != expect_v {
                return Err(MeshDescriptorError::AttributeLengthMismatch {
                    name: MeshAttributeName::Normal,
                    expected: expect_v,
                    actual: normals.len(),
                });
            }
            if indices.len() != ic as usize {
                return Err(MeshDescriptorError::GroupsDoNotTile {
                    covered: indices.len() as u64,
                    index_count: ic,
                });
            }
            for &i in indices {
                if i >= vc {
                    return Err(MeshDescriptorError::IndexOutOfRange {
                        index: i,
                        vertex_count: vc,
                    });
                }
            }
        }

        let mut covered: u64 = 0;
        for g in &self.groups {
            let end = g.start as u64 + g.count as u64;
            if end > ic as u64 {
                return Err(MeshDescriptorError::GroupOutOfRange {
                    start: g.start,
                    count: g.count,
                    index_count: ic,
                });
            }
            covered += g.count as u64;
        }
        if covered != ic as u64 {
            return Err(MeshDescriptorError::GroupsDoNotTile {
                covered,
                index_count: ic,
            });
        }
        Ok(())
    }
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

#[cfg(test)]
mod mesh_tests {
    use super::*;

    /// A minimal valid inline descriptor: one triangle (3 verts, 3 indices), one group.
    fn one_triangle() -> MeshPayloadDescriptor {
        MeshPayloadDescriptor {
            layout: MeshBufferLayout {
                vertex_count: 3,
                index_count: 3,
                index_width: MeshIndexWidth::U32,
                attributes: vec![
                    MeshAttribute {
                        name: MeshAttributeName::Position,
                        components: 3,
                        kind: MeshAttributeKind::F32,
                    },
                    MeshAttribute {
                        name: MeshAttributeName::Normal,
                        components: 3,
                        kind: MeshAttributeKind::F32,
                    },
                ],
            },
            groups: vec![MeshGroupDescriptor {
                material_slot: 1,
                start: 0,
                count: 3,
            }],
            bounds: MeshBoundsDescriptor {
                min: [0.0; 3],
                max: [1.0, 1.0, 0.0],
            },
            source: MeshPayloadSource::Inline {
                positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
                normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                indices: vec![0, 1, 2],
            },
        }
    }

    #[test]
    fn valid_inline_descriptor_passes() {
        assert_eq!(one_triangle().validate(), Ok(()));
    }

    #[test]
    fn handle_source_skips_inline_length_checks_but_groups_must_tile() {
        let mut d = one_triangle();
        d.source = MeshPayloadSource::Handle {
            buffer: 7,
            positions_byte_offset: 0,
            normals_byte_offset: 36,
            indices_byte_offset: 72,
        };
        assert_eq!(d.validate(), Ok(()));
    }

    #[test]
    fn wrong_position_length_is_classified() {
        let mut d = one_triangle();
        if let MeshPayloadSource::Inline { positions, .. } = &mut d.source {
            positions.pop();
        }
        assert!(matches!(
            d.validate(),
            Err(MeshDescriptorError::AttributeLengthMismatch {
                name: MeshAttributeName::Position,
                ..
            })
        ));
    }

    #[test]
    fn index_out_of_range_is_classified() {
        let mut d = one_triangle();
        if let MeshPayloadSource::Inline { indices, .. } = &mut d.source {
            indices[2] = 9;
        }
        assert_eq!(
            d.validate(),
            Err(MeshDescriptorError::IndexOutOfRange {
                index: 9,
                vertex_count: 3
            }),
        );
    }

    #[test]
    fn groups_must_tile_the_index_buffer() {
        let mut d = one_triangle();
        d.groups = vec![MeshGroupDescriptor {
            material_slot: 1,
            start: 0,
            count: 2,
        }];
        assert!(matches!(
            d.validate(),
            Err(MeshDescriptorError::GroupsDoNotTile { .. })
        ));
    }

    #[test]
    fn group_range_beyond_indices_is_classified() {
        let mut d = one_triangle();
        d.groups = vec![MeshGroupDescriptor {
            material_slot: 1,
            start: 2,
            count: 5,
        }];
        assert!(matches!(
            d.validate(),
            Err(MeshDescriptorError::GroupOutOfRange { .. })
        ));
    }

    #[test]
    fn replace_mesh_payload_diff_constructs() {
        let diff = RenderDiff::ReplaceMeshPayload {
            handle: RenderHandle::new(3),
            payload: one_triangle(),
        };
        assert!(matches!(diff, RenderDiff::ReplaceMeshPayload { .. }));
    }
}

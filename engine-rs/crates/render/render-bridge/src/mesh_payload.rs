//! Small deterministic geometry payloads used by scene projection.

use protocol_render::{
    MeshAttribute, MeshAttributeKind, MeshAttributeName, MeshBoundsDescriptor, MeshBufferLayout,
    MeshGroupDescriptor, MeshIndexWidth, MeshPayloadDescriptor, MeshPayloadSource, MeshProvenance,
};

/// Unit-quad placeholder shared until imported static geometry is available.
pub(crate) fn placeholder_quad_payload() -> MeshPayloadDescriptor {
    MeshPayloadDescriptor {
        layout: MeshBufferLayout {
            vertex_count: 4,
            index_count: 6,
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
            material_slot: 0,
            start: 0,
            count: 6,
        }],
        bounds: MeshBoundsDescriptor {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 0.0],
        },
        source: MeshPayloadSource::Inline {
            positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0],
            normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            indices: vec![0, 1, 2, 0, 2, 3],
        },
        provenance: MeshProvenance::StaticAsset,
    }
}

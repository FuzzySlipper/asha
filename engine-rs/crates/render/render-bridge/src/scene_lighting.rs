//! Stored scene-light pose composition and generic render projection.

use core_ids::SceneNodeId;
use core_math::Vec3;
use core_scene::{
    FlatSceneDocument, SceneLight, SceneLightShadowIntent, SceneNodeRecord, SceneTransform,
};
use protocol_render::{LightDescriptor, LightShadowIntent};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn project_scene_light(
    light: &SceneLight,
    transform: SceneTransform,
) -> LightDescriptor {
    let position = transform.translation.to_array();
    let direction = rotate_vector(transform.rotation, [0.0, 0.0, -1.0]);
    let shadow = |intent| match intent {
        SceneLightShadowIntent::Disabled => LightShadowIntent::Disabled,
        SceneLightShadowIntent::Requested => LightShadowIntent::Requested,
    };
    match light {
        SceneLight::Ambient {
            color,
            intensity,
            enabled,
            shadow_intent,
        } => LightDescriptor::Ambient {
            color: *color,
            intensity: *intensity,
            enabled: *enabled,
            shadow_intent: shadow(*shadow_intent),
        },
        SceneLight::Directional {
            color,
            intensity,
            enabled,
            shadow_intent,
        } => LightDescriptor::Directional {
            color: *color,
            intensity: *intensity,
            enabled: *enabled,
            direction,
            shadow_intent: shadow(*shadow_intent),
        },
        SceneLight::Point {
            color,
            intensity,
            enabled,
            range,
            decay,
            shadow_intent,
        } => LightDescriptor::Point {
            color: *color,
            intensity: *intensity,
            enabled: *enabled,
            position,
            range: *range,
            decay: *decay,
            shadow_intent: shadow(*shadow_intent),
        },
        SceneLight::Spot {
            color,
            intensity,
            enabled,
            range,
            decay,
            outer_angle_radians,
            penumbra,
            shadow_intent,
        } => LightDescriptor::Spot {
            color: *color,
            intensity: *intensity,
            enabled: *enabled,
            position,
            direction,
            range: *range,
            decay: *decay,
            outer_angle_radians: *outer_angle_radians,
            penumbra: *penumbra,
            shadow_intent: shadow(*shadow_intent),
        },
    }
}

pub(crate) fn light_kind(light: &LightDescriptor) -> u8 {
    match light {
        LightDescriptor::Ambient { .. } => 0,
        LightDescriptor::Directional { .. } => 1,
        LightDescriptor::Point { .. } => 2,
        LightDescriptor::Spot { .. } => 3,
    }
}

pub(crate) fn authored_world_transform(
    scene: &FlatSceneDocument,
    node: SceneNodeId,
) -> Option<SceneTransform> {
    let records: BTreeMap<u64, &SceneNodeRecord> = scene
        .nodes
        .iter()
        .map(|record| (record.id.raw(), record))
        .collect();
    let mut chain = Vec::new();
    let mut current = records.get(&node.raw()).copied()?;
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.id.raw()) {
            return None;
        }
        chain.push(current.transform);
        let Some(parent) = current.parent else { break };
        current = records.get(&parent.raw()).copied()?;
    }
    Some(
        chain
            .into_iter()
            .rev()
            .fold(SceneTransform::IDENTITY, compose_transform),
    )
}

fn compose_transform(parent: SceneTransform, local: SceneTransform) -> SceneTransform {
    let scaled = Vec3::new(
        local.translation.x * parent.scale.x,
        local.translation.y * parent.scale.y,
        local.translation.z * parent.scale.z,
    );
    let rotated = rotate_vector(parent.rotation, scaled.to_array());
    SceneTransform {
        translation: parent.translation + Vec3::new(rotated[0], rotated[1], rotated[2]),
        rotation: multiply_quat(parent.rotation, local.rotation),
        scale: Vec3::new(
            parent.scale.x * local.scale.x,
            parent.scale.y * local.scale.y,
            parent.scale.z * local.scale.z,
        ),
    }
}

fn multiply_quat(a: core_scene::Quat, b: core_scene::Quat) -> core_scene::Quat {
    core_scene::Quat::new(
        a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    )
}

fn rotate_vector(rotation: core_scene::Quat, vector: [f32; 3]) -> [f32; 3] {
    let inverse_length = rotation.norm_squared().sqrt().recip();
    let q = core_scene::Quat::new(
        rotation.x * inverse_length,
        rotation.y * inverse_length,
        rotation.z * inverse_length,
        rotation.w * inverse_length,
    );
    let v = core_scene::Quat::new(vector[0], vector[1], vector[2], 0.0);
    let conjugate = core_scene::Quat::new(-q.x, -q.y, -q.z, q.w);
    let rotated = multiply_quat(multiply_quat(q, v), conjugate);
    [rotated.x, rotated.y, rotated.z]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{ScenePresentation, ScenePresentationProjector};
    use core_catalog::Catalog;
    use core_ids::{RuntimeSessionId, SceneId};
    use core_scene::{NodeMetadata, Quat, SceneMetadata, SceneNodeKind, SpatialSessionState};
    use protocol_render::RenderDiff;

    #[test]
    fn hierarchy_projects_updates_and_destroys_one_retained_light() {
        let half = std::f32::consts::FRAC_1_SQRT_2;
        let light = SceneNodeRecord {
            id: SceneNodeId::new(2),
            parent: Some(SceneNodeId::new(1)),
            child_order: 0,
            transform: SceneTransform::new(Vec3::new(0.0, 0.0, -3.0), Quat::IDENTITY, Vec3::ONE),
            kind: SceneNodeKind::Light(SceneLight::Spot {
                color: [0.3, 0.5, 1.0],
                intensity: 4.0,
                enabled: true,
                range: Some(12.0),
                decay: 2.0,
                outer_angle_radians: 0.6,
                penumbra: 0.25,
                shadow_intent: SceneLightShadowIntent::Requested,
            }),
            metadata: NodeMetadata::default(),
        };
        let mut doc = FlatSceneDocument {
            id: SceneId::new(1),
            schema_version: 2,
            metadata: SceneMetadata {
                name: None,
                authoring_format_version: 2,
            },
            dependencies: vec![],
            nodes: vec![
                SceneNodeRecord {
                    id: SceneNodeId::new(1),
                    parent: None,
                    child_order: 0,
                    transform: SceneTransform::new(
                        Vec3::new(2.0, 0.0, 0.0),
                        Quat::new(0.0, half, 0.0, half),
                        Vec3::ONE,
                    ),
                    kind: SceneNodeKind::EmptyGroup,
                    metadata: NodeMetadata::default(),
                },
                light,
            ],
        };
        let world = SpatialSessionState::empty(RuntimeSessionId::new(1));
        let catalog = Catalog { entries: vec![] };
        let mut projector = ScenePresentationProjector::new();
        let project = |projector: &mut ScenePresentationProjector, doc: &FlatSceneDocument| {
            projector.project(&ScenePresentation {
                scene: doc,
                world: &world,
                catalog: &catalog,
                overrides: &BTreeMap::new(),
            })
        };

        let first = project(&mut projector, &doc);
        assert!(
            matches!(&first.ops[0], RenderDiff::CreateLight { handle, light: LightDescriptor::Spot { position, direction, intensity: 4.0, .. }, .. } if handle.raw() == 1 && (position[0] + 1.0).abs() < 0.0001 && (direction[0] + 1.0).abs() < 0.0001)
        );
        let node = doc
            .nodes
            .iter_mut()
            .find(|node| node.id == SceneNodeId::new(2))
            .unwrap();
        node.transform.translation.z = -4.0;
        if let SceneNodeKind::Light(SceneLight::Spot { intensity, .. }) = &mut node.kind {
            *intensity = 6.0;
        }
        let second = project(&mut projector, &doc);
        assert!(
            matches!(&second.ops[0], RenderDiff::UpdateLight { handle, light: LightDescriptor::Spot { intensity: 6.0, .. } } if handle.raw() == 1)
        );
        doc.nodes.retain(|node| node.id != SceneNodeId::new(2));
        assert!(
            matches!(project(&mut projector, &doc).ops.as_slice(), [RenderDiff::Destroy { handle }] if handle.raw() == 1)
        );
    }
}

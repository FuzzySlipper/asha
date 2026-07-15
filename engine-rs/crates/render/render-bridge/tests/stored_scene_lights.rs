use std::collections::BTreeMap;

use core_catalog::Catalog;
use core_ids::RuntimeSessionId;
use core_scene::{decode, validate, SpatialSessionState};
use protocol_render::{LightDescriptor, RenderDiff};
use render_bridge::{ScenePresentation, ScenePresentationProjector};

#[test]
fn public_consumer_projects_the_canonical_scene_without_a_renderer_backend() {
    let source = include_str!("../../../../../harness/fixtures/scenes/lights-v2.json");
    let scene = decode(source).expect("stored fixture decodes");
    assert!(validate(&scene).is_ok());
    let world = SpatialSessionState::empty(RuntimeSessionId::new(42));
    let catalog = Catalog { entries: vec![] };
    let overrides = BTreeMap::new();
    let frame = ScenePresentationProjector::new().project(&ScenePresentation {
        scene: &scene,
        world: &world,
        catalog: &catalog,
        overrides: &overrides,
    });
    let lights: Vec<&LightDescriptor> = frame
        .ops
        .iter()
        .filter_map(|operation| match operation {
            RenderDiff::CreateLight { light, .. } => Some(light),
            _ => None,
        })
        .collect();
    assert_eq!(lights.len(), 4);
    assert!(matches!(lights[0], LightDescriptor::Ambient { .. }));
    assert!(matches!(lights[1], LightDescriptor::Directional { .. }));
    assert!(matches!(lights[2], LightDescriptor::Point { .. }));
    assert!(matches!(lights[3], LightDescriptor::Spot { .. }));
}

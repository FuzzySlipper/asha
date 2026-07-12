//! Public-height prefab bootstrap and inspection for downstream gameplay hosts.
//!
//! Authoring tools provide canonical registry JSON plus explicit placement
//! commands. Validation and entity creation remain in the existing
//! serialization and ProjectBundle rule owners.

use std::collections::BTreeSet;

use core_entity::EntityStore;
use core_ids::{PrefabId, PrefabInstanceId};
use rule_project_bundle::{
    InstantiatePrefabCommand, PrefabInstantiationCatalog, PrefabPlacementOrigin,
    ProjectBundleLoadResult,
};
use serde::{Deserialize, Serialize};
use svc_serialization::{
    load_prefab_registry, PrefabInstanceRecord, PrefabOverride, PrefabOverrideValue,
    PrefabRegistryValidationContext, PrefabTransform,
};

use crate::GameplayRuntimeHostError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuntimePrefabCatalog {
    pub asset_ids: Vec<String>,
    pub entity_definition_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuntimePrefabBootstrap {
    pub registry_json: String,
    pub catalog: GameplayRuntimePrefabCatalog,
    pub placements: Vec<GameplayRuntimePrefabPlacement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuntimePrefabPlacement {
    pub command_id: String,
    pub origin: GameplayRuntimePrefabPlacementOrigin,
    pub instance: u64,
    pub prefab: u64,
    pub seed: u64,
    pub transform: GameplayRuntimePrefabTransform,
    pub overrides: Vec<GameplayRuntimePrefabOverride>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameplayRuntimePrefabPlacementOrigin {
    Authored,
    Player,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuntimePrefabTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl GameplayRuntimePrefabTransform {
    pub const IDENTITY: Self = Self {
        translation: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    };
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "field",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GameplayRuntimePrefabOverride {
    Transform {
        target_role: String,
        transform: GameplayRuntimePrefabTransform,
    },
    EntityDefinition {
        target_role: String,
        stable_id: String,
    },
    Asset {
        target_role: String,
        asset: String,
    },
    Material {
        target_role: String,
        asset: String,
    },
    Activation {
        target_role: String,
        active: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuntimePrefabReadout {
    pub state_hash: String,
    pub instances: Vec<GameplayRuntimePrefabInstanceReadout>,
    pub accepted_commands: Vec<GameplayRuntimePrefabCommandReadout>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuntimePrefabCommandReadout {
    pub command_id: String,
    pub instance: u64,
    pub prefab: u64,
    pub origin: GameplayRuntimePrefabPlacementOrigin,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuntimePrefabInstanceReadout {
    pub instance: u64,
    pub prefab: u64,
    pub origin: GameplayRuntimePrefabPlacementOrigin,
    pub provenance_hash: String,
    pub override_count: u32,
    pub parts: Vec<GameplayRuntimePrefabPartReadout>,
    pub roles: Vec<GameplayRuntimePrefabRoleReadout>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuntimePrefabPartReadout {
    pub part: u64,
    pub namespace: String,
    pub entity: u64,
    pub parent_entity: Option<u64>,
    pub translation: [f32; 3],
    pub source_kind: String,
    pub active: bool,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuntimePrefabRoleReadout {
    pub role: String,
    pub entity: u64,
}

pub(crate) fn apply_prefab_bootstrap(
    bundle: &mut ProjectBundleLoadResult,
    bootstrap: GameplayRuntimePrefabBootstrap,
) -> Result<(), GameplayRuntimeHostError> {
    let validation_context = PrefabRegistryValidationContext {
        asset_ids: bootstrap
            .catalog
            .asset_ids
            .into_iter()
            .collect::<BTreeSet<_>>(),
        entity_definition_ids: bootstrap
            .catalog
            .entity_definition_ids
            .into_iter()
            .collect::<BTreeSet<_>>(),
    };
    let registry = load_prefab_registry(&bootstrap.registry_json, &validation_context)
        .map_err(|error| GameplayRuntimeHostError::Prefab(error.to_string()))?;
    let catalog = PrefabInstantiationCatalog::from(&validation_context);
    let entities = bundle.runtime_entities.get_or_insert_with(EntityStore::new);
    for placement in bootstrap.placements {
        bundle
            .prefab_instances
            .instantiate(
                entities,
                &registry,
                &catalog,
                InstantiatePrefabCommand {
                    command_id: placement.command_id,
                    origin: placement.origin.into(),
                    record: PrefabInstanceRecord {
                        instance: PrefabInstanceId::new(placement.instance),
                        prefab: PrefabId::new(placement.prefab),
                        seed: placement.seed,
                        transform: placement.transform.into(),
                        overrides: placement.overrides.into_iter().map(Into::into).collect(),
                    },
                },
            )
            .map_err(|error| GameplayRuntimeHostError::Prefab(error.to_string()))?;
    }
    Ok(())
}

pub(crate) fn prefab_readout(bundle: &ProjectBundleLoadResult) -> GameplayRuntimePrefabReadout {
    let empty = EntityStore::new();
    let entities = bundle.runtime_entities.as_ref().unwrap_or(&empty);
    let snapshot = bundle.prefab_instances.snapshot(entities);
    GameplayRuntimePrefabReadout {
        state_hash: snapshot.state_hash,
        accepted_commands: snapshot
            .accepted_commands
            .iter()
            .map(|command| GameplayRuntimePrefabCommandReadout {
                command_id: command.command_id.clone(),
                instance: command.record.instance.raw(),
                prefab: command.record.prefab.raw(),
                origin: command.origin.into(),
            })
            .collect(),
        instances: snapshot
            .instances
            .iter()
            .map(|instance| GameplayRuntimePrefabInstanceReadout {
                instance: instance.record.instance.raw(),
                prefab: instance.record.prefab.raw(),
                origin: instance.origin.into(),
                provenance_hash: instance.provenance_hash.clone(),
                override_count: u32::try_from(instance.record.overrides.len()).unwrap_or(u32::MAX),
                parts: instance
                    .parts
                    .iter()
                    .map(|part| GameplayRuntimePrefabPartReadout {
                        part: part.part.raw(),
                        namespace: part.namespace.clone(),
                        entity: part.entity.raw(),
                        parent_entity: part.parent_entity.map(|entity| entity.raw()),
                        translation: part.transform.translation,
                        source_kind: match &part.source {
                            svc_serialization::PrefabPartSource::Scene { .. } => "scene",
                            svc_serialization::PrefabPartSource::EntityDefinition { .. } => {
                                "entityDefinition"
                            }
                            svc_serialization::PrefabPartSource::VoxelObject { .. } => {
                                "voxelObject"
                            }
                        }
                        .to_owned(),
                        active: part.active,
                        roles: part.roles.clone(),
                    })
                    .collect(),
                roles: instance
                    .role_map
                    .iter()
                    .map(|role| GameplayRuntimePrefabRoleReadout {
                        role: role.reference.role.clone(),
                        entity: role.entity.raw(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

impl From<GameplayRuntimePrefabPlacementOrigin> for PrefabPlacementOrigin {
    fn from(value: GameplayRuntimePrefabPlacementOrigin) -> Self {
        match value {
            GameplayRuntimePrefabPlacementOrigin::Authored => Self::Authored,
            GameplayRuntimePrefabPlacementOrigin::Player => Self::Player,
        }
    }
}

impl From<PrefabPlacementOrigin> for GameplayRuntimePrefabPlacementOrigin {
    fn from(value: PrefabPlacementOrigin) -> Self {
        match value {
            PrefabPlacementOrigin::Authored => Self::Authored,
            PrefabPlacementOrigin::Player => Self::Player,
        }
    }
}

impl From<GameplayRuntimePrefabTransform> for PrefabTransform {
    fn from(value: GameplayRuntimePrefabTransform) -> Self {
        Self {
            translation: value.translation,
            rotation: value.rotation,
            scale: value.scale,
        }
    }
}

impl From<GameplayRuntimePrefabOverride> for PrefabOverride {
    fn from(value: GameplayRuntimePrefabOverride) -> Self {
        match value {
            GameplayRuntimePrefabOverride::Transform {
                target_role,
                transform,
            } => Self {
                target_role,
                value: PrefabOverrideValue::Transform {
                    transform: transform.into(),
                },
            },
            GameplayRuntimePrefabOverride::EntityDefinition {
                target_role,
                stable_id,
            } => Self {
                target_role,
                value: PrefabOverrideValue::EntityDefinition { stable_id },
            },
            GameplayRuntimePrefabOverride::Asset { target_role, asset } => Self {
                target_role,
                value: PrefabOverrideValue::Asset { asset },
            },
            GameplayRuntimePrefabOverride::Material { target_role, asset } => Self {
                target_role,
                value: PrefabOverrideValue::Material { asset },
            },
            GameplayRuntimePrefabOverride::Activation {
                target_role,
                active,
            } => Self {
                target_role,
                value: PrefabOverrideValue::Activation { active },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BundleArtifacts, GameplayBindingEntityTargets, GameplayRuntimeHost,
        GameplayRuntimeProjectInput, LoadPlan, LoadStep,
    };
    use core_ids::{RuntimeSessionId, SceneId, SceneNodeId};
    use core_scene::{encode, SceneMetadata, SceneNode, SceneNodeKind, SceneTree};
    use gameplay_module_sdk::{
        GameplayModuleBindingRegistryBuilder, GameplayStaticCompositionBuilder,
    };

    fn prefab_project_input() -> GameplayRuntimeProjectInput {
        let scene = SceneTree {
            id: SceneId::new(44),
            schema_version: 1,
            metadata: SceneMetadata {
                name: Some("public-prefab-host".to_owned()),
                authoring_format_version: 1,
            },
            dependencies: Vec::new(),
            roots: vec![SceneNode::leaf(
                SceneNodeId::new(1),
                SceneNodeKind::EmptyGroup,
            )],
        };
        let mut composition = GameplayStaticCompositionBuilder::new();
        composition.include_standard_owner_events();
        GameplayRuntimeProjectInput {
            load_plan: LoadPlan {
                steps: vec![
                    LoadStep::ValidateVersions {
                        bundle_schema_version: 1,
                        protocol_version: 1,
                    },
                    LoadStep::LoadAssetLock {
                        artifact: "assets/lock.json".to_owned(),
                        asset_count: 0,
                    },
                    LoadStep::LoadSceneDocument {
                        artifact: "scene/scene.json".to_owned(),
                        scene: SceneId::new(44),
                    },
                    LoadStep::BootstrapScene {
                        scene: SceneId::new(44),
                        runtime_session: RuntimeSessionId::new(44),
                    },
                    LoadStep::ValidateFinalState,
                ],
            },
            artifacts: BundleArtifacts::new()
                .with_artifact("assets/lock.json", "{\"entries\":[]}")
                .with_artifact("scene/scene.json", encode(&scene.to_flat())),
            composition: composition.build().unwrap(),
            bindings: GameplayModuleBindingRegistryBuilder::new().build(),
            entity_targets: GameplayBindingEntityTargets::new(),
            spatial_entities: Vec::new(),
            declared_reads: Vec::new(),
            triggers: Vec::new(),
        }
    }

    fn prefab_bootstrap() -> GameplayRuntimePrefabBootstrap {
        GameplayRuntimePrefabBootstrap {
            registry_json: r#"{
  "schemaVersion": 1,
  "definitions": [{
    "id": 70,
    "schemaVersion": 1,
    "displayName": "Public console",
    "parts": [
      {
        "id": 1,
        "namespace": "body",
        "displayName": "Body",
        "parent": null,
        "transform": { "translation": [0, 0, 0], "rotation": [0, 0, 0, 1], "scale": [1, 1, 1] },
        "source": { "kind": "entityDefinition", "stableId": "fixture.console.body" }
      },
      {
        "id": 2,
        "namespace": "sensor",
        "displayName": "Sensor",
        "parent": 1,
        "transform": { "translation": [0, 1, 0], "rotation": [0, 0, 0, 1], "scale": [1, 1, 1] },
        "source": { "kind": "entityDefinition", "stableId": "fixture.console.sensor" }
      }
    ],
    "partRoles": [
      { "role": "console/body", "part": 1 },
      { "role": "interaction/sensor", "part": 2 }
    ],
    "variant": null
  }]
}"#
            .to_owned(),
            catalog: GameplayRuntimePrefabCatalog {
                asset_ids: Vec::new(),
                entity_definition_ids: vec![
                    "fixture.console.body".to_owned(),
                    "fixture.console.body.blue".to_owned(),
                    "fixture.console.body.red".to_owned(),
                    "fixture.console.sensor".to_owned(),
                ],
            },
            placements: vec![
                GameplayRuntimePrefabPlacement {
                    command_id: "place-console-authored".to_owned(),
                    origin: GameplayRuntimePrefabPlacementOrigin::Authored,
                    instance: 700,
                    prefab: 70,
                    seed: 11,
                    transform: GameplayRuntimePrefabTransform::IDENTITY,
                    overrides: vec![GameplayRuntimePrefabOverride::EntityDefinition {
                        target_role: "console/body".to_owned(),
                        stable_id: "fixture.console.body.blue".to_owned(),
                    }],
                },
                GameplayRuntimePrefabPlacement {
                    command_id: "place-console-player".to_owned(),
                    origin: GameplayRuntimePrefabPlacementOrigin::Player,
                    instance: 701,
                    prefab: 70,
                    seed: 12,
                    transform: GameplayRuntimePrefabTransform {
                        translation: [4.0, 0.0, 0.0],
                        ..GameplayRuntimePrefabTransform::IDENTITY
                    },
                    overrides: vec![GameplayRuntimePrefabOverride::EntityDefinition {
                        target_role: "console/body".to_owned(),
                        stable_id: "fixture.console.body.red".to_owned(),
                    }],
                },
            ],
        }
    }

    #[test]
    fn public_prefab_bootstrap_places_resolves_and_restores_multiple_instances() {
        let host = GameplayRuntimeHost::activate_project_with_prefabs(
            prefab_project_input(),
            prefab_bootstrap(),
        )
        .unwrap();
        let readout = host.prefab_readout();
        assert_eq!(readout.instances.len(), 2);
        assert_eq!(readout.accepted_commands.len(), 2);
        assert_eq!(
            readout.accepted_commands[1].origin,
            GameplayRuntimePrefabPlacementOrigin::Player
        );
        let first_sensor = readout.instances[0]
            .roles
            .iter()
            .find(|role| role.role == "interaction/sensor")
            .unwrap();
        let second_sensor = readout.instances[1]
            .roles
            .iter()
            .find(|role| role.role == "interaction/sensor")
            .unwrap();
        assert_ne!(first_sensor.entity, second_sensor.entity);
        assert_eq!(readout.instances[0].override_count, 1);
        assert_eq!(readout.instances[1].override_count, 1);

        let snapshot = host.compose_snapshot().unwrap();
        let restored = GameplayRuntimeHost::restore_project_with_prefabs(
            prefab_project_input(),
            prefab_bootstrap(),
            &snapshot.text,
        )
        .unwrap();
        assert_eq!(restored.prefab_readout(), readout);
        assert_eq!(
            restored.readout().runtime_host_hash,
            host.readout().runtime_host_hash
        );
    }
}

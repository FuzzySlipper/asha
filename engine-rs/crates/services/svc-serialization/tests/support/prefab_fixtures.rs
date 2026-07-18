//! Shared prefab-registry samples for golden tests and regenerators.

use core_ids::{PrefabId, PrefabPartId};
use std::collections::BTreeSet;
use svc_serialization::{
    PrefabDefinition, PrefabOverride, PrefabOverrideValue, PrefabPart, PrefabPartRoleBinding,
    PrefabPartSource, PrefabRegistry, PrefabRegistryValidationContext, PrefabTransform,
    PrefabVariantDelta, ValidatedPrefabRegistry, PREFAB_DEFINITION_SCHEMA_VERSION,
    PREFAB_REGISTRY_SCHEMA_VERSION,
};

pub fn context() -> PrefabRegistryValidationContext {
    PrefabRegistryValidationContext {
        asset_ids: BTreeSet::new(),
        entity_definition_ids: ["fixture.root".to_string()].into_iter().collect(),
    }
}

pub fn valid_registry() -> ValidatedPrefabRegistry {
    ValidatedPrefabRegistry::new(
        PrefabRegistry {
            schema_version: PREFAB_REGISTRY_SCHEMA_VERSION,
            definitions: vec![PrefabDefinition {
                id: PrefabId::new(7),
                schema_version: PREFAB_DEFINITION_SCHEMA_VERSION,
                display_name: "Fixture".into(),
                parts: vec![PrefabPart {
                    id: PrefabPartId::new(3),
                    namespace: "root".into(),
                    display_name: "Root".into(),
                    parent: None,
                    transform: PrefabTransform::IDENTITY,
                    source: PrefabPartSource::EntityDefinition {
                        stable_id: "fixture.root".into(),
                    },
                }],
                part_roles: vec![PrefabPartRoleBinding {
                    role: "gameplay-root".into(),
                    part: PrefabPartId::new(3),
                }],
                variant: None,
            }],
        },
        &context(),
    )
    .expect("fixture registry validates")
}

#[allow(dead_code)]
pub fn alias_removal_registry() -> PrefabRegistry {
    let base = PrefabDefinition {
        id: PrefabId::new(7),
        schema_version: PREFAB_DEFINITION_SCHEMA_VERSION,
        display_name: "Fixture".into(),
        parts: vec![PrefabPart {
            id: PrefabPartId::new(3),
            namespace: "root".into(),
            display_name: "Root".into(),
            parent: None,
            transform: PrefabTransform::IDENTITY,
            source: PrefabPartSource::EntityDefinition {
                stable_id: "fixture.root".into(),
            },
        }],
        part_roles: vec![
            PrefabPartRoleBinding {
                role: "gameplay-root".into(),
                part: PrefabPartId::new(3),
            },
            PrefabPartRoleBinding {
                role: "root-alias".into(),
                part: PrefabPartId::new(3),
            },
        ],
        variant: None,
    };
    PrefabRegistry {
        schema_version: PREFAB_REGISTRY_SCHEMA_VERSION,
        definitions: vec![
            base,
            PrefabDefinition {
                id: PrefabId::new(8),
                schema_version: PREFAB_DEFINITION_SCHEMA_VERSION,
                display_name: "Invalid Alias Removal".into(),
                parts: vec![],
                part_roles: vec![],
                variant: Some(PrefabVariantDelta {
                    variant_id: "damaged".into(),
                    base: PrefabId::new(7),
                    removed_roles: vec!["gameplay-root".into()],
                    overrides: vec![PrefabOverride {
                        target_role: "root-alias".into(),
                        value: PrefabOverrideValue::Transform {
                            transform: PrefabTransform::IDENTITY,
                        },
                    }],
                }),
            },
        ],
    }
}

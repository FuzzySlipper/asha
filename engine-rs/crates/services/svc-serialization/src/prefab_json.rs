//! Canonical JSON codec for the durable ProjectBundle prefab registry artifact.

use core_ids::{PrefabId, PrefabPartId};
use serde_json::{json, Map, Value};

use crate::prefab::{
    PrefabDefinition, PrefabOverride, PrefabOverrideValue, PrefabPart, PrefabPartRoleBinding,
    PrefabPartSource, PrefabRegistry, PrefabRegistryValidationContext, PrefabTransform,
    PrefabValidationReport, PrefabVariantDelta, ValidatedPrefabRegistry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefabRegistryDecodeError {
    Json(String),
    Field(String),
    UnknownSourceKind(String),
    UnknownOverrideField(String),
}

impl core::fmt::Display for PrefabRegistryDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Json(message) => write!(f, "invalid JSON: {message}"),
            Self::Field(message) => write!(f, "bad field: {message}"),
            Self::UnknownSourceKind(kind) => write!(f, "unknown prefab source kind `{kind}`"),
            Self::UnknownOverrideField(field) => {
                write!(f, "unknown prefab override field `{field}`")
            }
        }
    }
}

impl std::error::Error for PrefabRegistryDecodeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefabRegistryLoadError {
    Decode(PrefabRegistryDecodeError),
    Validation(PrefabValidationReport),
}

impl core::fmt::Display for PrefabRegistryLoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Decode(error) => write!(f, "could not decode prefab registry: {error}"),
            Self::Validation(report) => write!(
                f,
                "prefab registry failed validation with {} diagnostic(s)",
                report.diagnostics.len()
            ),
        }
    }
}

impl std::error::Error for PrefabRegistryLoadError {}

pub fn encode_prefab_registry(registry: &ValidatedPrefabRegistry) -> String {
    let canonical = registry.as_registry();
    let value = json!({
        "schemaVersion": canonical.schema_version,
        "definitions": canonical.definitions.iter().map(encode_definition).collect::<Vec<_>>(),
    });
    let mut encoded = serde_json::to_string_pretty(&value).expect("prefab registry values encode");
    encoded.push('\n');
    encoded
}

pub fn load_prefab_registry(
    input: &str,
    context: &PrefabRegistryValidationContext,
) -> Result<ValidatedPrefabRegistry, PrefabRegistryLoadError> {
    let registry = decode_prefab_registry(input).map_err(PrefabRegistryLoadError::Decode)?;
    ValidatedPrefabRegistry::new(registry, context).map_err(PrefabRegistryLoadError::Validation)
}

fn decode_prefab_registry(input: &str) -> Result<PrefabRegistry, PrefabRegistryDecodeError> {
    let value: Value = serde_json::from_str(input)
        .map_err(|error| PrefabRegistryDecodeError::Json(error.to_string()))?;
    require_keys(&value, "prefab registry", &["schemaVersion", "definitions"])?;
    let definitions = array(field(&value, "definitions")?, "definitions")?
        .iter()
        .map(decode_definition)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PrefabRegistry {
        schema_version: u32_field(&value, "schemaVersion")?,
        definitions,
    })
}

fn encode_definition(definition: &PrefabDefinition) -> Value {
    json!({
        "id": definition.id.raw(),
        "schemaVersion": definition.schema_version,
        "displayName": definition.display_name,
        "parts": definition.parts.iter().map(encode_part).collect::<Vec<_>>(),
        "partRoles": definition.part_roles.iter().map(|binding| json!({
            "role": binding.role,
            "part": binding.part.raw(),
        })).collect::<Vec<_>>(),
        "variant": definition.variant.as_ref().map(encode_variant),
    })
}

fn decode_definition(value: &Value) -> Result<PrefabDefinition, PrefabRegistryDecodeError> {
    require_keys(
        value,
        "prefab definition",
        &[
            "id",
            "schemaVersion",
            "displayName",
            "parts",
            "partRoles",
            "variant",
        ],
    )?;
    let parts = array(field(value, "parts")?, "parts")?
        .iter()
        .map(decode_part)
        .collect::<Result<Vec<_>, _>>()?;
    let part_roles = array(field(value, "partRoles")?, "partRoles")?
        .iter()
        .map(|binding| {
            require_keys(binding, "prefab part role", &["role", "part"])?;
            Ok(PrefabPartRoleBinding {
                role: string_field(binding, "role")?,
                part: PrefabPartId::new(u64_field(binding, "part")?),
            })
        })
        .collect::<Result<Vec<_>, PrefabRegistryDecodeError>>()?;
    let variant = match value.get("variant") {
        None | Some(Value::Null) => None,
        Some(variant) => Some(decode_variant(variant)?),
    };
    Ok(PrefabDefinition {
        id: PrefabId::new(u64_field(value, "id")?),
        schema_version: u32_field(value, "schemaVersion")?,
        display_name: string_field(value, "displayName")?,
        parts,
        part_roles,
        variant,
    })
}

fn encode_part(part: &PrefabPart) -> Value {
    json!({
        "id": part.id.raw(),
        "namespace": part.namespace,
        "displayName": part.display_name,
        "parent": part.parent.map(PrefabPartId::raw),
        "transform": encode_transform(part.transform),
        "source": encode_source(&part.source),
    })
}

fn decode_part(value: &Value) -> Result<PrefabPart, PrefabRegistryDecodeError> {
    require_keys(
        value,
        "prefab part",
        &[
            "id",
            "namespace",
            "displayName",
            "parent",
            "transform",
            "source",
        ],
    )?;
    Ok(PrefabPart {
        id: PrefabPartId::new(u64_field(value, "id")?),
        namespace: string_field(value, "namespace")?,
        display_name: string_field(value, "displayName")?,
        parent: optional_u64(value, "parent")?.map(PrefabPartId::new),
        transform: decode_transform(field(value, "transform")?)?,
        source: decode_source(field(value, "source")?)?,
    })
}

fn encode_source(source: &PrefabPartSource) -> Value {
    match source {
        PrefabPartSource::Scene { asset } => json!({ "kind": "scene", "asset": asset }),
        PrefabPartSource::EntityDefinition { stable_id } => {
            json!({ "kind": "entityDefinition", "stableId": stable_id })
        }
        PrefabPartSource::VoxelObject { asset } => {
            json!({ "kind": "voxelObject", "asset": asset })
        }
    }
}

fn decode_source(value: &Value) -> Result<PrefabPartSource, PrefabRegistryDecodeError> {
    match string_field(value, "kind")?.as_str() {
        "scene" => {
            require_keys(value, "scene prefab source", &["kind", "asset"])?;
            Ok(PrefabPartSource::Scene {
                asset: string_field(value, "asset")?,
            })
        }
        "entityDefinition" => {
            require_keys(
                value,
                "entity-definition prefab source",
                &["kind", "stableId"],
            )?;
            Ok(PrefabPartSource::EntityDefinition {
                stable_id: string_field(value, "stableId")?,
            })
        }
        "voxelObject" => {
            require_keys(value, "voxel-object prefab source", &["kind", "asset"])?;
            Ok(PrefabPartSource::VoxelObject {
                asset: string_field(value, "asset")?,
            })
        }
        other => Err(PrefabRegistryDecodeError::UnknownSourceKind(other.into())),
    }
}

fn encode_variant(variant: &PrefabVariantDelta) -> Value {
    json!({
        "variantId": variant.variant_id,
        "base": variant.base.raw(),
        "removedRoles": variant.removed_roles,
        "overrides": variant.overrides.iter().map(encode_override).collect::<Vec<_>>(),
    })
}

fn decode_variant(value: &Value) -> Result<PrefabVariantDelta, PrefabRegistryDecodeError> {
    require_keys(
        value,
        "prefab variant",
        &["variantId", "base", "removedRoles", "overrides"],
    )?;
    Ok(PrefabVariantDelta {
        variant_id: string_field(value, "variantId")?,
        base: PrefabId::new(u64_field(value, "base")?),
        removed_roles: string_array(field(value, "removedRoles")?, "removedRoles")?,
        overrides: array(field(value, "overrides")?, "overrides")?
            .iter()
            .map(decode_override)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn encode_override(item: &PrefabOverride) -> Value {
    let mut value = Map::new();
    value.insert("field".into(), Value::String(item.value.field().into()));
    match &item.value {
        PrefabOverrideValue::Transform { transform } => {
            value.insert("transform".into(), encode_transform(*transform));
        }
        PrefabOverrideValue::EntityDefinition { stable_id } => {
            value.insert("stableId".into(), Value::String(stable_id.clone()));
        }
        PrefabOverrideValue::Asset { asset } => {
            value.insert("asset".into(), Value::String(asset.clone()));
        }
        PrefabOverrideValue::Material { asset } => {
            value.insert("asset".into(), Value::String(asset.clone()));
        }
        PrefabOverrideValue::Activation { active } => {
            value.insert("active".into(), Value::Bool(*active));
        }
    }
    json!({
        "targetRole": item.target_role,
        "value": Value::Object(value),
    })
}

fn decode_override(value: &Value) -> Result<PrefabOverride, PrefabRegistryDecodeError> {
    require_keys(value, "prefab override", &["targetRole", "value"])?;
    let encoded_value = field(value, "value")?;
    let field_name = string_field(encoded_value, "field")?;
    let override_value = match field_name.as_str() {
        "transform" => {
            require_keys(
                encoded_value,
                "transform override value",
                &["field", "transform"],
            )?;
            PrefabOverrideValue::Transform {
                transform: decode_transform(field(encoded_value, "transform")?)?,
            }
        }
        "entityDefinition" => {
            require_keys(
                encoded_value,
                "entity-definition override value",
                &["field", "stableId"],
            )?;
            PrefabOverrideValue::EntityDefinition {
                stable_id: string_field(encoded_value, "stableId")?,
            }
        }
        "asset" => {
            require_keys(encoded_value, "asset override value", &["field", "asset"])?;
            PrefabOverrideValue::Asset {
                asset: string_field(encoded_value, "asset")?,
            }
        }
        "material" => {
            require_keys(
                encoded_value,
                "material override value",
                &["field", "asset"],
            )?;
            PrefabOverrideValue::Material {
                asset: string_field(encoded_value, "asset")?,
            }
        }
        "activation" => {
            require_keys(
                encoded_value,
                "activation override value",
                &["field", "active"],
            )?;
            PrefabOverrideValue::Activation {
                active: bool_field(encoded_value, "active")?,
            }
        }
        other => {
            return Err(PrefabRegistryDecodeError::UnknownOverrideField(
                other.into(),
            ))
        }
    };
    Ok(PrefabOverride {
        target_role: string_field(value, "targetRole")?,
        value: override_value,
    })
}

fn encode_transform(transform: PrefabTransform) -> Value {
    json!({
        "translation": transform.translation,
        "rotation": transform.rotation,
        "scale": transform.scale,
    })
}

fn decode_transform(value: &Value) -> Result<PrefabTransform, PrefabRegistryDecodeError> {
    require_keys(
        value,
        "prefab transform",
        &["translation", "rotation", "scale"],
    )?;
    Ok(PrefabTransform {
        translation: f32_array::<3>(field(value, "translation")?, "translation")?,
        rotation: f32_array::<4>(field(value, "rotation")?, "rotation")?,
        scale: f32_array::<3>(field(value, "scale")?, "scale")?,
    })
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, PrefabRegistryDecodeError> {
    value
        .get(key)
        .ok_or_else(|| PrefabRegistryDecodeError::Field(format!("missing field `{key}`")))
}

fn require_keys(
    value: &Value,
    label: &str,
    allowed: &[&str],
) -> Result<(), PrefabRegistryDecodeError> {
    let object = value
        .as_object()
        .ok_or_else(|| PrefabRegistryDecodeError::Field(format!("{label} must be an object")))?;
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(PrefabRegistryDecodeError::Field(format!(
                "unknown field `{key}` in {label}"
            )));
        }
    }
    Ok(())
}

fn string_field(value: &Value, key: &str) -> Result<String, PrefabRegistryDecodeError> {
    field(value, key)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| PrefabRegistryDecodeError::Field(format!("field `{key}` must be a string")))
}

fn u64_field(value: &Value, key: &str) -> Result<u64, PrefabRegistryDecodeError> {
    field(value, key)?.as_u64().ok_or_else(|| {
        PrefabRegistryDecodeError::Field(format!("field `{key}` must be a non-negative integer"))
    })
}

fn bool_field(value: &Value, key: &str) -> Result<bool, PrefabRegistryDecodeError> {
    field(value, key)?
        .as_bool()
        .ok_or_else(|| PrefabRegistryDecodeError::Field(format!("field `{key}` must be a boolean")))
}

fn u32_field(value: &Value, key: &str) -> Result<u32, PrefabRegistryDecodeError> {
    let raw = u64_field(value, key)?;
    u32::try_from(raw)
        .map_err(|_| PrefabRegistryDecodeError::Field(format!("field `{key}` exceeds u32 range")))
}

fn optional_u64(value: &Value, key: &str) -> Result<Option<u64>, PrefabRegistryDecodeError> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(number) => number.as_u64().map(Some).ok_or_else(|| {
            PrefabRegistryDecodeError::Field(format!("field `{key}` must be an integer or null"))
        }),
    }
}

fn array<'a>(value: &'a Value, label: &str) -> Result<&'a Vec<Value>, PrefabRegistryDecodeError> {
    value.as_array().ok_or_else(|| {
        PrefabRegistryDecodeError::Field(format!("field `{label}` must be an array"))
    })
}

fn string_array(value: &Value, label: &str) -> Result<Vec<String>, PrefabRegistryDecodeError> {
    array(value, label)?
        .iter()
        .map(|item| {
            item.as_str().map(str::to_string).ok_or_else(|| {
                PrefabRegistryDecodeError::Field(format!("field `{label}` must contain strings"))
            })
        })
        .collect()
}

fn f32_array<const N: usize>(
    value: &Value,
    label: &str,
) -> Result<[f32; N], PrefabRegistryDecodeError> {
    let values = array(value, label)?;
    if values.len() != N {
        return Err(PrefabRegistryDecodeError::Field(format!(
            "field `{label}` must contain {N} numbers"
        )));
    }
    let parsed = values
        .iter()
        .map(|item| {
            item.as_f64().map(|number| number as f32).ok_or_else(|| {
                PrefabRegistryDecodeError::Field(format!("field `{label}` must contain numbers"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    parsed.try_into().map_err(|_| {
        PrefabRegistryDecodeError::Field(format!("field `{label}` must contain {N} numbers"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prefab::PrefabPartReference;
    use std::collections::BTreeSet;

    fn registry() -> PrefabRegistry {
        PrefabRegistry {
            schema_version: 1,
            definitions: vec![PrefabDefinition {
                id: PrefabId::new(7),
                schema_version: 1,
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
        }
    }

    #[test]
    fn canonical_json_round_trip_is_a_fixed_point() {
        let context = PrefabRegistryValidationContext {
            asset_ids: BTreeSet::new(),
            entity_definition_ids: ["fixture.root".to_string()].into_iter().collect(),
        };
        let validated = ValidatedPrefabRegistry::new(registry(), &context).unwrap();
        let encoded = encode_prefab_registry(&validated);
        let decoded = load_prefab_registry(&encoded, &context).unwrap();
        assert_eq!(encode_prefab_registry(&decoded), encoded);
    }

    #[test]
    fn stable_reference_survives_json_round_trip() {
        let reference = PrefabPartReference {
            prefab: PrefabId::new(7),
            role: "gameplay-root".into(),
        };
        let context = PrefabRegistryValidationContext {
            asset_ids: BTreeSet::new(),
            entity_definition_ids: ["fixture.root".to_string()].into_iter().collect(),
        };
        let validated = ValidatedPrefabRegistry::new(registry(), &context).unwrap();
        let decoded = load_prefab_registry(&encode_prefab_registry(&validated), &context).unwrap();
        assert_eq!(decoded.as_registry().definitions[0].id, reference.prefab);
        assert_eq!(
            decoded.as_registry().definitions[0].part_roles[0].role,
            reference.role
        );
    }

    #[test]
    fn material_and_activation_overrides_round_trip_through_the_durable_codec() {
        let mut stored = registry();
        stored.definitions[0].parts[0].source = PrefabPartSource::VoxelObject {
            asset: "voxel-object/body".into(),
        };
        stored.definitions.push(PrefabDefinition {
            id: PrefabId::new(8),
            schema_version: 1,
            display_name: "Dormant steel fixture".into(),
            parts: vec![],
            part_roles: vec![],
            variant: Some(PrefabVariantDelta {
                variant_id: "lit".to_owned(),
                base: PrefabId::new(7),
                removed_roles: vec![],
                overrides: vec![
                    PrefabOverride {
                        target_role: "gameplay-root".into(),
                        value: PrefabOverrideValue::Material {
                            asset: "material/steel".into(),
                        },
                    },
                    PrefabOverride {
                        target_role: "gameplay-root".into(),
                        value: PrefabOverrideValue::Activation { active: false },
                    },
                ],
            }),
        });
        let context = PrefabRegistryValidationContext {
            asset_ids: ["voxel-object/body", "material/steel"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            entity_definition_ids: BTreeSet::new(),
        };
        let validated = ValidatedPrefabRegistry::new(stored, &context).unwrap();
        let encoded = encode_prefab_registry(&validated);
        let decoded = load_prefab_registry(&encoded, &context).unwrap();
        let overrides = &decoded.as_registry().definitions[1]
            .variant
            .as_ref()
            .unwrap()
            .overrides;
        assert!(matches!(
            overrides[0].value,
            PrefabOverrideValue::Activation { active: false }
        ));
        assert!(matches!(
            overrides[1].value,
            PrefabOverrideValue::Material { ref asset } if asset == "material/steel"
        ));
        assert_eq!(encode_prefab_registry(&decoded), encoded);
    }

    #[test]
    fn load_boundary_rejects_semantically_invalid_decoded_content() {
        let context = PrefabRegistryValidationContext {
            asset_ids: BTreeSet::new(),
            entity_definition_ids: ["fixture.root".to_string()].into_iter().collect(),
        };
        let invalid = r#"{"schemaVersion":99,"definitions":[]}"#;
        let error = load_prefab_registry(invalid, &context).unwrap_err();
        assert!(matches!(error, PrefabRegistryLoadError::Validation(_)));
    }

    #[test]
    fn load_boundary_rejects_unknown_fields_before_validation() {
        let context = PrefabRegistryValidationContext {
            asset_ids: BTreeSet::new(),
            entity_definition_ids: ["fixture.root".to_string()].into_iter().collect(),
        };
        let validated = ValidatedPrefabRegistry::new(registry(), &context).unwrap();
        let mut encoded: Value = serde_json::from_str(&encode_prefab_registry(&validated)).unwrap();
        encoded["definitions"][0]["parts"][0]["source"]["browserAccepted"] = Value::Bool(true);
        let error = load_prefab_registry(&encoded.to_string(), &context).unwrap_err();
        assert!(error
            .to_string()
            .contains("unknown field `browserAccepted`"));
    }
}

//! Durable codec for prefab-instance metadata embedded in the Session snapshot.

use core_ids::{EntityId, PrefabId, PrefabInstanceId, PrefabPartId, SceneNodeId};
use serde_json::{json, Value};
use svc_serialization::{
    PrefabInstanceRecord, PrefabOverride, PrefabOverrideValue, PrefabPartReference,
    PrefabPartSource, PrefabTransform,
};

use crate::prefab_instance::{
    InstantiatePrefabCommand, PrefabInstanceSnapshot, PrefabPartResolution, PrefabPlacementOrigin,
    ResolvedPrefabInstance, ResolvedPrefabPart,
};

pub(crate) const SESSION_PREFAB_FIELD: &str = "prefabInstances";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefabSnapshotDecodeError {
    Json(String),
    Field(String),
    UnknownOrigin(String),
    UnknownSource(String),
    UnknownOverride(String),
}

impl core::fmt::Display for PrefabSnapshotDecodeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for PrefabSnapshotDecodeError {}

pub(crate) fn embed_prefab_snapshot(
    session_text: &str,
    snapshot: &PrefabInstanceSnapshot,
) -> String {
    let mut root: Value = serde_json::from_str(session_text).expect("entity snapshot is JSON");
    root.as_object_mut()
        .expect("entity snapshot root is an object")
        .insert(SESSION_PREFAB_FIELD.into(), encode_snapshot(snapshot));
    let mut encoded = serde_json::to_string_pretty(&root).expect("prefab snapshot values encode");
    encoded.push('\n');
    encoded
}

pub(crate) fn decode_embedded_prefab_snapshot(
    session_text: &str,
) -> Result<Option<PrefabInstanceSnapshot>, PrefabSnapshotDecodeError> {
    let root: Value = serde_json::from_str(session_text)
        .map_err(|error| PrefabSnapshotDecodeError::Json(error.to_string()))?;
    root.get(SESSION_PREFAB_FIELD)
        .map(decode_snapshot)
        .transpose()
}

fn encode_snapshot(snapshot: &PrefabInstanceSnapshot) -> Value {
    json!({
        "schemaVersion": snapshot.schema_version,
        "acceptedCommands": snapshot.accepted_commands.iter().map(encode_command).collect::<Vec<_>>(),
        "instances": snapshot.instances.iter().map(encode_instance).collect::<Vec<_>>(),
        "stateHash": snapshot.state_hash,
    })
}

fn decode_snapshot(value: &Value) -> Result<PrefabInstanceSnapshot, PrefabSnapshotDecodeError> {
    Ok(PrefabInstanceSnapshot {
        schema_version: u32_field(value, "schemaVersion")?,
        accepted_commands: array_field(value, "acceptedCommands")?
            .iter()
            .map(decode_command)
            .collect::<Result<_, _>>()?,
        instances: array_field(value, "instances")?
            .iter()
            .map(decode_instance)
            .collect::<Result<_, _>>()?,
        state_hash: string_field(value, "stateHash")?,
    })
}

fn encode_command(command: &InstantiatePrefabCommand) -> Value {
    json!({
        "commandId": command.command_id,
        "origin": origin_label(command.origin),
        "record": encode_record(&command.record),
    })
}

fn decode_command(value: &Value) -> Result<InstantiatePrefabCommand, PrefabSnapshotDecodeError> {
    Ok(InstantiatePrefabCommand {
        command_id: string_field(value, "commandId")?,
        origin: decode_origin(&string_field(value, "origin")?)?,
        record: decode_record(field(value, "record")?)?,
    })
}

fn encode_record(record: &PrefabInstanceRecord) -> Value {
    json!({
        "instance": record.instance.raw(),
        "prefab": record.prefab.raw(),
        "seed": record.seed,
        "transform": encode_transform(record.transform),
        "overrides": record.overrides.iter().map(encode_override).collect::<Vec<_>>(),
    })
}

fn decode_record(value: &Value) -> Result<PrefabInstanceRecord, PrefabSnapshotDecodeError> {
    Ok(PrefabInstanceRecord {
        instance: PrefabInstanceId::new(u64_field(value, "instance")?),
        prefab: PrefabId::new(u64_field(value, "prefab")?),
        seed: u64_field(value, "seed")?,
        transform: decode_transform(field(value, "transform")?)?,
        overrides: array_field(value, "overrides")?
            .iter()
            .map(decode_override)
            .collect::<Result<_, _>>()?,
    })
}

fn encode_instance(instance: &ResolvedPrefabInstance) -> Value {
    json!({
        "record": encode_record(&instance.record),
        "origin": origin_label(instance.origin),
        "parts": instance.parts.iter().map(encode_part).collect::<Vec<_>>(),
        "roleMap": instance.role_map.iter().map(encode_resolution).collect::<Vec<_>>(),
        "effectiveOverrides": instance.effective_overrides.iter().map(encode_override).collect::<Vec<_>>(),
        "provenanceHash": instance.provenance_hash,
    })
}

fn decode_instance(value: &Value) -> Result<ResolvedPrefabInstance, PrefabSnapshotDecodeError> {
    Ok(ResolvedPrefabInstance {
        record: decode_record(field(value, "record")?)?,
        origin: decode_origin(&string_field(value, "origin")?)?,
        parts: array_field(value, "parts")?
            .iter()
            .map(decode_part)
            .collect::<Result<_, _>>()?,
        role_map: array_field(value, "roleMap")?
            .iter()
            .map(decode_resolution)
            .collect::<Result<_, _>>()?,
        effective_overrides: array_field(value, "effectiveOverrides")?
            .iter()
            .map(decode_override)
            .collect::<Result<_, _>>()?,
        provenance_hash: string_field(value, "provenanceHash")?,
    })
}

fn encode_part(part: &ResolvedPrefabPart) -> Value {
    json!({
        "part": part.part.raw(),
        "namespace": part.namespace,
        "entity": part.entity.raw(),
        "node": part.node.raw(),
        "parentEntity": part.parent_entity.map(EntityId::raw),
        "transform": encode_transform(part.transform),
        "source": encode_source(&part.source),
        "materialOverride": part.material_override,
        "active": part.active,
        "roles": part.roles,
    })
}

fn decode_part(value: &Value) -> Result<ResolvedPrefabPart, PrefabSnapshotDecodeError> {
    Ok(ResolvedPrefabPart {
        part: PrefabPartId::new(u64_field(value, "part")?),
        namespace: string_field(value, "namespace")?,
        entity: EntityId::new(u64_field(value, "entity")?),
        node: SceneNodeId::new(u64_field(value, "node")?),
        parent_entity: optional_u64(value, "parentEntity")?.map(EntityId::new),
        transform: decode_transform(field(value, "transform")?)?,
        source: decode_source(field(value, "source")?)?,
        material_override: optional_string(value, "materialOverride")?,
        active: bool_field(value, "active")?,
        roles: string_array(value, "roles")?,
    })
}

fn encode_resolution(resolution: &PrefabPartResolution) -> Value {
    json!({
        "prefab": resolution.reference.prefab.raw(),
        "role": resolution.reference.role,
        "entity": resolution.entity.raw(),
        "node": resolution.node.raw(),
    })
}

fn decode_resolution(value: &Value) -> Result<PrefabPartResolution, PrefabSnapshotDecodeError> {
    Ok(PrefabPartResolution {
        reference: PrefabPartReference {
            prefab: PrefabId::new(u64_field(value, "prefab")?),
            role: string_field(value, "role")?,
        },
        entity: EntityId::new(u64_field(value, "entity")?),
        node: SceneNodeId::new(u64_field(value, "node")?),
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

fn decode_source(value: &Value) -> Result<PrefabPartSource, PrefabSnapshotDecodeError> {
    match string_field(value, "kind")?.as_str() {
        "scene" => Ok(PrefabPartSource::Scene {
            asset: string_field(value, "asset")?,
        }),
        "entityDefinition" => Ok(PrefabPartSource::EntityDefinition {
            stable_id: string_field(value, "stableId")?,
        }),
        "voxelObject" => Ok(PrefabPartSource::VoxelObject {
            asset: string_field(value, "asset")?,
        }),
        other => Err(PrefabSnapshotDecodeError::UnknownSource(other.into())),
    }
}

fn encode_override(item: &PrefabOverride) -> Value {
    let value = match &item.value {
        PrefabOverrideValue::Transform { transform } => {
            json!({ "field": "transform", "transform": encode_transform(*transform) })
        }
        PrefabOverrideValue::EntityDefinition { stable_id } => {
            json!({ "field": "entityDefinition", "stableId": stable_id })
        }
        PrefabOverrideValue::Asset { asset } => json!({ "field": "asset", "asset": asset }),
        PrefabOverrideValue::Material { asset } => {
            json!({ "field": "material", "asset": asset })
        }
        PrefabOverrideValue::Activation { active } => {
            json!({ "field": "activation", "active": active })
        }
    };
    json!({ "targetRole": item.target_role, "value": value })
}

fn decode_override(value: &Value) -> Result<PrefabOverride, PrefabSnapshotDecodeError> {
    let encoded = field(value, "value")?;
    let kind = string_field(encoded, "field")?;
    let override_value = match kind.as_str() {
        "transform" => PrefabOverrideValue::Transform {
            transform: decode_transform(field(encoded, "transform")?)?,
        },
        "entityDefinition" => PrefabOverrideValue::EntityDefinition {
            stable_id: string_field(encoded, "stableId")?,
        },
        "asset" => PrefabOverrideValue::Asset {
            asset: string_field(encoded, "asset")?,
        },
        "material" => PrefabOverrideValue::Material {
            asset: string_field(encoded, "asset")?,
        },
        "activation" => PrefabOverrideValue::Activation {
            active: bool_field(encoded, "active")?,
        },
        other => return Err(PrefabSnapshotDecodeError::UnknownOverride(other.into())),
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

fn decode_transform(value: &Value) -> Result<PrefabTransform, PrefabSnapshotDecodeError> {
    Ok(PrefabTransform {
        translation: f32_array::<3>(value, "translation")?,
        rotation: f32_array::<4>(value, "rotation")?,
        scale: f32_array::<3>(value, "scale")?,
    })
}

fn origin_label(origin: PrefabPlacementOrigin) -> &'static str {
    match origin {
        PrefabPlacementOrigin::Authored => "authored",
        PrefabPlacementOrigin::Player => "player",
    }
}

fn decode_origin(value: &str) -> Result<PrefabPlacementOrigin, PrefabSnapshotDecodeError> {
    match value {
        "authored" => Ok(PrefabPlacementOrigin::Authored),
        "player" => Ok(PrefabPlacementOrigin::Player),
        other => Err(PrefabSnapshotDecodeError::UnknownOrigin(other.into())),
    }
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a Value, PrefabSnapshotDecodeError> {
    value
        .get(name)
        .ok_or_else(|| PrefabSnapshotDecodeError::Field(format!("missing `{name}`")))
}

fn string_field(value: &Value, name: &str) -> Result<String, PrefabSnapshotDecodeError> {
    field(value, name)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| PrefabSnapshotDecodeError::Field(format!("`{name}` must be a string")))
}

fn bool_field(value: &Value, name: &str) -> Result<bool, PrefabSnapshotDecodeError> {
    field(value, name)?
        .as_bool()
        .ok_or_else(|| PrefabSnapshotDecodeError::Field(format!("`{name}` must be a bool")))
}

fn u64_field(value: &Value, name: &str) -> Result<u64, PrefabSnapshotDecodeError> {
    field(value, name)?
        .as_u64()
        .ok_or_else(|| PrefabSnapshotDecodeError::Field(format!("`{name}` must be a u64")))
}

fn u32_field(value: &Value, name: &str) -> Result<u32, PrefabSnapshotDecodeError> {
    u64_field(value, name)?
        .try_into()
        .map_err(|_| PrefabSnapshotDecodeError::Field(format!("`{name}` must be a u32")))
}

fn array_field<'a>(value: &'a Value, name: &str) -> Result<&'a [Value], PrefabSnapshotDecodeError> {
    field(value, name)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| PrefabSnapshotDecodeError::Field(format!("`{name}` must be an array")))
}

fn optional_u64(value: &Value, name: &str) -> Result<Option<u64>, PrefabSnapshotDecodeError> {
    match field(value, name)? {
        Value::Null => Ok(None),
        value => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| PrefabSnapshotDecodeError::Field(format!("`{name}` must be a u64"))),
    }
}

fn optional_string(value: &Value, name: &str) -> Result<Option<String>, PrefabSnapshotDecodeError> {
    match field(value, name)? {
        Value::Null => Ok(None),
        value => value
            .as_str()
            .map(|item| Some(item.to_owned()))
            .ok_or_else(|| PrefabSnapshotDecodeError::Field(format!("`{name}` must be a string"))),
    }
}

fn string_array(value: &Value, name: &str) -> Result<Vec<String>, PrefabSnapshotDecodeError> {
    array_field(value, name)?
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                PrefabSnapshotDecodeError::Field(format!("`{name}` entries must be strings"))
            })
        })
        .collect()
}

fn f32_array<const N: usize>(
    value: &Value,
    name: &str,
) -> Result<[f32; N], PrefabSnapshotDecodeError> {
    let values = array_field(value, name)?;
    let converted = values
        .iter()
        .map(|item| {
            item.as_f64().map(|number| number as f32).ok_or_else(|| {
                PrefabSnapshotDecodeError::Field(format!("`{name}` entries must be numbers"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    converted
        .try_into()
        .map_err(|_| PrefabSnapshotDecodeError::Field(format!("`{name}` must contain {N} values")))
}

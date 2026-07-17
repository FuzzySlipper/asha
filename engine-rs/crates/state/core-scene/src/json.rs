//! Std-only canonical JSON encode/decode for the flat scene document.
//!
//! The workspace has zero external dependencies, so — like
//! `render-bridge/src/json.rs` — this hand-writes the exact JSON shape that TS
//! authoring/visualization tools read and write. [`encode`] emits a
//! deterministic, canonicalized document (nodes sorted by stable id, fixed field
//! order); [`decode`] parses authored JSON back into a [`FlatSceneDocument`] so
//! Rust authority validates what TS authored. Encode∘decode is a fixed point on
//! a canonicalized document, which the golden-fixture test pins.

use core_assets::{AssetHash, AssetId, AssetReference, AssetVersionReq};
use core_ids::{SceneId, SceneNodeId};
use core_math::Vec3;

use crate::document::{
    FlatSceneDocument, NodeMetadata, SceneBootstrapBindings, SceneCatalogBinding,
    SceneEntityInstance, SceneEntityReference, SceneGeneratorBinding, SceneMetadata, SceneNodeKind,
    SceneNodeRecord,
};
use crate::transform::{Quat, SceneTransform};
use crate::{SceneLight, SceneLightShadowIntent};

// ── Encode ────────────────────────────────────────────────────────────────────

/// Encode a document as canonical JSON (LF newlines, trailing newline). The
/// input is canonicalized first, so equivalent documents encode byte-identically.
pub fn encode(doc: &FlatSceneDocument) -> String {
    let doc = doc.canonical();
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"schemaVersion\": {},\n", doc.schema_version));
    out.push_str(&format!("  \"id\": {},\n", doc.id.raw()));

    out.push_str("  \"metadata\": ");
    encode_metadata(&mut out, &doc.metadata);
    out.push_str(",\n");

    out.push_str("  \"dependencies\": ");
    encode_dependencies(&mut out, &doc.dependencies);
    out.push_str(",\n");

    out.push_str("  \"nodes\": [");
    if doc.nodes.is_empty() {
        out.push(']');
    } else {
        out.push('\n');
        for (i, node) in doc.nodes.iter().enumerate() {
            out.push_str("    ");
            encode_node(&mut out, node);
            if i + 1 < doc.nodes.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]");
    }
    out.push('\n');
    out.push_str("}\n");
    out
}

fn encode_metadata(out: &mut String, meta: &SceneMetadata) {
    out.push_str("{ \"name\": ");
    encode_opt_str(out, meta.name.as_deref());
    out.push_str(&format!(
        ", \"authoringFormatVersion\": {} }}",
        meta.authoring_format_version
    ));
}

fn encode_dependencies(out: &mut String, deps: &[AssetReference]) {
    if deps.is_empty() {
        out.push_str("[]");
        return;
    }
    out.push_str("[\n");
    for (i, dep) in deps.iter().enumerate() {
        out.push_str("    ");
        encode_asset_ref(out, dep);
        if i + 1 < deps.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]");
}

fn encode_node(out: &mut String, node: &SceneNodeRecord) {
    out.push_str(&format!("{{ \"id\": {}, \"parent\": ", node.id.raw()));
    match node.parent {
        Some(p) => out.push_str(&p.raw().to_string()),
        None => out.push_str("null"),
    }
    out.push_str(&format!(
        ", \"childOrder\": {}, \"label\": ",
        node.child_order
    ));
    encode_opt_str(out, node.metadata.label.as_deref());
    out.push_str(", \"tags\": ");
    encode_str_array(out, &node.metadata.tags);
    out.push_str(", \"transform\": ");
    encode_transform(out, &node.transform);
    out.push_str(", \"kind\": ");
    encode_kind(out, &node.kind);
    out.push_str(" }");
}

fn encode_kind(out: &mut String, kind: &SceneNodeKind) {
    out.push_str(&format!("{{ \"kind\": \"{}\"", kind.tag()));
    if let Some(asset) = kind.asset() {
        out.push_str(", \"asset\": ");
        encode_asset_ref(out, asset);
    }
    if let SceneNodeKind::Light(light) = kind {
        out.push_str(", \"sceneLight\": ");
        encode_light(out, light);
    }
    if let SceneNodeKind::EntityInstance(instance) = kind {
        out.push_str(", \"instance\": ");
        encode_entity_instance(out, instance);
    }
    if let SceneNodeKind::Bootstrap(bindings) = kind {
        out.push_str(", \"bindings\": ");
        encode_bootstrap_bindings(out, bindings);
    }
    out.push_str(" }");
}

fn encode_entity_instance(out: &mut String, instance: &SceneEntityInstance) {
    out.push_str("{ \"instanceId\": ");
    encode_opt_str(out, Some(&instance.instance_id));
    out.push_str(", \"reference\": ");
    match &instance.reference {
        SceneEntityReference::EntityDefinition { stable_id } => {
            out.push_str("{ \"kind\": \"entityDefinition\", \"stableId\": ");
            encode_opt_str(out, Some(stable_id));
            out.push_str(" }");
        }
        SceneEntityReference::Prefab {
            prefab_id,
            variant_id,
        } => {
            out.push_str(&format!(
                "{{ \"kind\": \"prefab\", \"prefabId\": {prefab_id}, \"variantId\": "
            ));
            encode_opt_str(out, variant_id.as_deref());
            out.push_str(" }");
        }
    }
    out.push_str(", \"spawnMarkerId\": ");
    encode_opt_str(out, instance.spawn_marker_id.as_deref());
    out.push_str(" }");
}

fn encode_bootstrap_bindings(out: &mut String, bindings: &SceneBootstrapBindings) {
    out.push_str("{ \"generator\": ");
    match &bindings.generator {
        Some(generator) => {
            out.push_str("{ \"providerId\": ");
            encode_opt_str(out, Some(&generator.provider_id));
            out.push_str(", \"presetId\": ");
            encode_opt_str(out, Some(&generator.preset_id));
            out.push_str(&format!(", \"seed\": {} }}", generator.seed));
        }
        None => out.push_str("null"),
    }
    out.push_str(", \"catalogs\": [");
    for (index, catalog) in bindings.catalogs.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str("{ \"bindingId\": ");
        encode_opt_str(out, Some(&catalog.binding_id));
        out.push_str(", \"catalogId\": ");
        encode_opt_str(out, Some(&catalog.catalog_id));
        out.push_str(", \"sourcePath\": ");
        encode_opt_str(out, Some(&catalog.source_path));
        out.push_str(" }");
    }
    out.push_str("] }");
}

fn encode_light(out: &mut String, light: &SceneLight) {
    let (kind, color, intensity, enabled, shadow) = match light {
        SceneLight::Ambient {
            color,
            intensity,
            enabled,
            shadow_intent,
        } => ("ambient", color, intensity, enabled, shadow_intent),
        SceneLight::Directional {
            color,
            intensity,
            enabled,
            shadow_intent,
        } => ("directional", color, intensity, enabled, shadow_intent),
        SceneLight::Point {
            color,
            intensity,
            enabled,
            shadow_intent,
            ..
        } => ("point", color, intensity, enabled, shadow_intent),
        SceneLight::Spot {
            color,
            intensity,
            enabled,
            shadow_intent,
            ..
        } => ("spot", color, intensity, enabled, shadow_intent),
    };
    out.push_str(&format!("{{ \"kind\": \"{kind}\", \"color\": "));
    encode_f32_array(out, color);
    out.push_str(&format!(
        ", \"intensity\": {}, \"enabled\": {}, \"shadowIntent\": \"{}\"",
        fmt_f32(*intensity),
        enabled,
        match shadow {
            SceneLightShadowIntent::Disabled => "disabled",
            SceneLightShadowIntent::Requested => "requested",
        }
    ));
    match light {
        SceneLight::Point { range, decay, .. } => encode_light_range(out, *range, *decay),
        SceneLight::Spot {
            range,
            decay,
            outer_angle_radians,
            penumbra,
            ..
        } => {
            encode_light_range(out, *range, *decay);
            out.push_str(&format!(
                ", \"outerAngleRadians\": {}, \"penumbra\": {}",
                fmt_f32(*outer_angle_radians),
                fmt_f32(*penumbra)
            ));
        }
        SceneLight::Ambient { .. } | SceneLight::Directional { .. } => {}
    }
    out.push_str(" }");
}

fn encode_light_range(out: &mut String, range: Option<f32>, decay: f32) {
    out.push_str(", \"range\": ");
    match range {
        Some(value) => out.push_str(&fmt_f32(value)),
        None => out.push_str("null"),
    }
    out.push_str(&format!(", \"decay\": {}", fmt_f32(decay)));
}

fn encode_f32_array(out: &mut String, values: &[f32]) {
    out.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&fmt_f32(*value));
    }
    out.push(']');
}

fn encode_asset_ref(out: &mut String, r: &AssetReference) {
    out.push_str(&format!(
        "{{ \"id\": \"{}\", \"version\": ",
        r.id().as_str()
    ));
    match r.version() {
        AssetVersionReq::Any => out.push_str("{ \"req\": \"any\" }"),
        AssetVersionReq::Exact(v) => {
            out.push_str(&format!("{{ \"req\": \"exact\", \"value\": {v} }}"))
        }
        AssetVersionReq::AtLeast(v) => {
            out.push_str(&format!("{{ \"req\": \"atLeast\", \"value\": {v} }}"))
        }
    }
    out.push_str(", \"hash\": ");
    match r.hash() {
        Some(h) => out.push_str(&format!("\"{}\"", h.as_str())),
        None => out.push_str("null"),
    }
    out.push_str(" }");
}

fn encode_transform(out: &mut String, t: &SceneTransform) {
    out.push_str("{ \"translation\": ");
    encode_vec3(out, t.translation);
    out.push_str(", \"rotation\": ");
    out.push_str(&format!(
        "[{}, {}, {}, {}]",
        fmt_f32(t.rotation.x),
        fmt_f32(t.rotation.y),
        fmt_f32(t.rotation.z),
        fmt_f32(t.rotation.w)
    ));
    out.push_str(", \"scale\": ");
    encode_vec3(out, t.scale);
    out.push_str(" }");
}

fn encode_vec3(out: &mut String, v: Vec3) {
    out.push_str(&format!(
        "[{}, {}, {}]",
        fmt_f32(v.x),
        fmt_f32(v.y),
        fmt_f32(v.z)
    ));
}

fn encode_str_array(out: &mut String, items: &[String]) {
    out.push('[');
    for (i, s) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("\"{}\"", escape(s)));
    }
    out.push(']');
}

fn encode_opt_str(out: &mut String, s: Option<&str>) {
    match s {
        Some(s) => out.push_str(&format!("\"{}\"", escape(s))),
        None => out.push_str("null"),
    }
}

/// Shortest round-trippable rendering of an `f32` (Rust's `Display` is
/// deterministic). Used so canonical output is stable across runs/platforms.
fn fmt_f32(v: f32) -> String {
    format!("{v}")
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

// ── Decode ────────────────────────────────────────────────────────────────────

/// Why decoding a scene JSON document failed (structural, before semantic
/// validation). Semantic problems (cycles, wrong-kind refs) surface from
/// [`crate::validate`] instead.
#[derive(Debug, Clone, PartialEq)]
pub enum SceneDecodeError {
    /// The bytes were not valid JSON.
    Json(String),
    /// A required field was missing or had the wrong type.
    Field(String),
    /// An asset id/hash string in the document was malformed.
    Asset(String),
    /// A `kind` discriminant was not recognized.
    UnknownKind(String),
    /// A `version.req` discriminant was not recognized.
    UnknownVersionReq(String),
    /// The removed Demo-only scene shape was supplied. Callers can route a
    /// migration diagnostic without confusing it with malformed canonical JSON.
    LegacyDemoScene,
}

/// Decode canonical/authored scene JSON into a flat document. The result is
/// **not** canonicalized or validated; call [`FlatSceneDocument::canonicalize`]
/// and [`crate::validate`] as needed.
pub fn decode(input: &str) -> Result<FlatSceneDocument, SceneDecodeError> {
    let json = Json::parse(input).map_err(SceneDecodeError::Json)?;
    if json.get("kind").and_then(Json::as_str) == Some("SceneDocument")
        && json.get("sceneId").is_some()
        && json.get("placements").is_some()
    {
        return Err(SceneDecodeError::LegacyDemoScene);
    }
    require_keys(
        &json,
        &["schemaVersion", "id", "metadata", "dependencies", "nodes"],
        "scene document",
    )?;
    let schema_version = field_u64(&json, "schemaVersion")? as u32;
    let id = SceneId::new(field_u64(&json, "id")?);
    let metadata = decode_metadata(field(&json, "metadata")?)?;
    let dependencies = decode_dependencies(field(&json, "dependencies")?)?;

    let nodes_json = field(&json, "nodes")?
        .as_array()
        .ok_or_else(|| SceneDecodeError::Field("nodes must be an array".into()))?;
    let mut nodes = Vec::with_capacity(nodes_json.len());
    for n in nodes_json {
        nodes.push(decode_node(n)?);
    }

    Ok(FlatSceneDocument {
        id,
        schema_version,
        metadata,
        dependencies,
        nodes,
    })
}

fn decode_metadata(j: &Json) -> Result<SceneMetadata, SceneDecodeError> {
    require_keys(j, &["name", "authoringFormatVersion"], "scene metadata")?;
    Ok(SceneMetadata {
        name: opt_str(j, "name")?,
        authoring_format_version: field_u64(j, "authoringFormatVersion")? as u32,
    })
}

fn decode_dependencies(j: &Json) -> Result<Vec<AssetReference>, SceneDecodeError> {
    let arr = j
        .as_array()
        .ok_or_else(|| SceneDecodeError::Field("dependencies must be an array".into()))?;
    arr.iter().map(decode_asset_ref).collect()
}

fn decode_node(j: &Json) -> Result<SceneNodeRecord, SceneDecodeError> {
    require_keys(
        j,
        &[
            "id",
            "parent",
            "childOrder",
            "label",
            "tags",
            "transform",
            "kind",
        ],
        "scene node",
    )?;
    let id = SceneNodeId::new(field_u64(j, "id")?);
    let parent = match j.get("parent") {
        Some(Json::Null) | None => None,
        Some(Json::Num(_)) => Some(SceneNodeId::new(field_u64(j, "parent")?)),
        Some(_) => {
            return Err(SceneDecodeError::Field(
                "parent must be a number or null".into(),
            ))
        }
    };
    let child_order = field_u64(j, "childOrder")? as u32;
    let metadata = NodeMetadata {
        label: opt_str(j, "label")?,
        tags: decode_str_array(j.get("tags"))?,
    };
    let transform = decode_transform(field(j, "transform")?)?;
    let kind = decode_kind(field(j, "kind")?)?;
    Ok(SceneNodeRecord {
        id,
        parent,
        child_order,
        transform,
        kind,
        metadata,
    })
}

fn decode_kind(j: &Json) -> Result<SceneNodeKind, SceneDecodeError> {
    let tag = j
        .get("kind")
        .and_then(Json::as_str)
        .ok_or_else(|| SceneDecodeError::Field("kind.kind must be a string".into()))?;
    let asset = || -> Result<AssetReference, SceneDecodeError> {
        decode_asset_ref(
            j.get("asset")
                .ok_or_else(|| SceneDecodeError::Field(format!("kind `{tag}` requires `asset`")))?,
        )
    };
    match tag {
        "emptyGroup" => {
            require_keys(j, &["kind"], "emptyGroup kind")?;
            Ok(SceneNodeKind::EmptyGroup)
        }
        "staticMesh" => {
            require_keys(j, &["kind", "asset"], "staticMesh kind")?;
            Ok(SceneNodeKind::StaticMesh(asset()?))
        }
        "sprite" => {
            require_keys(j, &["kind", "asset"], "sprite kind")?;
            Ok(SceneNodeKind::Sprite(asset()?))
        }
        "voxelVolume" => {
            require_keys(j, &["kind", "asset"], "voxelVolume kind")?;
            Ok(SceneNodeKind::VoxelVolume(asset()?))
        }
        "light" => {
            require_keys(j, &["kind", "sceneLight"], "light kind")?;
            Ok(SceneNodeKind::Light(decode_light(field(j, "sceneLight")?)?))
        }
        "entityInstance" => {
            require_keys(j, &["kind", "instance"], "entityInstance kind")?;
            Ok(SceneNodeKind::EntityInstance(decode_entity_instance(
                field(j, "instance")?,
            )?))
        }
        "bootstrap" => {
            require_keys(j, &["kind", "bindings"], "bootstrap kind")?;
            Ok(SceneNodeKind::Bootstrap(decode_bootstrap_bindings(field(
                j, "bindings",
            )?)?))
        }
        other => Err(SceneDecodeError::UnknownKind(other.to_string())),
    }
}

fn decode_entity_instance(j: &Json) -> Result<SceneEntityInstance, SceneDecodeError> {
    require_keys(
        j,
        &["instanceId", "reference", "spawnMarkerId"],
        "entity instance",
    )?;
    let reference_json = field(j, "reference")?;
    let reference_kind = field(reference_json, "kind")?.as_str().ok_or_else(|| {
        SceneDecodeError::Field("instance.reference.kind must be a string".into())
    })?;
    let reference = match reference_kind {
        "entityDefinition" => {
            require_keys(
                reference_json,
                &["kind", "stableId"],
                "entity definition reference",
            )?;
            SceneEntityReference::EntityDefinition {
                stable_id: field_str(reference_json, "stableId")?,
            }
        }
        "prefab" => {
            require_keys(
                reference_json,
                &["kind", "prefabId", "variantId"],
                "prefab reference",
            )?;
            SceneEntityReference::Prefab {
                prefab_id: field_u64(reference_json, "prefabId")?,
                variant_id: opt_str(reference_json, "variantId")?,
            }
        }
        other => return Err(SceneDecodeError::UnknownKind(other.to_string())),
    };
    Ok(SceneEntityInstance {
        instance_id: field_str(j, "instanceId")?,
        reference,
        spawn_marker_id: opt_str(j, "spawnMarkerId")?,
    })
}

fn decode_bootstrap_bindings(j: &Json) -> Result<SceneBootstrapBindings, SceneDecodeError> {
    require_keys(j, &["generator", "catalogs"], "bootstrap bindings")?;
    let generator = match j.get("generator") {
        Some(Json::Null) | None => None,
        Some(value) => {
            require_keys(
                value,
                &["providerId", "presetId", "seed"],
                "generator binding",
            )?;
            Some(SceneGeneratorBinding {
                provider_id: field_str(value, "providerId")?,
                preset_id: field_str(value, "presetId")?,
                seed: field_u64(value, "seed")?,
            })
        }
    };
    let catalogs_json = field(j, "catalogs")?
        .as_array()
        .ok_or_else(|| SceneDecodeError::Field("bindings.catalogs must be an array".into()))?;
    let catalogs = catalogs_json
        .iter()
        .map(|value| {
            require_keys(
                value,
                &["bindingId", "catalogId", "sourcePath"],
                "catalog binding",
            )?;
            Ok(SceneCatalogBinding {
                binding_id: field_str(value, "bindingId")?,
                catalog_id: field_str(value, "catalogId")?,
                source_path: field_str(value, "sourcePath")?,
            })
        })
        .collect::<Result<Vec<_>, SceneDecodeError>>()?;
    Ok(SceneBootstrapBindings {
        generator,
        catalogs,
    })
}

fn decode_light(j: &Json) -> Result<SceneLight, SceneDecodeError> {
    let kind = field(j, "kind")?
        .as_str()
        .ok_or_else(|| SceneDecodeError::Field("light.kind must be a string".into()))?;
    let color = decode_f32_array::<3>(field(j, "color")?, "light.color")?;
    let intensity = num(field(j, "intensity")?)?;
    let enabled = field_bool(j, "enabled")?;
    let shadow_intent = match field(j, "shadowIntent")?.as_str() {
        Some("disabled") => SceneLightShadowIntent::Disabled,
        Some("requested") => SceneLightShadowIntent::Requested,
        _ => {
            return Err(SceneDecodeError::Field(
                "light.shadowIntent must be `disabled` or `requested`".into(),
            ))
        }
    };
    match kind {
        "ambient" => {
            require_keys(
                j,
                &["kind", "color", "intensity", "enabled", "shadowIntent"],
                "ambient light",
            )?;
            Ok(SceneLight::Ambient {
                color,
                intensity,
                enabled,
                shadow_intent,
            })
        }
        "directional" => {
            require_keys(
                j,
                &["kind", "color", "intensity", "enabled", "shadowIntent"],
                "directional light",
            )?;
            Ok(SceneLight::Directional {
                color,
                intensity,
                enabled,
                shadow_intent,
            })
        }
        "point" => {
            require_keys(
                j,
                &[
                    "kind",
                    "color",
                    "intensity",
                    "enabled",
                    "shadowIntent",
                    "range",
                    "decay",
                ],
                "point light",
            )?;
            Ok(SceneLight::Point {
                color,
                intensity,
                enabled,
                range: opt_num(j, "range")?,
                decay: num(field(j, "decay")?)?,
                shadow_intent,
            })
        }
        "spot" => {
            require_keys(
                j,
                &[
                    "kind",
                    "color",
                    "intensity",
                    "enabled",
                    "shadowIntent",
                    "range",
                    "decay",
                    "outerAngleRadians",
                    "penumbra",
                ],
                "spot light",
            )?;
            Ok(SceneLight::Spot {
                color,
                intensity,
                enabled,
                range: opt_num(j, "range")?,
                decay: num(field(j, "decay")?)?,
                outer_angle_radians: num(field(j, "outerAngleRadians")?)?,
                penumbra: num(field(j, "penumbra")?)?,
                shadow_intent,
            })
        }
        other => Err(SceneDecodeError::UnknownKind(format!("light.{other}"))),
    }
}

fn decode_asset_ref(j: &Json) -> Result<AssetReference, SceneDecodeError> {
    require_keys(j, &["id", "version", "hash"], "asset reference")?;
    let id_str = j
        .get("id")
        .and_then(Json::as_str)
        .ok_or_else(|| SceneDecodeError::Field("asset.id must be a string".into()))?;
    let id = AssetId::parse(id_str).map_err(|e| SceneDecodeError::Asset(e.to_string()))?;

    let version = match j.get("version") {
        None | Some(Json::Null) => AssetVersionReq::Any,
        Some(v) => {
            require_keys(v, &["req", "value"], "asset version")?;
            let req = v
                .get("req")
                .and_then(Json::as_str)
                .ok_or_else(|| SceneDecodeError::Field("version.req must be a string".into()))?;
            match req {
                "any" => AssetVersionReq::Any,
                "exact" => AssetVersionReq::Exact(field_u64(v, "value")? as u32),
                "atLeast" => AssetVersionReq::AtLeast(field_u64(v, "value")? as u32),
                other => return Err(SceneDecodeError::UnknownVersionReq(other.to_string())),
            }
        }
    };

    let hash = match j.get("hash") {
        None | Some(Json::Null) => None,
        Some(Json::Str(s)) => {
            Some(AssetHash::parse(s).map_err(|e| SceneDecodeError::Asset(e.to_string()))?)
        }
        Some(_) => {
            return Err(SceneDecodeError::Field(
                "hash must be a string or null".into(),
            ))
        }
    };

    Ok(AssetReference::new(id, version, hash))
}

fn decode_transform(j: &Json) -> Result<SceneTransform, SceneDecodeError> {
    require_keys(j, &["translation", "rotation", "scale"], "scene transform")?;
    let translation = decode_vec3(field(j, "translation")?)?;
    let rot = field(j, "rotation")?
        .as_array()
        .filter(|a| a.len() == 4)
        .ok_or_else(|| SceneDecodeError::Field("rotation must be a 4-array".into()))?;
    let rotation = Quat::new(num(&rot[0])?, num(&rot[1])?, num(&rot[2])?, num(&rot[3])?);
    let scale = decode_vec3(field(j, "scale")?)?;
    Ok(SceneTransform {
        translation,
        rotation,
        scale,
    })
}

fn decode_vec3(j: &Json) -> Result<Vec3, SceneDecodeError> {
    let a = j
        .as_array()
        .filter(|a| a.len() == 3)
        .ok_or_else(|| SceneDecodeError::Field("vec3 must be a 3-array".into()))?;
    Ok(Vec3::new(num(&a[0])?, num(&a[1])?, num(&a[2])?))
}

fn decode_str_array(j: Option<&Json>) -> Result<Vec<String>, SceneDecodeError> {
    match j {
        None | Some(Json::Null) => Ok(Vec::new()),
        Some(Json::Arr(items)) => items
            .iter()
            .map(|i| {
                i.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| SceneDecodeError::Field("tag must be a string".into()))
            })
            .collect(),
        Some(_) => Err(SceneDecodeError::Field("tags must be an array".into())),
    }
}

// ── Small typed-field helpers over `Json` ─────────────────────────────────────

fn field<'a>(j: &'a Json, key: &str) -> Result<&'a Json, SceneDecodeError> {
    j.get(key)
        .ok_or_else(|| SceneDecodeError::Field(format!("missing field `{key}`")))
}

fn field_u64(j: &Json, key: &str) -> Result<u64, SceneDecodeError> {
    field(j, key)?.as_u64().ok_or_else(|| {
        SceneDecodeError::Field(format!("field `{key}` must be a non-negative integer"))
    })
}

fn field_bool(j: &Json, key: &str) -> Result<bool, SceneDecodeError> {
    match field(j, key)? {
        Json::Bool(value) => Ok(*value),
        _ => Err(SceneDecodeError::Field(format!(
            "field `{key}` must be a boolean"
        ))),
    }
}

fn opt_num(j: &Json, key: &str) -> Result<Option<f32>, SceneDecodeError> {
    match j.get(key) {
        Some(Json::Null) | None => Ok(None),
        Some(value) => num(value).map(Some),
    }
}

fn decode_f32_array<const N: usize>(j: &Json, label: &str) -> Result<[f32; N], SceneDecodeError> {
    let values = j
        .as_array()
        .filter(|values| values.len() == N)
        .ok_or_else(|| SceneDecodeError::Field(format!("{label} must be a {N}-array")))?;
    let mut result = [0.0; N];
    for (index, value) in values.iter().enumerate() {
        result[index] = num(value)?;
    }
    Ok(result)
}

fn require_keys(j: &Json, allowed: &[&str], label: &str) -> Result<(), SceneDecodeError> {
    let Json::Obj(entries) = j else {
        return Err(SceneDecodeError::Field(format!(
            "{label} must be an object"
        )));
    };
    let mut seen = std::collections::BTreeSet::new();
    for (key, _) in entries {
        if !allowed.contains(&key.as_str()) {
            return Err(SceneDecodeError::Field(format!(
                "{label} contains unknown field `{key}`"
            )));
        }
        if !seen.insert(key) {
            return Err(SceneDecodeError::Field(format!(
                "{label} contains duplicate field `{key}`"
            )));
        }
    }
    Ok(())
}

fn opt_str(j: &Json, key: &str) -> Result<Option<String>, SceneDecodeError> {
    match j.get(key) {
        None | Some(Json::Null) => Ok(None),
        Some(Json::Str(s)) => Ok(Some(s.clone())),
        Some(_) => Err(SceneDecodeError::Field(format!(
            "field `{key}` must be a string or null"
        ))),
    }
}

fn field_str(j: &Json, key: &str) -> Result<String, SceneDecodeError> {
    match field(j, key)? {
        Json::Str(value) => Ok(value.clone()),
        _ => Err(SceneDecodeError::Field(format!(
            "field `{key}` must be a string"
        ))),
    }
}

fn num(j: &Json) -> Result<f32, SceneDecodeError> {
    match j {
        Json::Num(n) => Ok(*n as f32),
        _ => Err(SceneDecodeError::Field("expected a number".into())),
    }
}

// ── Minimal JSON value + parser (std-only) ────────────────────────────────────

/// A parsed JSON value. Supports the object/array/string/number/bool/null subset
/// the scene contract uses, with the common string escapes.
#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn parse(input: &str) -> Result<Json, String> {
        let chars: Vec<char> = input.chars().collect();
        let mut p = Parser { chars, pos: 0 };
        p.skip_ws();
        let v = p.value()?;
        p.skip_ws();
        if p.pos != p.chars.len() {
            return Err(format!("trailing input at position {}", p.pos));
        }
        Ok(v)
    }

    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            Json::Num(n) if n.fract() == 0.0 && *n >= 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(items) => Some(items),
            _ => None,
        }
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(Json::Str(self.string()?)),
            Some('t') | Some('f') => self.boolean(),
            Some('n') => self.null(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            other => Err(format!("unexpected {other:?} at {}", self.pos)),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.expect('{')?;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(Json::Obj(entries));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            self.skip_ws();
            self.expect(':')?;
            let val = self.value()?;
            entries.push((key, val));
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some('}') => break,
                other => return Err(format!("expected ',' or '}}', got {other:?}")),
            }
        }
        Ok(Json::Obj(entries))
    }

    fn array(&mut self) -> Result<Json, String> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_ws();
            match self.bump() {
                Some(',') => continue,
                Some(']') => break,
                other => return Err(format!("expected ',' or ']', got {other:?}")),
            }
        }
        Ok(Json::Arr(items))
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.bump() {
                Some('"') => break,
                Some('\\') => match self.bump() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    other => return Err(format!("bad escape {other:?}")),
                },
                Some(c) => out.push(c),
                None => return Err("unterminated string".into()),
            }
        }
        Ok(out)
    }

    fn boolean(&mut self) -> Result<Json, String> {
        if self.consume("true") {
            Ok(Json::Bool(true))
        } else if self.consume("false") {
            Ok(Json::Bool(false))
        } else {
            Err(format!("bad literal at {}", self.pos))
        }
    }

    fn null(&mut self) -> Result<Json, String> {
        if self.consume("null") {
            Ok(Json::Null)
        } else {
            Err(format!("bad literal at {}", self.pos))
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-')
        {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>()
            .map(Json::Num)
            .map_err(|_| format!("bad number `{s}`"))
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.bump() == Some(c) {
            Ok(())
        } else {
            Err(format!("expected '{c}' at {}", self.pos))
        }
    }

    fn consume(&mut self, lit: &str) -> bool {
        let end = self.pos + lit.len();
        if end <= self.chars.len() && self.chars[self.pos..end].iter().collect::<String>() == lit {
            self.pos = end;
            true
        } else {
            false
        }
    }
}

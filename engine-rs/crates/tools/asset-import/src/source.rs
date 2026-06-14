//! The documented offline source-mesh format and its deterministic parser.
//!
//! # Supported source format (schema version 1)
//!
//! The first-pass importer reads ASHA's own small JSON **source-mesh** format
//! (`*.mesh.json`), chosen over a glTF/glb binary subset for the first pass so the
//! import is dependency-free and trivially deterministic. A glTF-subset front-end
//! can later target this same importer core. The format is, exactly:
//!
//! ```json
//! {
//!   "schemaVersion": 1,
//!   "name": "import-fixture-a",          // base name; importer emits mesh/<name>
//!   "positions": [x, y, z, ...],         // 3 floats per vertex
//!   "normals":   [x, y, z, ...],         // 3 floats per vertex (same vertex count)
//!   "indices":   [i, ...],               // triangle list; length a multiple of 3
//!   "materials": [                       // optional; defaults to one flat slot 0
//!     { "slot": 0, "name": "surface-a", "color": [r, g, b, a], "texture": "surface-a" }
//!   ],
//!   "groups": [                          // optional; defaults to one group over all
//!     { "materialSlot": 0, "start": 0, "count": 3 }
//!   ],
//!   "collision": "visualOnly"            // "visualOnly" | "aabbFallback" | { "proxy": "id" }
//! }
//! ```
//!
//! # Explicit non-goals / rejected features
//!
//! Triangle lists only; separate position/normal streams only; no UV/colour/joint
//! attributes; no animations, skins, morph targets, cameras, or lights. Any of
//! those recognised-but-unsupported keys is **rejected with a classified
//! diagnostic**, never silently dropped. There is no runtime import path.

use crate::diagnostic::{ImportCode, ImportDiagnostic};
use crate::json::Json;

/// The schema version this importer understands.
pub const SUPPORTED_SCHEMA: u64 = 1;

/// Source top-level keys that name recognised-but-unsupported features.
const UNSUPPORTED_KEYS: &[&str] = &["animations", "skins", "morphTargets", "cameras", "lights"];

/// A material slot declared by the source.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceMaterial {
    pub slot: u16,
    pub name: String,
    pub color: [f32; 4],
    pub texture: Option<String>,
}

/// A draw group binding an index range to a material slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceGroup {
    pub material_slot: u16,
    pub start: u32,
    pub count: u32,
}

/// The collision policy the source requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceCollision {
    VisualOnly,
    AabbFallback,
    Proxy(String),
}

/// A parsed, structurally-loaded source mesh (not yet validated for import).
#[derive(Debug, Clone, PartialEq)]
pub struct SourceMesh {
    pub schema_version: u64,
    pub name: String,
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
    pub materials: Vec<SourceMaterial>,
    pub groups: Vec<SourceGroup>,
    pub collision: SourceCollision,
}

/// The result of parsing a source document: the mesh (when fatal-error-free) and
/// any classified diagnostics gathered along the way.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceParse {
    pub mesh: Option<SourceMesh>,
    pub diagnostics: Vec<ImportDiagnostic>,
}

fn num_array(value: &Json) -> Option<Vec<f64>> {
    value
        .as_array()?
        .iter()
        .map(Json::as_f64)
        .collect::<Option<Vec<f64>>>()
}

fn parse_color(value: Option<&Json>) -> [f32; 4] {
    match value.and_then(num_array) {
        Some(v) if v.len() == 4 => [v[0] as f32, v[1] as f32, v[2] as f32, v[3] as f32],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

/// Parse the documented source-mesh JSON. `locus` is the source path (or a label)
/// used in diagnostics. Recognised-unsupported features and malformed structure are
/// reported as classified errors rather than panics or silent drops.
pub fn parse_source(text: &str, locus: &str) -> SourceParse {
    let mut diagnostics = Vec::new();

    let root = match Json::parse(text) {
        Ok(v) => v,
        Err(e) => {
            diagnostics.push(ImportDiagnostic::error(
                ImportCode::MalformedSource,
                locus,
                format!("source is not valid JSON: {e}"),
                "fix the source JSON syntax",
            ));
            return SourceParse {
                mesh: None,
                diagnostics,
            };
        }
    };

    // Recognised-but-unsupported feature keys are rejected, not ignored.
    for key in root.keys() {
        if UNSUPPORTED_KEYS.contains(&key) {
            diagnostics.push(ImportDiagnostic::error(
                ImportCode::UnsupportedFeature,
                format!("{locus}#{key}"),
                format!("source feature `{key}` is not supported by this importer"),
                "remove the feature from the source or extend the importer scope",
            ));
        }
    }

    let schema_version = root
        .get("schemaVersion")
        .and_then(Json::as_u64)
        .unwrap_or(0);
    if schema_version != SUPPORTED_SCHEMA {
        diagnostics.push(ImportDiagnostic::error(
            ImportCode::UnsupportedSchema,
            locus,
            format!("source schema version {schema_version} is unsupported"),
            format!("re-author the source at schema version {SUPPORTED_SCHEMA}"),
        ));
    }

    let name = root
        .get("name")
        .and_then(Json::as_str)
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        diagnostics.push(ImportDiagnostic::error(
            ImportCode::MalformedSource,
            locus,
            "source is missing a non-empty `name`".to_string(),
            "add a `name` for the generated asset id",
        ));
    }

    let positions: Vec<f32> = root
        .get("positions")
        .and_then(num_array)
        .map(|v| v.into_iter().map(|n| n as f32).collect())
        .unwrap_or_default();
    let normals: Vec<f32> = root
        .get("normals")
        .and_then(num_array)
        .map(|v| v.into_iter().map(|n| n as f32).collect())
        .unwrap_or_default();
    let indices: Vec<u32> = root
        .get("indices")
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Json::as_u64)
                .map(|n| n as u32)
                .collect()
        })
        .unwrap_or_default();

    let materials = parse_materials(root.get("materials"));
    let groups = parse_groups(root.get("groups"), indices.len() as u32);
    let collision = parse_collision(root.get("collision"));

    // A fatal diagnostic means we do not hand back a mesh to import.
    let fatal = diagnostics.iter().any(ImportDiagnostic::is_error);
    let mesh = if fatal {
        None
    } else {
        Some(SourceMesh {
            schema_version,
            name,
            positions,
            normals,
            indices,
            materials,
            groups,
            collision,
        })
    };

    SourceParse { mesh, diagnostics }
}

fn parse_materials(value: Option<&Json>) -> Vec<SourceMaterial> {
    let Some(items) = value.and_then(Json::as_array) else {
        // Default: a single flat white slot 0, so a mesh with no materials still
        // has a bound slot.
        return vec![SourceMaterial {
            slot: 0,
            name: "default".to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
            texture: None,
        }];
    };
    items
        .iter()
        .enumerate()
        .map(|(i, m)| SourceMaterial {
            slot: m.get("slot").and_then(Json::as_u64).unwrap_or(i as u64) as u16,
            name: m
                .get("name")
                .and_then(Json::as_str)
                .unwrap_or("default")
                .to_string(),
            color: parse_color(m.get("color")),
            texture: m
                .get("texture")
                .filter(|t| !t.is_null())
                .and_then(Json::as_str)
                .map(str::to_string),
        })
        .collect()
}

fn parse_groups(value: Option<&Json>, index_count: u32) -> Vec<SourceGroup> {
    match value.and_then(Json::as_array) {
        Some(items) if !items.is_empty() => items
            .iter()
            .map(|g| SourceGroup {
                material_slot: g.get("materialSlot").and_then(Json::as_u64).unwrap_or(0) as u16,
                start: g.get("start").and_then(Json::as_u64).unwrap_or(0) as u32,
                count: g.get("count").and_then(Json::as_u64).unwrap_or(0) as u32,
            })
            .collect(),
        // Default: one group over the whole index buffer, bound to slot 0.
        _ => vec![SourceGroup {
            material_slot: 0,
            start: 0,
            count: index_count,
        }],
    }
}

fn parse_collision(value: Option<&Json>) -> SourceCollision {
    match value {
        Some(Json::Str(s)) if s == "aabbFallback" => SourceCollision::AabbFallback,
        Some(Json::Obj(_)) => value
            .and_then(|v| v.get("proxy"))
            .and_then(Json::as_str)
            .map(|p| SourceCollision::Proxy(p.to_string()))
            .unwrap_or(SourceCollision::VisualOnly),
        _ => SourceCollision::VisualOnly,
    }
}

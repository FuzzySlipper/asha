//! Flat-canonical scene validation with a classified report.
//!
//! Validation runs on the [`FlatSceneDocument`] (the canonical form). Every
//! failure is a typed [`SceneValidationError`] so a future protocol diagnostic
//! can route on the variant rather than parse prose (scene-capability-01,
//! "Rust validation and error classification").

use std::collections::{HashMap, HashSet};

use core_assets::AssetKind;
use core_ids::SceneNodeId;

use crate::document::{FlatSceneDocument, SceneNodeKind};
use crate::transform::TransformInvalid;
use crate::SceneLightInvalid;

/// One classified validation failure.
#[derive(Debug, Clone, PartialEq)]
pub enum SceneValidationError {
    /// Two node records share a stable id.
    DuplicateNodeId { id: SceneNodeId },
    /// A record names a parent that is not present in the document.
    UnknownParent {
        node: SceneNodeId,
        parent: SceneNodeId,
    },
    /// The parent pointers form a cycle; `path` lists the ids in cycle order.
    Cycle { path: Vec<SceneNodeId> },
    /// A node's initial transform is invalid.
    InvalidTransform {
        node: SceneNodeId,
        reason: TransformInvalid,
    },
    /// A node references an asset of the wrong kind for its variant.
    AssetKindMismatch {
        node: SceneNodeId,
        expected: AssetKind,
        actual: AssetKind,
    },
    /// A stored light has malformed fields or a scaled pose.
    InvalidLight {
        node: SceneNodeId,
        reason: SceneLightInvalid,
    },
}

impl SceneValidationError {
    /// Short, stable label for diagnostics/serialization.
    pub fn label(&self) -> &'static str {
        match self {
            SceneValidationError::DuplicateNodeId { .. } => "duplicate-node-id",
            SceneValidationError::UnknownParent { .. } => "unknown-parent",
            SceneValidationError::Cycle { .. } => "cycle",
            SceneValidationError::InvalidTransform { .. } => "invalid-transform",
            SceneValidationError::AssetKindMismatch { .. } => "asset-kind-mismatch",
            SceneValidationError::InvalidLight { .. } => "invalid-light",
        }
    }
}

/// The outcome of validating a document: every error found, not just the first.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SceneValidationReport {
    pub errors: Vec<SceneValidationError>,
}

impl SceneValidationReport {
    /// `true` if no errors were found.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validate a flat scene document, returning every classified error.
pub fn validate(doc: &FlatSceneDocument) -> SceneValidationReport {
    let mut errors = Vec::new();

    // 1. Duplicate stable ids. `known` is the set of all ids present (used by the
    //    parent/cycle checks below); `seen`/`reported` track duplicates so each
    //    colliding id is reported exactly once.
    let mut known: HashSet<u64> = HashSet::new();
    let mut seen: HashSet<u64> = HashSet::new();
    let mut reported: HashSet<u64> = HashSet::new();
    for rec in &doc.nodes {
        let raw = rec.id.raw();
        known.insert(raw);
        if !seen.insert(raw) && reported.insert(raw) {
            errors.push(SceneValidationError::DuplicateNodeId { id: rec.id });
        }
    }

    // 2. Per-node checks: unknown parent, transform, asset kind.
    for rec in &doc.nodes {
        if let Some(parent) = rec.parent {
            if !known.contains(&parent.raw()) {
                errors.push(SceneValidationError::UnknownParent {
                    node: rec.id,
                    parent,
                });
            }
        }
        if let Err(reason) = rec.transform.validate() {
            errors.push(SceneValidationError::InvalidTransform {
                node: rec.id,
                reason,
            });
        }
        if let (Some(expected), Some(asset)) = (rec.kind.expected_asset_kind(), rec.kind.asset()) {
            if asset.kind() != expected {
                errors.push(SceneValidationError::AssetKindMismatch {
                    node: rec.id,
                    expected,
                    actual: asset.kind(),
                });
            }
        }
        if let SceneNodeKind::Light(light) = &rec.kind {
            let result = if doc.schema_version < 2 || doc.metadata.authoring_format_version < 2 {
                Err(SceneLightInvalid::RequiresSchema2)
            } else if rec.transform.scale != core_math::Vec3::ONE {
                Err(SceneLightInvalid::NonUnitScale)
            } else {
                light.validate()
            };
            if let Err(reason) = result {
                errors.push(SceneValidationError::InvalidLight {
                    node: rec.id,
                    reason,
                });
            }
        }
    }

    // 3. Cycles via the parent map. Only meaningful with present parents; an
    //    unknown parent is already reported above and terminates a walk.
    detect_cycles(doc, &known, &mut errors);

    SceneValidationReport { errors }
}

fn detect_cycles(
    doc: &FlatSceneDocument,
    known: &HashSet<u64>,
    errors: &mut Vec<SceneValidationError>,
) {
    // Last-wins parent map (duplicate ids are reported separately).
    let mut parent_of: HashMap<u64, Option<u64>> = HashMap::new();
    let mut id_of: HashMap<u64, SceneNodeId> = HashMap::new();
    for rec in &doc.nodes {
        parent_of.insert(rec.id.raw(), rec.parent.map(|p| p.raw()));
        id_of.insert(rec.id.raw(), rec.id);
    }

    let mut acyclic: HashSet<u64> = HashSet::new();
    let mut cyclic: HashSet<u64> = HashSet::new();

    // Walk starts in ascending id order so any reported cycle path is
    // deterministic regardless of hash-map iteration order.
    let mut starts: Vec<u64> = parent_of.keys().copied().collect();
    starts.sort_unstable();

    for start in starts {
        if acyclic.contains(&start) || cyclic.contains(&start) {
            continue;
        }
        let mut order: Vec<u64> = Vec::new();
        let mut local: HashSet<u64> = HashSet::new();
        let mut cur = start;
        loop {
            if cyclic.contains(&cur) {
                break;
            }
            if acyclic.contains(&cur) {
                acyclic.extend(order.iter().copied());
                break;
            }
            if local.contains(&cur) {
                // Cycle: from the first occurrence of `cur` to the end.
                let pos = order.iter().position(|&x| x == cur).unwrap();
                let path: Vec<SceneNodeId> = order[pos..].iter().map(|raw| id_of[raw]).collect();
                for raw in &order[pos..] {
                    cyclic.insert(*raw);
                }
                errors.push(SceneValidationError::Cycle { path });
                break;
            }
            local.insert(cur);
            order.push(cur);
            match parent_of.get(&cur).copied().flatten() {
                None => {
                    acyclic.extend(order.iter().copied());
                    break;
                }
                Some(p) => {
                    if !known.contains(&p) {
                        // Unknown parent: not a cycle, already reported.
                        acyclic.extend(order.iter().copied());
                        break;
                    }
                    cur = p;
                }
            }
        }
    }
}

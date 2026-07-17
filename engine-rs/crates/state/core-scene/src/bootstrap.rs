//! Atomic scene → authority bootstrap (scene-capability-01, "Bootstrap posture").
//!
//! Bootstrap is **one atomic authority initialization**, not N ordinary
//! create-entity commands. The flow is two-phase so authority is never partially
//! mutated:
//!
//! 1. [`BootstrapPlan::prepare`] validates the whole document and the schema/asset
//!    context and builds a deterministic plan. Any failure returns here, before a
//!    [`SpatialSessionState`] exists.
//! 2. [`BootstrapPlan::apply`] turns the plan into a populated [`SpatialSessionState`] plus
//!    a single [`BootstrapRecord`] — the one replay/audit unit for the whole
//!    initialization, with a deterministic world hash.
//!
//! Entity ids are allocated deterministically (ascending scene-node id from a
//! base), and scene initial transforms are copied into authority runtime
//! transforms. After bootstrap the world is authority-owned and free to diverge
//! from the scene document (see [`SpatialSessionState::set_transform`]).

use std::collections::{BTreeMap, BTreeSet};

use core_ids::{EntityId, RuntimeSessionId, SceneId, SceneNodeId};

use crate::document::{
    FlatSceneDocument, SceneBootstrapBindings, SceneEntityReference, SceneNodeKind,
};
use crate::json::encode;
use crate::spatial_session::{SpatialSessionHash, SpatialSessionState};
use crate::validate::{validate, SceneValidationReport};
use crate::SceneTransform;

/// The scene schema version this bootstrap understands. A real migration policy
/// (scene-capability-01, "Decisions to make") is future work; for now an
/// unsupported version fails closed rather than guessing.
pub const SUPPORTED_SCHEMA_VERSION: u32 = 3;

/// The default first entity id allocated to scene-sourced entities.
pub const DEFAULT_BASE_ENTITY_ID: EntityId = EntityId::new(1);

/// Why a scene could not be prepared for bootstrap. Returned *before* any
/// authority state is created, so a rejected scene never partially mutates a
/// world.
#[derive(Debug, Clone, PartialEq)]
pub enum BootstrapError {
    /// The scene failed structural/semantic validation.
    Invalid(SceneValidationReport),
    /// The document's schema version is not supported by this engine build.
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    /// This document carries typed stored references and therefore cannot be
    /// prepared without an immutable resolution registry.
    ResolutionContextRequired,
    /// One or more stored references did not resolve. The whole bootstrap is
    /// rejected before any RuntimeSession state exists.
    UnresolvedReferences {
        errors: Vec<BootstrapReferenceError>,
    },
    /// A caller-supplied allocator base and authored node id could not be
    /// combined without overflowing the durable EntityId domain.
    EntityAllocationOverflow {
        base_entity: EntityId,
        node: SceneNodeId,
    },
}

/// Immutable identities available to one scene bootstrap. Hosts assemble this
/// only from already-validated ProjectBundle registries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BootstrapResolutionContext {
    pub entity_definition_ids: BTreeSet<String>,
    pub prefab_ids: BTreeSet<u64>,
    pub spawn_marker_ids: BTreeSet<String>,
    pub generator_presets: BTreeSet<(String, String)>,
    pub catalog_ids: BTreeSet<String>,
}

/// One classified stored-reference failure from atomic bootstrap preparation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapReferenceError {
    UnknownEntityDefinition {
        node: SceneNodeId,
        stable_id: String,
    },
    UnknownPrefab {
        node: SceneNodeId,
        prefab_id: u64,
    },
    UnknownSpawnMarker {
        node: SceneNodeId,
        marker_id: String,
    },
    UnknownGeneratorPreset {
        node: SceneNodeId,
        provider_id: String,
        preset_id: String,
    },
    UnknownCatalog {
        node: SceneNodeId,
        binding_id: String,
        catalog_id: String,
    },
}

/// Stable canonical identity of the exact stored scene used for bootstrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneContentHash(pub u64);

/// One node's place in the deterministic bootstrap plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlannedEntity {
    /// The authored scene node this entity comes from.
    pub node: SceneNodeId,
    /// The runtime entity id allocated for it.
    pub entity: EntityId,
}

/// A validated, deterministic bootstrap plan. Holding one is proof the scene
/// passed validation; [`BootstrapPlan::apply`] is therefore infallible.
#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapPlan {
    scene_id: SceneId,
    runtime_session_id: RuntimeSessionId,
    schema_version: u32,
    /// Node→entity allocations in ascending scene-node id order.
    allocations: Vec<PlannedEntity>,
    /// The canonicalized document the plan was built from (carries transforms).
    doc: FlatSceneDocument,
    scene_content_hash: SceneContentHash,
    resolved_instances: Vec<ResolvedEntityInstance>,
    bootstrap_bindings: Option<SceneBootstrapBindings>,
}

/// Resolved stored placement evidence retained by the plan and bootstrap
/// record. Both local and composed world transforms are explicit so consumers
/// never have to guess which plane a value belongs to.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedEntityInstance {
    pub node: SceneNodeId,
    pub entity: EntityId,
    pub instance_id: String,
    pub reference: SceneEntityReference,
    pub spawn_marker_id: Option<String>,
    pub local_transform: SceneTransform,
    pub world_transform: SceneTransform,
}

impl BootstrapPlan {
    /// Validate `doc` and build a deterministic plan that bootstraps into
    /// `runtime_session_id`, allocating entity ids from [`DEFAULT_BASE_ENTITY_ID`].
    pub fn prepare(
        doc: &FlatSceneDocument,
        runtime_session_id: RuntimeSessionId,
    ) -> Result<BootstrapPlan, BootstrapError> {
        Self::prepare_with_base(doc, runtime_session_id, DEFAULT_BASE_ENTITY_ID)
    }

    /// Like [`BootstrapPlan::prepare`] but with an explicit base entity id, for
    /// callers threading their own allocator.
    pub fn prepare_with_base(
        doc: &FlatSceneDocument,
        runtime_session_id: RuntimeSessionId,
        base_entity: EntityId,
    ) -> Result<BootstrapPlan, BootstrapError> {
        Self::prepare_internal(doc, runtime_session_id, base_entity, None)
    }

    /// Validate and resolve a document with typed EntityDefinition/prefab,
    /// spawn-marker, generator, and catalog inputs.
    pub fn prepare_resolved(
        doc: &FlatSceneDocument,
        runtime_session_id: RuntimeSessionId,
        resolution: &BootstrapResolutionContext,
    ) -> Result<BootstrapPlan, BootstrapError> {
        Self::prepare_resolved_with_base(
            doc,
            runtime_session_id,
            DEFAULT_BASE_ENTITY_ID,
            resolution,
        )
    }

    /// Resolved variant with an explicit runtime entity allocator base.
    pub fn prepare_resolved_with_base(
        doc: &FlatSceneDocument,
        runtime_session_id: RuntimeSessionId,
        base_entity: EntityId,
        resolution: &BootstrapResolutionContext,
    ) -> Result<BootstrapPlan, BootstrapError> {
        Self::prepare_internal(doc, runtime_session_id, base_entity, Some(resolution))
    }

    fn prepare_internal(
        doc: &FlatSceneDocument,
        runtime_session_id: RuntimeSessionId,
        base_entity: EntityId,
        resolution: Option<&BootstrapResolutionContext>,
    ) -> Result<BootstrapPlan, BootstrapError> {
        if !(1..=SUPPORTED_SCHEMA_VERSION).contains(&doc.schema_version) {
            return Err(BootstrapError::UnsupportedSchemaVersion {
                found: doc.schema_version,
                supported: SUPPORTED_SCHEMA_VERSION,
            });
        }
        let report = validate(doc);
        if !report.is_ok() {
            return Err(BootstrapError::Invalid(report));
        }

        // Canonicalize so allocation order (ascending node id) is deterministic
        // regardless of the authoring order the document arrived in.
        let doc = doc.canonical();
        let allocations: Vec<PlannedEntity> = doc
            .nodes
            .iter()
            .enumerate()
            .map(|(index, record)| {
                // Schema 3 scene-node ids are durable authored instance/source
                // identities. Preserve that stability while still respecting a
                // caller allocator base. Legacy schemas retain compact ordinal
                // allocation for compatibility.
                let offset = if doc.schema_version >= 3 {
                    record.id.raw()
                } else {
                    index as u64 + 1
                };
                let entity = base_entity
                    .raw()
                    .checked_sub(1)
                    .and_then(|base| base.checked_add(offset))
                    .map(EntityId::new)
                    .ok_or(BootstrapError::EntityAllocationOverflow {
                        base_entity,
                        node: record.id,
                    })?;
                Ok(PlannedEntity {
                    node: record.id,
                    entity,
                })
            })
            .collect::<Result<_, BootstrapError>>()?;

        let needs_resolution = doc.nodes.iter().any(|record| {
            matches!(
                &record.kind,
                SceneNodeKind::EntityInstance(_) | SceneNodeKind::Bootstrap(_)
            )
        });
        if needs_resolution && resolution.is_none() {
            return Err(BootstrapError::ResolutionContextRequired);
        }
        if let Some(context) = resolution {
            let errors = resolve_references(&doc, context);
            if !errors.is_empty() {
                return Err(BootstrapError::UnresolvedReferences { errors });
            }
        }

        let world_transforms = authored_world_transforms(&doc);
        let resolved_instances = doc
            .nodes
            .iter()
            .zip(allocations.iter())
            .filter_map(|(record, allocation)| {
                let SceneNodeKind::EntityInstance(instance) = &record.kind else {
                    return None;
                };
                Some(ResolvedEntityInstance {
                    node: record.id,
                    entity: allocation.entity,
                    instance_id: instance.instance_id.clone(),
                    reference: instance.reference.clone(),
                    spawn_marker_id: instance.spawn_marker_id.clone(),
                    local_transform: record.transform,
                    world_transform: world_transforms[&record.id.raw()],
                })
            })
            .collect();
        let bootstrap_bindings = doc.nodes.iter().find_map(|record| {
            let SceneNodeKind::Bootstrap(bindings) = &record.kind else {
                return None;
            };
            Some(bindings.clone())
        });
        let scene_content_hash = SceneContentHash(hash_bytes(encode(&doc).as_bytes()));

        Ok(BootstrapPlan {
            scene_id: doc.id,
            runtime_session_id,
            schema_version: doc.schema_version,
            allocations,
            doc,
            scene_content_hash,
            resolved_instances,
            bootstrap_bindings,
        })
    }

    /// The node→entity allocations, in deterministic order.
    pub fn allocations(&self) -> &[PlannedEntity] {
        &self.allocations
    }

    pub fn resolved_instances(&self) -> &[ResolvedEntityInstance] {
        &self.resolved_instances
    }

    /// Apply the plan as one atomic initialization: populate a fresh world with
    /// every scene-sourced entity (initial transforms copied in) and return it
    /// alongside the single [`BootstrapRecord`] for replay/audit.
    pub fn apply(&self) -> (SpatialSessionState, BootstrapRecord) {
        let mut world = SpatialSessionState::empty(self.runtime_session_id);
        // `allocations` is parallel to `doc.nodes` (both canonical order).
        let world_transforms = authored_world_transforms(&self.doc);
        for (alloc, rec) in self.allocations.iter().zip(self.doc.nodes.iter()) {
            debug_assert_eq!(alloc.node, rec.id);
            let inserted = world.insert_scene_entity(
                alloc.entity,
                alloc.node,
                world_transforms[&rec.id.raw()],
            );
            debug_assert!(
                inserted,
                "validated plan must allocate unique entities/nodes"
            );
        }
        let record = BootstrapRecord {
            scene_id: self.scene_id,
            runtime_session_id: self.runtime_session_id,
            schema_version: self.schema_version,
            node_count: self.doc.nodes.len(),
            entity_count: world.entity_count(),
            spatial_session_hash: world.hash(),
            source_trace: self.allocations.clone(),
            scene_content_hash: self.scene_content_hash,
            resolved_instances: self.resolved_instances.clone(),
            bootstrap_bindings: self.bootstrap_bindings.clone(),
        };
        (world, record)
    }
}

/// Convenience: prepare and apply in one call. Errors if the scene is invalid.
pub fn bootstrap_scene(
    doc: &FlatSceneDocument,
    runtime_session_id: RuntimeSessionId,
) -> Result<(SpatialSessionState, BootstrapRecord), BootstrapError> {
    Ok(BootstrapPlan::prepare(doc, runtime_session_id)?.apply())
}

/// The single replay/audit unit recorded for one scene bootstrap. Replay sees
/// this one initialization unit, **not** N ordinary create events.
#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapRecord {
    pub scene_id: SceneId,
    pub runtime_session_id: RuntimeSessionId,
    pub schema_version: u32,
    pub node_count: usize,
    pub entity_count: usize,
    /// Deterministic fingerprint of the world produced by this bootstrap.
    pub spatial_session_hash: SpatialSessionHash,
    /// The source trace `scene node → runtime entity`. Render-handle/projection
    /// metadata is appended later, at projection time (scene-capability-01).
    pub source_trace: Vec<PlannedEntity>,
    /// Identity of the exact canonical stored scene bytes used to prepare this
    /// fresh runtime authority state.
    pub scene_content_hash: SceneContentHash,
    /// Resolved runtime placements, retaining stored reference and transform
    /// evidence without making the stored document live authority.
    pub resolved_instances: Vec<ResolvedEntityInstance>,
    /// Resolved scene-wide inputs retained for replay/audit correlation.
    pub bootstrap_bindings: Option<SceneBootstrapBindings>,
}

fn resolve_references(
    doc: &FlatSceneDocument,
    context: &BootstrapResolutionContext,
) -> Vec<BootstrapReferenceError> {
    let mut errors = Vec::new();
    for record in &doc.nodes {
        match &record.kind {
            SceneNodeKind::EntityInstance(instance) => {
                match &instance.reference {
                    SceneEntityReference::EntityDefinition { stable_id } => {
                        if !context.entity_definition_ids.contains(stable_id) {
                            errors.push(BootstrapReferenceError::UnknownEntityDefinition {
                                node: record.id,
                                stable_id: stable_id.clone(),
                            });
                        }
                    }
                    SceneEntityReference::Prefab { prefab_id, .. } => {
                        if !context.prefab_ids.contains(prefab_id) {
                            errors.push(BootstrapReferenceError::UnknownPrefab {
                                node: record.id,
                                prefab_id: *prefab_id,
                            });
                        }
                    }
                }
                if let Some(marker_id) = &instance.spawn_marker_id {
                    if !context.spawn_marker_ids.contains(marker_id) {
                        errors.push(BootstrapReferenceError::UnknownSpawnMarker {
                            node: record.id,
                            marker_id: marker_id.clone(),
                        });
                    }
                }
            }
            SceneNodeKind::Bootstrap(bindings) => {
                if let Some(generator) = &bindings.generator {
                    if !context
                        .generator_presets
                        .contains(&(generator.provider_id.clone(), generator.preset_id.clone()))
                    {
                        errors.push(BootstrapReferenceError::UnknownGeneratorPreset {
                            node: record.id,
                            provider_id: generator.provider_id.clone(),
                            preset_id: generator.preset_id.clone(),
                        });
                    }
                }
                for catalog in &bindings.catalogs {
                    if !context.catalog_ids.contains(&catalog.catalog_id) {
                        errors.push(BootstrapReferenceError::UnknownCatalog {
                            node: record.id,
                            binding_id: catalog.binding_id.clone(),
                            catalog_id: catalog.catalog_id.clone(),
                        });
                    }
                }
            }
            SceneNodeKind::EmptyGroup
            | SceneNodeKind::StaticMesh(_)
            | SceneNodeKind::Sprite(_)
            | SceneNodeKind::VoxelVolume(_)
            | SceneNodeKind::Light(_) => {}
        }
    }
    errors
}

fn authored_world_transforms(doc: &FlatSceneDocument) -> BTreeMap<u64, SceneTransform> {
    let records: BTreeMap<u64, _> = doc
        .nodes
        .iter()
        .map(|record| (record.id.raw(), record))
        .collect();
    let mut result = BTreeMap::new();
    for record in &doc.nodes {
        let mut chain = Vec::new();
        let mut current = record;
        loop {
            chain.push(current.transform);
            let Some(parent) = current.parent else { break };
            current = records[&parent.raw()];
        }
        let world = chain
            .into_iter()
            .rev()
            .fold(SceneTransform::IDENTITY, SceneTransform::compose);
        result.insert(record.id.raw(), world);
    }
    result
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl BootstrapRecord {
    /// Stable label identifying this as one bootstrap semantic unit in audit logs.
    pub fn replay_unit_label(&self) -> &'static str {
        "scene.bootstrap"
    }
}

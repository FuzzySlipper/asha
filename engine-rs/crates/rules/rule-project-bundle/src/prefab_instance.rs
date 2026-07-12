//! Deterministic authoritative prefab expansion over the validated stored registry.

use core_assets::{AssetId, AssetKind};
use core_entity::{
    EntityLifecycleCommand, EntitySnapshot, EntitySource, EntityStore, EntityTransform, Quat,
};
use core_ids::{EntityId, PrefabId, PrefabInstanceId, PrefabPartId, SceneNodeId};
use core_math::Vec3;
use std::collections::{BTreeMap, BTreeSet};
use svc_serialization::{
    PrefabInstanceRecord, PrefabOverride, PrefabOverrideValue, PrefabPart, PrefabPartReference,
    PrefabPartRoleBinding, PrefabPartSource, PrefabRegistry, PrefabRegistryValidationContext,
    PrefabTransform, ValidatedPrefabRegistry,
};

pub const PREFAB_INSTANCE_SNAPSHOT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefabPlacementOrigin {
    Authored,
    Player,
}

impl PrefabPlacementOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::Authored => "authored",
            Self::Player => "player",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstantiatePrefabCommand {
    pub command_id: String,
    pub origin: PrefabPlacementOrigin,
    pub record: PrefabInstanceRecord,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrefabInstantiationCatalog {
    pub asset_ids: BTreeSet<String>,
    pub entity_definition_ids: BTreeSet<String>,
}

impl From<&PrefabRegistryValidationContext> for PrefabInstantiationCatalog {
    fn from(value: &PrefabRegistryValidationContext) -> Self {
        Self {
            asset_ids: value.asset_ids.clone(),
            entity_definition_ids: value.entity_definition_ids.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPrefabPart {
    pub part: PrefabPartId,
    pub namespace: String,
    pub entity: EntityId,
    pub node: SceneNodeId,
    pub parent_entity: Option<EntityId>,
    pub transform: PrefabTransform,
    pub source: PrefabPartSource,
    pub material_override: Option<String>,
    pub active: bool,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefabPartResolution {
    pub reference: PrefabPartReference,
    pub entity: EntityId,
    pub node: SceneNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefabInstantiationFact {
    InstanceCreated {
        prefab: PrefabId,
        instance: PrefabInstanceId,
        origin: PrefabPlacementOrigin,
        provenance_hash: String,
    },
    PartCreated {
        prefab: PrefabId,
        instance: PrefabInstanceId,
        part: PrefabPartId,
        entity: EntityId,
        node: SceneNodeId,
        roles: Vec<String>,
        source_kind: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPrefabInstance {
    pub record: PrefabInstanceRecord,
    pub origin: PrefabPlacementOrigin,
    pub parts: Vec<ResolvedPrefabPart>,
    pub role_map: Vec<PrefabPartResolution>,
    pub effective_overrides: Vec<PrefabOverride>,
    pub provenance_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefabInstantiationReceipt {
    pub command_id: String,
    pub instance: PrefabInstanceId,
    pub origin: PrefabPlacementOrigin,
    pub part_count: usize,
    pub state_hash_before: String,
    pub state_hash_after: String,
    pub provenance_hash: String,
    pub facts: Vec<PrefabInstantiationFact>,
    pub receipt_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefabInstantiationError {
    InvalidCommandId,
    MissingPrefab(PrefabId),
    DuplicateInstance(PrefabInstanceId),
    UnknownOverrideRole(String),
    DuplicateEffectiveOverride { role: String, field: String },
    InvalidOverrideValue { role: String, field: String },
    IdCollision { kind: &'static str, id: u64 },
    EntityRejected(EntityId),
    SnapshotVersion(u32),
    SnapshotDiverged,
}

impl core::fmt::Display for PrefabInstantiationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for PrefabInstantiationError {}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefabInstanceSnapshot {
    pub schema_version: u32,
    pub accepted_commands: Vec<InstantiatePrefabCommand>,
    pub instances: Vec<ResolvedPrefabInstance>,
    pub state_hash: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PrefabInstanceAuthority {
    instances: BTreeMap<PrefabInstanceId, ResolvedPrefabInstance>,
    accepted_commands: Vec<InstantiatePrefabCommand>,
}

impl PrefabInstanceAuthority {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn instance(&self, id: PrefabInstanceId) -> Option<&ResolvedPrefabInstance> {
        self.instances.get(&id)
    }

    pub fn instances(&self) -> impl Iterator<Item = &ResolvedPrefabInstance> {
        self.instances.values()
    }

    pub fn resolve_part(
        &self,
        instance: PrefabInstanceId,
        reference: &PrefabPartReference,
    ) -> Option<&PrefabPartResolution> {
        self.instances
            .get(&instance)?
            .role_map
            .iter()
            .find(|item| item.reference == *reference)
    }

    pub fn state_hash(&self, entities: &EntityStore) -> String {
        let mut hash = Fnv1a::new();
        hash.write_u64(prefab_entity_hash(entities, &self.instances));
        for command in &self.accepted_commands {
            hash_command(&mut hash, command);
        }
        for (id, instance) in &self.instances {
            hash.write_u64(id.raw());
            hash.write_u64(instance.record.prefab.raw());
            hash.write_u64(instance.record.seed);
            hash.write_str(instance.origin.label());
            hash_transform(&mut hash, instance.record.transform);
            for part in &instance.parts {
                hash.write_u64(part.part.raw());
                hash.write_u64(part.entity.raw());
                hash.write_u64(part.node.raw());
                hash.write_str(&part.namespace);
                match part.parent_entity {
                    Some(parent) => {
                        hash.write_u8(1);
                        hash.write_u64(parent.raw());
                    }
                    None => hash.write_u8(0),
                }
                hash_transform(&mut hash, part.transform);
                hash_source(&mut hash, &part.source);
                match &part.material_override {
                    Some(asset) => {
                        hash.write_u8(1);
                        hash.write_str(asset);
                    }
                    None => hash.write_u8(0),
                }
                hash.write_u8(u8::from(part.active));
                for role in &part.roles {
                    hash.write_str(role);
                }
            }
            for resolution in &instance.role_map {
                hash.write_str(&resolution.reference.role);
                hash.write_u64(resolution.entity.raw());
                hash.write_u64(resolution.node.raw());
            }
            for item in &instance.effective_overrides {
                hash_override(&mut hash, item);
            }
            hash.write_str(&instance.provenance_hash);
        }
        stable_hash(hash.finish())
    }

    pub fn instantiate(
        &mut self,
        entities: &mut EntityStore,
        registry: &ValidatedPrefabRegistry,
        catalog: &PrefabInstantiationCatalog,
        command: InstantiatePrefabCommand,
    ) -> Result<PrefabInstantiationReceipt, PrefabInstantiationError> {
        let state_hash_before = self.state_hash(entities);
        let mut staged = self.clone();
        let mut staged_entities = entities.clone();
        let receipt = staged.instantiate_staged(
            &mut staged_entities,
            registry,
            catalog,
            command,
            state_hash_before,
        )?;
        *self = staged;
        *entities = staged_entities;
        Ok(receipt)
    }

    pub fn snapshot(&self, entities: &EntityStore) -> PrefabInstanceSnapshot {
        PrefabInstanceSnapshot {
            schema_version: PREFAB_INSTANCE_SNAPSHOT_VERSION,
            accepted_commands: self.accepted_commands.clone(),
            instances: self.instances.values().cloned().collect(),
            state_hash: self.state_hash(entities),
        }
    }

    pub fn restore(
        registry: &ValidatedPrefabRegistry,
        catalog: &PrefabInstantiationCatalog,
        snapshot: &PrefabInstanceSnapshot,
    ) -> Result<(Self, EntityStore), PrefabInstantiationError> {
        if snapshot.schema_version != PREFAB_INSTANCE_SNAPSHOT_VERSION {
            return Err(PrefabInstantiationError::SnapshotVersion(
                snapshot.schema_version,
            ));
        }
        let mut authority = Self::new();
        let mut entities = EntityStore::new();
        for command in &snapshot.accepted_commands {
            authority.instantiate(&mut entities, registry, catalog, command.clone())?;
        }
        if authority.instances.values().cloned().collect::<Vec<_>>() != snapshot.instances
            || authority.state_hash(&entities) != snapshot.state_hash
        {
            return Err(PrefabInstantiationError::SnapshotDiverged);
        }
        Ok((authority, entities))
    }

    /// Restore the prefab-instance index beside an already-decoded owning Session
    /// EntityStore. This is the ProjectBundle save/reload path: resolved role and
    /// override metadata is durable, while the normal Session snapshot remains the
    /// sole owner of entity records. Any mismatch fails closed.
    pub fn restore_persisted(
        snapshot: &PrefabInstanceSnapshot,
        entities: &EntityStore,
    ) -> Result<Self, PrefabInstantiationError> {
        if snapshot.schema_version != PREFAB_INSTANCE_SNAPSHOT_VERSION {
            return Err(PrefabInstantiationError::SnapshotVersion(
                snapshot.schema_version,
            ));
        }
        let mut instances = BTreeMap::new();
        for instance in &snapshot.instances {
            if instances
                .insert(instance.record.instance, instance.clone())
                .is_some()
            {
                return Err(PrefabInstantiationError::SnapshotDiverged);
            }
        }
        if instances.len() != snapshot.accepted_commands.len() {
            return Err(PrefabInstantiationError::SnapshotDiverged);
        }
        for command in &snapshot.accepted_commands {
            let Some(instance) = instances.get(&command.record.instance) else {
                return Err(PrefabInstantiationError::SnapshotDiverged);
            };
            if instance.record != command.record || instance.origin != command.origin {
                return Err(PrefabInstantiationError::SnapshotDiverged);
            }
            validate_persisted_instance(instance, entities)?;
        }
        let indexed_entities = instances
            .values()
            .flat_map(|instance| instance.parts.iter().map(|part| part.entity))
            .collect::<BTreeSet<_>>();
        let persisted_entities = entities
            .snapshot()
            .records
            .into_iter()
            .filter_map(|record| {
                matches!(record.core.source, EntitySource::PrefabInstance { .. })
                    .then_some(record.core.id)
            })
            .collect::<BTreeSet<_>>();
        if indexed_entities != persisted_entities {
            return Err(PrefabInstantiationError::SnapshotDiverged);
        }
        let authority = Self {
            instances,
            accepted_commands: snapshot.accepted_commands.clone(),
        };
        if authority.state_hash(entities) != snapshot.state_hash {
            return Err(PrefabInstantiationError::SnapshotDiverged);
        }
        Ok(authority)
    }

    fn instantiate_staged(
        &mut self,
        entities: &mut EntityStore,
        registry: &ValidatedPrefabRegistry,
        catalog: &PrefabInstantiationCatalog,
        command: InstantiatePrefabCommand,
        state_hash_before: String,
    ) -> Result<PrefabInstantiationReceipt, PrefabInstantiationError> {
        if command.command_id.trim().is_empty() {
            return Err(PrefabInstantiationError::InvalidCommandId);
        }
        if self.instances.contains_key(&command.record.instance) {
            return Err(PrefabInstantiationError::DuplicateInstance(
                command.record.instance,
            ));
        }
        let EffectivePrefabDefinition {
            mut parts,
            roles,
            mut effective_overrides,
            mut material_overrides,
            mut activation,
        } = effective_definition(registry, command.record.prefab)?;
        apply_overrides(
            &mut parts,
            &roles,
            &command.record.overrides,
            catalog,
            &mut material_overrides,
            &mut activation,
        )?;
        effective_overrides.extend(command.record.overrides.clone());
        effective_overrides.sort_by(|left, right| {
            (left.target_role.as_str(), left.value.field())
                .cmp(&(right.target_role.as_str(), right.value.field()))
        });

        let roles_by_part = index_roles(&roles);
        let mut world_transforms = BTreeMap::new();
        let indexed_parts = parts
            .iter()
            .map(|part| (part.id, part))
            .collect::<BTreeMap<_, _>>();
        for part in &parts {
            resolve_world_transform(
                part.id,
                &indexed_parts,
                command.record.transform,
                &mut world_transforms,
            );
        }

        let mut resolved_parts = Vec::new();
        let mut part_targets = BTreeMap::new();
        let mut node_targets = BTreeMap::new();
        let placement_hash = transform_hash(command.record.transform);
        for part in &parts {
            let entity = EntityId::new(derived_id(0xe1, &command.record, part.id, placement_hash));
            let node = SceneNodeId::new(derived_id(0xa7, &command.record, part.id, placement_hash));
            if entities.contains(entity) || part_targets.values().any(|item| *item == entity) {
                return Err(PrefabInstantiationError::IdCollision {
                    kind: "entity",
                    id: entity.raw(),
                });
            }
            if node_targets.values().any(|item| *item == node) {
                return Err(PrefabInstantiationError::IdCollision {
                    kind: "node",
                    id: node.raw(),
                });
            }
            part_targets.insert(part.id, entity);
            node_targets.insert(part.id, node);
        }
        for part in &parts {
            let entity = part_targets[&part.id];
            let node = node_targets[&part.id];
            let part_roles = roles_by_part.get(&part.id).cloned().unwrap_or_default();
            let canonical_role = part_roles.first().cloned();
            let source = EntitySource::PrefabInstance {
                prefab: command.record.prefab,
                instance: command.record.instance,
                part: part.id,
                role: canonical_role,
            };
            entities
                .apply(EntityLifecycleCommand::Create {
                    id: entity,
                    source,
                    labels: Vec::new(),
                })
                .map_err(|_| PrefabInstantiationError::EntityRejected(entity))?;
            let transform = world_transforms[&part.id];
            if !entities.attach_transform(entity, to_entity_transform(transform)) {
                return Err(PrefabInstantiationError::EntityRejected(entity));
            }
            let active = activation.get(&part.id).copied().unwrap_or(true);
            if !active {
                entities
                    .apply(EntityLifecycleCommand::Disable { id: entity })
                    .map_err(|_| PrefabInstantiationError::EntityRejected(entity))?;
            }
            resolved_parts.push(ResolvedPrefabPart {
                part: part.id,
                namespace: part.namespace.clone(),
                entity,
                node,
                parent_entity: part
                    .parent
                    .and_then(|parent| part_targets.get(&parent).copied()),
                transform,
                source: part.source.clone(),
                material_override: material_overrides.get(&part.id).cloned(),
                active,
                roles: part_roles,
            });
        }

        let mut role_map = roles
            .iter()
            .map(|binding| {
                let part = resolved_parts
                    .iter()
                    .find(|part| part.part == binding.part)
                    .expect("validated effective role targets a retained part");
                PrefabPartResolution {
                    reference: PrefabPartReference {
                        prefab: command.record.prefab,
                        role: binding.role.clone(),
                    },
                    entity: part.entity,
                    node: part.node,
                }
            })
            .collect::<Vec<_>>();
        role_map.sort_by(|left, right| left.reference.role.cmp(&right.reference.role));
        resolved_parts.sort_by_key(|part| part.part.raw());
        let provenance_hash = provenance_hash(&command.record, &resolved_parts, &role_map);
        let mut facts = vec![PrefabInstantiationFact::InstanceCreated {
            prefab: command.record.prefab,
            instance: command.record.instance,
            origin: command.origin,
            provenance_hash: provenance_hash.clone(),
        }];
        facts.extend(
            resolved_parts
                .iter()
                .map(|part| PrefabInstantiationFact::PartCreated {
                    prefab: command.record.prefab,
                    instance: command.record.instance,
                    part: part.part,
                    entity: part.entity,
                    node: part.node,
                    roles: part.roles.clone(),
                    source_kind: part.source.kind().to_owned(),
                }),
        );
        self.instances.insert(
            command.record.instance,
            ResolvedPrefabInstance {
                record: command.record.clone(),
                origin: command.origin,
                parts: resolved_parts,
                role_map,
                effective_overrides,
                provenance_hash: provenance_hash.clone(),
            },
        );
        self.accepted_commands.push(command.clone());
        let state_hash_after = self.state_hash(entities);
        let receipt_hash = stable_hash_fields(&[
            &command.command_id,
            command.origin.label(),
            &command.record.instance.raw().to_string(),
            &state_hash_before,
            &state_hash_after,
            &provenance_hash,
            &facts.len().to_string(),
        ]);
        Ok(PrefabInstantiationReceipt {
            command_id: command.command_id,
            instance: command.record.instance,
            origin: command.origin,
            part_count: self.instances[&command.record.instance].parts.len(),
            state_hash_before,
            state_hash_after,
            provenance_hash,
            facts,
            receipt_hash,
        })
    }
}

fn prefab_entity_hash(
    entities: &EntityStore,
    instances: &BTreeMap<PrefabInstanceId, ResolvedPrefabInstance>,
) -> u64 {
    let ids = instances
        .values()
        .flat_map(|instance| instance.parts.iter().map(|part| part.entity))
        .collect::<BTreeSet<_>>();
    let records = entities
        .snapshot()
        .records
        .into_iter()
        .filter(|record| ids.contains(&record.core.id))
        .collect();
    EntityStore::from_snapshot(EntitySnapshot { records })
        .hash()
        .0
}

fn validate_persisted_instance(
    instance: &ResolvedPrefabInstance,
    entities: &EntityStore,
) -> Result<(), PrefabInstantiationError> {
    let records = entities
        .snapshot()
        .records
        .into_iter()
        .map(|record| (record.core.id, record))
        .collect::<BTreeMap<_, _>>();
    let mut parts = BTreeMap::new();
    for part in &instance.parts {
        if parts.insert(part.part, part).is_some() {
            return Err(PrefabInstantiationError::SnapshotDiverged);
        }
        let Some(record) = records.get(&part.entity) else {
            return Err(PrefabInstantiationError::SnapshotDiverged);
        };
        let expected_role = part.roles.first().cloned();
        if record.core.source
            != (EntitySource::PrefabInstance {
                prefab: instance.record.prefab,
                instance: instance.record.instance,
                part: part.part,
                role: expected_role,
            })
        {
            return Err(PrefabInstantiationError::SnapshotDiverged);
        }
    }
    let mut roles = BTreeSet::new();
    for resolution in &instance.role_map {
        if resolution.reference.prefab != instance.record.prefab
            || !roles.insert(resolution.reference.role.as_str())
            || !instance.parts.iter().any(|part| {
                part.entity == resolution.entity
                    && part.node == resolution.node
                    && part.roles.contains(&resolution.reference.role)
            })
        {
            return Err(PrefabInstantiationError::SnapshotDiverged);
        }
    }
    Ok(())
}

struct EffectivePrefabDefinition {
    parts: Vec<PrefabPart>,
    roles: Vec<PrefabPartRoleBinding>,
    effective_overrides: Vec<PrefabOverride>,
    material_overrides: BTreeMap<PrefabPartId, String>,
    activation: BTreeMap<PrefabPartId, bool>,
}

fn effective_definition(
    registry: &ValidatedPrefabRegistry,
    prefab: PrefabId,
) -> Result<EffectivePrefabDefinition, PrefabInstantiationError> {
    let definitions = registry
        .as_registry()
        .definitions
        .iter()
        .map(|definition| (definition.id, definition))
        .collect::<BTreeMap<_, _>>();
    let definition = definitions
        .get(&prefab)
        .copied()
        .ok_or(PrefabInstantiationError::MissingPrefab(prefab))?;
    let Some(variant) = &definition.variant else {
        return Ok(EffectivePrefabDefinition {
            parts: definition.parts.clone(),
            roles: definition.part_roles.clone(),
            effective_overrides: Vec::new(),
            material_overrides: BTreeMap::new(),
            activation: BTreeMap::new(),
        });
    };
    let base = definitions
        .get(&variant.base)
        .copied()
        .expect("validated variant base exists");
    let mut parts = base.parts.clone();
    let mut roles = base.part_roles.clone();
    let mut material_overrides = BTreeMap::new();
    let mut activation = BTreeMap::new();
    let removed_parts = variant
        .removed_roles
        .iter()
        .filter_map(|role| roles.iter().find(|item| item.role == *role))
        .map(|item| item.part)
        .collect::<BTreeSet<_>>();
    parts.retain(|part| !removed_parts.contains(&part.id));
    roles.retain(|binding| !removed_parts.contains(&binding.part));
    apply_overrides(
        &mut parts,
        &roles,
        &variant.overrides,
        &PrefabInstantiationCatalog {
            asset_ids: registry_asset_ids(registry.as_registry()),
            entity_definition_ids: registry_entity_definition_ids(registry.as_registry()),
        },
        &mut material_overrides,
        &mut activation,
    )?;
    Ok(EffectivePrefabDefinition {
        parts,
        roles,
        effective_overrides: variant.overrides.clone(),
        material_overrides,
        activation,
    })
}

fn registry_asset_ids(registry: &PrefabRegistry) -> BTreeSet<String> {
    registry
        .definitions
        .iter()
        .flat_map(|definition| {
            definition
                .parts
                .iter()
                .filter_map(|part| match &part.source {
                    PrefabPartSource::Scene { asset } | PrefabPartSource::VoxelObject { asset } => {
                        Some(asset.clone())
                    }
                    PrefabPartSource::EntityDefinition { .. } => None,
                })
                .chain(
                    definition
                        .variant
                        .iter()
                        .flat_map(|variant| variant.overrides.iter())
                        .filter_map(|item| match &item.value {
                            PrefabOverrideValue::Asset { asset }
                            | PrefabOverrideValue::Material { asset } => Some(asset.clone()),
                            _ => None,
                        }),
                )
        })
        .collect()
}

fn registry_entity_definition_ids(registry: &PrefabRegistry) -> BTreeSet<String> {
    registry
        .definitions
        .iter()
        .flat_map(|definition| {
            definition
                .parts
                .iter()
                .filter_map(|part| match &part.source {
                    PrefabPartSource::EntityDefinition { stable_id } => Some(stable_id.clone()),
                    _ => None,
                })
                .chain(
                    definition
                        .variant
                        .iter()
                        .flat_map(|variant| variant.overrides.iter())
                        .filter_map(|item| match &item.value {
                            PrefabOverrideValue::EntityDefinition { stable_id } => {
                                Some(stable_id.clone())
                            }
                            _ => None,
                        }),
                )
        })
        .collect()
}

fn apply_overrides(
    parts: &mut [PrefabPart],
    roles: &[PrefabPartRoleBinding],
    overrides: &[PrefabOverride],
    catalog: &PrefabInstantiationCatalog,
    material_overrides: &mut BTreeMap<PrefabPartId, String>,
    activation: &mut BTreeMap<PrefabPartId, bool>,
) -> Result<(), PrefabInstantiationError> {
    let role_index = roles
        .iter()
        .map(|binding| (binding.role.as_str(), binding.part))
        .collect::<BTreeMap<_, _>>();
    let mut targets = BTreeSet::new();
    for item in overrides {
        let part_id = role_index
            .get(item.target_role.as_str())
            .copied()
            .ok_or_else(|| {
                PrefabInstantiationError::UnknownOverrideRole(item.target_role.clone())
            })?;
        let key = (part_id, item.value.field());
        if !targets.insert(key) {
            return Err(PrefabInstantiationError::DuplicateEffectiveOverride {
                role: item.target_role.clone(),
                field: item.value.field().to_owned(),
            });
        }
        let part = parts
            .iter_mut()
            .find(|part| part.id == part_id)
            .ok_or_else(|| {
                PrefabInstantiationError::UnknownOverrideRole(item.target_role.clone())
            })?;
        match &item.value {
            PrefabOverrideValue::Transform { transform } if transform_valid(*transform) => {
                part.transform = *transform;
            }
            PrefabOverrideValue::EntityDefinition { stable_id }
                if matches!(part.source, PrefabPartSource::EntityDefinition { .. })
                    && catalog.entity_definition_ids.contains(stable_id) =>
            {
                part.source = PrefabPartSource::EntityDefinition {
                    stable_id: stable_id.clone(),
                };
            }
            PrefabOverrideValue::Asset { asset }
                if catalog.asset_ids.contains(asset)
                    && asset_matches_source(asset, &part.source) =>
            {
                part.source = match part.source {
                    PrefabPartSource::Scene { .. } => PrefabPartSource::Scene {
                        asset: asset.clone(),
                    },
                    PrefabPartSource::VoxelObject { .. } => PrefabPartSource::VoxelObject {
                        asset: asset.clone(),
                    },
                    PrefabPartSource::EntityDefinition { .. } => unreachable!(),
                };
            }
            PrefabOverrideValue::Material { asset }
                if catalog.asset_ids.contains(asset)
                    && material_matches_source(asset, &part.source) =>
            {
                material_overrides.insert(part_id, asset.clone());
            }
            PrefabOverrideValue::Activation { active } => {
                activation.insert(part_id, *active);
            }
            _ => {
                return Err(PrefabInstantiationError::InvalidOverrideValue {
                    role: item.target_role.clone(),
                    field: item.value.field().to_owned(),
                })
            }
        }
    }
    Ok(())
}

fn asset_matches_source(asset: &str, source: &PrefabPartSource) -> bool {
    let expected = match source {
        PrefabPartSource::Scene { .. } => AssetKind::Scene,
        PrefabPartSource::VoxelObject { .. } => AssetKind::VoxelObject,
        PrefabPartSource::EntityDefinition { .. } => return false,
    };
    AssetId::parse(asset).is_ok_and(|id| id.kind() == expected)
}

fn material_matches_source(asset: &str, source: &PrefabPartSource) -> bool {
    !matches!(source, PrefabPartSource::EntityDefinition { .. })
        && AssetId::parse(asset).is_ok_and(|id| id.kind() == AssetKind::Material)
}

fn index_roles(roles: &[PrefabPartRoleBinding]) -> BTreeMap<PrefabPartId, Vec<String>> {
    let mut result: BTreeMap<PrefabPartId, Vec<String>> = BTreeMap::new();
    for binding in roles {
        result
            .entry(binding.part)
            .or_default()
            .push(binding.role.clone());
    }
    for values in result.values_mut() {
        values.sort();
    }
    result
}

fn resolve_world_transform(
    part_id: PrefabPartId,
    parts: &BTreeMap<PrefabPartId, &PrefabPart>,
    placement: PrefabTransform,
    resolved: &mut BTreeMap<PrefabPartId, PrefabTransform>,
) -> PrefabTransform {
    if let Some(value) = resolved.get(&part_id) {
        return *value;
    }
    let part = parts[&part_id];
    let parent = part
        .parent
        .map(|parent| resolve_world_transform(parent, parts, placement, resolved))
        .unwrap_or(placement);
    let value = compose_transform(parent, part.transform);
    resolved.insert(part_id, value);
    value
}

fn compose_transform(parent: PrefabTransform, local: PrefabTransform) -> PrefabTransform {
    let scaled = [
        local.translation[0] * parent.scale[0],
        local.translation[1] * parent.scale[1],
        local.translation[2] * parent.scale[2],
    ];
    let rotated = rotate_vector(parent.rotation, scaled);
    PrefabTransform {
        translation: [
            parent.translation[0] + rotated[0],
            parent.translation[1] + rotated[1],
            parent.translation[2] + rotated[2],
        ],
        rotation: multiply_quaternion(parent.rotation, local.rotation),
        scale: [
            parent.scale[0] * local.scale[0],
            parent.scale[1] * local.scale[1],
            parent.scale[2] * local.scale[2],
        ],
    }
}

fn multiply_quaternion(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

fn rotate_vector(q: [f32; 4], value: [f32; 3]) -> [f32; 3] {
    let u = [q[0], q[1], q[2]];
    let uv = cross(u, value);
    let uuv = cross(u, uv);
    [
        value[0] + 2.0 * (q[3] * uv[0] + uuv[0]),
        value[1] + 2.0 * (q[3] * uv[1] + uuv[1]),
        value[2] + 2.0 * (q[3] * uv[2] + uuv[2]),
    ]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn to_entity_transform(value: PrefabTransform) -> EntityTransform {
    EntityTransform {
        translation: Vec3::new(
            value.translation[0],
            value.translation[1],
            value.translation[2],
        ),
        rotation: Quat {
            x: value.rotation[0],
            y: value.rotation[1],
            z: value.rotation[2],
            w: value.rotation[3],
        },
        scale: Vec3::new(value.scale[0], value.scale[1], value.scale[2]),
    }
}

fn transform_valid(value: PrefabTransform) -> bool {
    value
        .translation
        .into_iter()
        .chain(value.rotation)
        .chain(value.scale)
        .all(f32::is_finite)
        && value.scale.into_iter().all(|axis| axis != 0.0)
}

fn derived_id(
    domain: u8,
    record: &PrefabInstanceRecord,
    part: PrefabPartId,
    placement_hash: u64,
) -> u64 {
    let mut hash = Fnv1a::new();
    hash.write_u8(domain);
    hash.write_u64(record.prefab.raw());
    hash.write_u64(record.instance.raw());
    hash.write_u64(record.seed);
    hash.write_u64(part.raw());
    hash.write_u64(placement_hash);
    // Public generated ids cross the JSON/TypeScript border as numbers. Keep
    // derived runtime ids inside the exactly representable integer range so a
    // save/reload cannot silently round them to a different entity or node.
    let value = hash.finish() & ((1_u64 << 53) - 1);
    if value == 0 {
        1
    } else {
        value
    }
}

fn provenance_hash(
    record: &PrefabInstanceRecord,
    parts: &[ResolvedPrefabPart],
    roles: &[PrefabPartResolution],
) -> String {
    let mut hash = Fnv1a::new();
    hash.write_u64(record.prefab.raw());
    hash.write_u64(record.instance.raw());
    hash.write_u64(record.seed);
    for part in parts {
        hash.write_u64(part.part.raw());
        hash.write_u64(part.entity.raw());
        hash.write_u64(part.node.raw());
    }
    for role in roles {
        hash.write_str(&role.reference.role);
        hash.write_u64(role.entity.raw());
    }
    stable_hash(hash.finish())
}

fn transform_hash(value: PrefabTransform) -> u64 {
    let mut hash = Fnv1a::new();
    hash_transform(&mut hash, value);
    hash.finish()
}

fn hash_transform(hash: &mut Fnv1a, value: PrefabTransform) {
    for item in value
        .translation
        .into_iter()
        .chain(value.rotation)
        .chain(value.scale)
    {
        hash.write_u32(item.to_bits());
    }
}

fn hash_source(hash: &mut Fnv1a, source: &PrefabPartSource) {
    match source {
        PrefabPartSource::Scene { asset } => {
            hash.write_u8(1);
            hash.write_str(asset);
        }
        PrefabPartSource::EntityDefinition { stable_id } => {
            hash.write_u8(2);
            hash.write_str(stable_id);
        }
        PrefabPartSource::VoxelObject { asset } => {
            hash.write_u8(3);
            hash.write_str(asset);
        }
    }
}

fn hash_override(hash: &mut Fnv1a, item: &PrefabOverride) {
    hash.write_str(&item.target_role);
    hash.write_str(item.value.field());
    match &item.value {
        PrefabOverrideValue::Transform { transform } => hash_transform(hash, *transform),
        PrefabOverrideValue::EntityDefinition { stable_id } => hash.write_str(stable_id),
        PrefabOverrideValue::Asset { asset } | PrefabOverrideValue::Material { asset } => {
            hash.write_str(asset)
        }
        PrefabOverrideValue::Activation { active } => hash.write_u8(u8::from(*active)),
    }
}

fn hash_command(hash: &mut Fnv1a, command: &InstantiatePrefabCommand) {
    hash.write_str(&command.command_id);
    hash.write_str(command.origin.label());
    hash.write_u64(command.record.instance.raw());
    hash.write_u64(command.record.prefab.raw());
    hash.write_u64(command.record.seed);
    hash_transform(hash, command.record.transform);
    for item in &command.record.overrides {
        hash_override(hash, item);
    }
}

fn stable_hash_fields(fields: &[&str]) -> String {
    let mut hash = Fnv1a::new();
    for field in fields {
        hash.write_str(field);
    }
    stable_hash(hash.finish())
}

fn stable_hash(value: u64) -> String {
    format!("fnv1a64:{value:016x}")
}

struct Fnv1a(u64);

impl Fnv1a {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }

    fn write_u8(&mut self, value: u8) {
        self.0 ^= u64::from(value);
        self.0 = self.0.wrapping_mul(0x100000001b3);
    }

    fn write_u32(&mut self, value: u32) {
        for byte in value.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    fn write_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        for byte in value.bytes() {
            self.write_u8(byte);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

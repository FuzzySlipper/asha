//! Live runtime world authority produced by scene bootstrap.
//!
//! A [`WorldState`] is the live authority that scene-capability-01 distinguishes
//! from the authored `SceneDocument`: it owns **runtime** transforms (seeded from
//! the scene's initial transforms at bootstrap, then authority-owned and free to
//! diverge) and the source trace `scene node → runtime entity`. The authored
//! document is never mutated by runtime movement.
//!
//! This crate's `core-state::StateStore` owns abstract entity/tag state; world
//! transforms and provenance are a separate concern, so they live here rather
//! than being bolted onto that store.

use std::collections::BTreeMap;

use core_ids::{EntityId, SceneNodeId, WorldId};

use crate::transform::SceneTransform;

/// A compact, deterministic fingerprint of a [`WorldState`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldHash(pub u64);

/// One entity's runtime state in the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityRuntime {
    /// Authority-owned runtime transform. Seeded from the scene initial transform
    /// at bootstrap; mutating it does not touch the scene document.
    pub transform: SceneTransform,
    /// The scene node this entity was bootstrapped from, or `None` for an entity
    /// created at runtime (no authored provenance).
    pub source_node: Option<SceneNodeId>,
}

/// Live world authority: entity runtime transforms plus scene-node provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldState {
    id: WorldId,
    /// `BTreeMap` for deterministic iteration (stable hashing without a sort pass).
    entities: BTreeMap<EntityId, EntityRuntime>,
    /// Reverse trace: scene node (raw) → runtime entity.
    node_to_entity: BTreeMap<u64, EntityId>,
}

impl WorldState {
    /// An empty world with no entities.
    pub fn empty(id: WorldId) -> Self {
        Self {
            id,
            entities: BTreeMap::new(),
            node_to_entity: BTreeMap::new(),
        }
    }

    pub fn id(&self) -> WorldId {
        self.id
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Insert a scene-sourced entity. Returns `false` (no-op) if the entity id or
    /// the source node is already present, so bootstrap stays one-to-one.
    pub(crate) fn insert_scene_entity(
        &mut self,
        entity: EntityId,
        node: SceneNodeId,
        transform: SceneTransform,
    ) -> bool {
        if self.entities.contains_key(&entity) || self.node_to_entity.contains_key(&node.raw()) {
            return false;
        }
        self.entities.insert(
            entity,
            EntityRuntime {
                transform,
                source_node: Some(node),
            },
        );
        self.node_to_entity.insert(node.raw(), entity);
        true
    }

    /// Create an entity at runtime with no scene provenance (`source_node` is
    /// `None`). Returns `false` if the id is already present.
    pub fn create_runtime_entity(&mut self, entity: EntityId, transform: SceneTransform) -> bool {
        if self.entities.contains_key(&entity) {
            return false;
        }
        self.entities.insert(
            entity,
            EntityRuntime {
                transform,
                source_node: None,
            },
        );
        true
    }

    /// The runtime record for `entity`, if present.
    pub fn entity(&self, entity: EntityId) -> Option<&EntityRuntime> {
        self.entities.get(&entity)
    }

    /// The runtime transform for `entity`, if present.
    pub fn transform(&self, entity: EntityId) -> Option<SceneTransform> {
        self.entities.get(&entity).map(|e| e.transform)
    }

    /// The scene node `entity` was bootstrapped from, if any.
    pub fn source_node(&self, entity: EntityId) -> Option<SceneNodeId> {
        self.entities.get(&entity).and_then(|e| e.source_node)
    }

    /// The runtime entity a scene node bootstrapped into, if any.
    pub fn entity_for_node(&self, node: SceneNodeId) -> Option<EntityId> {
        self.node_to_entity.get(&node.raw()).copied()
    }

    /// Overwrite an entity's runtime transform (authority-owned movement).
    /// Returns `false` if the entity is unknown. Never touches scene documents.
    pub fn set_transform(&mut self, entity: EntityId, transform: SceneTransform) -> bool {
        match self.entities.get_mut(&entity) {
            Some(rec) => {
                rec.transform = transform;
                true
            }
            None => false,
        }
    }

    /// Entities in ascending id order.
    pub fn entities(&self) -> impl Iterator<Item = (EntityId, &EntityRuntime)> {
        self.entities.iter().map(|(id, rec)| (*id, rec))
    }

    /// Deterministic FNV-1a fingerprint of the world: id, then each entity (in
    /// ascending id order) with its transform bits and source node. Mirrors the
    /// `core-snapshot` hashing approach so fingerprints are stable across runs.
    pub fn hash(&self) -> WorldHash {
        let mut h = Fnv1a::new();
        h.write_u64(self.id.raw());
        h.write_u8(0x01); // entities section
        for (id, rec) in &self.entities {
            h.write_u64(id.raw());
            hash_transform(&mut h, &rec.transform);
            match rec.source_node {
                Some(n) => {
                    h.write_u8(1);
                    h.write_u64(n.raw());
                }
                None => h.write_u8(0),
            }
        }
        WorldHash(h.finish())
    }
}

fn hash_transform(h: &mut Fnv1a, t: &SceneTransform) {
    for f in [
        t.translation.x,
        t.translation.y,
        t.translation.z,
        t.rotation.x,
        t.rotation.y,
        t.rotation.z,
        t.rotation.w,
        t.scale.x,
        t.scale.y,
        t.scale.z,
    ] {
        h.write_u64(f.to_bits() as u64);
    }
}

// ── FNV-1a hasher (mirrors core-snapshot's deterministic fingerprint) ─────────

const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

struct Fnv1a(u64);

impl Fnv1a {
    fn new() -> Self {
        Fnv1a(FNV_OFFSET)
    }

    fn write_u8(&mut self, b: u8) {
        self.0 ^= b as u64;
        self.0 = self.0.wrapping_mul(FNV_PRIME);
    }

    fn write_u64(&mut self, v: u64) {
        for byte in v.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

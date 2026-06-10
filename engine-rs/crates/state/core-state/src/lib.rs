//! Authoritative state store for the ASHA simulation engine.
//!
//! # Lane
//!
//! `rust-state` — may depend on `core-ids` and `core-error`. Must not
//! reference protocol, render, UI, or any TypeScript package.
//!
//! # Design
//!
//! [`StateStore`] is the single authoritative owner of abstract entity state.
//! Callers (validators, appliers, the sim kernel) mutate state exclusively
//! through the methods below; no interior mutability or direct field writes
//! are exposed outside this crate.
//!
//! All maps use [`BTreeMap`] for deterministic iteration order, which keeps
//! snapshot hashing stable across runs without requiring sort passes.
//!
//! ID allocation is the caller's responsibility; the store records whatever
//! ID it is given and rejects duplicates with a `false` return value rather
//! than panicking.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use core_ids::{EntityId, ModeId, ProcessId, SignalId, SubjectId, TagId};

// ── Record types ─────────────────────────────────────────────────────────────

/// Stored state for one abstract entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRecord {
    pub id: EntityId,
    /// Tags currently applied to this entity.
    pub tags: BTreeSet<TagId>,
}

/// Stored state for one authority subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectRecord {
    pub id: SubjectId,
}

/// Stored state for one ongoing process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRecord {
    pub id: ProcessId,
    /// Active mode, if any.
    pub mode: Option<ModeId>,
}

/// Stored state for a discrete mode variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeRecord {
    pub id: ModeId,
}

/// Stored state for a signal type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalRecord {
    pub id: SignalId,
}

/// Stored state for a tag label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRecord {
    pub id: TagId,
}

// ── StateStore ────────────────────────────────────────────────────────────────

/// Authoritative owner of all abstract entity state.
///
/// Mutation is only possible through the typed methods; callers cannot reach
/// into the internal maps directly. Lookups return `Option` — a missing or
/// removed ID is a lookup failure, never a panic.
#[derive(Debug, Default)]
pub struct StateStore {
    entities: BTreeMap<EntityId, EntityRecord>,
    subjects: BTreeMap<SubjectId, SubjectRecord>,
    processes: BTreeMap<ProcessId, ProcessRecord>,
    modes: BTreeMap<ModeId, ModeRecord>,
    signals: BTreeMap<SignalId, SignalRecord>,
    tags: BTreeMap<TagId, TagRecord>,
}

impl StateStore {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Entity ────────────────────────────────────────────────────────────

    /// Insert a new entity. Returns `false` if `id` already exists.
    pub fn insert_entity(&mut self, id: EntityId) -> bool {
        if self.entities.contains_key(&id) {
            return false;
        }
        self.entities.insert(
            id,
            EntityRecord {
                id,
                tags: BTreeSet::new(),
            },
        );
        true
    }

    /// Look up an entity by ID.
    pub fn entity(&self, id: EntityId) -> Option<&EntityRecord> {
        self.entities.get(&id)
    }

    /// Mutable access to an entity record.
    pub fn entity_mut(&mut self, id: EntityId) -> Option<&mut EntityRecord> {
        self.entities.get_mut(&id)
    }

    /// Remove an entity. Returns `false` if it did not exist.
    pub fn remove_entity(&mut self, id: EntityId) -> bool {
        self.entities.remove(&id).is_some()
    }

    /// Iterate over all live entities in deterministic order.
    pub fn entities(&self) -> impl Iterator<Item = &EntityRecord> {
        self.entities.values()
    }

    /// Number of live entities.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    // ── Subject ───────────────────────────────────────────────────────────

    /// Insert a new subject. Returns `false` if `id` already exists.
    pub fn insert_subject(&mut self, id: SubjectId) -> bool {
        if self.subjects.contains_key(&id) {
            return false;
        }
        self.subjects.insert(id, SubjectRecord { id });
        true
    }

    pub fn subject(&self, id: SubjectId) -> Option<&SubjectRecord> {
        self.subjects.get(&id)
    }

    pub fn remove_subject(&mut self, id: SubjectId) -> bool {
        self.subjects.remove(&id).is_some()
    }

    pub fn subjects(&self) -> impl Iterator<Item = &SubjectRecord> {
        self.subjects.values()
    }

    // ── Process ───────────────────────────────────────────────────────────

    /// Insert a new process. Returns `false` if `id` already exists.
    pub fn insert_process(&mut self, id: ProcessId) -> bool {
        if self.processes.contains_key(&id) {
            return false;
        }
        self.processes.insert(id, ProcessRecord { id, mode: None });
        true
    }

    pub fn process(&self, id: ProcessId) -> Option<&ProcessRecord> {
        self.processes.get(&id)
    }

    pub fn process_mut(&mut self, id: ProcessId) -> Option<&mut ProcessRecord> {
        self.processes.get_mut(&id)
    }

    pub fn remove_process(&mut self, id: ProcessId) -> bool {
        self.processes.remove(&id).is_some()
    }

    pub fn processes(&self) -> impl Iterator<Item = &ProcessRecord> {
        self.processes.values()
    }

    // ── Mode ──────────────────────────────────────────────────────────────

    /// Insert a new mode. Returns `false` if `id` already exists.
    pub fn insert_mode(&mut self, id: ModeId) -> bool {
        if self.modes.contains_key(&id) {
            return false;
        }
        self.modes.insert(id, ModeRecord { id });
        true
    }

    pub fn mode(&self, id: ModeId) -> Option<&ModeRecord> {
        self.modes.get(&id)
    }

    pub fn remove_mode(&mut self, id: ModeId) -> bool {
        self.modes.remove(&id).is_some()
    }

    pub fn modes(&self) -> impl Iterator<Item = &ModeRecord> {
        self.modes.values()
    }

    // ── Signal ────────────────────────────────────────────────────────────

    /// Insert a new signal. Returns `false` if `id` already exists.
    pub fn insert_signal(&mut self, id: SignalId) -> bool {
        if self.signals.contains_key(&id) {
            return false;
        }
        self.signals.insert(id, SignalRecord { id });
        true
    }

    pub fn signal(&self, id: SignalId) -> Option<&SignalRecord> {
        self.signals.get(&id)
    }

    pub fn remove_signal(&mut self, id: SignalId) -> bool {
        self.signals.remove(&id).is_some()
    }

    pub fn signals(&self) -> impl Iterator<Item = &SignalRecord> {
        self.signals.values()
    }

    // ── Tag ───────────────────────────────────────────────────────────────

    /// Insert a new tag definition. Returns `false` if `id` already exists.
    pub fn insert_tag(&mut self, id: TagId) -> bool {
        if self.tags.contains_key(&id) {
            return false;
        }
        self.tags.insert(id, TagRecord { id });
        true
    }

    pub fn tag(&self, id: TagId) -> Option<&TagRecord> {
        self.tags.get(&id)
    }

    pub fn remove_tag(&mut self, id: TagId) -> bool {
        self.tags.remove(&id).is_some()
    }

    pub fn tags(&self) -> impl Iterator<Item = &TagRecord> {
        self.tags.values()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core_ids::{EntityId, ModeId, ProcessId, SignalId, SubjectId, TagId};

    // ── Entity fixture ────────────────────────────────────────────────────

    #[test]
    fn create_entity_fixture() {
        let mut store = StateStore::new();
        let id = EntityId::new(1);
        assert!(store.insert_entity(id));
        let rec = store.entity(id).expect("entity must exist after insert");
        assert_eq!(rec.id, id);
        assert!(rec.tags.is_empty());
        assert_eq!(store.entity_count(), 1);
    }

    #[test]
    fn create_entity_duplicate_rejected() {
        let mut store = StateStore::new();
        let id = EntityId::new(1);
        assert!(store.insert_entity(id));
        assert!(!store.insert_entity(id));
        assert_eq!(store.entity_count(), 1);
    }

    #[test]
    fn update_entity_fixture() {
        let mut store = StateStore::new();
        let eid = EntityId::new(10);
        let tid = TagId::new(99);
        store.insert_entity(eid);
        store.insert_tag(tid);

        let rec = store.entity_mut(eid).expect("entity must exist");
        rec.tags.insert(tid);

        let rec = store.entity(eid).expect("entity must still exist");
        assert!(rec.tags.contains(&tid));
    }

    #[test]
    fn delete_entity_fixture() {
        let mut store = StateStore::new();
        let id = EntityId::new(5);
        store.insert_entity(id);
        assert!(store.remove_entity(id));
        assert!(
            store.entity(id).is_none(),
            "lookup after delete must return None"
        );
        assert_eq!(store.entity_count(), 0);
    }

    #[test]
    fn delete_nonexistent_entity_returns_false() {
        let mut store = StateStore::new();
        assert!(!store.remove_entity(EntityId::new(999)));
    }

    #[test]
    fn entity_iteration_is_deterministic() {
        let mut store = StateStore::new();
        for raw in [3u64, 1, 4, 1, 5, 9, 2, 6] {
            store.insert_entity(EntityId::new(raw));
        }
        let ids: Vec<u64> = store.entities().map(|r| r.id.raw()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted, "BTreeMap must iterate in ascending key order");
    }

    // ── Subject fixture ───────────────────────────────────────────────────

    #[test]
    fn create_and_delete_subject() {
        let mut store = StateStore::new();
        let id = SubjectId::new(1);
        assert!(store.insert_subject(id));
        assert!(store.subject(id).is_some());
        assert!(store.remove_subject(id));
        assert!(store.subject(id).is_none());
    }

    // ── Process fixture ───────────────────────────────────────────────────

    #[test]
    fn process_mode_update() {
        let mut store = StateStore::new();
        let pid = ProcessId::new(1);
        let mid = ModeId::new(7);
        store.insert_process(pid);
        store.insert_mode(mid);

        assert!(store.process(pid).unwrap().mode.is_none());
        store.process_mut(pid).unwrap().mode = Some(mid);
        assert_eq!(store.process(pid).unwrap().mode, Some(mid));
    }

    // ── Signal / Tag fixtures ─────────────────────────────────────────────

    #[test]
    fn create_and_delete_signal() {
        let mut store = StateStore::new();
        let id = SignalId::new(42);
        assert!(store.insert_signal(id));
        assert!(store.signal(id).is_some());
        assert!(store.remove_signal(id));
        assert!(store.signal(id).is_none());
    }

    #[test]
    fn create_and_delete_tag() {
        let mut store = StateStore::new();
        let id = TagId::new(3);
        assert!(store.insert_tag(id));
        assert!(store.tag(id).is_some());
        assert!(store.remove_tag(id));
        assert!(store.tag(id).is_none());
    }

    // ── Stale-reference safety ────────────────────────────────────────────

    #[test]
    fn stale_entity_id_is_lookup_failure_not_panic() {
        let mut store = StateStore::new();
        let id = EntityId::new(77);
        store.insert_entity(id);
        store.remove_entity(id);
        // Must return None, not panic.
        assert!(store.entity(id).is_none());
        assert!(store.entity_mut(id).is_none());
    }
}

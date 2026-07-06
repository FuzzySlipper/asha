//! Rule-owned relationship authority facade.
//!
//! # Lane
//!
//! `rust-rule` — validates and applies generic entity relationship operations
//! without renderer, UI, TypeScript, or product-domain concepts.
//!
//! # Design
//!
//! `core-entity` already owns the durable relationship tables and low-level
//! fail-closed mutation (`EntityStore::apply_relation`). This crate is the rule
//! assignment cell around that substrate: it exposes an explicit request/readout
//! API, classifies diagnostics for orchestrators, and proves callers use the
//! existing core semantics instead of growing a second relation graph.

#![forbid(unsafe_code)]

use core_entity::{EntityStore, RelationCommand, RelationError, RelationKind};
use core_error::ErrorCategory;
use core_ids::EntityId;

/// A typed relationship operation proposed to the rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationshipRequest {
    pub command: RelationCommand,
}

impl RelationshipRequest {
    pub const fn attach_transform_parent(child: EntityId, parent: EntityId) -> Self {
        Self {
            command: RelationCommand::AttachTransformParent { child, parent },
        }
    }

    pub const fn detach_transform_parent(child: EntityId) -> Self {
        Self {
            command: RelationCommand::DetachTransformParent { child },
        }
    }

    pub const fn set_containment(member: EntityId, container: EntityId) -> Self {
        Self {
            command: RelationCommand::SetContainment { member, container },
        }
    }

    pub const fn clear_containment(member: EntityId) -> Self {
        Self {
            command: RelationCommand::ClearContainment { member },
        }
    }

    pub const fn set_derived_from(derived: EntityId, origin: EntityId) -> Self {
        Self {
            command: RelationCommand::SetDerivedFrom { derived, origin },
        }
    }

    pub const fn set_render_group(member: EntityId) -> Self {
        Self {
            command: RelationCommand::SetRenderGroup { member },
        }
    }

    pub fn kind(self) -> RelationKind {
        relation_kind(self.command)
    }

    pub fn subject(self) -> EntityId {
        relation_subject(self.command)
    }

    pub fn target(self) -> Option<EntityId> {
        relation_target(self.command)
    }
}

impl From<RelationCommand> for RelationshipRequest {
    fn from(command: RelationCommand) -> Self {
        Self { command }
    }
}

/// A successful relationship rule mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationshipApplied {
    pub kind: RelationKind,
    pub subject: EntityId,
    pub target: Option<EntityId>,
}

/// A non-mutating preview result for a proposed relationship operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationshipPreview {
    pub applied: RelationshipApplied,
}

/// Current relationship pointers for an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationshipReadout {
    pub entity: EntityId,
    pub transform_parent: Option<EntityId>,
    pub container: Option<EntityId>,
    pub derived_from: Option<EntityId>,
}

/// Typed rejection from the relationship rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationshipRuleError {
    pub source: RelationError,
}

impl RelationshipRuleError {
    pub fn category(self) -> ErrorCategory {
        match self.source {
            RelationError::UnknownEntity { .. } | RelationError::NoSuchRelation { .. } => {
                ErrorCategory::NotFound
            }
            RelationError::Tombstoned { .. } => ErrorCategory::Conflict,
            RelationError::Cycle { .. }
            | RelationError::NotTransformEligible { .. }
            | RelationError::SelfRelation { .. } => ErrorCategory::Invalid,
            RelationError::ProjectionOnly { .. } => ErrorCategory::Unsupported,
        }
    }

    pub fn code(self) -> &'static str {
        match self.source {
            RelationError::UnknownEntity { .. } => "unknown_entity",
            RelationError::Tombstoned { .. } => "tombstoned_entity",
            RelationError::Cycle { .. } => "relationship_cycle",
            RelationError::NotTransformEligible { .. } => "not_transform_eligible",
            RelationError::SelfRelation { .. } => "self_relationship",
            RelationError::NoSuchRelation { .. } => "no_such_relationship",
            RelationError::ProjectionOnly { .. } => "projection_only_relationship",
        }
    }
}

impl From<RelationError> for RelationshipRuleError {
    fn from(source: RelationError) -> Self {
        Self { source }
    }
}

impl core::fmt::Display for RelationshipRuleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.source)
    }
}

impl std::error::Error for RelationshipRuleError {}

/// Read the current relationship pointers for `entity`.
pub fn read_relationships(store: &EntityStore, entity: EntityId) -> RelationshipReadout {
    RelationshipReadout {
        entity,
        transform_parent: store.transform_parent_of(entity),
        container: store.containment(entity).map(|c| c.container),
        derived_from: store.derived_from(entity),
    }
}

/// Validate a relationship operation without mutating the caller's store.
pub fn preview_relationship(
    store: &EntityStore,
    request: RelationshipRequest,
) -> Result<RelationshipPreview, RelationshipRuleError> {
    let mut scratch = store.clone();
    let applied = apply_relationship(&mut scratch, request)?;
    Ok(RelationshipPreview { applied })
}

/// Validate and apply a relationship operation atomically through `core-entity`.
pub fn apply_relationship(
    store: &mut EntityStore,
    request: RelationshipRequest,
) -> Result<RelationshipApplied, RelationshipRuleError> {
    store.apply_relation(request.command)?;
    Ok(RelationshipApplied {
        kind: request.kind(),
        subject: request.subject(),
        target: request.target(),
    })
}

fn relation_kind(command: RelationCommand) -> RelationKind {
    match command {
        RelationCommand::AttachTransformParent { .. }
        | RelationCommand::DetachTransformParent { .. } => RelationKind::TransformParent,
        RelationCommand::SetContainment { .. } | RelationCommand::ClearContainment { .. } => {
            RelationKind::Containment
        }
        RelationCommand::SetDerivedFrom { .. } => RelationKind::SourceAncestry,
        RelationCommand::SetRenderGroup { .. } => RelationKind::RenderGrouping,
    }
}

fn relation_subject(command: RelationCommand) -> EntityId {
    match command {
        RelationCommand::AttachTransformParent { child, .. }
        | RelationCommand::DetachTransformParent { child } => child,
        RelationCommand::SetContainment { member, .. }
        | RelationCommand::ClearContainment { member }
        | RelationCommand::SetRenderGroup { member } => member,
        RelationCommand::SetDerivedFrom { derived, .. } => derived,
    }
}

fn relation_target(command: RelationCommand) -> Option<EntityId> {
    match command {
        RelationCommand::AttachTransformParent { parent, .. } => Some(parent),
        RelationCommand::SetContainment { container, .. } => Some(container),
        RelationCommand::SetDerivedFrom { origin, .. } => Some(origin),
        RelationCommand::DetachTransformParent { .. }
        | RelationCommand::ClearContainment { .. }
        | RelationCommand::SetRenderGroup { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_entity::{EntityLifecycleCommand, EntitySource, EntityTransform};

    fn entity(id: u64) -> EntityId {
        EntityId::new(id)
    }

    fn create_logical(store: &mut EntityStore, id: u64) -> EntityId {
        let entity = entity(id);
        store
            .apply(EntityLifecycleCommand::Create {
                id: entity,
                source: EntitySource::RuntimeCreated { by: None },
                labels: vec![],
            })
            .unwrap();
        entity
    }

    fn create_spatial(store: &mut EntityStore, id: u64) -> EntityId {
        let entity = create_logical(store, id);
        assert!(store.attach_transform(entity, EntityTransform::IDENTITY));
        entity
    }

    #[test]
    fn transform_parent_attach_and_detach_use_core_entity_tables() {
        let mut store = EntityStore::new();
        let parent = create_spatial(&mut store, 1);
        let child = create_spatial(&mut store, 2);

        let applied = apply_relationship(
            &mut store,
            RelationshipRequest::attach_transform_parent(child, parent),
        )
        .unwrap();

        assert_eq!(applied.kind, RelationKind::TransformParent);
        assert_eq!(applied.subject, child);
        assert_eq!(applied.target, Some(parent));
        assert_eq!(
            read_relationships(&store, child).transform_parent,
            Some(parent)
        );

        let detached = apply_relationship(
            &mut store,
            RelationshipRequest::detach_transform_parent(child),
        )
        .unwrap();

        assert_eq!(detached.target, None);
        assert_eq!(read_relationships(&store, child).transform_parent, None);
    }

    #[test]
    fn preview_validates_without_mutating() {
        let mut store = EntityStore::new();
        let container = create_logical(&mut store, 1);
        let member = create_logical(&mut store, 2);

        let preview = preview_relationship(
            &store,
            RelationshipRequest::set_containment(member, container),
        )
        .unwrap();

        assert_eq!(preview.applied.kind, RelationKind::Containment);
        assert_eq!(
            read_relationships(&store, member).container,
            None,
            "preview must not mutate authoritative state"
        );
    }

    #[test]
    fn containment_cycle_is_rejected_by_core_semantics() {
        let mut store = EntityStore::new();
        let a = create_logical(&mut store, 1);
        let b = create_logical(&mut store, 2);
        apply_relationship(&mut store, RelationshipRequest::set_containment(b, a)).unwrap();

        let err =
            apply_relationship(&mut store, RelationshipRequest::set_containment(a, b)).unwrap_err();

        assert_eq!(
            err.source,
            RelationError::Cycle {
                kind: RelationKind::Containment,
                at: a
            }
        );
        assert_eq!(err.category(), ErrorCategory::Invalid);
        assert_eq!(err.code(), "relationship_cycle");
    }

    #[test]
    fn transform_attachment_requires_spatial_endpoints() {
        let mut store = EntityStore::new();
        let child = create_spatial(&mut store, 1);
        let parent = create_logical(&mut store, 2);

        let err = apply_relationship(
            &mut store,
            RelationshipRequest::attach_transform_parent(child, parent),
        )
        .unwrap_err();

        assert_eq!(
            err.source,
            RelationError::NotTransformEligible { id: parent }
        );
        assert_eq!(err.category(), ErrorCategory::Invalid);
    }

    #[test]
    fn source_ancestry_records_dependency_trace() {
        let mut store = EntityStore::new();
        let origin = create_logical(&mut store, 1);
        let derived = create_logical(&mut store, 2);

        let applied = apply_relationship(
            &mut store,
            RelationshipRequest::set_derived_from(derived, origin),
        )
        .unwrap();

        assert_eq!(applied.kind, RelationKind::SourceAncestry);
        assert_eq!(
            read_relationships(&store, derived).derived_from,
            Some(origin)
        );
    }

    #[test]
    fn unknown_and_tombstoned_endpoints_fail_closed() {
        let mut store = EntityStore::new();
        let child = create_spatial(&mut store, 1);

        let unknown = apply_relationship(
            &mut store,
            RelationshipRequest::attach_transform_parent(child, entity(99)),
        )
        .unwrap_err();
        assert_eq!(
            unknown.source,
            RelationError::UnknownEntity { id: entity(99) }
        );
        assert_eq!(unknown.category(), ErrorCategory::NotFound);

        let parent = create_spatial(&mut store, 2);
        store
            .apply(EntityLifecycleCommand::Destroy { id: parent })
            .unwrap();

        let tombstoned = apply_relationship(
            &mut store,
            RelationshipRequest::attach_transform_parent(child, parent),
        )
        .unwrap_err();
        assert_eq!(tombstoned.source, RelationError::Tombstoned { id: parent });
        assert_eq!(tombstoned.category(), ErrorCategory::Conflict);
    }

    #[test]
    fn self_relationship_and_missing_detach_are_rejected() {
        let mut store = EntityStore::new();
        let child = create_spatial(&mut store, 1);

        let self_relation = apply_relationship(
            &mut store,
            RelationshipRequest::attach_transform_parent(child, child),
        )
        .unwrap_err();
        assert_eq!(
            self_relation.source,
            RelationError::SelfRelation {
                kind: RelationKind::TransformParent,
                id: child
            }
        );

        let missing = apply_relationship(
            &mut store,
            RelationshipRequest::detach_transform_parent(child),
        )
        .unwrap_err();
        assert_eq!(
            missing.source,
            RelationError::NoSuchRelation {
                kind: RelationKind::TransformParent,
                id: child
            }
        );
    }

    #[test]
    fn render_grouping_fails_closed_as_projection_only() {
        let mut store = EntityStore::new();
        let member = create_logical(&mut store, 1);

        let err = apply_relationship(&mut store, RelationshipRequest::set_render_group(member))
            .unwrap_err();

        assert_eq!(
            err.source,
            RelationError::ProjectionOnly {
                kind: RelationKind::RenderGrouping
            }
        );
        assert_eq!(err.category(), ErrorCategory::Unsupported);
        assert_eq!(err.code(), "projection_only_relationship");
    }
}

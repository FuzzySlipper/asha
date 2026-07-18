// @asha/devtools — scene/world outliner and inspector read models (#2377).
//
// Observational, **read-only** projections of authored scene data + projected
// runtime authority. These build a display tree and per-node/per-entity inspection
// views from generated contracts; they never mutate authority and own no scene
// copy that could become a second truth.
//
// Authoritative shapes (`FlatSceneDocument`, `SceneSourceTrace`, ...) come from
// `@asha/contracts`. The runtime-entity projection (`RuntimeEntityProjection`)
// is a plain mirror of authority state carried over the bridge as projected data
// — devtools never reads Rust authority directly (mirrors `SceneReportSummary`).

import type {
  AssetReference,
  EntityId,
  FlatSceneDocument,
  SceneNodeId,
  SceneNodeKind,
  SceneNodeRecord,
  SceneSourceTrace,
  SceneTransform,
} from '@asha/contracts';

// ── Projected runtime authority (mirror, not truth) ─────────────────────────────

/** Lifecycle of a projected runtime entity (mirrors core-entity authority). */
export type RuntimeLifecycle = 'active' | 'disabled' | 'tombstoned';

/**
 * A plain mirror of one runtime entity's authority record, as projected over the
 * bridge. `transform` is present only for spatial entities; `sourceNode` is set
 * for entities bootstrapped from a scene node and null for runtime-created ones.
 */
export interface RuntimeEntityProjection {
  readonly entityId: EntityId;
  readonly lifecycle: RuntimeLifecycle;
  readonly transform: SceneTransform | null;
  readonly sourceNode: SceneNodeId | null;
}

/** The full read-model input: authored scene + projected runtime + source traces. */
export interface OutlinerInput {
  readonly scene: FlatSceneDocument;
  readonly entities: readonly RuntimeEntityProjection[];
  readonly sourceTraces: readonly SceneSourceTrace[];
}

// ── Correlation between an authored node and runtime authority ──────────────────

/** How an authored scene node relates to projected runtime authority. */
export type NodeCorrelation =
  // No source trace: the node was not bootstrapped into an entity (e.g. a pure
  // group). Not an error on its own.
  | { readonly kind: 'authoredOnly' }
  // A live runtime entity was bootstrapped from this node.
  | { readonly kind: 'matched'; readonly entityId: EntityId; readonly lifecycle: 'active' | 'disabled' }
  // The scene-sourced entity exists but is tombstoned — surfaced, never hidden.
  | { readonly kind: 'destroyed'; readonly entityId: EntityId }
  // A trace exists but the entity is absent from the projection (dangling).
  | { readonly kind: 'danglingTrace'; readonly entityId: EntityId };

/** One node in the display tree: the authored record plus correlation + children. */
export interface OutlinerNode {
  readonly node: SceneNodeRecord;
  readonly correlation: NodeCorrelation;
  readonly children: readonly OutlinerNode[];
}

/** A runtime entity with no authored scene source (created at runtime). */
export interface RuntimeOnlyEntity {
  readonly entityId: EntityId;
  readonly lifecycle: RuntimeLifecycle;
  readonly hasTransform: boolean;
}

/** A classified outliner readout. Missing/stale correlations are never silent. */
export type OutlinerDiagnosticCode =
  | 'orphanedNode' // parent id is not present in the document
  | 'danglingSourceTrace' // a trace points at an entity absent from the projection
  | 'destroyedSceneEntity' // a scene-sourced entity is tombstoned
  | 'danglingEntitySource'; // a runtime entity names a scene node that is absent

export interface OutlinerDiagnostic {
  readonly code: OutlinerDiagnosticCode;
  readonly sceneNode: SceneNodeId | null;
  readonly entityId: EntityId | null;
  readonly detail: string;
}

/** The full outliner read model: a tree, runtime-only entities, and diagnostics. */
export interface OutlinerModel {
  /** Roots in authored `childOrder`, then ascending id for ties. */
  readonly roots: readonly OutlinerNode[];
  /** Authored nodes whose parent is absent — shown explicitly, not dropped. */
  readonly orphans: readonly OutlinerNode[];
  /** Runtime entities with no scene source. */
  readonly runtimeOnly: readonly RuntimeOnlyEntity[];
  readonly diagnostics: readonly OutlinerDiagnostic[];
}

// ── Tree construction ──────────────────────────────────────────────────────────

function sortRecords(records: readonly SceneNodeRecord[]): SceneNodeRecord[] {
  // Stable display order: authored childOrder first, then ascending id for ties.
  return [...records].sort((a, b) => {
    if (a.childOrder !== b.childOrder) {
      return a.childOrder - b.childOrder;
    }
    return (a.id as number) - (b.id as number);
  });
}

function correlate(
  nodeId: SceneNodeId,
  traceByNode: ReadonlyMap<number, EntityId>,
  entityById: ReadonlyMap<number, RuntimeEntityProjection>,
): NodeCorrelation {
  const tracedEntity = traceByNode.get(nodeId as number);
  if (tracedEntity === undefined) {
    return { kind: 'authoredOnly' };
  }
  const entity = entityById.get(tracedEntity as number);
  if (entity === undefined) {
    return { kind: 'danglingTrace', entityId: tracedEntity };
  }
  if (entity.lifecycle === 'tombstoned') {
    return { kind: 'destroyed', entityId: tracedEntity };
  }
  return { kind: 'matched', entityId: tracedEntity, lifecycle: entity.lifecycle };
}

/**
 * Build the outliner read model: a parent/childOrder tree of authored nodes with
 * runtime correlation, the set of runtime-created entities, and a classified list
 * of every missing/stale correlation (never a silent omission).
 */
export function buildOutlinerModel(input: OutlinerInput): OutlinerModel {
  const records = input.scene.nodes;
  const known = new Set<number>(records.map((r) => r.id as number));

  const traceByNode = new Map<number, EntityId>();
  for (const trace of input.sourceTraces) {
    traceByNode.set(trace.sceneNodeId as number, trace.runtimeEntityId);
  }
  const entityById = new Map<number, RuntimeEntityProjection>();
  for (const entity of input.entities) {
    entityById.set(entity.entityId as number, entity);
  }

  const childrenOf = new Map<number, SceneNodeRecord[]>();
  const orphanRecords: SceneNodeRecord[] = [];
  for (const rec of records) {
    if (rec.parent === null) {
      continue;
    }
    if (!known.has(rec.parent as number)) {
      orphanRecords.push(rec);
      continue;
    }
    const bucket = childrenOf.get(rec.parent as number);
    if (bucket === undefined) {
      childrenOf.set(rec.parent as number, [rec]);
    } else {
      bucket.push(rec);
    }
  }

  const diagnostics: OutlinerDiagnostic[] = [];

  const build = (rec: SceneNodeRecord): OutlinerNode => {
    const correlation = correlate(rec.id, traceByNode, entityById);
    if (correlation.kind === 'danglingTrace') {
      diagnostics.push({
        code: 'danglingSourceTrace',
        sceneNode: rec.id,
        entityId: correlation.entityId,
        detail: `scene node ${rec.id as number} traces to entity ${correlation.entityId as number}, which is absent from the runtime projection`,
      });
    } else if (correlation.kind === 'destroyed') {
      diagnostics.push({
        code: 'destroyedSceneEntity',
        sceneNode: rec.id,
        entityId: correlation.entityId,
        detail: `scene node ${rec.id as number} was bootstrapped to entity ${correlation.entityId as number}, now tombstoned`,
      });
    }
    const kids = sortRecords(childrenOf.get(rec.id as number) ?? []).map(build);
    return { node: rec, correlation, children: kids };
  };

  const rootRecords = sortRecords(records.filter((r) => r.parent === null));
  const roots = rootRecords.map(build);
  const orphans = sortRecords(orphanRecords).map(build);

  for (const rec of orphanRecords) {
    diagnostics.push({
      code: 'orphanedNode',
      sceneNode: rec.id,
      entityId: null,
      detail: `scene node ${rec.id as number} names absent parent ${rec.parent as number}`,
    });
  }

  const runtimeOnly: RuntimeOnlyEntity[] = [];
  for (const entity of input.entities) {
    if (entity.sourceNode === null) {
      runtimeOnly.push({
        entityId: entity.entityId,
        lifecycle: entity.lifecycle,
        hasTransform: entity.transform !== null,
      });
    } else if (!known.has(entity.sourceNode as number)) {
      diagnostics.push({
        code: 'danglingEntitySource',
        sceneNode: entity.sourceNode,
        entityId: entity.entityId,
        detail: `entity ${entity.entityId as number} names source node ${entity.sourceNode as number}, absent from the scene`,
      });
    }
  }
  runtimeOnly.sort((a, b) => (a.entityId as number) - (b.entityId as number));

  return { roots, orphans, runtimeOnly, diagnostics };
}

// ── Node / entity inspection ────────────────────────────────────────────────────

/** A scalar field comparison between an authored value and runtime authority. */
export interface TransformComparison {
  readonly authored: SceneTransform;
  readonly runtime: SceneTransform | null;
  /** True when both exist and differ (component-wise) — a runtime override. */
  readonly diverged: boolean;
}

function transformsEqual(a: SceneTransform, b: SceneTransform): boolean {
  const tripleEqual = (x: readonly number[], y: readonly number[]): boolean =>
    x.length === y.length && x.every((v, i) => Object.is(v, y[i]));
  return (
    tripleEqual(a.translation, b.translation) &&
    tripleEqual(a.rotation, b.rotation) &&
    tripleEqual(a.scale, b.scale)
  );
}

/** Asset references a node depends on, keyed by what they are bound to. */
export interface NodeAssetRefs {
  readonly kindTag: SceneNodeKind['kind'];
  /** The asset backing an asset-kind node (staticMesh/sprite/voxelVolume). */
  readonly asset: AssetReference | null;
}

function assetOf(kind: SceneNodeKind): AssetReference | null {
  switch (kind.kind) {
    case 'staticMesh':
    case 'sprite':
    case 'voxelVolume':
      return kind.asset;
    case 'emptyGroup':
    case 'light':
    case 'marker':
    case 'entityInstance':
    case 'bootstrap':
      return null;
  }
}

/** The inspector read model for one authored scene node. */
export interface NodeInspection {
  readonly node: SceneNodeRecord;
  readonly correlation: NodeCorrelation;
  /** Authored initial transform vs runtime authority transform (when both exist). */
  readonly transform: TransformComparison;
  readonly assetRefs: NodeAssetRefs;
}

/**
 * Inspect one authored scene node: its authored fields, the runtime authority
 * fields (transform), asset refs, and correlation. Returns null for an unknown id.
 */
export function inspectNode(input: OutlinerInput, nodeId: SceneNodeId): NodeInspection | null {
  const record = input.scene.nodes.find((r) => (r.id as number) === (nodeId as number));
  if (record === undefined) {
    return null;
  }
  const traceByNode = new Map<number, EntityId>();
  for (const trace of input.sourceTraces) {
    traceByNode.set(trace.sceneNodeId as number, trace.runtimeEntityId);
  }
  const entityById = new Map<number, RuntimeEntityProjection>();
  for (const entity of input.entities) {
    entityById.set(entity.entityId as number, entity);
  }
  const correlation = correlate(nodeId, traceByNode, entityById);

  let runtimeTransform: SceneTransform | null = null;
  if (correlation.kind === 'matched' || correlation.kind === 'destroyed') {
    runtimeTransform = entityById.get(correlation.entityId as number)?.transform ?? null;
  }

  const transform: TransformComparison = {
    authored: record.transform,
    runtime: runtimeTransform,
    diverged: runtimeTransform !== null && !transformsEqual(record.transform, runtimeTransform),
  };

  return {
    node: record,
    correlation,
    transform,
    assetRefs: { kindTag: record.kind.kind, asset: assetOf(record.kind) },
  };
}

/** The inspector read model for one runtime entity. */
export interface EntityInspection {
  readonly entity: RuntimeEntityProjection;
  /** The authored node it was sourced from, if the source resolves in the scene. */
  readonly sourceNode: SceneNodeRecord | null;
  /** True when the entity names a source node that the scene does not contain. */
  readonly danglingSource: boolean;
}

/** Inspect one runtime entity. Returns null for an unknown id. */
export function inspectEntity(input: OutlinerInput, entityId: EntityId): EntityInspection | null {
  const entity = input.entities.find((e) => (e.entityId as number) === (entityId as number));
  if (entity === undefined) {
    return null;
  }
  if (entity.sourceNode === null) {
    return { entity, sourceNode: null, danglingSource: false };
  }
  const sourceNode = input.scene.nodes.find((r) => (r.id as number) === (entity.sourceNode as number)) ?? null;
  return { entity, sourceNode, danglingSource: sourceNode === null };
}

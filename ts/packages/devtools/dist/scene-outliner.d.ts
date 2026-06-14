import type { AssetReference, EntityId, FlatSceneDocument, SceneNodeId, SceneNodeKind, SceneNodeRecord, SceneSourceTrace, SceneTransform } from '@asha/contracts';
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
/** How an authored scene node relates to projected runtime authority. */
export type NodeCorrelation = {
    readonly kind: 'authoredOnly';
} | {
    readonly kind: 'matched';
    readonly entityId: EntityId;
    readonly lifecycle: 'active' | 'disabled';
} | {
    readonly kind: 'destroyed';
    readonly entityId: EntityId;
} | {
    readonly kind: 'danglingTrace';
    readonly entityId: EntityId;
};
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
export type OutlinerDiagnosticCode = 'orphanedNode' | 'danglingSourceTrace' | 'destroyedSceneEntity' | 'danglingEntitySource';
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
/**
 * Build the outliner read model: a parent/childOrder tree of authored nodes with
 * runtime correlation, the set of runtime-created entities, and a classified list
 * of every missing/stale correlation (never a silent omission).
 */
export declare function buildOutlinerModel(input: OutlinerInput): OutlinerModel;
/** A scalar field comparison between an authored value and runtime authority. */
export interface TransformComparison {
    readonly authored: SceneTransform;
    readonly runtime: SceneTransform | null;
    /** True when both exist and differ (component-wise) — a runtime override. */
    readonly diverged: boolean;
}
/** Asset references a node depends on, keyed by what they are bound to. */
export interface NodeAssetRefs {
    readonly kindTag: SceneNodeKind['kind'];
    /** The asset backing an asset-kind node (staticMesh/sprite/voxelVolume). */
    readonly asset: AssetReference | null;
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
export declare function inspectNode(input: OutlinerInput, nodeId: SceneNodeId): NodeInspection | null;
/** The inspector read model for one runtime entity. */
export interface EntityInspection {
    readonly entity: RuntimeEntityProjection;
    /** The authored node it was sourced from, if the source resolves in the scene. */
    readonly sourceNode: SceneNodeRecord | null;
    /** True when the entity names a source node that the scene does not contain. */
    readonly danglingSource: boolean;
}
/** Inspect one runtime entity. Returns null for an unknown id. */
export declare function inspectEntity(input: OutlinerInput, entityId: EntityId): EntityInspection | null;
//# sourceMappingURL=scene-outliner.d.ts.map
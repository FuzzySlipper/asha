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
// ── Tree construction ──────────────────────────────────────────────────────────
function sortRecords(records) {
    // Stable display order: authored childOrder first, then ascending id for ties.
    return [...records].sort((a, b) => {
        if (a.childOrder !== b.childOrder) {
            return a.childOrder - b.childOrder;
        }
        return a.id - b.id;
    });
}
function correlate(nodeId, traceByNode, entityById) {
    const tracedEntity = traceByNode.get(nodeId);
    if (tracedEntity === undefined) {
        return { kind: 'authoredOnly' };
    }
    const entity = entityById.get(tracedEntity);
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
export function buildOutlinerModel(input) {
    const records = input.scene.nodes;
    const known = new Set(records.map((r) => r.id));
    const traceByNode = new Map();
    for (const trace of input.sourceTraces) {
        traceByNode.set(trace.sceneNodeId, trace.runtimeEntityId);
    }
    const entityById = new Map();
    for (const entity of input.entities) {
        entityById.set(entity.entityId, entity);
    }
    const childrenOf = new Map();
    const orphanRecords = [];
    for (const rec of records) {
        if (rec.parent === null) {
            continue;
        }
        if (!known.has(rec.parent)) {
            orphanRecords.push(rec);
            continue;
        }
        const bucket = childrenOf.get(rec.parent);
        if (bucket === undefined) {
            childrenOf.set(rec.parent, [rec]);
        }
        else {
            bucket.push(rec);
        }
    }
    const diagnostics = [];
    const build = (rec) => {
        const correlation = correlate(rec.id, traceByNode, entityById);
        if (correlation.kind === 'danglingTrace') {
            diagnostics.push({
                code: 'danglingSourceTrace',
                sceneNode: rec.id,
                entityId: correlation.entityId,
                detail: `scene node ${rec.id} traces to entity ${correlation.entityId}, which is absent from the runtime projection`,
            });
        }
        else if (correlation.kind === 'destroyed') {
            diagnostics.push({
                code: 'destroyedSceneEntity',
                sceneNode: rec.id,
                entityId: correlation.entityId,
                detail: `scene node ${rec.id} was bootstrapped to entity ${correlation.entityId}, now tombstoned`,
            });
        }
        const kids = sortRecords(childrenOf.get(rec.id) ?? []).map(build);
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
            detail: `scene node ${rec.id} names absent parent ${rec.parent}`,
        });
    }
    const runtimeOnly = [];
    for (const entity of input.entities) {
        if (entity.sourceNode === null) {
            runtimeOnly.push({
                entityId: entity.entityId,
                lifecycle: entity.lifecycle,
                hasTransform: entity.transform !== null,
            });
        }
        else if (!known.has(entity.sourceNode)) {
            diagnostics.push({
                code: 'danglingEntitySource',
                sceneNode: entity.sourceNode,
                entityId: entity.entityId,
                detail: `entity ${entity.entityId} names source node ${entity.sourceNode}, absent from the scene`,
            });
        }
    }
    runtimeOnly.sort((a, b) => a.entityId - b.entityId);
    return { roots, orphans, runtimeOnly, diagnostics };
}
function transformsEqual(a, b) {
    const tripleEqual = (x, y) => x.length === y.length && x.every((v, i) => Object.is(v, y[i]));
    return (tripleEqual(a.translation, b.translation) &&
        tripleEqual(a.rotation, b.rotation) &&
        tripleEqual(a.scale, b.scale));
}
function assetOf(kind) {
    return kind.kind === 'emptyGroup' ? null : kind.asset;
}
/**
 * Inspect one authored scene node: its authored fields, the runtime authority
 * fields (transform), asset refs, and correlation. Returns null for an unknown id.
 */
export function inspectNode(input, nodeId) {
    const record = input.scene.nodes.find((r) => r.id === nodeId);
    if (record === undefined) {
        return null;
    }
    const traceByNode = new Map();
    for (const trace of input.sourceTraces) {
        traceByNode.set(trace.sceneNodeId, trace.runtimeEntityId);
    }
    const entityById = new Map();
    for (const entity of input.entities) {
        entityById.set(entity.entityId, entity);
    }
    const correlation = correlate(nodeId, traceByNode, entityById);
    let runtimeTransform = null;
    if (correlation.kind === 'matched' || correlation.kind === 'destroyed') {
        runtimeTransform = entityById.get(correlation.entityId)?.transform ?? null;
    }
    const transform = {
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
/** Inspect one runtime entity. Returns null for an unknown id. */
export function inspectEntity(input, entityId) {
    const entity = input.entities.find((e) => e.entityId === entityId);
    if (entity === undefined) {
        return null;
    }
    if (entity.sourceNode === null) {
        return { entity, sourceNode: null, danglingSource: false };
    }
    const sourceNode = input.scene.nodes.find((r) => r.id === entity.sourceNode) ?? null;
    return { entity, sourceNode, danglingSource: sourceNode === null };
}
//# sourceMappingURL=scene-outliner.js.map
// Import/typecheck smoke for @asha/contracts.
//
// This is the proof for the Phase 2 exit criterion "a TypeScript package can
// import generated branded IDs and command unions" (see
// governance/protocol-border-consumers.md). It is NOT part of the public API
// (index.ts does not re-export it). Its only job is to fail `tsc` if the
// generated contracts stop being importable or usable — proving that branded
// IDs and the command/view/diff/replay unions compile when consumed exactly as
// a downstream package would consume them, with no policy, renderer, UI,
// bridge, Electron, or browser globals in scope.
//
// It is value-level on purpose: constructing real union values exercises the
// discriminants and field shapes, not just the type names.
import { entityId, modeId, tagId, renderHandle, stepIndex, replayHash, REPLAY_FORMAT_VERSION, } from './index.js';
// Branded IDs are nominally typed and built through their constructors.
const entity = entityId(1);
// A command authored the way a policy would author it.
const addTag = {
    domain: 'entity',
    command: { kind: 'addTag', id: entity, tag: tagId(2) },
};
const envelope = { kind: 'policy', command: addTag };
// A read-only view value.
const view = {
    entities: [{ id: entity, tags: [tagId(2)] }],
    subjects: [],
    processes: [],
    modes: [modeId(3)],
    signals: [],
    tags: [tagId(2)],
};
const outcome = { status: 'accepted' };
// A retained-mode render diff value: create an abstract cube node, then destroy.
const createDiff = {
    op: 'create',
    handle: renderHandle(5),
    parent: null,
    node: {
        geometry: { shape: 'cube' },
        material: { color: [1, 1, 1, 1], wireframe: false },
        transform: {
            translation: [0, 0, 0],
            rotation: [0, 0, 0, 1],
            scale: [1, 1, 1],
        },
        visible: true,
        layer: 'scene',
        metadata: { source: entity, tags: [tagId(2)], label: 'cube' },
    },
};
const diff = { op: 'destroy', handle: renderHandle(5) };
// A replay record value, with the format version sourced from the contract.
const record = {
    formatVersion: REPLAY_FORMAT_VERSION,
    initialHash: replayHash(0),
    steps: [
        {
            index: stepIndex(0),
            command: envelope,
            outcome: { status: 'accepted', events: [{ event: 'entityCreated', id: entity }] },
            postHash: replayHash(1),
        },
    ],
    snapshots: [],
};
// A diagnostic report value, authored the way a devtools panel would consume
// one: a broken source trace pointing at a missing sprite texture, plus a
// fatal corrupt-bundle report. Proves the generated diagnostic contracts are
// importable and usable (scene-capability-06, #2330).
const missingAsset = {
    scope: 'scene',
    severity: 'error',
    code: 'sceneAssetMissing',
    reference: 'person-spawn-03',
    source: {
        sceneNodeId: 3,
        runtimeEntityId: 456,
        assetId: 'sprite/hard-hat',
        chunkCoord: null,
        renderHandle: 43,
        bundlePath: null,
    },
    message: 'scene node references a sprite the catalog does not contain',
    remedy: { action: 'provideAsset', detail: 'add sprite/hard-hat to the catalog' },
};
const corruptArtifact = {
    scope: 'worldBundle',
    severity: 'fatal',
    code: 'corruptBundleArtifact',
    reference: 'chunks/0_0_0.snap',
    source: {
        sceneNodeId: null,
        runtimeEntityId: null,
        assetId: null,
        chunkCoord: [0, 0, 0],
        renderHandle: null,
        bundlePath: 'chunks/0_0_0.snap',
    },
    message: 'durable artifact failed its content hash',
    remedy: { action: 'restoreArtifact', detail: 'restore from a known-good bundle copy' },
};
const reportSet = { reports: [missingAsset, corruptArtifact] };
const trace = {
    renderHandle: 43,
    sceneNodeId: 3,
    runtimeEntityId: 456,
    assetId: 'sprite/hard-hat',
    assetResolved: false,
};
const resources = {
    liveHandles: 2,
    geometries: 1,
    materials: 1,
    spriteInstances: 1,
    spritesUpdatedLastTick: 1,
    resourcesCreated: 4,
    resourcesDisposed: 4,
    fallbackMaterials: 0,
};
// Exported so the values are "used" (lint-clean) and tree-shakeable. Consumers
// of @asha/contracts never see this — it is not re-exported by index.ts.
export const __contractSmoke = {
    entity,
    addTag,
    envelope,
    view,
    outcome,
    createDiff,
    diff,
    record,
    reportSet,
    trace,
    resources,
};
//# sourceMappingURL=smoke.js.map
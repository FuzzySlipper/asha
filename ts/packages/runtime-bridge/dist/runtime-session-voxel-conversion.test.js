import { test } from 'node:test';
import assert from 'node:assert/strict';
import { RuntimeBridgeError, createRuntimeSessionFacade } from './index.js';
import { createMockRuntimeBridge } from './mock.js';
import { createMockRuntimeSession } from './reference.js';
function sessionInput() {
    return {
        sessionId: 'runtime-session.asha-demo.reference',
        seed: 17,
        project: {
            gameId: 'asha-demo',
            workspaceId: 'workspace.local',
        },
        projectBundle: {
            bundleSchemaVersion: 1,
            protocolVersion: 1,
            sceneId: 42,
        },
    };
}
function voxelConversionPlanRequest() {
    return {
        source: {
            assetId: 'mesh/quad',
            assetKind: 'mesh',
            assetVersion: 1,
            sourceHash: 'sha256:quad',
            meshPrimitive: null,
        },
        target: {
            grid: 7,
            volumeAssetId: 'voxel/generated',
            origin: { x: 0, y: 0, z: 0 },
        },
        settings: {
            mode: 'surface',
            fitPolicy: 'contain',
            originPolicy: 'target_min',
            resolution: [4, 4, 1],
            voxelSize: 1,
            maxOutputVoxels: 16,
            transform: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
            materialMap: {
                entries: [
                    {
                        sourceMaterialSlot: 0,
                        sourceMaterialId: 'mat/a',
                        voxelMaterial: 3,
                    },
                ],
                defaultVoxelMaterial: 3,
            },
        },
    };
}
void test('RuntimeSession voxel conversion facade methods are typed and fail closed until wired', () => {
    const request = voxelConversionPlanRequest();
    const referenceSession = createMockRuntimeSession();
    assert.throws(() => referenceSession.planVoxelConversion(request), (error) => error instanceof RuntimeBridgeError && error.kind === 'not_initialized');
    referenceSession.initialize(sessionInput());
    assert.throws(() => referenceSession.planVoxelConversion(request), (error) => error instanceof RuntimeBridgeError && error.kind === 'operation_unimplemented');
    assert.throws(() => referenceSession.previewVoxelConversion({
        planId: 'plan',
        expectedPlanHash: 'hash',
    }), (error) => error instanceof RuntimeBridgeError && error.kind === 'operation_unimplemented');
    assert.throws(() => referenceSession.applyVoxelConversion({
        planId: 'plan',
        expectedPlanHash: 'hash',
        expectedPreviewHash: null,
    }), (error) => error instanceof RuntimeBridgeError && error.kind === 'operation_unimplemented');
    assert.throws(() => referenceSession.exportVoxelConversionEvidence([
        {
            kind: 'plan',
            uri: 'asha://voxel-conversion/plan/plan',
            contentHash: 'fnv1a64:0000000000000000',
        },
    ]), (error) => error instanceof RuntimeBridgeError && error.kind === 'operation_unimplemented');
    const rustSession = createRuntimeSessionFacade({ bridge: createMockRuntimeBridge(), mode: 'rust' });
    rustSession.initialize(sessionInput());
    assert.throws(() => rustSession.planVoxelConversion(request), (error) => error instanceof RuntimeBridgeError && error.kind === 'operation_unimplemented');
});
//# sourceMappingURL=runtime-session-voxel-conversion.test.js.map
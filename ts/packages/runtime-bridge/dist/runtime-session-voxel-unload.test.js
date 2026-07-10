import { test } from 'node:test';
import assert from 'node:assert/strict';
import { RuntimeBridgeError, createRuntimeSessionFacade, } from './index.js';
import { MockRuntimeBridge } from './mock.js';
const UNLOAD_REQUEST = {
    grid: 7,
    volumeAssetId: 'voxel/converted-room',
    expectedSessionHash: 'fnv1a64:session-before-unload',
};
function sessionInput() {
    return {
        sessionId: 'runtime-session.voxel-unload.test',
        seed: 17,
        project: {
            gameId: 'asha-test',
            workspaceId: 'workspace.local',
        },
        projectBundle: {
            bundleSchemaVersion: 1,
            protocolVersion: 1,
            sceneId: 42,
        },
    };
}
class CapturingVoxelUnloadBridge extends MockRuntimeBridge {
    request = null;
    unloadVoxelVolumeAsset(request) {
        this.request = request;
        return {
            request,
            unloaded: true,
            modelId: 'voxel-model:grid:7:volume:voxel/converted-room',
            volumeAssetId: request.volumeAssetId,
            grid: request.grid,
            removedVoxelCount: 128,
            sessionHash: 'fnv1a64:session-after-unload',
            replayHash: 'fnv1a64:replay-after-unload',
            diagnostics: [],
        };
    }
}
void test('Rust-backed RuntimeSession forwards voxel volume unload and returns its authority receipt', () => {
    const bridge = new CapturingVoxelUnloadBridge();
    const session = createRuntimeSessionFacade({ bridge, mode: 'rust' });
    session.initialize(sessionInput());
    const receipt = session.unloadVoxelVolumeAsset(UNLOAD_REQUEST);
    assert.deepEqual(bridge.request, UNLOAD_REQUEST);
    assert.equal(receipt.unloaded, true);
    assert.equal(receipt.removedVoxelCount, 128);
    assert.equal(receipt.request.expectedSessionHash, UNLOAD_REQUEST.expectedSessionHash);
    assert.equal(receipt.sessionHash, 'fnv1a64:session-after-unload');
});
void test('reference RuntimeSession fails closed for voxel volume unload', () => {
    const session = createRuntimeSessionFacade({
        bridge: new MockRuntimeBridge(),
        mode: 'reference',
    });
    session.initialize(sessionInput());
    assert.throws(() => session.unloadVoxelVolumeAsset(UNLOAD_REQUEST), (error) => error instanceof RuntimeBridgeError && error.kind === 'operation_unimplemented');
});
//# sourceMappingURL=runtime-session-voxel-unload.test.js.map
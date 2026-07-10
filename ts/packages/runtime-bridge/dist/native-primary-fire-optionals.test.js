import assert from 'node:assert/strict';
import { test } from 'node:test';
import { NativeRuntimeBridge } from './native.js';
const HASH = 'fnv1a64:0000000000000001';
void test('native primary-fire optionals normalize omitted napi values to null', () => {
    const addon = new Proxy({}, {
        get: (_target, property) => {
            if (property === 'initializeEngine') {
                return () => 1;
            }
            if (property === 'applyFpsPrimaryFire') {
                return () => ({
                    backend: 'engine_bridge_rust',
                    authoritySurface: 'runtime_session.fps.primary_fire.v0',
                    mutationOwner: 'rule-lifecycle + svc-combat',
                    workspaceTrace: ['blocked'],
                    shooter: 101,
                    lifecycleStatus: { state: 'active' },
                    entityHash: HASH,
                    healthHash: HASH,
                    replayHash: HASH,
                });
            }
            return undefined;
        },
    });
    const bridge = new NativeRuntimeBridge(addon);
    bridge.initializeEngine({ seed: 7 });
    const result = bridge.applyFpsPrimaryFire({
        tick: 9,
        origin: [0, 0, 0],
        direction: [0, 0, 1],
    });
    assert.equal(result.target, null);
    assert.equal(result.targetHealthBefore, null);
    assert.equal(result.targetHealthAfter, null);
    assert.equal(result.targetRenderVisible, null);
});
//# sourceMappingURL=native-primary-fire-optionals.test.js.map
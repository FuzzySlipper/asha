import { test } from 'node:test';
import assert from 'node:assert/strict';
import { telemetryOverlayHandle, } from '@asha/contracts';
import { applyAshaRuntimeProjectionFrame } from './audio-host.js';
import { AshaLiveTelemetryCollector, AshaTelemetryOverlayHost, } from './telemetry-host.js';
class FakeOverlaySink {
    rendered = [];
    destroyed = [];
    render(handle, descriptor, snapshot) {
        this.rendered.push({ handle, descriptor, snapshot });
    }
    destroy(handle) {
        this.destroyed.push(handle);
    }
}
function frame(op) {
    return {
        schemaVersion: 1,
        authorityTick: 8,
        scene: { ops: [] },
        presentation: {
            replayScope: 'excludedFromReplayTruth',
            ops: [{
                    domain: 'telemetryOverlay',
                    meta: { sequence: 0, origin: null },
                    op,
                }],
        },
    };
}
function descriptor() {
    return {
        title: 'ASHA runtime',
        corner: 'topRight',
        refreshIntervalMs: 250,
        maxFrameTimeSamples: 3,
        visible: true,
    };
}
void test('headless live telemetry omits unavailable counters and preserves bounded history', () => {
    const collector = new AshaLiveTelemetryCollector({
        expectedCounters: ['entityCount', 'drawCallCount', 'renderDiffCount'],
        maxFrameTimeSamples: 2,
    });
    collector.sample({
        authorityTick: 4,
        frameTimeMs: 16,
        counters: { entityCount: 2, drawCallCount: null, renderDiffCount: 5 },
    });
    const snapshot = collector.sample({
        authorityTick: 5,
        frameTimeMs: 17,
        counters: { entityCount: 3, renderDiffCount: 7 },
    });
    assert.deepEqual(snapshot.frameTimeHistoryMs, [16, 17]);
    assert.deepEqual(snapshot.metrics.map((metric) => metric.counter), [
        'frameTimeMs',
        'entityCount',
        'renderDiffCount',
    ]);
    assert.equal(snapshot.diagnostics[0]?.code, 'counterUnavailable');
    assert.equal(snapshot.diagnostics[0]?.counter, 'drawCallCount');
    assert.deepEqual(collector.readSnapshot(), snapshot);
});
void test('telemetry overlay projects the same snapshot and local toggle changes no sample', () => {
    const collector = new AshaLiveTelemetryCollector({
        expectedCounters: ['entityCount', 'activeParticleCount'],
    });
    const sink = new FakeOverlaySink();
    const host = new AshaTelemetryOverlayHost({ collector, sink });
    const handle = telemetryOverlayHandle(1);
    const created = host.applyPresentation(frame({
        op: 'create',
        handle,
        descriptor: descriptor(),
    }).presentation);
    assert.equal(created.applied, 1);
    assert.equal(created.readout.activeOverlays, 1);
    const snapshot = host.sample({
        authorityTick: 8,
        frameTimeMs: 16.5,
        counters: { entityCount: 2, activeParticleCount: 12 },
    }, 250);
    assert.deepEqual(sink.rendered.at(-1)?.snapshot, snapshot);
    assert.equal(host.toggleVisible(handle), false);
    assert.deepEqual(collector.readSnapshot(), snapshot, 'toggle is projection-local');
    assert.equal(sink.rendered.at(-1)?.descriptor.visible, false);
    const updated = host.applyPresentation(frame({
        op: 'update',
        handle,
        patch: {
            title: null,
            corner: 'bottomRight',
            refreshIntervalMs: null,
            maxFrameTimeSamples: null,
            visible: true,
        },
    }).presentation);
    assert.equal(updated.applied, 1);
    assert.equal(sink.rendered.at(-1)?.descriptor.corner, 'bottomRight');
    host.applyPresentation(frame({ op: 'destroy', handle }).presentation);
    assert.deepEqual(sink.destroyed, [handle]);
});
void test('missing overlay realization does not block scene or other telemetry access', async () => {
    const collector = new AshaLiveTelemetryCollector({ expectedCounters: ['entityCount'] });
    const snapshot = collector.sample({
        authorityTick: 8,
        frameTimeMs: 16,
        counters: { entityCount: 2 },
    });
    let sceneApplied = false;
    const receipt = await applyAshaRuntimeProjectionFrame(frame({
        op: 'create',
        handle: telemetryOverlayHandle(2),
        descriptor: descriptor(),
    }), {
        applyScene: () => { sceneApplied = true; },
    });
    assert.equal(sceneApplied, true);
    assert.equal(receipt.telemetryOverlay.diagnostics[0]?.code, 'unavailableHost');
    assert.deepEqual(collector.readSnapshot(), snapshot);
});
//# sourceMappingURL=telemetry-host.test.js.map
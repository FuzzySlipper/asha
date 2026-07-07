import { test } from 'node:test';
import assert from 'node:assert/strict';
import { cameraHandle } from '@asha/contracts';
import { GENERATED_TUNNEL_FIRE_HIT_READOUT, TINY_GENERATED_TUNNEL_READOUT, buildCombatFeedbackProjection, defaultCombatFeedbackIntent, } from './index.js';
void test('@asha/runtime-session exposes semantic readouts without a bridge backend', () => {
    const envelope = {
        kind: 'runtime_action_intent.v0',
        action: 'primary_fire',
        phase: 'pressed',
        camera: cameraHandle(1),
        tick: 3,
        source: 'programmatic',
        pressed: true,
    };
    const projection = buildCombatFeedbackProjection({
        ...defaultCombatFeedbackIntent(envelope),
        sequenceId: 7,
        combatReadout: GENERATED_TUNNEL_FIRE_HIT_READOUT,
    });
    assert.equal(TINY_GENERATED_TUNNEL_READOUT.status, 'available');
    assert.equal(projection.trace.result, 'hit');
    assert.equal(projection.intent.accepted, true);
});
//# sourceMappingURL=index.test.js.map
import assert from 'node:assert/strict';
import test from 'node:test';
import * as browserEntry from './browser.js';
void test('browser-safe root exports the resolved time and pause composition surface', () => {
    assert.equal(typeof browserEntry.ResolvedPauseContextConsumer, 'function');
    assert.equal(typeof browserEntry.ResolvedTimeControlConsumer, 'function');
    assert.equal(typeof browserEntry.timeControlCommandFromResolvedAction, 'function');
    assert.deepEqual(browserEntry.TIME_CONTROL_INPUT_ACTIONS, {
        pause: 'runtime.time.pause',
        resume: 'runtime.time.resume',
        stepOne: 'runtime.time.step_one',
    });
    const session = {};
    assert.ok(new browserEntry.ResolvedPauseContextConsumer(session));
    assert.ok(new browserEntry.ResolvedTimeControlConsumer(session));
});
//# sourceMappingURL=browser-entry.test.js.map
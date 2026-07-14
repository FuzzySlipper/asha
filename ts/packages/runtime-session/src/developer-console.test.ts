import assert from 'node:assert/strict';
import test from 'node:test';
import type { DeveloperConsoleSnapshot } from '@asha/contracts';
import {
  projectDeveloperConsoleActivity,
  projectDeveloperConsolePullDown,
} from './developer-console.js';

const snapshot: DeveloperConsoleSnapshot = {
  schemaVersion: 1,
  records: [
    {
      sequence: 0,
      severity: 'info',
      category: 'capability',
      source: 'authority',
      message: 'runtime capabilities attached',
      correlation: 'session:1',
      authorityTick: 0,
      session: 'engine:1',
      detail: { code: 'capability_attached', operation: 'initialize_engine', resourceKind: null, resourceId: null, reason: null },
    },
    {
      sequence: 1,
      severity: 'warning',
      category: 'resource',
      source: 'projection',
      message: 'audio clip unavailable; effect omitted',
      correlation: 'shot:7',
      authorityTick: 9,
      session: 'engine:1',
      detail: { code: 'resource_degraded', operation: 'read_projection_frame', resourceKind: 'audio', resourceId: 'audio/missing', reason: 'asset unavailable' },
    },
  ],
  droppedRecordCount: 2,
  firstSequence: 0,
  nextSequence: 2,
  snapshotHash: 'fnv1a64:fixture',
};

void test('game pull-down and Studio activity projections share runtime records but keep UI messages local', () => {
  const localMessages = [{ id: 'studio:save', severity: 'info' as const, message: 'Saved scene locally' }];
  const pullDown = projectDeveloperConsolePullDown(snapshot, localMessages);
  const activity = projectDeveloperConsoleActivity(snapshot, localMessages);

  assert.equal(pullDown.runtime.length, 2);
  assert.equal(activity.runtime.length, 1);
  assert.equal(activity.runtime[0]?.correlation, 'shot:7');
  assert.deepEqual(pullDown.localUi, activity.localUi);
  assert.equal(pullDown.localUi[0]?.channel, 'localUi');
  assert.equal(pullDown.runtime.some((entry) => entry.text.includes('Saved scene locally')), false);
  assert.equal(pullDown.droppedRuntimeRecordCount, 2);
});

import assert from 'node:assert/strict';
import test from 'node:test';
import type { DeveloperConsoleSnapshot } from '@asha/contracts';
import {
  readGamePullDownConsole,
  readStudioActivityStatus,
} from './developer-console-consumer.js';

void test('downstream game and Studio examples use only the public RuntimeSession root', () => {
  const snapshot: DeveloperConsoleSnapshot = {
    schemaVersion: 1,
    records: [{
      sequence: 4,
      severity: 'warning',
      category: 'resource',
      source: 'projection',
      message: 'font unavailable',
      correlation: 'overlay:2',
      authorityTick: 12,
      session: 'engine:9',
      detail: { code: 'resource_degraded', operation: 'read_projection_frame', resourceKind: 'font', resourceId: 'font/hud', reason: 'missing' },
    }],
    droppedRecordCount: 0,
    firstSequence: 4,
    nextSequence: 5,
    snapshotHash: 'fnv1a64:consumer-fixture',
  };
  const session = { readDeveloperConsole: () => snapshot };
  const local = [{ id: 'studio:save', severity: 'info' as const, message: 'Saved locally' }];

  assert.equal(readGamePullDownConsole(session, local).runtime.length, 1);
  assert.equal(readStudioActivityStatus(session, local).runtime.length, 1);
  assert.equal(readStudioActivityStatus(session, local).localUi[0]?.channel, 'localUi');
});

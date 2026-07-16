import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  createDefaultBrowserInputCatalog,
  type RuntimeBridge,
} from './index.js';
import { createMockRuntimeBridge } from './reference.js';

type ProviderFactory = () => RuntimeBridge;

function exerciseInputReplay(createProvider: ProviderFactory) {
  const provider = createProvider();
  provider.initializeEngine({ seed: 7 });
  provider.configureInputSession({
    catalog: createDefaultBrowserInputCatalog(),
    initialContexts: ['gameplay'],
  });
  const resolved = provider.submitRawInput({
    sequence: 0,
    platformKind: 'keyboardKey',
    control: 'KeyW',
    phase: 'pressed',
    value: { kind: 'button', pressed: true },
  });
  assert.equal(resolved.accepted, true);
  assert.ok(resolved.record);

  const replayed = provider.replayResolvedInputAction(resolved.record);
  assert.equal(replayed.accepted, true);
  assert.deepEqual(replayed.action, resolved.action);

  const duplicate = provider.replayResolvedInputAction(resolved.record);
  assert.equal(duplicate.accepted, false);
  assert.equal(duplicate.diagnostics[0]?.code, 'replayAlreadyDelivered');
  return { resolved, replayed, duplicate };
}

function exercisePublicProvider(createProvider: ProviderFactory = createMockRuntimeBridge) {
  const provider = createProvider();
  provider.initializeEngine({ seed: 11 });

  const before = provider.readSceneObjectSnapshot();
  const root = before.objects[0];
  assert.ok(root);

  const accepted = provider.applySceneObjectCommand({
    expectedDocumentHash: before.documentHash,
    command: { kind: 'rename', id: root.id, label: 'Provider-regression root' },
  });
  assert.equal(accepted.accepted, true);
  assert.equal(accepted.rejection, null);

  const after = provider.readSceneObjectSnapshot();
  assert.equal(after.objects[0]?.label, 'Provider-regression root');
  assert.notEqual(after.documentHash, before.documentHash);

  const stale = provider.applySceneObjectCommand({
    expectedDocumentHash: before.documentHash,
    command: { kind: 'select', id: root.id },
  });
  assert.equal(stale.accepted, false);
  assert.equal(stale.rejection?.code, 'stale-scene-object-snapshot');
  assert.deepEqual(provider.readSceneObjectSnapshot(), after);

  const firstReplay = exerciseInputReplay(createProvider);
  const secondReplay = exerciseInputReplay(createProvider);
  assert.deepEqual(secondReplay.resolved.record, firstReplay.resolved.record);
  assert.equal(secondReplay.replayed.replayHash, firstReplay.replayed.replayHash);
  assert.equal(secondReplay.duplicate.replayHash, firstReplay.duplicate.replayHash);
}

void test('public provider accepts and rejects stored mutation and replays deterministically', () => {
  exercisePublicProvider();
});

void test('shape-compatible provider with broken stale-write behavior fails the regression', () => {
  const brokenProvider: ProviderFactory = () => {
    const provider = createMockRuntimeBridge();
    const applySceneObjectCommand = provider.applySceneObjectCommand.bind(provider);
    provider.applySceneObjectCommand = (
      ...parameters: Parameters<RuntimeBridge['applySceneObjectCommand']>
    ) => {
      const result = applySceneObjectCommand(...parameters);
      if (result.accepted) return result;
      return { ...result, accepted: true, rejection: null };
    };
    return provider;
  };

  assert.throws(() => exercisePublicProvider(brokenProvider), /true !== false/u);
});

// Smoke harness tests: a passing mock run carries trustworthy evidence; failures
// are categorized to the exact subsystem (#2395/#2396/#2397/#2398).

import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  createMockRuntimeBridge,
  RuntimeBridgeError,
  type RuntimeBridge,
} from '@asha/runtime-bridge';

import { authorityBootBridge, runSmoke } from './harness.js';
import { formatResult } from './result.js';
import { FIXTURE_WORLD, fixtureRenderFrame, fixtureWorldHash } from './fixtures.js';

function mockBoot() {
  return {
    bridge: createMockRuntimeBridge(),
    mode: 'mock' as const,
    intent: 'reference' as const,
    nativeAvailable: false,
  };
}

test('mock run passes and reports trustworthy evidence', () => {
  const result = runSmoke({ bootBridge: mockBoot });
  assert.equal(result.ok, true);
  assert.equal(result.runtimeMode, 'mock');
  assert.equal(result.smokeMode, 'reference');
  assert.equal(result.outcome, 'mock_reference_passed');
  assert.equal(result.nativeAvailable, false);
  // Capabilities honestly distinguish real (renderer) from mock-backed.
  assert.equal(result.capabilities.renderer, 'ok');
  assert.equal(result.capabilities.worldLoad, 'mock');
  assert.equal(result.capabilities.projection, 'mock');
  // Deterministic fixture evidence.
  assert.equal(result.fixture.id, FIXTURE_WORLD.sceneId);
  assert.equal(result.fixture.worldHash, fixtureWorldHash(FIXTURE_WORLD));
  // Real load → projection → render and edit/save stages all ran.
  assert.deepEqual(
    result.stages.map((s) => s.name),
    ['boot', 'load', 'render', 'edit-save'],
  );
  assert.ok(result.stages.every((s) => s.ok));
  assert.equal(result.render.applied, true);
  assert.ok(result.render.sceneNodes > 0);
  assert.equal(result.failures.length, 0);
});

test('formatResult is deterministic and lists every stage', () => {
  const a = formatResult(runSmoke({ bootBridge: mockBoot }));
  const b = formatResult(runSmoke({ bootBridge: mockBoot }));
  assert.equal(a, b);
  assert.match(a, /asha-smoke: PASS/);
  assert.match(a, /stage render: ok/);
  assert.match(a, /stage edit-save: ok/);
});

/** A bridge that delegates to a real mock but lets one method be overridden. */
function bridgeWith(overrides: Partial<RuntimeBridge>): RuntimeBridge {
  const base = createMockRuntimeBridge();
  return {
    initializeEngine: base.initializeEngine.bind(base),
    stepSimulation: base.stepSimulation.bind(base),
    submitCommands: base.submitCommands.bind(base),
    readRenderDiffs: base.readRenderDiffs.bind(base),
    getBuffer: base.getBuffer.bind(base),
    releaseBuffer: base.releaseBuffer.bind(base),
    loadWorldBundle: base.loadWorldBundle.bind(base),
    saveCurrentWorld: base.saveCurrentWorld.bind(base),
    getCompositionStatus: base.getCompositionStatus.bind(base),
    unloadWorld: base.unloadWorld.bind(base),
    loadReplayFixture: base.loadReplayFixture.bind(base),
    runReplayStep: base.runReplayStep.bind(base),
    ...overrides,
  };
}

test('a failing world load is categorized to the load subsystem, not a blank success', () => {
  const failing = bridgeWith({
    loadWorldBundle: () => ({ loadedWorld: null, fatalCount: 1, totalCount: 1, blocksLoad: true }),
  });
  const result = runSmoke({
    bootBridge: () => ({ bridge: failing, mode: 'mock', intent: 'reference', nativeAvailable: false }),
  });
  assert.equal(result.ok, false);
  assert.equal(result.outcome, 'failed');
  assert.equal(result.capabilities.worldLoad, 'unavailable');
  const loadFailure = result.failures.find((f) => f.category === 'load_failure');
  assert.ok(loadFailure, 'expected a classified load_failure');
  assert.ok(loadFailure!.nextStep.length > 0, 'failure carries an actionable next step');
});

test('a thrown bridge load surfaces a classified failure', () => {
  const throwing = bridgeWith({
    loadWorldBundle: () => {
      throw new RuntimeBridgeError('invalid_input', 'bad bundle');
    },
  });
  const result = runSmoke({
    bootBridge: () => ({ bridge: throwing, mode: 'mock', intent: 'reference', nativeAvailable: false }),
  });
  assert.equal(result.ok, false);
  assert.ok(result.failures.some((f) => f.category === 'load_failure'));
});

// ── Authority-path smoke (#2424) ──────────────────────────────────────────────

/** An authority-capable bridge: a mock that serves real render diffs through the
 *  facade, standing in for a wired native runtime in tests. */
function authorityBridge(): RuntimeBridge {
  return bridgeWith({ readRenderDiffs: () => fixtureRenderFrame() });
}

test('authority run reads diffs through the facade and earns native_authority_passed', () => {
  const result = runSmoke({
    bootBridge: () => ({
      bridge: authorityBridge(),
      mode: 'native',
      intent: 'authority',
      nativeAvailable: true,
    }),
  });
  assert.equal(result.ok, true);
  assert.equal(result.smokeMode, 'authority');
  assert.equal(result.outcome, 'native_authority_passed');
  // Capabilities report real (not mock) once the authority path passes.
  assert.equal(result.capabilities.worldLoad, 'ok');
  assert.equal(result.capabilities.projection, 'ok');
  // The render stage consumed bridge.readRenderDiffs, not the local fixture frame.
  const render = result.stages.find((s) => s.name === 'render');
  assert.ok(render?.detail.includes('bridge.readRenderDiffs'));
  assert.ok(result.render.sceneNodes > 0);
});

test('authority run fails closed (not blank success) when readRenderDiffs is empty', () => {
  // A fail-closed native bridge (post-#2423) whose projection is not wired: the
  // mock returns an empty frame; authority intent must classify, not pass.
  const result = runSmoke({
    bootBridge: () => ({
      bridge: createMockRuntimeBridge(),
      mode: 'native',
      intent: 'authority',
      nativeAvailable: true,
    }),
  });
  assert.equal(result.ok, false);
  assert.equal(result.outcome, 'failed');
  assert.ok(result.failures.some((f) => f.category === 'missing_native_bridge'));
});

test('authority boot fails closed and honest when the native addon is unavailable', (t) => {
  // The real authority boot in offline CI: no native addon → classified failure,
  // never downgraded to a mock pass.
  const boot = authorityBootBridge();
  if (boot.bridge !== null) {
    t.skip('native addon is built in this environment; offline-failure path not exercised');
    return;
  }
  assert.equal(boot.nativeAvailable, false);
  const result = runSmoke({ bootBridge: authorityBootBridge });
  assert.equal(result.ok, false);
  assert.equal(result.smokeMode, 'authority');
  assert.equal(result.outcome, 'failed');
  assert.ok(result.failures.some((f) => f.category === 'missing_native_bridge'));
  assert.equal(result.capabilities.runtimeBridge, 'unavailable');
});

test('real native authority boot fails closed at an unwired op (no mock success)', (t) => {
  // When the native addon IS built, the authority path still must not pass on
  // mock behaviour: post-#2423 the native facade fail-closes unwired ops, so the
  // load stage fails honestly rather than reporting a blank success.
  const boot = authorityBootBridge();
  if (boot.bridge === null) {
    t.skip('native addon not built; honest-failure path covered by the offline test');
    return;
  }
  const result = runSmoke({ bootBridge: authorityBootBridge });
  assert.equal(result.smokeMode, 'authority');
  assert.equal(result.runtimeMode, 'native');
  assert.equal(result.ok, false);
  assert.equal(result.outcome, 'failed');
  assert.ok(result.failures.some((f) => f.category === 'missing_native_bridge'));
});

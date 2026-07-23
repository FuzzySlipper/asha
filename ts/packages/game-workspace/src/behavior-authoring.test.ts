import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  AUTHORED_PREDICATE_STATE_IS,
  AUTHORED_SIGNAL_PREFAB_PART_INTERACTED,
  AUTHORED_VERB_SET_CAPABILITY_ACTIVE,
  AUTHORED_VERB_SET_RELATIVE_TRANSLATION,
  AUTHORED_VERB_TRANSITION_STATE,
} from '@asha/contracts';

import {
  authoredBehavior,
  compileAshaAuthoredBehaviorPackage,
  createAshaAuthoredBehaviorDocument,
} from './index.js';

function doorDraft() {
  const door = authoredBehavior.sceneEntity('demo.door');
  const machine = authoredBehavior.stateMachine(
    'door',
    'demo.door',
    'closed',
    [authoredBehavior.state('open'), authoredBehavior.state('closed')],
    [
      authoredBehavior.transition('close', 'open', 'closed'),
      authoredBehavior.transition('open', 'closed', 'open'),
    ],
  );
  const behavior = authoredBehavior.behavior(
    'switch-opens-door',
    authoredBehavior.prefabPartInteracted(
      authoredBehavior.prefabPart('demo.console', 'switch'),
    ),
    [authoredBehavior.whenState('door', 'closed')],
    [
      authoredBehavior.step('open-now', [
        authoredBehavior.transitionState('door', 'open'),
        authoredBehavior.setRelativeTranslation(door, [0, 3, 0]),
        authoredBehavior.setCapabilityActive(door, 'collision', false),
      ]),
      authoredBehavior.afterTicks('close-later', 'open-now', 120, [
        authoredBehavior.transitionState('door', 'close'),
        authoredBehavior.setRelativeTranslation(door, [0, 0, 0]),
        authoredBehavior.setCapabilityActive(door, 'collision', true),
      ]),
    ],
  );
  return {
    packageId: 'demo.doors',
    stateMachines: [machine],
    behaviors: [behavior],
  } as const;
}

const source = {
  sourceModule: '@demo/gameplay',
  sourcePath: 'src/content/doors.ts',
} as const;

void test('the consumer-shaped door declaration lowers only to Rust semantic identities', () => {
  const first = compileAshaAuthoredBehaviorPackage(doorDraft(), source);
  const second = compileAshaAuthoredBehaviorPackage(doorDraft(), source);

  assert.deepEqual(first, second);
  assert.match(first.provenance.sourceHash, /^fnv1a64:[0-9a-f]{16}$/);
  assert.equal(first.stateMachines[0]?.states[0]?.stateId, 'closed');
  assert.equal(first.behaviors[0]?.signal.signal.semanticId, AUTHORED_SIGNAL_PREFAB_PART_INTERACTED);
  assert.equal(first.behaviors[0]?.conditions[0]?.predicate.semanticId, AUTHORED_PREDICATE_STATE_IS);
  assert.deepEqual(
    first.behaviors[0]?.steps[1]?.operations.map((operation) => operation.verb.semanticId),
    [
      AUTHORED_VERB_TRANSITION_STATE,
      AUTHORED_VERB_SET_RELATIVE_TRANSLATION,
      AUTHORED_VERB_SET_CAPABILITY_ACTIVE,
    ],
  );
  assert.equal(Object.isFrozen(first), true);
  assert.equal(Object.isFrozen(first.stateMachines), true);
  assert.equal(Object.isFrozen(first.stateMachines[0]?.states[0]), true);
  assert.equal(Object.isFrozen(first.behaviors[0]?.signal), true);
  assert.equal(Object.isFrozen(first.behaviors[0]?.steps), true);
  assert.equal(Object.isFrozen(first.behaviors[0]?.steps[1]?.operations[1]?.arguments), true);

  const document = createAshaAuthoredBehaviorDocument('behavior.demo-doors', doorDraft(), source);
  assert.equal(document.kind, 'behaviorPackage');
  assert.equal(document.documentId, 'behavior.demo-doors');
});

void test('published gameplay events need no Engine-specific signal helper', () => {
  const signal = authoredBehavior.publishedEvent('asha.combat', 'entity-defeated');

  assert.deepEqual(signal, {
    signal: {
      semanticId: 'asha.combat.entity-defeated',
      version: 1,
    },
    arguments: [],
  });
  assert.equal(Object.isFrozen(signal), true);
  assert.equal(Object.isFrozen(signal.arguments), true);
});

void test('authored behavior preparation rejects executable and ambient values', () => {
  const executable = doorDraft() as unknown as Record<string, unknown>;
  executable['callback'] = () => 'run at runtime';
  assert.throws(
    () => compileAshaAuthoredBehaviorPackage(
      executable as unknown as ReturnType<typeof doorDraft>,
      source,
    ),
    /data only/,
  );

  const ambient = doorDraft() as unknown as Record<string, unknown>;
  ambient['browserObject'] = new URL('https://example.invalid');
  assert.throws(
    () => compileAshaAuthoredBehaviorPackage(
      ambient as unknown as ReturnType<typeof doorDraft>,
      source,
    ),
    /plain project data/,
  );
});

void test('authored behavior delayed work enforces the generated Engine budget', () => {
  assert.throws(
    () => authoredBehavior.afterTicks('later', 'now', 3_601, [],),
    /delayTicks/,
  );
});

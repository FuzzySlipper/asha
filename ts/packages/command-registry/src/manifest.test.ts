import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  COMMAND_IDS,
  COMMAND_MANIFEST,
  requireKnownCommand,
  validateCommandDefinition,
  validateCommandManifest,
  validateExampleAgainstSchema,
  type DraftStudioCommandDefinition,
} from './index.js';

const REQUIRED_IDS = [
  'session.list_scenarios',
  'session.start',
  'session.load_scenario',
  'inspection.session_status',
  'inspection.world_summary',
  'inspection.editor_state',
  'selection.voxel_from_screen_point',
  'inspection.voxel',
  'preview.voxel_brush',
  'authority.voxel.apply_brush',
  'inspection.last_command_result',
  'render.capture_before_after',
  'export.agent_readout',
] as const;

test('manifest contains the V1 stable command ids in reviewable order', () => {
  assert.deepEqual(COMMAND_IDS, REQUIRED_IDS);
  assert.equal(new Set(COMMAND_IDS).size, COMMAND_IDS.length);
});

test('manifest entries include all required metadata and validate cleanly', () => {
  assert.deepEqual(validateCommandManifest(COMMAND_MANIFEST), []);
  for (const command of COMMAND_MANIFEST) {
    assert.equal(command.version, 1);
    assert.ok(command.label.length > 0);
    assert.ok(command.summary.length > 0);
    assert.ok(command.menuPath.length > 0);
    assert.ok(command.commandPalette.keywords.length > 0);
    assert.ok(command.artifacts.length > 0);
    assert.equal(command.owningLane, 'ts-command-registry');
    assert.equal(command.owningPackage, '@asha/command-registry');
    assert.equal(command.compatibility.commandRegistry, 'command-registry.v0');
  }
});

test('non-hidden agent exposure requires GUI mirror metadata', () => {
  for (const command of COMMAND_MANIFEST) {
    if (command.agentExposure.kind !== 'hidden') {
      assert.equal(command.guiMirror.required, true, command.id);
      assert.ok(command.guiMirror.menuPath.length > 0, command.id);
      assert.ok(command.guiMirror.commandPaletteVisible || command.guiMirror.panel !== undefined, command.id);
    }
  }
});

test('command schemas are fail-closed and contain no freeform object payloads', () => {
  for (const command of COMMAND_MANIFEST) {
    const issues = validateCommandDefinition(command).filter((issue) => issue.message.includes('allowExtraFields'));
    assert.deepEqual(issues, [], command.id);
  }
});

test('typed examples match declared input and output schemas', () => {
  for (const command of COMMAND_MANIFEST) {
    assert.deepEqual(
      validateExampleAgainstSchema(command.id, 'typedInputExample', command.typedInputExample, command.inputSchema.shape),
      [],
      `${command.id} input example`,
    );
    assert.deepEqual(
      validateExampleAgainstSchema(command.id, 'typedOutputExample', command.typedOutputExample, command.outputSchema.shape),
      [],
      `${command.id} output example`,
    );
  }
});

test('mutating, writing, and capture commands are not advertised as read-only to agents', () => {
  const nonReadOnlyByImpact = COMMAND_MANIFEST.filter(
    (command) => command.operationClass !== 'read_only' || command.stateImpact.authority === 'mutate' || command.stateImpact.editor === 'mutate' || command.stateImpact.render === 'capture' || command.stateImpact.workspace === 'write',
  );
  assert.ok(nonReadOnlyByImpact.length > 0);
  for (const command of nonReadOnlyByImpact) {
    assert.notEqual(command.agentExposure.kind, 'read_only', command.id);
  }
  assert.equal(requireKnownCommand('session.start', COMMAND_MANIFEST).agentExposure.kind, 'workspace_io');
  assert.equal(requireKnownCommand('session.load_scenario', COMMAND_MANIFEST).agentExposure.kind, 'workspace_io');
});

test('selection command uses screen-point camera request, not a caller-supplied pick ray', () => {
  const select = requireKnownCommand('selection.voxel_from_screen_point', COMMAND_MANIFEST);
  assert.deepEqual(select.inputContractRefs, [{ package: '@asha/contracts', exportName: 'ScreenPointToPickRayRequest' }]);
  assert.deepEqual(select.outputContractRefs, [{ package: '@asha/contracts', exportName: 'VoxelSelectionSnapshot' }]);
  const inputSchema = JSON.stringify(select.inputSchema);
  assert.ok(inputSchema.includes('ScreenPointToPickRayRequest'));
  assert.equal(inputSchema.includes('"exportName":"PickRay"'), false);
  assert.deepEqual(select.runtimeRequirements, [{ kind: 'runtime_bridge_operation', operation: 'select_voxel' }, { kind: 'editor_store' }]);
});

test('validation rejects read-only exposure for non-read-only or mutating impacts', () => {
  const start = requireKnownCommand('session.start', COMMAND_MANIFEST);
  const broken: DraftStudioCommandDefinition = { ...start, agentExposure: { kind: 'read_only' } };
  const issues = validateCommandDefinition(broken);
  assert.ok(issues.some((issue) => issue.field === 'agentExposure' && issue.message.includes('read_only exposure')));
});

test('validation rejects output schemas that do not describe typed outputs', () => {
  const world = requireKnownCommand('inspection.world_summary', COMMAND_MANIFEST);
  const broken = validateExampleAgainstSchema(
    world.id,
    'typedOutputExample',
    world.typedOutputExample,
    { kind: 'object', allowExtraFields: false, fields: [{ name: 'artifactId', required: true, shape: { kind: 'scalar', scalar: 'artifact_ref' }, summary: 'Wrong artifact-only output.' }] },
  );
  assert.deepEqual(broken, [{ commandId: 'inspection.world_summary', field: 'typedOutputExample', message: 'typedOutputExample does not match its declared schema' }]);
});

test('validation rejects missing metadata and open object schemas', () => {
  const broken: DraftStudioCommandDefinition = {
    id: 'inspection.world_summary',
    version: 1,
    inputSchema: {
      name: 'BrokenInput',
      version: 1,
      shape: {
        kind: 'object',
        allowExtraFields: true as false,
        fields: [],
      },
    },
  };
  const issues = validateCommandDefinition(broken);
  assert.ok(issues.some((issue) => issue.field === 'label'));
  assert.ok(issues.some((issue) => issue.field === 'operationClass'));
  assert.ok(issues.some((issue) => issue.field === 'inputSchema.shape'));
});

test('unknown command ids are rejected rather than treated as dynamic method names', () => {
  assert.throws(() => requireKnownCommand('authority.voxel.delete_everything', COMMAND_MANIFEST), /Unknown ASHA studio command id/);
});

test('authority command uses typed voxel contracts and guarded retry/idempotency posture', () => {
  const apply = requireKnownCommand('authority.voxel.apply_brush', COMMAND_MANIFEST);
  assert.equal(apply.operationClass, 'authority_mutating');
  assert.deepEqual(apply.inputContractRefs, [{ package: '@asha/contracts', exportName: 'VoxelCommand' }]);
  assert.equal(apply.agentExposure.kind, 'authority_mutating');
  assert.equal(apply.retry, 'safe_to_retry_if_state_hash_unchanged');
  assert.equal(apply.idempotency.kind, 'conditional');
});

test('manifest golden stays stable and reviewable', () => {
  const goldenPath = join(process.cwd(), 'src', 'manifest.golden.json');
  const expected = readFileSync(goldenPath, 'utf8');
  const actual = `${JSON.stringify(COMMAND_MANIFEST, null, 2)}\n`;
  assert.equal(actual, expected);
});

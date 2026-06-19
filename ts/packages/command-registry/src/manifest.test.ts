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

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

import type { PrefabRegistry } from '@asha/contracts';
import {
  decodeAndValidateAshaPrefabRegistrySourceDocument,
  serializeAshaPrefabRegistrySource,
  validateAshaPrefabRegistrySourceDocument,
} from './index.js';

const fixtureContext = {
  assetIds: [],
  entityDefinitionIds: ['fixture.root'],
} as const;

function fixture(name: string): unknown {
  const url = new URL(`../../../../harness/fixtures/project-bundle/${name}`, import.meta.url);
  return JSON.parse(readFileSync(url, 'utf8')) as unknown;
}

void test('unknown prefab source becomes a typed canonical registry only after validation', () => {
  const source: unknown = fixture('prefab-registry.valid.json');
  const result = decodeAndValidateAshaPrefabRegistrySourceDocument(source, fixtureContext);
  assert.equal(result.ok, true);
  if (!result.ok) {
    throw new Error('committed valid prefab source should decode');
  }

  const registry: PrefabRegistry = result.registry;
  assert.equal(registry.definitions[0]?.id, 7);
  assert.equal(result.authority, 'typescript_early_diagnostic_only');
  const canonicalJson = serializeAshaPrefabRegistrySource(registry);
  assert.deepEqual(JSON.parse(canonicalJson), source);
  const fixedPoint = decodeAndValidateAshaPrefabRegistrySourceDocument(
    JSON.parse(canonicalJson) as unknown,
    fixtureContext,
  );
  assert.equal(fixedPoint.ok, true);
  if (!fixedPoint.ok) {
    throw new Error('canonical source should decode again');
  }
  assert.equal(serializeAshaPrefabRegistrySource(fixedPoint.registry), canonicalJson);
});

void test('unsupported schema and invalid variant roles fail atomically', () => {
  const unsupported = decodeAndValidateAshaPrefabRegistrySourceDocument(
    fixture('prefab-registry.invalid-schema.json'),
    fixtureContext,
  );
  assert.equal(unsupported.ok, false);
  assert.equal(unsupported.registry, null);
  assert.equal(
    unsupported.diagnostics.some((item) => item.code === 'unsupportedRegistrySchema'),
    true,
  );

  const missingRole = decodeAndValidateAshaPrefabRegistrySourceDocument(
    fixture('prefab-registry.invalid-missing-role-variant.json'),
    fixtureContext,
  );
  assert.equal(missingRole.ok, false);
  assert.equal(missingRole.registry, null);
  const codes = new Set(missingRole.diagnostics.map((item) => item.code));
  assert.equal(codes.has('unknownRemovedRole'), true);
  assert.equal(codes.has('invalidOverrideTarget'), true);
});

void test('alias removal golden retains part-level deletion safety diagnostics', () => {
  const result = decodeAndValidateAshaPrefabRegistrySourceDocument(
    fixture('prefab-registry.invalid-alias-removal.json'),
    fixtureContext,
  );
  assert.equal(result.ok, false);
  assert.equal(result.registry, null);
  const codes = new Set(result.diagnostics.map((item) => item.code));
  assert.equal(codes.has('unsafePartRemoval'), true);
  assert.equal(codes.has('deletedRoleReferenced'), true);
});

void test('malformed nested source fields fail during bounded decoding', () => {
  const result = decodeAndValidateAshaPrefabRegistrySourceDocument(
    fixture('prefab-registry.invalid-source-shape.json'),
    fixtureContext,
  );
  assert.deepEqual(result, {
    ok: false,
    registry: null,
    diagnostics: [{
      code: 'invalidSourceDocument',
      path: 'definitions[0].parts[0].transform.scale',
      message: 'expected exactly 3 numbers',
    }],
    authority: 'typescript_early_diagnostic_only',
  });
});

void test('typed validator and unknown decoder share complete semantic policy', () => {
  const validResult = decodeAndValidateAshaPrefabRegistrySourceDocument(
    fixture('prefab-registry.valid.json'),
    fixtureContext,
  );
  assert.equal(validResult.ok, true);
  if (!validResult.ok) {
    throw new Error('valid source should decode');
  }
  assert.deepEqual(
    validateAshaPrefabRegistrySourceDocument(validResult.registry, fixtureContext),
    [],
  );
});

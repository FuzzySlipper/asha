import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  buildAshaAuthoringPersistenceContract,
  resolveAshaAuthoringWriteTarget,
} from './index.js';
import { validManifest } from './test-support.js';

void test('authoring persistence contract exposes bounded public write scopes', () => {
  const manifest = validManifest();
  const contract = buildAshaAuthoringPersistenceContract(manifest);

  assert.equal(contract.contractVersion, 'authoring-persistence.v0');
  assert.deepEqual(
    contract.writeScopes.map((scope) => scope.operationKind),
    [
      'authoring.scene.save_source',
      'authoring.catalog.save_source',
      'authoring.asset.save_source',
      'authoring.policy.save_source',
    ],
  );
  assert.deepEqual(contract.writeScopes.find((scope) => scope.operationKind === 'authoring.scene.save_source')?.allowedRoots, ['scenes']);
  assert.deepEqual(contract.writeScopes.find((scope) => scope.operationKind === 'authoring.catalog.save_source')?.allowedRoots, ['packages/game-catalogs']);
  assert.deepEqual(contract.writeScopes.find((scope) => scope.operationKind === 'authoring.asset.save_source')?.allowedRoots, ['assets']);
  assert.deepEqual(contract.writeScopes.find((scope) => scope.operationKind === 'authoring.policy.save_source')?.allowedRoots, []);
  assert.ok(contract.forbiddenRoots.includes('harness/out'));
  assert.ok(contract.nonClaims.includes('not_repo_crawler'));
  assert.equal(contract.diagnostics.some((diagnostic) => diagnostic.code === 'unsupported_operation'), true);
});

void test('authoring write target resolver accepts normalized scene catalog and asset paths', () => {
  const manifest = validManifest();

  const scene = resolveAshaAuthoringWriteTarget(manifest, {
    operationKind: 'authoring.scene.save_source',
    relativePath: './scenes/demo.scene.json',
  });
  assert.equal(scene.ok, true);
  if (!scene.ok) {
    throw new Error('scene authoring path should resolve');
  }
  assert.equal(scene.normalizedPath, 'scenes/demo.scene.json');
  assert.equal(scene.allowedRoot, 'scenes');
  assert.equal(scene.requiredValidator, 'validateAshaProofSceneSourceDocument');

  const catalog = resolveAshaAuthoringWriteTarget(manifest, {
    operationKind: 'authoring.catalog.save_source',
    relativePath: 'packages/game-catalogs/catalog.json',
  });
  assert.equal(catalog.ok, true);
  if (!catalog.ok) {
    throw new Error('catalog authoring path should resolve');
  }
  assert.equal(catalog.format, 'asset-catalog-json.v1');
  assert.equal(catalog.requiredValidator, 'validateAshaGameAssetCatalog');

  const asset = resolveAshaAuthoringWriteTarget(manifest, {
    operationKind: 'authoring.asset.save_source',
    relativePath: 'assets/meshes/demo.mesh.json',
  });
  assert.equal(asset.ok, true);
  if (!asset.ok) {
    throw new Error('asset authoring path should resolve');
  }
  assert.equal(asset.format, 'inline-asset-json.v1');
});

void test('authoring write target resolver fails closed on disallowed paths and hatches', () => {
  const manifest = validManifest();

  const generated = resolveAshaAuthoringWriteTarget(manifest, {
    operationKind: 'authoring.scene.save_source',
    relativePath: 'harness/out/proof.scene.json',
  });
  assert.equal(generated.ok, false);
  assert.equal(
    !generated.ok && generated.diagnostics.some((diagnostic) => diagnostic.code === 'forbidden_generated_path'),
    true,
  );

  const traversal = resolveAshaAuthoringWriteTarget(manifest, {
    operationKind: 'authoring.catalog.save_source',
    relativePath: '../asha-engine/private/catalog.json',
  });
  assert.equal(traversal.ok, false);
  assert.equal(
    !traversal.ok && traversal.diagnostics.some((diagnostic) => diagnostic.code === 'disallowed_path'),
    true,
  );

  const wrongExtension = resolveAshaAuthoringWriteTarget(manifest, {
    operationKind: 'authoring.asset.save_source',
    relativePath: 'assets/meshes/demo.txt',
  });
  assert.equal(wrongExtension.ok, false);
  assert.equal(
    !wrongExtension.ok && wrongExtension.diagnostics.some((diagnostic) => diagnostic.code === 'invalid_extension'),
    true,
  );

  const privateTransport = resolveAshaAuthoringWriteTarget(manifest, {
    operationKind: 'authoring.asset.save_source',
    relativePath: 'assets/@asha/native-bridge/native-bridge.node',
  });
  assert.equal(privateTransport.ok, false);
  assert.equal(
    !privateTransport.ok && privateTransport.diagnostics.some((diagnostic) => diagnostic.code === 'private_transport_hint'),
    true,
  );
});

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { buildAshaGamePublishAssetManifest, resolveAshaGameAssetForDev, validateAshaGameAssetCatalog, } from './index.js';
import { validCatalog, validManifest } from './test-support.js';
void test('asset catalog validates and resolves a dev resource by catalog id', () => {
    const catalog = validCatalog();
    const existingFiles = new Set(catalog.entries.map((entry) => entry.source));
    const sourceHashes = new Map(catalog.entries.map((entry) => [entry.source, entry.importMetadata.sourceHash]));
    const validation = validateAshaGameAssetCatalog(catalog, validManifest(), (path) => existingFiles.has(path), { sourceHash: (path) => sourceHashes.get(path) ?? null });
    assert.equal(validation.ok, true);
    const resolution = resolveAshaGameAssetForDev(catalog, 'mesh.demo-cube', sourceHashes.get('assets/meshes/demo-cube.mesh.json'));
    assert.deepEqual(resolution, {
        assetId: 'mesh.demo-cube',
        sourcePath: 'assets/meshes/demo-cube.mesh.json',
        sourceHash: 'sha256:mesh',
        devCacheKey: 'dev-cache/static_mesh/mesh.demo-cube/sha256-mesh',
        generatedArtifactVersion: 'asset-import.v1',
        importStatus: 'clean',
        publishOutputKey: 'meshes/demo-cube.mesh.json',
    });
    const publishManifest = buildAshaGamePublishAssetManifest(catalog);
    assert.deepEqual(publishManifest.dependencyOrder, [
        'texture.demo-checker',
        'material.demo-copper',
        'mesh.demo-cube',
    ]);
    assert.deepEqual(publishManifest.entries.map((entry) => entry.assetId), [
        'mesh.demo-cube',
        'material.demo-copper',
        'texture.demo-checker',
    ]);
});
void test('asset catalog reports stale import metadata in validation and dev resolution', () => {
    const catalog = validCatalog();
    const validation = validateAshaGameAssetCatalog(catalog, validManifest(), () => true, { sourceHash: (path) => (path === 'assets/meshes/demo-cube.mesh.json' ? 'sha256:changed' : catalog.entries.find((entry) => entry.source === path)?.importMetadata?.sourceHash ?? null) });
    assert.equal(validation.ok, false);
    if (validation.ok) {
        throw new Error('stale import metadata should fail validation');
    }
    assert.equal(validation.diagnostics.some((diagnostic) => diagnostic.code === 'stale_import_metadata'), true);
    const resolution = resolveAshaGameAssetForDev(catalog, 'mesh.demo-cube', 'sha256:changed');
    assert.equal(resolution?.sourcePath, 'assets/meshes/demo-cube.mesh.json');
    assert.equal(resolution?.sourceHash, 'sha256:changed');
    assert.equal(resolution?.importStatus, 'stale');
});
void test('asset catalog fails closed for missing file, duplicate id, forbidden path, unsupported kind, and wrong kind profile', () => {
    const catalog = {
        schemaVersion: 1,
        entries: [
            { ...validCatalog().entries[0] },
            { ...validCatalog().entries[0], source: '../asha-engine/private.bin', kind: 'shader' },
            { ...validCatalog().entries[1], importProfile: 'inline-static-mesh.v0', publish: { include: true, outputKey: 'meshes/not-a-material.mesh.json' } },
        ],
    };
    const validation = validateAshaGameAssetCatalog(catalog, validManifest(), () => false);
    assert.equal(validation.ok, false);
    if (validation.ok) {
        throw new Error('invalid catalog should fail validation');
    }
    assert.equal(validation.diagnostics.some((diagnostic) => diagnostic.code === 'missing_asset_file'), true);
    assert.equal(validation.diagnostics.some((diagnostic) => diagnostic.code === 'duplicate_asset_id'), true);
    assert.equal(validation.diagnostics.some((diagnostic) => diagnostic.code === 'forbidden_asset_path'), true);
    assert.equal(validation.diagnostics.some((diagnostic) => diagnostic.code === 'unsupported_asset_kind'), true);
    assert.equal(validation.diagnostics.some((diagnostic) => diagnostic.code === 'invalid_asset_entry' && diagnostic.path.endsWith('.importProfile')), true);
    assert.equal(validation.diagnostics.some((diagnostic) => diagnostic.code === 'invalid_asset_entry' && diagnostic.path.endsWith('.publish.outputKey')), true);
});
void test('asset catalog dependency graph fails closed for missing dependency and cycles', () => {
    const missing = {
        ...validCatalog(),
        entries: [
            { ...validCatalog().entries[0], dependencies: ['material.missing', 'material.missing'] },
            ...validCatalog().entries.slice(1),
        ],
    };
    const missingValidation = validateAshaGameAssetCatalog(missing, validManifest(), (path) => path.startsWith('assets/'));
    assert.equal(missingValidation.ok, false);
    if (missingValidation.ok) {
        throw new Error('missing dependency should fail validation');
    }
    assert.equal(missingValidation.diagnostics.some((diagnostic) => diagnostic.code === 'missing_asset_dependency'), true);
    assert.equal(missingValidation.diagnostics.some((diagnostic) => diagnostic.code === 'duplicate_asset_dependency'), true);
    const cyclic = {
        ...validCatalog(),
        entries: validCatalog().entries.map((entry) => entry.id === 'texture.demo-checker' ? { ...entry, dependencies: ['mesh.demo-cube'] } : entry),
    };
    const cyclicValidation = validateAshaGameAssetCatalog(cyclic, validManifest(), (path) => path.startsWith('assets/'));
    assert.equal(cyclicValidation.ok, false);
    if (cyclicValidation.ok) {
        throw new Error('dependency cycle should fail validation');
    }
    assert.equal(cyclicValidation.diagnostics.some((diagnostic) => diagnostic.code === 'asset_dependency_cycle'), true);
});
//# sourceMappingURL=assets.test.js.map
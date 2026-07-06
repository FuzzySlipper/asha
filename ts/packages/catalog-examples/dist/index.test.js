import { test } from 'node:test';
import assert from 'node:assert/strict';
import { CATALOG_EXAMPLE_AUTHORITY_BOUNDARY, GENERATED_TUNNEL_ASSET_CATALOG_EXAMPLE, GENERATED_TUNNEL_CATALOG_EXAMPLE_BUNDLE, buildInvalidGeneratedTunnelGameplayPresetExample, readGeneratedTunnelCatalogExampleReadout, validateGeneratedTunnelCatalogExample, validateInvalidGeneratedTunnelGameplayPresetExample, } from './index.js';
void test('valid generated tunnel catalog example validates through catalog-core', () => {
    const report = validateGeneratedTunnelCatalogExample();
    const readout = readGeneratedTunnelCatalogExampleReadout();
    assert.equal(report.kind, 'fps_gameplay_preset_validation.v0');
    assert.equal(report.valid, true);
    assert.deepEqual(report.diagnostics, []);
    assert.equal(report.rejectedHash, null);
    assert.ok(report.readout);
    assert.equal(report.readout.preset.presetId, 'asha.generated_tunnel.default_fps.v0');
    assert.equal(readout.kind, 'fps_gameplay_preset_catalog_readout.v0');
    assert.equal(readout.catalog, GENERATED_TUNNEL_CATALOG_EXAMPLE_BUNDLE.gameplayCatalog);
    assert.equal(readout.hashes.defaultPresetHash, report.readout.hashes.presetHash);
});
void test('asset catalog example exercises generated contract shape without runtime authority', () => {
    const [materialEntry, meshEntry] = GENERATED_TUNNEL_ASSET_CATALOG_EXAMPLE.entries;
    assert.equal(GENERATED_TUNNEL_CATALOG_EXAMPLE_BUNDLE.kind, 'catalog_example_bundle.v0');
    assert.equal(GENERATED_TUNNEL_CATALOG_EXAMPLE_BUNDLE.generatedAssetCatalog, GENERATED_TUNNEL_ASSET_CATALOG_EXAMPLE);
    assert.ok(materialEntry);
    assert.ok(meshEntry);
    assert.equal(materialEntry.kind, 'material');
    assert.ok(materialEntry.material);
    assert.equal(materialEntry.material.render.uvStrategy, 'flat');
    assert.equal(materialEntry.material.collision.structuralClass, 'structural');
    assert.equal(meshEntry.kind, 'mesh');
    assert.equal(meshEntry.material, null);
    assert.deepEqual(meshEntry.dependencies, [
        {
            id: 'material/generated-tunnel/wall-grey',
            version: { req: 'exact', value: 1 },
            hash: 'fnv1a64:8cfd3f4c579e0d41',
        },
    ]);
    assert.ok(CATALOG_EXAMPLE_AUTHORITY_BOUNDARY.doesNotOwn.includes('runtime_authority'));
    assert.ok(CATALOG_EXAMPLE_AUTHORITY_BOUNDARY.doesNotOwn.includes('state_mutation'));
});
void test('invalid generated tunnel fixture builder is rejected by catalog-core', () => {
    const invalidPreset = buildInvalidGeneratedTunnelGameplayPresetExample();
    const report = validateInvalidGeneratedTunnelGameplayPresetExample();
    const diagnostics = report.diagnostics.map((diagnostic) => `${diagnostic.code}:${diagnostic.path}`);
    assert.equal(invalidPreset.presetId, 'asha.generated_tunnel.default_fps.v0');
    assert.equal(report.valid, false);
    assert.equal(report.readout, null);
    assert.ok(diagnostics.includes('arbitraryPayloadRejected:preset.payload'));
    assert.ok(diagnostics.includes('invalidNumberRange:playerController.moveSpeedUnitsPerSecond'));
    assert.ok(diagnostics.includes('invalidNumberRange:playerController.collisionHalfExtents.1'));
    assert.ok(diagnostics.includes('invalidIntegerRange:weapon.cooldownTicks'));
    assert.ok(diagnostics.includes('invalidIntegerRange:encounter.enemyCount'));
    assert.ok(diagnostics.includes('duplicateReference:encounter.spawnMarkerIds.1'));
});
//# sourceMappingURL=index.test.js.map
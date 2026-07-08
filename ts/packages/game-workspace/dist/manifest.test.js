import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ASHA_GAME_WORKSPACE_COMPATIBILITY, parseAshaGameManifestToml, validateAshaConsumerCompatibility, } from './index.js';
import { fixture } from './test-support.js';
void test('validates the golden asha.game.toml manifest', () => {
    const result = parseAshaGameManifestToml(fixture('asha.game.toml'));
    assert.equal(result.ok, true);
    if (!result.ok) {
        throw new Error('golden manifest should validate');
    }
    assert.equal(result.manifest.asha.engineVersion, '0.1.0');
    assert.equal(result.manifest.runtime.devtoolsEndpoint, 'ws://127.0.0.1:7391');
    assert.equal(result.manifest.runtime.backendMode, 'reference');
    assert.equal(result.manifest.runtime.backendProfile, 'reference');
    assert.deepEqual(result.manifest.runtime.backendProofRefs, []);
    assert.deepEqual(result.manifest.studio.allowedSourceWrites, ['scenes', 'assets', 'packages/game-catalogs']);
    assert.deepEqual(result.manifest.devResourceProfile.localRoots, ['assets', 'packages/game-catalogs']);
    assert.equal(result.manifest.devResourceProfile.cacheDir, 'dist/dev-cache');
    assert.equal(result.manifest.devResourceProfile.resolutionPolicy, 'prefer-source');
    assert.equal(result.manifest.publishResourceProfile.outputDir, 'dist/resources');
    assert.equal(result.manifest.publishResourceProfile.archiveDir, 'dist/archive');
    assert.equal(result.manifest.publishResourceProfile.resolutionPolicy, 'locked');
});
void test('fails closed when required workspace roots are missing', () => {
    const result = parseAshaGameManifestToml(fixture('invalid-missing-roots.toml'));
    assert.equal(result.ok, false);
    if (result.ok) {
        throw new Error('missing roots should fail validation');
    }
    assert.equal(result.diagnostics.some((diagnostic) => diagnostic.code === 'missing_root' && diagnostic.path === 'workspace.scene_roots'), true);
    assert.equal(result.diagnostics.some((diagnostic) => diagnostic.code === 'invalid_write_scope'), true);
});
void test('fails closed on disallowed Studio source-write roots', () => {
    const result = parseAshaGameManifestToml(fixture('invalid-source-write.toml'));
    assert.equal(result.ok, false);
    if (result.ok) {
        throw new Error('private write scope should fail validation');
    }
    assert.equal(result.diagnostics.some((diagnostic) => diagnostic.code === 'invalid_path'), true);
    assert.equal(result.diagnostics.some((diagnostic) => diagnostic.code === 'invalid_write_scope'), true);
});
void test('classifies bad versions and unsupported devtools endpoints', () => {
    const manifest = fixture('asha.game.toml')
        .replace('engine_version = "0.1.0"', 'engine_version = "latest"')
        .replace('devtools_endpoint = "ws://127.0.0.1:7391"', 'devtools_endpoint = "https://example.com/devtools"');
    const result = parseAshaGameManifestToml(manifest);
    assert.equal(result.ok, false);
    if (result.ok) {
        throw new Error('bad version and endpoint should fail validation');
    }
    assert.equal(result.diagnostics.some((diagnostic) => diagnostic.code === 'bad_version'), true);
    assert.equal(result.diagnostics.some((diagnostic) => diagnostic.code === 'unsupported_endpoint'), true);
});
void test('manifest accepts selected native backend mode with public proof refs', () => {
    const manifest = fixture('asha.game.toml')
        .replace('backend_mode = "reference"', 'backend_mode = "native"')
        .replace('backend_profile = "reference"', 'backend_profile = "native.napi.launcher.v1"')
        .replace('backend_proof_refs = []', 'backend_proof_refs = ["proof:dev-authority-smoke"]');
    const result = parseAshaGameManifestToml(manifest);
    assert.equal(result.ok, true);
    if (!result.ok) {
        throw new Error('native backend manifest should validate');
    }
    assert.equal(result.manifest.runtime.backendMode, 'native');
    assert.deepEqual(result.manifest.runtime.backendProofRefs, ['proof:dev-authority-smoke']);
});
void test('manifest fails closed on unsupported or unproved backend modes', () => {
    const wasm = parseAshaGameManifestToml(fixture('asha.game.toml').replace('backend_mode = "reference"', 'backend_mode = "wasm"'));
    assert.equal(wasm.ok, false);
    assert.equal(!wasm.ok && wasm.diagnostics.some((diagnostic) => diagnostic.code === 'unsupported_backend_mode' && diagnostic.path === 'runtime.backend_mode'), true);
    const nativeMissingProof = parseAshaGameManifestToml(fixture('asha.game.toml')
        .replace('backend_mode = "reference"', 'backend_mode = "native"')
        .replace('backend_profile = "reference"', 'backend_profile = "native.napi.launcher.v1"'));
    assert.equal(nativeMissingProof.ok, false);
    assert.equal(!nativeMissingProof.ok && nativeMissingProof.diagnostics.some((diagnostic) => diagnostic.code === 'missing_backend_ref' && diagnostic.path === 'runtime.backend_proof_refs'), true);
});
void test('manifest rejects private transport hints in backend selection', () => {
    const manifest = fixture('asha.game.toml')
        .replace('wasm_or_native_entry = "dist/runtime/index.js"', 'wasm_or_native_entry = "@asha/native-bridge/native-bridge.node"')
        .replace('backend_profile = "reference"', 'backend_profile = "@asha/native-bridge"');
    const result = parseAshaGameManifestToml(manifest);
    assert.equal(result.ok, false);
    assert.equal(!result.ok && result.diagnostics.some((diagnostic) => diagnostic.code === 'private_transport_hint' && diagnostic.path === 'runtime.wasm_or_native_entry'), true);
    assert.equal(!result.ok && result.diagnostics.some((diagnostic) => diagnostic.code === 'private_transport_hint' && diagnostic.path === 'runtime.backend_profile'), true);
});
void test('fails closed when the publish resource profile is missing', () => {
    const result = parseAshaGameManifestToml(fixture('invalid-missing-publish-profile.toml'));
    assert.equal(result.ok, false);
    if (result.ok) {
        throw new Error('missing publish resource profile should fail validation');
    }
    assert.equal(result.diagnostics.some((diagnostic) => diagnostic.code === 'missing_required_field' && diagnostic.path === 'publish_resource_profile'), true);
});
void test('fails closed when publish resource paths point into dev-local roots', () => {
    const result = parseAshaGameManifestToml(fixture('invalid-dev-root-leakage.toml'));
    assert.equal(result.ok, false);
    if (result.ok) {
        throw new Error('publish resource paths inside dev roots should fail validation');
    }
    assert.equal(result.diagnostics.some((diagnostic) => diagnostic.code === 'invalid_resource_profile' && diagnostic.path === 'publish_resource_profile.output_dir'), true);
    assert.equal(result.diagnostics.some((diagnostic) => diagnostic.code === 'invalid_resource_profile' && diagnostic.path === 'publish_resource_profile.archive_dir'), true);
});
void test('validates compatible ASHA consumer metadata against the manifest', () => {
    const result = parseAshaGameManifestToml(fixture('asha.game.toml'));
    assert.equal(result.ok, true);
    if (!result.ok) {
        throw new Error('golden manifest should validate');
    }
    const compatibility = validateAshaConsumerCompatibility(result.manifest, ASHA_GAME_WORKSPACE_COMPATIBILITY);
    assert.equal(compatibility.ok, true);
    if (!compatibility.ok) {
        throw new Error('golden compatibility metadata should validate');
    }
    assert.equal(compatibility.metadata.runtimeBridge.compatibilityVersion, 'runtime-bridge.v0');
});
void test('fails closed on incompatible consumer metadata versions', () => {
    const result = parseAshaGameManifestToml(fixture('asha.game.toml'));
    assert.equal(result.ok, true);
    if (!result.ok) {
        throw new Error('golden manifest should validate');
    }
    const compatibility = validateAshaConsumerCompatibility(result.manifest, {
        ...ASHA_GAME_WORKSPACE_COMPATIBILITY,
        contracts: { compatibilityVersion: 'contracts.v0', packageVersion: '9.9.9' },
    });
    assert.equal(compatibility.ok, false);
    if (compatibility.ok) {
        throw new Error('incompatible metadata should fail validation');
    }
    assert.equal(compatibility.diagnostics.some((diagnostic) => diagnostic.code === 'incompatible_version' && diagnostic.path === 'asha.contracts_version'), true);
});
void test('fails closed when compatibility metadata is missing', () => {
    const result = parseAshaGameManifestToml(fixture('asha.game.toml'));
    assert.equal(result.ok, true);
    if (!result.ok) {
        throw new Error('golden manifest should validate');
    }
    const compatibility = validateAshaConsumerCompatibility(result.manifest, {
        contracts: ASHA_GAME_WORKSPACE_COMPATIBILITY.contracts,
    });
    assert.equal(compatibility.ok, false);
    if (compatibility.ok) {
        throw new Error('missing metadata should fail validation');
    }
    assert.equal(compatibility.diagnostics.some((diagnostic) => diagnostic.code === 'missing_metadata' && diagnostic.path === 'runtimeBridge'), true);
});
//# sourceMappingURL=manifest.test.js.map
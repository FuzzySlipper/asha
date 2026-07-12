export { validateAshaConsumerCompatibility } from './manifest-compatibility.js';
export { ASHA_GAME_WORKSPACE_COMPATIBILITY } from './manifest-types.js';
import { manifestDiagnostic as diag, } from './manifest-types.js';
import { parseTomlSubset } from './manifest-toml.js';
const REQUIRED_SECTIONS = ['asha', 'workspace', 'runtime', 'studio', 'publish', 'dev_resource_profile', 'publish_resource_profile'];
const VERSION_PATTERN = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/;
const LOCAL_WEBSOCKET_ENDPOINT_PATTERN = /^wss?:\/\/(?:127\.0\.0\.1|localhost):\d+(?:\/[A-Za-z0-9._~:/?#[\]@!$&'()*+,;=-]*)?$/;
export function parseAshaGameManifestToml(toml) {
    const parsed = parseTomlSubset(toml);
    if (!parsed.ok) {
        return { ok: false, diagnostics: parsed.diagnostics };
    }
    return decodeAndValidateManifest(parsed.document);
}
function decodeAndValidateManifest(document) {
    const diagnostics = [];
    for (const section of REQUIRED_SECTIONS) {
        if (document[section] === undefined) {
            diagnostics.push(diag('missing_required_field', section, `missing [${section}] section`));
        }
    }
    const manifest = {
        asha: {
            engineVersion: getString(document, 'asha', 'engine_version', diagnostics),
            contractsVersion: getString(document, 'asha', 'contracts_version', diagnostics),
            runtimeBridgeVersion: getString(document, 'asha', 'runtime_bridge_version', diagnostics),
            devtoolsProtocolVersion: getString(document, 'asha', 'devtools_protocol_version', diagnostics),
            publishArtifactFormatVersion: getString(document, 'asha', 'publish_artifact_format_version', diagnostics),
            engineSource: getString(document, 'asha', 'engine_source', diagnostics),
        },
        workspace: {
            sceneRoots: getStringArray(document, 'workspace', 'scene_roots', diagnostics),
            prefabRoots: getStringArray(document, 'workspace', 'prefab_roots', diagnostics),
            assetRoots: getStringArray(document, 'workspace', 'asset_roots', diagnostics),
            replayRoots: getStringArray(document, 'workspace', 'replay_roots', diagnostics),
            catalogPackages: getStringArray(document, 'workspace', 'catalog_packages', diagnostics),
            policyPackages: getStringArray(document, 'workspace', 'policy_packages', diagnostics),
        },
        runtime: {
            devCommand: getString(document, 'runtime', 'dev_command', diagnostics),
            devtoolsEndpoint: getString(document, 'runtime', 'devtools_endpoint', diagnostics),
            wasmOrNativeEntry: getString(document, 'runtime', 'wasm_or_native_entry', diagnostics),
            backendMode: getBackendMode(document, diagnostics),
            backendProfile: getString(document, 'runtime', 'backend_profile', diagnostics),
            backendProofRefs: getStringArray(document, 'runtime', 'backend_proof_refs', diagnostics),
        },
        studio: {
            workspaceMode: getBoolean(document, 'studio', 'workspace_mode', diagnostics),
            attachEnabled: getBoolean(document, 'studio', 'attach_enabled', diagnostics),
            allowedSourceWrites: getStringArray(document, 'studio', 'allowed_source_writes', diagnostics),
        },
        publish: {
            command: getString(document, 'publish', 'command', diagnostics),
            artifactDir: getString(document, 'publish', 'artifact_dir', diagnostics),
            verifyCommand: getString(document, 'publish', 'verify_command', diagnostics),
        },
        devResourceProfile: {
            localRoots: getStringArray(document, 'dev_resource_profile', 'local_roots', diagnostics),
            cacheDir: getString(document, 'dev_resource_profile', 'cache_dir', diagnostics),
            resolutionPolicy: getString(document, 'dev_resource_profile', 'resolution_policy', diagnostics),
        },
        publishResourceProfile: {
            outputDir: getString(document, 'publish_resource_profile', 'output_dir', diagnostics),
            archiveDir: getString(document, 'publish_resource_profile', 'archive_dir', diagnostics),
            resolutionPolicy: getString(document, 'publish_resource_profile', 'resolution_policy', diagnostics),
        },
    };
    validateManifest(manifest, diagnostics);
    return diagnostics.length === 0 ? { ok: true, manifest, diagnostics: [] } : { ok: false, diagnostics };
}
function validateManifest(manifest, diagnostics) {
    validateVersion(manifest.asha.engineVersion, 'asha.engine_version', diagnostics);
    validateVersion(manifest.asha.contractsVersion, 'asha.contracts_version', diagnostics);
    validateVersion(manifest.asha.runtimeBridgeVersion, 'asha.runtime_bridge_version', diagnostics);
    validateNonEmptyRoots(manifest.workspace.sceneRoots, 'workspace.scene_roots', diagnostics);
    validateNonEmptyRoots(manifest.workspace.prefabRoots, 'workspace.prefab_roots', diagnostics);
    validateNonEmptyRoots(manifest.workspace.assetRoots, 'workspace.asset_roots', diagnostics);
    validateNonEmptyRoots(manifest.workspace.replayRoots, 'workspace.replay_roots', diagnostics);
    validateEngineSource(manifest.asha.engineSource, 'asha.engine_source', diagnostics);
    validatePath(manifest.runtime.wasmOrNativeEntry, 'runtime.wasm_or_native_entry', diagnostics);
    validateBackendMode(manifest, diagnostics);
    validatePath(manifest.publish.artifactDir, 'publish.artifact_dir', diagnostics);
    validateResourceProfiles(manifest, diagnostics);
    if (!LOCAL_WEBSOCKET_ENDPOINT_PATTERN.test(manifest.runtime.devtoolsEndpoint)) {
        diagnostics.push(diag('unsupported_endpoint', 'runtime.devtools_endpoint', 'devtools endpoint must be a local ws:// or wss:// URL with an explicit port'));
    }
    const writeRoots = [
        ...manifest.workspace.sceneRoots,
        ...manifest.workspace.prefabRoots,
        ...manifest.workspace.assetRoots,
        ...manifest.workspace.catalogPackages,
        ...manifest.workspace.policyPackages,
    ];
    for (const writeScope of manifest.studio.allowedSourceWrites) {
        validatePath(writeScope, 'studio.allowed_source_writes', diagnostics);
        if (!writeRoots.some((root) => isSameOrChildPath(writeScope, root))) {
            diagnostics.push(diag('invalid_write_scope', 'studio.allowed_source_writes', `write scope "${writeScope}" is not within a declared workspace root`));
        }
    }
}
function validateResourceProfiles(manifest, diagnostics) {
    validateNonEmptyRoots(manifest.devResourceProfile.localRoots, 'dev_resource_profile.local_roots', diagnostics);
    validatePath(manifest.devResourceProfile.cacheDir, 'dev_resource_profile.cache_dir', diagnostics);
    validatePath(manifest.publishResourceProfile.outputDir, 'publish_resource_profile.output_dir', diagnostics);
    validatePath(manifest.publishResourceProfile.archiveDir, 'publish_resource_profile.archive_dir', diagnostics);
    const workspaceRoots = [
        ...manifest.workspace.sceneRoots,
        ...manifest.workspace.prefabRoots,
        ...manifest.workspace.assetRoots,
        ...manifest.workspace.replayRoots,
        ...manifest.workspace.catalogPackages,
        ...manifest.workspace.policyPackages,
    ];
    for (const root of manifest.devResourceProfile.localRoots) {
        if (!workspaceRoots.some((workspaceRoot) => isSameOrChildPath(root, workspaceRoot))) {
            diagnostics.push(diag('invalid_resource_profile', 'dev_resource_profile.local_roots', `dev resource root "${root}" is not within a declared workspace root`));
        }
    }
    for (const [path, value] of [
        ['publish_resource_profile.output_dir', manifest.publishResourceProfile.outputDir],
        ['publish_resource_profile.archive_dir', manifest.publishResourceProfile.archiveDir],
    ]) {
        if (workspaceRoots.some((root) => isSameOrChildPath(value, root))) {
            diagnostics.push(diag('invalid_resource_profile', path, `publish resource path "${value}" must not be inside a dev-local workspace root`));
        }
    }
    if (manifest.devResourceProfile.resolutionPolicy !== 'prefer-source') {
        diagnostics.push(diag('invalid_resource_profile', 'dev_resource_profile.resolution_policy', 'dev resolution_policy must be "prefer-source"'));
    }
    if (manifest.publishResourceProfile.resolutionPolicy !== 'locked') {
        diagnostics.push(diag('invalid_resource_profile', 'publish_resource_profile.resolution_policy', 'publish resolution_policy must be "locked"'));
    }
}
function validateVersion(version, path, diagnostics) {
    if (!VERSION_PATTERN.test(version)) {
        diagnostics.push(diag('bad_version', path, `version "${version}" must be semver-like x.y.z`));
    }
}
function validateNonEmptyRoots(roots, path, diagnostics) {
    if (roots.length === 0) {
        diagnostics.push(diag('missing_root', path, 'at least one root is required'));
    }
    for (const root of roots) {
        validatePath(root, path, diagnostics);
    }
}
function validatePath(pathValue, path, diagnostics) {
    if (pathValue.length === 0 || pathValue.startsWith('/') || pathValue.split('/').includes('..')) {
        diagnostics.push(diag('invalid_path', path, `path "${pathValue}" must be non-empty, relative, and remain inside the game workspace`));
    }
}
function validateEngineSource(engineSource, path, diagnostics) {
    if (engineSource.length === 0 || engineSource.includes('engine-rs/crates') || engineSource.includes('/src/')) {
        diagnostics.push(diag('invalid_path', path, 'engine source must be a package/version or repo root path, not an ASHA internal source path'));
    }
}
function containsPrivateTransportHint(value) {
    return [
        '@asha/native-bridge',
        '@asha/wasm-bridge',
        '@asha/wasm-replay-bridge',
        'native-bridge.node',
        'wasm.memory',
        'engine-rs/',
        '/src/',
    ].some((hint) => value.includes(hint));
}
function isSameOrChildPath(candidate, root) {
    return candidate === root || candidate.startsWith(`${root}/`);
}
function validateBackendMode(manifest, diagnostics) {
    const { backendMode, backendProfile, backendProofRefs, wasmOrNativeEntry } = manifest.runtime;
    if (containsPrivateTransportHint(wasmOrNativeEntry)) {
        diagnostics.push(diag('private_transport_hint', 'runtime.wasm_or_native_entry', 'runtime entry must point at a public launcher/facade entry, not a raw private transport'));
    }
    if (containsPrivateTransportHint(backendProfile)) {
        diagnostics.push(diag('private_transport_hint', 'runtime.backend_profile', 'backend profile must not name private transports or ASHA internals'));
    }
    if (backendMode === 'reference') {
        if (backendProfile !== 'reference') {
            diagnostics.push(diag('unsupported_backend_mode', 'runtime.backend_profile', 'reference backend mode must use backend_profile = "reference"'));
        }
        return;
    }
    if (backendMode === 'native') {
        if (backendProfile.length === 0 || backendProfile === 'reference') {
            diagnostics.push(diag('missing_backend_ref', 'runtime.backend_profile', 'native backend mode requires a selected backend profile id'));
        }
        if (backendProofRefs.length === 0) {
            diagnostics.push(diag('missing_backend_ref', 'runtime.backend_proof_refs', 'native backend mode requires at least one public proof/evidence ref'));
        }
        return;
    }
    diagnostics.push(diag('unsupported_backend_mode', 'runtime.backend_mode', 'wasm backend mode is declared but deferred until a public WASM runtime facade is approved'));
}
function getString(document, section, key, diagnostics) {
    const value = document[section]?.[key];
    if (typeof value !== 'string') {
        diagnostics.push(diag('missing_required_field', `${section}.${key}`, 'expected a string field'));
        return '';
    }
    return value;
}
function getBoolean(document, section, key, diagnostics) {
    const value = document[section]?.[key];
    if (typeof value !== 'boolean') {
        diagnostics.push(diag('missing_required_field', `${section}.${key}`, 'expected a boolean field'));
        return false;
    }
    return value;
}
function getStringArray(document, section, key, diagnostics) {
    const value = document[section]?.[key];
    if (!Array.isArray(value) || !value.every((entry) => typeof entry === 'string')) {
        diagnostics.push(diag('missing_required_field', `${section}.${key}`, 'expected a string array field'));
        return [];
    }
    return value;
}
function getBackendMode(document, diagnostics) {
    const value = document['runtime']?.['backend_mode'];
    if (value === 'reference' || value === 'native' || value === 'wasm') {
        return value;
    }
    diagnostics.push(diag('unsupported_backend_mode', 'runtime.backend_mode', 'backend_mode must be one of reference, native, or wasm'));
    return 'reference';
}
//# sourceMappingURL=manifest.js.map
export function buildAshaAuthoringPersistenceContract(manifest) {
    const writeScopes = authoringWriteScopes(manifest);
    return {
        contractVersion: 'authoring-persistence.v0',
        writeScopes,
        forbiddenRoots: ['harness/out', 'node_modules', '.git', '../asha-engine', '../asha-studio'],
        diagnostics: writeScopes.flatMap((scope) => scope.operationKind === 'authoring.policy.save_source'
            ? [authoringDiag('unsupported_operation', scope.operationKind, 'policy authoring is reserved until a policy schema contract exists')]
            : []),
        nonClaims: [
            'not_repo_crawler',
            'not_private_asset_database',
            'not_runtime_authority',
            'not_generated_artifact_source',
        ],
    };
}
export function resolveAshaAuthoringWriteTarget(manifest, request) {
    const diagnostics = [];
    const normalizedPath = normalizeAuthoringPath(request.relativePath, diagnostics);
    const scope = authoringWriteScopes(manifest).find((candidate) => candidate.operationKind === request.operationKind);
    if (scope === undefined) {
        diagnostics.push(authoringDiag('unsupported_operation', 'operationKind', `unsupported authoring operation "${request.operationKind}"`));
        return { ok: false, diagnostics };
    }
    if (scope.operationKind === 'authoring.policy.save_source') {
        diagnostics.push(authoringDiag('unsupported_operation', request.operationKind, 'policy authoring is reserved until a policy schema contract exists'));
    }
    if (normalizedPath !== null) {
        if (isGeneratedOrPrivateAuthoringPath(normalizedPath)) {
            diagnostics.push(authoringDiag('forbidden_generated_path', 'relativePath', `authoring save cannot target generated/private path "${normalizedPath}"`));
        }
        if (containsPrivateTransportHint(normalizedPath)) {
            diagnostics.push(authoringDiag('private_transport_hint', 'relativePath', `authoring save cannot target private transport path "${normalizedPath}"`));
        }
        const allowedRoot = scope.allowedRoots.find((root) => isSameOrChildPath(normalizedPath, root));
        if (allowedRoot === undefined) {
            diagnostics.push(authoringDiag('disallowed_path', 'relativePath', `path "${normalizedPath}" is outside allowed roots for ${request.operationKind}`));
        }
        validateAuthoringExtension(scope, normalizedPath, diagnostics);
        if (diagnostics.length === 0 && allowedRoot !== undefined) {
            return {
                ok: true,
                operationKind: request.operationKind,
                normalizedPath,
                allowedRoot,
                format: scope.format,
                requiredValidator: scope.requiredValidator,
                diagnostics: [],
            };
        }
    }
    return { ok: false, diagnostics };
}
function authoringWriteScopes(manifest) {
    const allowed = new Set(manifest.studio.allowedSourceWrites);
    const allowedRoots = (roots) => roots.filter((root) => [...allowed].some((writeRoot) => isSameOrChildPath(writeRoot, root) || isSameOrChildPath(root, writeRoot)));
    return [
        {
            operationKind: 'authoring.scene.save_source',
            allowedRoots: allowedRoots(manifest.workspace.sceneRoots),
            format: 'proof-scene-json.v1',
            requiredValidator: 'validateAshaProofSceneSourceDocument',
        },
        {
            operationKind: 'authoring.prefab.save_source',
            allowedRoots: allowedRoots(manifest.workspace.prefabRoots),
            format: 'prefab-registry-json.v1',
            requiredValidator: 'validateAshaPrefabRegistrySourceDocument',
        },
        {
            operationKind: 'authoring.catalog.save_source',
            allowedRoots: allowedRoots(manifest.workspace.catalogPackages),
            format: 'asset-catalog-json.v1',
            requiredValidator: 'validateAshaGameAssetCatalog',
        },
        {
            operationKind: 'authoring.asset.save_source',
            allowedRoots: allowedRoots(manifest.workspace.assetRoots),
            format: 'inline-asset-json.v1',
            requiredValidator: 'validateAshaCatalogAssetPayload',
        },
        {
            operationKind: 'authoring.policy.save_source',
            allowedRoots: allowedRoots(manifest.workspace.policyPackages),
            format: 'policy-json.deferred',
            requiredValidator: 'deferred-policy-schema-contract',
        },
    ];
}
function normalizeAuthoringPath(value, diagnostics) {
    const replaced = value.replace(/\\/g, '/');
    const parts = replaced.split('/');
    if (value.length === 0 || replaced.startsWith('/') || parts.includes('..')) {
        diagnostics.push(authoringDiag('disallowed_path', 'relativePath', `path "${value}" must be non-empty, relative, and remain inside the game workspace`));
        return null;
    }
    const normalized = [];
    for (const part of parts) {
        if (part.length === 0 || part === '.')
            continue;
        normalized.push(part);
    }
    if (normalized.length === 0) {
        diagnostics.push(authoringDiag('disallowed_path', 'relativePath', `path "${value}" must name a file`));
        return null;
    }
    return normalized.join('/');
}
function isGeneratedOrPrivateAuthoringPath(value) {
    return value === '.git'
        || value.startsWith('.git/')
        || value === 'harness/out'
        || value.startsWith('harness/out/')
        || value === 'node_modules'
        || value.startsWith('node_modules/')
        || value.startsWith('../asha-engine')
        || value.startsWith('../asha-studio');
}
function validateAuthoringExtension(scope, normalizedPath, diagnostics) {
    if (scope.operationKind === 'authoring.scene.save_source' && !normalizedPath.endsWith('.scene.json')) {
        diagnostics.push(authoringDiag('invalid_extension', 'relativePath', 'scene authoring saves must target *.scene.json'));
    }
    if (scope.operationKind === 'authoring.prefab.save_source') {
        const registryPathAllowed = scope.allowedRoots.some((root) => normalizedPath === `${root}/registry.json`);
        if (!registryPathAllowed) {
            diagnostics.push(authoringDiag('invalid_extension', 'relativePath', 'prefab authoring saves must target registry.json in a prefab root'));
        }
    }
    if (scope.operationKind === 'authoring.catalog.save_source') {
        const catalogPathAllowed = scope.allowedRoots.some((root) => normalizedPath === `${root}/catalog.json`);
        if (!catalogPathAllowed) {
            diagnostics.push(authoringDiag('invalid_extension', 'relativePath', 'catalog authoring saves must target catalog.json in a catalog package root'));
        }
    }
    if (scope.operationKind === 'authoring.asset.save_source'
        && !(normalizedPath.endsWith('.mesh.json')
            || normalizedPath.endsWith('.material.json')
            || normalizedPath.endsWith('.texture.json'))) {
        diagnostics.push(authoringDiag('invalid_extension', 'relativePath', 'asset authoring saves must target *.mesh.json, *.material.json, or *.texture.json'));
    }
}
function isSameOrChildPath(candidate, root) {
    return candidate === root || candidate.startsWith(`${root}/`);
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
function authoringDiag(code, path, message) {
    return { code, path, message };
}
//# sourceMappingURL=authoring.js.map
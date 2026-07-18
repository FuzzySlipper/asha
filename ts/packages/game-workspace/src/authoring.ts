import type { AshaGameManifest } from './manifest.js';

export type AshaAuthoringOperationKind =
  | 'authoring.scene.save_source'
  | 'authoring.prefab.save_source'
  | 'authoring.catalog.save_source'
  | 'authoring.asset.save_source'
  | 'authoring.policy.save_source';

export type AshaAuthoringSourceFormat =
  | 'proof-scene-json.v1'
  | 'prefab-registry-json.v1'
  | 'asset-catalog-json.v1'
  | 'inline-asset-json.v1'
  | 'policy-json.deferred';

export type AshaAuthoringDiagnosticCode =
  | 'unsupported_operation'
  | 'disallowed_path'
  | 'invalid_extension'
  | 'forbidden_generated_path'
  | 'private_transport_hint'
  | 'stale_file_hash'
  | 'invalid_schema';

export interface AshaAuthoringDiagnostic {
  readonly code: AshaAuthoringDiagnosticCode;
  readonly path: string;
  readonly message: string;
}

export interface AshaAuthoringWriteScope {
  readonly operationKind: AshaAuthoringOperationKind;
  readonly allowedRoots: readonly string[];
  readonly format: AshaAuthoringSourceFormat;
  readonly requiredValidator: string;
}

export interface AshaAuthoringPersistenceContract {
  readonly contractVersion: 'authoring-persistence.v0';
  readonly writeScopes: readonly AshaAuthoringWriteScope[];
  readonly forbiddenRoots: readonly string[];
  readonly diagnostics: readonly AshaAuthoringDiagnostic[];
  readonly nonClaims: readonly string[];
}

export interface AshaAuthoringSaveRequest {
  readonly operationKind: AshaAuthoringOperationKind;
  readonly relativePath: string;
  readonly expectedPreviousHash: string | null;
  readonly payloadText: string;
}

export interface AshaAuthoringSaveReadback {
  readonly operationKind: AshaAuthoringOperationKind;
  readonly normalizedPath: string;
  readonly allowedRoot: string;
  readonly previousFileHash: string | null;
  readonly nextFileHash: string;
  readonly semanticDiffHash: string;
  readonly validationDiagnosticsHash: string;
  readonly dependentReadbackHashes: readonly string[];
}

export type AshaAuthoringSaveResult =
  | {
      readonly ok: true;
      readonly readback: AshaAuthoringSaveReadback;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly diagnostics: readonly AshaAuthoringDiagnostic[];
    };

export type AshaAuthoringWriteTargetResolution =
  | {
      readonly ok: true;
      readonly operationKind: AshaAuthoringOperationKind;
      readonly normalizedPath: string;
      readonly allowedRoot: string;
      readonly format: AshaAuthoringSourceFormat;
      readonly requiredValidator: string;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly diagnostics: readonly AshaAuthoringDiagnostic[];
    };

export function buildAshaAuthoringPersistenceContract(manifest: AshaGameManifest): AshaAuthoringPersistenceContract {
  const writeScopes = authoringWriteScopes(manifest);
  return {
    contractVersion: 'authoring-persistence.v0',
    writeScopes,
    forbiddenRoots: ['harness/out', 'node_modules', '.git', '../asha-engine', '../asha-studio'],
    diagnostics: writeScopes.flatMap((scope) =>
      scope.operationKind === 'authoring.policy.save_source'
        ? [authoringDiag('unsupported_operation', scope.operationKind, 'policy authoring is reserved until a policy schema contract exists')]
        : [],
    ),
    nonClaims: [
      'not_repo_crawler',
      'not_private_asset_database',
      'not_runtime_authority',
      'not_generated_artifact_source',
    ],
  };
}

export function resolveAshaAuthoringWriteTarget(
  manifest: AshaGameManifest,
  request: Pick<AshaAuthoringSaveRequest, 'operationKind' | 'relativePath'>,
): AshaAuthoringWriteTargetResolution {
  const diagnostics: AshaAuthoringDiagnostic[] = [];
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

function authoringWriteScopes(manifest: AshaGameManifest): readonly AshaAuthoringWriteScope[] {
  const allowed = new Set(manifest.studio.allowedSourceWrites);
  const allowedRoots = (roots: readonly string[]) => roots.filter((root) =>
    [...allowed].some((writeRoot) => isSameOrChildPath(writeRoot, root) || isSameOrChildPath(root, writeRoot)),
  );
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

function normalizeAuthoringPath(value: string, diagnostics: AshaAuthoringDiagnostic[]): string | null {
  const replaced = value.replace(/\\/g, '/');
  const parts = replaced.split('/');
  if (value.length === 0 || replaced.startsWith('/') || parts.includes('..')) {
    diagnostics.push(authoringDiag('disallowed_path', 'relativePath', `path "${value}" must be non-empty, relative, and remain inside the game workspace`));
    return null;
  }
  const normalized: string[] = [];
  for (const part of parts) {
    if (part.length === 0 || part === '.') continue;
    normalized.push(part);
  }
  if (normalized.length === 0) {
    diagnostics.push(authoringDiag('disallowed_path', 'relativePath', `path "${value}" must name a file`));
    return null;
  }
  return normalized.join('/');
}

function isGeneratedOrPrivateAuthoringPath(value: string): boolean {
  return value === '.git'
    || value.startsWith('.git/')
    || value === 'harness/out'
    || value.startsWith('harness/out/')
    || value === 'node_modules'
    || value.startsWith('node_modules/')
    || value.startsWith('../asha-engine')
    || value.startsWith('../asha-studio');
}

function validateAuthoringExtension(
  scope: AshaAuthoringWriteScope,
  normalizedPath: string,
  diagnostics: AshaAuthoringDiagnostic[],
): void {
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
  if (
    scope.operationKind === 'authoring.asset.save_source'
    && !(
      normalizedPath.endsWith('.mesh.json')
      || normalizedPath.endsWith('.material.json')
      || normalizedPath.endsWith('.texture.json')
      || normalizedPath.endsWith('.avxl.json')
    )
  ) {
    diagnostics.push(authoringDiag(
      'invalid_extension',
      'relativePath',
      'asset authoring saves must target *.mesh.json, *.material.json, *.texture.json, or *.avxl.json',
    ));
  }
}

function isSameOrChildPath(candidate: string, root: string): boolean {
  return candidate === root || candidate.startsWith(`${root}/`);
}

function containsPrivateTransportHint(value: string): boolean {
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

function authoringDiag(code: AshaAuthoringDiagnosticCode, path: string, message: string): AshaAuthoringDiagnostic {
  return { code, path, message };
}

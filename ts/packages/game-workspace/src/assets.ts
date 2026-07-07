import type { AshaGameManifest } from './manifest.js';

export type AshaGameAssetKind = 'static_mesh' | 'material' | 'texture' | 'scene';

export interface AshaGameAssetCatalogEntry {
  readonly id: string;
  readonly kind: AshaGameAssetKind;
  readonly source: string;
  readonly importProfile: string | null;
  readonly importMetadata?: {
    readonly sourceHash: string;
    readonly cacheKey: string;
    readonly generatedArtifactVersion: string;
  };
  readonly dependencies?: readonly string[];
  readonly publish: {
    readonly include: boolean;
    readonly outputKey: string;
  };
  readonly diagnostics: {
    readonly owner: string;
    readonly notes: readonly string[];
  };
}

export interface AshaGameAssetCatalog {
  readonly schemaVersion: 1;
  readonly entries: readonly AshaGameAssetCatalogEntry[];
}

export type AshaGameAssetCatalogDiagnosticCode =
  | 'duplicate_asset_id'
  | 'missing_asset_file'
  | 'forbidden_asset_path'
  | 'unsupported_asset_kind'
  | 'missing_asset_dependency'
  | 'duplicate_asset_dependency'
  | 'asset_dependency_cycle'
  | 'stale_import_metadata'
  | 'invalid_asset_entry';

export interface AshaGameAssetCatalogDiagnostic {
  readonly code: AshaGameAssetCatalogDiagnosticCode;
  readonly path: string;
  readonly message: string;
}

export type AshaGameAssetCatalogValidation =
  | {
      readonly ok: true;
      readonly catalog: AshaGameAssetCatalog;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly diagnostics: readonly AshaGameAssetCatalogDiagnostic[];
    };

export interface AshaGameAssetDevResolution {
  readonly assetId: string;
  readonly sourcePath: string;
  readonly sourceHash: string | null;
  readonly devCacheKey: string;
  readonly generatedArtifactVersion: string | null;
  readonly importStatus: 'clean' | 'stale' | 'missing_metadata' | 'unknown';
  readonly publishOutputKey: string;
}

export interface AshaGameAssetCatalogValidationOptions {
  readonly sourceHash?: (path: string) => string | null;
}

export interface AshaGamePublishAssetManifest {
  readonly schemaVersion: 1;
  readonly dependencyOrder: readonly string[];
  readonly entries: readonly {
    readonly assetId: string;
    readonly kind: AshaGameAssetKind;
    readonly sourcePath: string;
    readonly outputKey: string;
  }[];
}

export function validateAshaGameAssetCatalog(
  catalog: AshaGameAssetCatalog,
  manifest: AshaGameManifest,
  fileExists: (path: string) => boolean,
  options: AshaGameAssetCatalogValidationOptions = {},
): AshaGameAssetCatalogValidation {
  const diagnostics: AshaGameAssetCatalogDiagnostic[] = [];
  const seen = new Set<string>();
  for (const [index, entry] of catalog.entries.entries()) {
    const path = `entries[${index}]`;
    if (entry.id.length === 0 || entry.source.length === 0 || entry.publish.outputKey.length === 0) {
      diagnostics.push(assetDiag('invalid_asset_entry', path, 'asset id, source, and publish output key are required'));
    }
    if (seen.has(entry.id)) {
      diagnostics.push(assetDiag('duplicate_asset_id', `${path}.id`, `duplicate asset id "${entry.id}"`));
    }
    seen.add(entry.id);
    if (!isSupportedAssetKind(entry.kind)) {
      diagnostics.push(assetDiag('unsupported_asset_kind', `${path}.kind`, `unsupported asset kind "${entry.kind}"`));
    } else {
      validateKindSpecificAssetEntry(entry, path, diagnostics);
    }
    if (!manifest.workspace.assetRoots.some((root) => isSameOrChildPath(entry.source, root))) {
      diagnostics.push(assetDiag('forbidden_asset_path', `${path}.source`, `asset source "${entry.source}" is outside manifest asset roots`));
    } else if (!fileExists(entry.source)) {
      diagnostics.push(assetDiag('missing_asset_file', `${path}.source`, `asset source does not exist: ${entry.source}`));
    }
    validateImportMetadata(entry, path, options, diagnostics);
  }
  validateAssetDependencyGraph(catalog, diagnostics);

  return diagnostics.length === 0 ? { ok: true, catalog, diagnostics: [] } : { ok: false, diagnostics };
}

export function resolveAshaGameAssetForDev(
  catalog: AshaGameAssetCatalog,
  assetId: string,
  sourceHash?: string | null,
): AshaGameAssetDevResolution | null {
  const entry = catalog.entries.find((candidate) => candidate.id === assetId);
  if (entry === undefined) {
    return null;
  }
  const observedSourceHash = sourceHash ?? entry.importMetadata?.sourceHash ?? null;
  const metadata = entry.importMetadata;
  const importStatus =
    metadata === undefined
      ? 'missing_metadata'
      : sourceHash === undefined || sourceHash === null
        ? 'unknown'
        : sourceHash === metadata.sourceHash
          ? 'clean'
          : 'stale';
  return {
    assetId: entry.id,
    sourcePath: entry.source,
    sourceHash: observedSourceHash,
    devCacheKey: metadata?.cacheKey ?? `dev-cache/${entry.kind}/${entry.id}`,
    generatedArtifactVersion: metadata?.generatedArtifactVersion ?? null,
    importStatus,
    publishOutputKey: entry.publish.outputKey,
  };
}

export function buildAshaGamePublishAssetManifest(catalog: AshaGameAssetCatalog): AshaGamePublishAssetManifest {
  const dependencyOrder = orderAssetDependencies(catalog).filter((assetId) => {
    const entry = catalog.entries.find((candidate) => candidate.id === assetId);
    return entry?.publish.include === true;
  });
  return {
    schemaVersion: 1,
    dependencyOrder,
    entries: catalog.entries
      .filter((entry) => entry.publish.include)
      .map((entry) => ({
        assetId: entry.id,
        kind: entry.kind,
        sourcePath: entry.source,
        outputKey: entry.publish.outputKey,
      })),
  };
}

function validateAssetDependencyGraph(
  catalog: AshaGameAssetCatalog,
  diagnostics: AshaGameAssetCatalogDiagnostic[],
): void {
  const ids = new Set(catalog.entries.map((entry) => entry.id));
  for (const [index, entry] of catalog.entries.entries()) {
    const seen = new Set<string>();
    for (const dependency of entry.dependencies ?? []) {
      if (seen.has(dependency)) {
        diagnostics.push(assetDiag('duplicate_asset_dependency', `entries[${index}].dependencies`, `asset "${entry.id}" repeats dependency "${dependency}"`));
      }
      seen.add(dependency);
      if (!ids.has(dependency)) {
        diagnostics.push(assetDiag('missing_asset_dependency', `entries[${index}].dependencies`, `asset "${entry.id}" depends on missing asset "${dependency}"`));
      }
    }
  }

  const visiting = new Set<string>();
  const visited = new Set<string>();
  const byId = new Map(catalog.entries.map((entry) => [entry.id, entry]));
  function visit(assetId: string, trail: readonly string[]): void {
    if (visited.has(assetId)) return;
    if (visiting.has(assetId)) {
      diagnostics.push(assetDiag('asset_dependency_cycle', 'entries.dependencies', `asset dependency cycle: ${[...trail, assetId].join(' -> ')}`));
      return;
    }
    visiting.add(assetId);
    const entry = byId.get(assetId);
    for (const dependency of entry?.dependencies ?? []) {
      if (byId.has(dependency)) visit(dependency, [...trail, assetId]);
    }
    visiting.delete(assetId);
    visited.add(assetId);
  }
  for (const entry of catalog.entries) visit(entry.id, []);
}

function orderAssetDependencies(catalog: AshaGameAssetCatalog): readonly string[] {
  const byId = new Map(catalog.entries.map((entry) => [entry.id, entry]));
  const visited = new Set<string>();
  const ordered: string[] = [];
  function visit(assetId: string): void {
    if (visited.has(assetId)) return;
    visited.add(assetId);
    const entry = byId.get(assetId);
    for (const dependency of entry?.dependencies ?? []) {
      if (byId.has(dependency)) visit(dependency);
    }
    ordered.push(assetId);
  }
  for (const entry of catalog.entries) visit(entry.id);
  return ordered;
}

function isSameOrChildPath(candidate: string, root: string): boolean {
  return candidate === root || candidate.startsWith(`${root}/`);
}

function isSupportedAssetKind(kind: string): kind is AshaGameAssetKind {
  return kind === 'static_mesh' || kind === 'material' || kind === 'texture' || kind === 'scene';
}

function validateKindSpecificAssetEntry(
  entry: AshaGameAssetCatalogEntry,
  path: string,
  diagnostics: AshaGameAssetCatalogDiagnostic[],
): void {
  const expected = {
    static_mesh: { importProfile: 'inline-static-mesh.v0', outputPrefix: 'meshes/', outputSuffix: '.mesh.json' },
    material: { importProfile: 'inline-material.v0', outputPrefix: 'materials/', outputSuffix: '.material.json' },
    texture: { importProfile: 'inline-texture.v0', outputPrefix: 'textures/', outputSuffix: '.texture.json' },
    scene: { importProfile: 'flat-scene.v0', outputPrefix: 'scenes/', outputSuffix: '.scene.json' },
  }[entry.kind];

  if (entry.importProfile !== expected.importProfile) {
    diagnostics.push(assetDiag('invalid_asset_entry', `${path}.importProfile`, `${entry.kind} assets require importProfile "${expected.importProfile}"`));
  }
  if (!entry.publish.outputKey.startsWith(expected.outputPrefix) || !entry.publish.outputKey.endsWith(expected.outputSuffix)) {
    diagnostics.push(assetDiag('invalid_asset_entry', `${path}.publish.outputKey`, `${entry.kind} publish output must match ${expected.outputPrefix}*${expected.outputSuffix}`));
  }
}

function validateImportMetadata(
  entry: AshaGameAssetCatalogEntry,
  path: string,
  options: AshaGameAssetCatalogValidationOptions,
  diagnostics: AshaGameAssetCatalogDiagnostic[],
): void {
  const metadata = entry.importMetadata;
  if (metadata === undefined) return;
  if (metadata.sourceHash.length === 0 || metadata.cacheKey.length === 0 || metadata.generatedArtifactVersion.length === 0) {
    diagnostics.push(assetDiag('invalid_asset_entry', `${path}.importMetadata`, 'sourceHash, cacheKey, and generatedArtifactVersion are required when import metadata is present'));
    return;
  }
  if (options.sourceHash !== undefined) {
    const observed = options.sourceHash(entry.source);
    if (observed !== null && observed !== metadata.sourceHash) {
      diagnostics.push(assetDiag('stale_import_metadata', `${path}.importMetadata.sourceHash`, `asset "${entry.id}" import metadata hash is stale`));
    }
  }
}

function assetDiag(code: AshaGameAssetCatalogDiagnosticCode, path: string, message: string): AshaGameAssetCatalogDiagnostic {
  return { code, path, message };
}

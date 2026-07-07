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
export type AshaGameAssetCatalogDiagnosticCode = 'duplicate_asset_id' | 'missing_asset_file' | 'forbidden_asset_path' | 'unsupported_asset_kind' | 'missing_asset_dependency' | 'duplicate_asset_dependency' | 'asset_dependency_cycle' | 'stale_import_metadata' | 'invalid_asset_entry';
export interface AshaGameAssetCatalogDiagnostic {
    readonly code: AshaGameAssetCatalogDiagnosticCode;
    readonly path: string;
    readonly message: string;
}
export type AshaGameAssetCatalogValidation = {
    readonly ok: true;
    readonly catalog: AshaGameAssetCatalog;
    readonly diagnostics: readonly [];
} | {
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
export declare function validateAshaGameAssetCatalog(catalog: AshaGameAssetCatalog, manifest: AshaGameManifest, fileExists: (path: string) => boolean, options?: AshaGameAssetCatalogValidationOptions): AshaGameAssetCatalogValidation;
export declare function resolveAshaGameAssetForDev(catalog: AshaGameAssetCatalog, assetId: string, sourceHash?: string | null): AshaGameAssetDevResolution | null;
export declare function buildAshaGamePublishAssetManifest(catalog: AshaGameAssetCatalog): AshaGamePublishAssetManifest;
//# sourceMappingURL=assets.d.ts.map
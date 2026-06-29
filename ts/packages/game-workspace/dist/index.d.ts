export interface AshaGameManifest {
    readonly asha: {
        readonly engineVersion: string;
        readonly contractsVersion: string;
        readonly runtimeBridgeVersion: string;
        readonly devtoolsProtocolVersion: string;
        readonly publishArtifactFormatVersion: string;
        readonly engineSource: string;
    };
    readonly workspace: {
        readonly sceneRoots: readonly string[];
        readonly assetRoots: readonly string[];
        readonly replayRoots: readonly string[];
        readonly catalogPackages: readonly string[];
        readonly policyPackages: readonly string[];
    };
    readonly runtime: {
        readonly devCommand: string;
        readonly devtoolsEndpoint: string;
        readonly wasmOrNativeEntry: string;
        readonly backendMode: AshaGameRuntimeBackendMode;
        readonly backendProfile: string;
        readonly backendProofRefs: readonly string[];
    };
    readonly studio: {
        readonly workspaceMode: boolean;
        readonly attachEnabled: boolean;
        readonly allowedSourceWrites: readonly string[];
    };
    readonly publish: {
        readonly command: string;
        readonly artifactDir: string;
        readonly verifyCommand: string;
    };
    readonly devResourceProfile: {
        readonly localRoots: readonly string[];
        readonly cacheDir: string;
        readonly resolutionPolicy: string;
    };
    readonly publishResourceProfile: {
        readonly outputDir: string;
        readonly archiveDir: string;
        readonly resolutionPolicy: string;
    };
}
export type AshaGameManifestDiagnosticCode = 'toml_parse_error' | 'missing_required_field' | 'missing_root' | 'bad_version' | 'unsupported_endpoint' | 'unsupported_backend_mode' | 'missing_backend_ref' | 'private_transport_hint' | 'invalid_write_scope' | 'invalid_resource_profile' | 'invalid_path';
export type AshaGameRuntimeBackendMode = 'reference' | 'native' | 'wasm';
export interface AshaGameManifestDiagnostic {
    readonly code: AshaGameManifestDiagnosticCode;
    readonly path: string;
    readonly message: string;
}
export type AshaConsumerCompatibilityDiagnosticCode = 'missing_metadata' | 'incompatible_version';
export interface AshaConsumerCompatibilityDiagnostic {
    readonly code: AshaConsumerCompatibilityDiagnosticCode;
    readonly path: string;
    readonly message: string;
}
export interface AshaCompatibilitySurfaceMetadata {
    readonly compatibilityVersion: string;
    readonly packageVersion: string;
}
export interface AshaProtocolCompatibilityMetadata {
    readonly compatibilityVersion: string;
}
export interface AshaConsumerCompatibilityMetadata {
    readonly contracts: AshaCompatibilitySurfaceMetadata;
    readonly runtimeBridge: AshaCompatibilitySurfaceMetadata;
    readonly devtoolsProtocol: AshaProtocolCompatibilityMetadata;
    readonly publishArtifact: AshaProtocolCompatibilityMetadata;
}
export type AshaConsumerCompatibilityValidation = {
    readonly ok: true;
    readonly metadata: AshaConsumerCompatibilityMetadata;
    readonly diagnostics: readonly [];
} | {
    readonly ok: false;
    readonly diagnostics: readonly AshaConsumerCompatibilityDiagnostic[];
};
export declare const ASHA_GAME_WORKSPACE_COMPATIBILITY: AshaConsumerCompatibilityMetadata;
export type AshaGameManifestValidation = {
    readonly ok: true;
    readonly manifest: AshaGameManifest;
    readonly diagnostics: readonly [];
} | {
    readonly ok: false;
    readonly diagnostics: readonly AshaGameManifestDiagnostic[];
};
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
export type AshaAuthoringOperationKind = 'authoring.scene.save_source' | 'authoring.catalog.save_source' | 'authoring.asset.save_source' | 'authoring.policy.save_source';
export type AshaAuthoringSourceFormat = 'proof-scene-json.v1' | 'asset-catalog-json.v1' | 'inline-asset-json.v1' | 'policy-json.deferred';
export type AshaAuthoringDiagnosticCode = 'unsupported_operation' | 'disallowed_path' | 'invalid_extension' | 'forbidden_generated_path' | 'private_transport_hint' | 'stale_file_hash' | 'invalid_schema';
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
export type AshaAuthoringSaveResult = {
    readonly ok: true;
    readonly readback: AshaAuthoringSaveReadback;
    readonly diagnostics: readonly [];
} | {
    readonly ok: false;
    readonly diagnostics: readonly AshaAuthoringDiagnostic[];
};
export type AshaAuthoringWriteTargetResolution = {
    readonly ok: true;
    readonly operationKind: AshaAuthoringOperationKind;
    readonly normalizedPath: string;
    readonly allowedRoot: string;
    readonly format: AshaAuthoringSourceFormat;
    readonly requiredValidator: string;
    readonly diagnostics: readonly [];
} | {
    readonly ok: false;
    readonly diagnostics: readonly AshaAuthoringDiagnostic[];
};
export declare function parseAshaGameManifestToml(toml: string): AshaGameManifestValidation;
export declare function validateAshaConsumerCompatibility(manifest: AshaGameManifest, metadata: Partial<AshaConsumerCompatibilityMetadata>): AshaConsumerCompatibilityValidation;
export declare function validateAshaGameAssetCatalog(catalog: AshaGameAssetCatalog, manifest: AshaGameManifest, fileExists: (path: string) => boolean, options?: AshaGameAssetCatalogValidationOptions): AshaGameAssetCatalogValidation;
export declare function resolveAshaGameAssetForDev(catalog: AshaGameAssetCatalog, assetId: string, sourceHash?: string | null): AshaGameAssetDevResolution | null;
export declare function buildAshaGamePublishAssetManifest(catalog: AshaGameAssetCatalog): AshaGamePublishAssetManifest;
export declare function buildAshaAuthoringPersistenceContract(manifest: AshaGameManifest): AshaAuthoringPersistenceContract;
export declare function resolveAshaAuthoringWriteTarget(manifest: AshaGameManifest, request: Pick<AshaAuthoringSaveRequest, 'operationKind' | 'relativePath'>): AshaAuthoringWriteTargetResolution;
//# sourceMappingURL=index.d.ts.map
import type { AshaGameManifest } from './manifest.js';
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
export declare function buildAshaAuthoringPersistenceContract(manifest: AshaGameManifest): AshaAuthoringPersistenceContract;
export declare function resolveAshaAuthoringWriteTarget(manifest: AshaGameManifest, request: Pick<AshaAuthoringSaveRequest, 'operationKind' | 'relativePath'>): AshaAuthoringWriteTargetResolution;
//# sourceMappingURL=authoring.d.ts.map
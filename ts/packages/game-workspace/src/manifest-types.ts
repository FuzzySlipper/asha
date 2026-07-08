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

export type AshaGameManifestDiagnosticCode =
  | 'toml_parse_error'
  | 'missing_required_field'
  | 'missing_root'
  | 'bad_version'
  | 'unsupported_endpoint'
  | 'unsupported_backend_mode'
  | 'missing_backend_ref'
  | 'private_transport_hint'
  | 'invalid_write_scope'
  | 'invalid_resource_profile'
  | 'invalid_path';

export type AshaGameRuntimeBackendMode = 'reference' | 'native' | 'wasm';

export interface AshaGameManifestDiagnostic {
  readonly code: AshaGameManifestDiagnosticCode;
  readonly path: string;
  readonly message: string;
}

export type AshaConsumerCompatibilityDiagnosticCode =
  | 'missing_metadata'
  | 'incompatible_version';

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

export type AshaConsumerCompatibilityValidation =
  | {
      readonly ok: true;
      readonly metadata: AshaConsumerCompatibilityMetadata;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly diagnostics: readonly AshaConsumerCompatibilityDiagnostic[];
    };

export const ASHA_GAME_WORKSPACE_COMPATIBILITY: AshaConsumerCompatibilityMetadata = {
  contracts: { compatibilityVersion: 'contracts.v0', packageVersion: '0.1.0' },
  runtimeBridge: { compatibilityVersion: 'runtime-bridge.v0', packageVersion: '0.1.0' },
  devtoolsProtocol: { compatibilityVersion: 'devtools-protocol.v0' },
  publishArtifact: { compatibilityVersion: 'publish-artifact.v0' },
};

export type AshaGameManifestValidation =
  | {
      readonly ok: true;
      readonly manifest: AshaGameManifest;
      readonly diagnostics: readonly [];
    }
  | {
      readonly ok: false;
      readonly diagnostics: readonly AshaGameManifestDiagnostic[];
    };

export function manifestDiagnostic(
  code: AshaGameManifestDiagnosticCode,
  path: string,
  message: string,
): AshaGameManifestDiagnostic {
  return { code, path, message };
}

export function consumerCompatibilityDiagnostic(
  code: AshaConsumerCompatibilityDiagnosticCode,
  path: string,
  message: string,
): AshaConsumerCompatibilityDiagnostic {
  return { code, path, message };
}

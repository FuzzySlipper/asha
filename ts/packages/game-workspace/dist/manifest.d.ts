export { validateAshaConsumerCompatibility } from './manifest-compatibility.js';
export type { AshaCompatibilitySurfaceMetadata, AshaConsumerCompatibilityDiagnostic, AshaConsumerCompatibilityDiagnosticCode, AshaConsumerCompatibilityMetadata, AshaConsumerCompatibilityValidation, AshaGameManifest, AshaGameManifestDiagnostic, AshaGameManifestDiagnosticCode, AshaGameManifestValidation, AshaGameRuntimeBackendMode, AshaProtocolCompatibilityMetadata, } from './manifest-types.js';
export { ASHA_GAME_WORKSPACE_COMPATIBILITY } from './manifest-types.js';
import { type AshaGameManifestValidation } from './manifest-types.js';
export declare function parseAshaGameManifestToml(toml: string): AshaGameManifestValidation;
//# sourceMappingURL=manifest.d.ts.map
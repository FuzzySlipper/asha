import { type AshaGameManifestDiagnostic } from './manifest-types.js';
export type TomlScalar = string | boolean | readonly string[];
export type TomlSection = Record<string, TomlScalar>;
export type TomlDocument = Record<string, TomlSection>;
export declare function parseTomlSubset(toml: string): {
    readonly ok: true;
    readonly document: TomlDocument;
} | {
    readonly ok: false;
    readonly diagnostics: readonly AshaGameManifestDiagnostic[];
};
//# sourceMappingURL=manifest-toml.d.ts.map
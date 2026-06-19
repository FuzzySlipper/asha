import type { StudioCommandDefinition } from './types.js';
export type DraftStudioCommandDefinition = Partial<StudioCommandDefinition<object, object>>;
export interface ManifestValidationIssue {
    readonly commandId: string;
    readonly field: string;
    readonly message: string;
}
export declare function validateCommandDefinition(definition: DraftStudioCommandDefinition): readonly ManifestValidationIssue[];
export declare function validateCommandManifest(manifest: readonly DraftStudioCommandDefinition[]): readonly ManifestValidationIssue[];
export declare function requireKnownCommand(id: string, manifest: readonly StudioCommandDefinition<object, object>[]): StudioCommandDefinition<object, object>;
//# sourceMappingURL=validation.d.ts.map
import type { GameplayModuleBindingRegistry, PrefabDefinition, PrefabDiagnostic, PrefabId, PrefabInstanceId, PrefabInstanceRecord, PrefabOverride, PrefabPart, PrefabPartId, PrefabPartRoleBinding, PrefabRegistry, PrefabTransform } from '@asha/contracts';
export type AshaPrefabAuthoringDiagnosticCode = 'invalidDefinition' | 'duplicatePrefab' | 'missingPrefab' | 'prefabInUse' | 'duplicatePart' | 'duplicateRole' | 'danglingRole' | 'duplicateInstance' | 'unknownOverrideRole';
export interface AshaPrefabAuthoringDiagnostic {
    readonly code: AshaPrefabAuthoringDiagnosticCode;
    readonly path: string;
    readonly message: string;
}
export interface AshaPrefabAuthoringState {
    readonly registry: PrefabRegistry;
    readonly instances: readonly AshaPrefabAuthoredInstance[];
    readonly selectedPrefab: PrefabId | null;
    readonly gameplayBindings: GameplayModuleBindingRegistry | null;
}
export interface AshaPrefabAuthoredInstance {
    readonly origin: 'authored' | 'player';
    readonly record: PrefabInstanceRecord;
}
export type AshaPrefabAuthoringCommand = {
    readonly kind: 'createPrefab';
    readonly definition: PrefabDefinition;
} | {
    readonly kind: 'replacePrefab';
    readonly definition: PrefabDefinition;
} | {
    readonly kind: 'deletePrefab';
    readonly prefab: PrefabId;
} | {
    readonly kind: 'instantiatePrefab';
    readonly origin: 'authored' | 'player';
    readonly record: PrefabInstanceRecord;
};
export type AshaPrefabAuthoringResult = {
    readonly ok: true;
    readonly command: AshaPrefabAuthoringCommand;
    readonly state: AshaPrefabAuthoringState;
    readonly readout: AshaPrefabAuthoringReadout;
    readonly diagnostics: readonly [];
} | {
    readonly ok: false;
    readonly command: AshaPrefabAuthoringCommand;
    readonly state: AshaPrefabAuthoringState;
    readonly diagnostics: readonly AshaPrefabAuthoringDiagnostic[];
};
export interface AshaPrefabAuthoringReadout {
    readonly registrySchemaVersion: number;
    readonly definitions: readonly AshaPrefabBrowserItem[];
    readonly selected: AshaPrefabDefinitionReadout | null;
    readonly instances: readonly AshaPrefabInstanceReadout[];
    readonly configurations: readonly AshaPrefabConfigurationReadout[];
    readonly bindings: readonly AshaPrefabBindingReadout[];
    readonly nonClaims: readonly ['nestedPrefabs', 'propagatingDefinitionEdits', 'runtimeAuthority'];
}
export interface AshaPrefabBrowserItem {
    readonly prefab: PrefabId;
    readonly displayName: string;
    readonly partCount: number;
    readonly roleCount: number;
    readonly variantBase: PrefabId | null;
}
export interface AshaPrefabDefinitionReadout extends AshaPrefabBrowserItem {
    readonly parts: readonly AshaPrefabPartReadout[];
    readonly roles: readonly PrefabPartRoleBinding[];
}
export interface AshaPrefabPartReadout {
    readonly part: PrefabPartId;
    readonly namespace: string;
    readonly displayName: string;
    readonly parent: PrefabPartId | null;
    readonly roles: readonly string[];
    readonly sourceKind: PrefabPart['source']['kind'];
}
export interface AshaPrefabInstanceReadout {
    readonly instance: PrefabInstanceId;
    readonly prefab: PrefabId;
    readonly origin: 'authored' | 'player';
    readonly overrideFields: readonly string[];
}
export interface AshaPrefabConfigurationReadout {
    readonly configurationId: string;
    readonly moduleId: string;
    readonly configHash: string;
}
export interface AshaPrefabBindingReadout {
    readonly bindingId: string;
    readonly moduleId: string;
    readonly configurationId: string;
    readonly prefab: PrefabId;
    readonly role: string | null;
    readonly enabled: boolean;
    readonly instanceOverrides: readonly {
        readonly instance: PrefabInstanceId;
        readonly configurationId: string | null;
        readonly enabled: boolean | null;
    }[];
}
export declare const ASHA_PREFAB_IDENTITY_TRANSFORM: PrefabTransform;
export declare function createAshaPrefabAuthoringState(gameplayBindings?: GameplayModuleBindingRegistry | null): AshaPrefabAuthoringState;
export declare function buildAshaPrefabDefinition(input: {
    readonly id: PrefabId;
    readonly displayName: string;
    readonly parts: readonly PrefabPart[];
    readonly partRoles: readonly PrefabPartRoleBinding[];
    readonly variant?: PrefabDefinition['variant'];
}): PrefabDefinition;
export declare function buildAshaPrefabPart(input: {
    readonly id: PrefabPartId;
    readonly namespace: string;
    readonly displayName: string;
    readonly parent?: PrefabPartId | null;
    readonly transform?: PrefabTransform;
    readonly source: PrefabPart['source'];
}): PrefabPart;
export declare function createAshaPrefabCommand(definition: PrefabDefinition): AshaPrefabAuthoringCommand;
export declare function replaceAshaPrefabCommand(definition: PrefabDefinition): AshaPrefabAuthoringCommand;
export declare function deleteAshaPrefabCommand(prefab: PrefabId): AshaPrefabAuthoringCommand;
export declare function instantiateAshaPrefabCommand(input: {
    readonly origin: 'authored' | 'player';
    readonly instance: PrefabInstanceId;
    readonly prefab: PrefabId;
    readonly seed: number;
    readonly transform?: PrefabTransform;
    readonly overrides?: readonly PrefabOverride[];
}): AshaPrefabAuthoringCommand;
export declare function selectAshaPrefab(state: AshaPrefabAuthoringState, prefab: PrefabId | null): AshaPrefabAuthoringState;
export declare function applyAshaPrefabAuthoringCommand(state: AshaPrefabAuthoringState, command: AshaPrefabAuthoringCommand): AshaPrefabAuthoringResult;
export declare function readAshaPrefabAuthoring(state: AshaPrefabAuthoringState): AshaPrefabAuthoringReadout;
export declare function serializeAshaPrefabRegistrySource(registry: PrefabRegistry): string;
export declare function validateAshaPrefabRegistrySourceDocument(registry: PrefabRegistry): readonly PrefabDiagnostic[];
//# sourceMappingURL=prefab-authoring.d.ts.map
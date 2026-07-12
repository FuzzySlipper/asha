import type { ProjectId, RuntimeSessionId, SceneId } from './scene.js';
import type { VoxelCoord, VoxelValue } from './voxel.js';
export type PrefabId = number & {
    readonly __brand: 'PrefabId';
};
export declare const prefabId: (raw: number) => PrefabId;
export type PrefabPartId = number & {
    readonly __brand: 'PrefabPartId';
};
export declare const prefabPartId: (raw: number) => PrefabPartId;
export type PrefabInstanceId = number & {
    readonly __brand: 'PrefabInstanceId';
};
export declare const prefabInstanceId: (raw: number) => PrefabInstanceId;
export type ArtifactClass = 'durable' | 'generated' | 'cache';
export type KnownArtifactRole = 'sceneDocument' | 'assetLock' | 'prefabRegistry' | 'sessionStateSnapshot' | 'voxelChunkSnapshot' | 'voxelEditLog' | 'voxelEditHistory' | 'voxelAnnotationLayer' | 'replayRecord' | 'generatedMetadata' | 'cache';
export type LoadStage = 'versions' | 'assetLock' | 'sceneDocument' | 'terrainGeneration' | 'voxelEdits' | 'voxelAnnotations' | 'bootstrap' | 'sessionStateSnapshot' | 'finalValidation';
export type SuggestedAction = 'keepEdit' | 'reviewConflict';
export interface ArtifactEntry {
    readonly path: string;
    readonly class: ArtifactClass;
    readonly role: string;
    readonly contentHash: string | null;
}
export interface GeneratorMetadata {
    readonly seed: number;
    readonly version: number;
    readonly params: string;
}
export interface ProjectSection {
    readonly id: ProjectId;
    readonly name: string | null;
}
export interface SceneSection {
    readonly id: SceneId;
    readonly schemaVersion: number;
    readonly artifact: string;
}
export interface AssetLockSection {
    readonly artifact: string;
    readonly assetCount: number;
}
export interface ProjectBundleManifest {
    readonly bundleSchemaVersion: number;
    readonly protocolVersion: number;
    readonly project: ProjectSection;
    readonly scene: SceneSection;
    readonly assetLock: AssetLockSection;
    readonly generator: GeneratorMetadata;
    readonly artifacts: readonly ArtifactEntry[];
}
export declare const GAMEPLAY_TRIGGER_DEFINITION_SCHEMA_VERSION = 1;
export interface GameplayTriggerDefinition {
    readonly schemaVersion: number;
    readonly entity: number;
    readonly scope: string;
    readonly tags: readonly string[];
}
export declare const PREFAB_REGISTRY_SCHEMA_VERSION = 1;
export declare const PREFAB_DEFINITION_SCHEMA_VERSION = 1;
export type PrefabDiagnosticCode = 'unsupportedRegistrySchema' | 'unsupportedDefinitionSchema' | 'duplicatePrefabId' | 'missingDisplayName' | 'duplicatePartId' | 'invalidPartNamespace' | 'duplicatePartNamespace' | 'missingParentPart' | 'partHierarchyCycle' | 'invalidPartTransform' | 'unknownAsset' | 'assetKindMismatch' | 'unknownEntityDefinition' | 'invalidPartRole' | 'duplicatePartRole' | 'danglingPartRole' | 'missingBasePrefab' | 'variantCycle' | 'variantDepthExceeded' | 'variantDefinesParts' | 'unknownRemovedRole' | 'duplicateRemovedRole' | 'unsafePartRemoval' | 'invalidOverrideTarget' | 'duplicateOverride' | 'invalidOverrideValue' | 'deletedRoleReferenced';
export interface PrefabTransform {
    readonly translation: readonly [number, number, number];
    readonly rotation: readonly [number, number, number, number];
    readonly scale: readonly [number, number, number];
}
export type PrefabPartSource = {
    readonly kind: 'scene';
    readonly asset: string;
} | {
    readonly kind: 'entityDefinition';
    readonly stableId: string;
} | {
    readonly kind: 'voxelObject';
    readonly asset: string;
};
export interface PrefabPart {
    readonly id: PrefabPartId;
    readonly namespace: string;
    readonly displayName: string;
    readonly parent: PrefabPartId | null;
    readonly transform: PrefabTransform;
    readonly source: PrefabPartSource;
}
export interface PrefabPartRoleBinding {
    readonly role: string;
    readonly part: PrefabPartId;
}
export type PrefabOverrideValue = {
    readonly field: 'transform';
    readonly transform: PrefabTransform;
} | {
    readonly field: 'entityDefinition';
    readonly stableId: string;
} | {
    readonly field: 'asset';
    readonly asset: string;
} | {
    readonly field: 'material';
    readonly asset: string;
} | {
    readonly field: 'activation';
    readonly active: boolean;
};
export interface PrefabOverride {
    readonly targetRole: string;
    readonly value: PrefabOverrideValue;
}
export interface PrefabVariantDelta {
    readonly base: PrefabId;
    readonly removedRoles: readonly string[];
    readonly overrides: readonly PrefabOverride[];
}
export interface PrefabDefinition {
    readonly id: PrefabId;
    readonly schemaVersion: number;
    readonly displayName: string;
    readonly parts: readonly PrefabPart[];
    readonly partRoles: readonly PrefabPartRoleBinding[];
    readonly variant: PrefabVariantDelta | null;
}
export interface PrefabRegistry {
    readonly schemaVersion: number;
    readonly definitions: readonly PrefabDefinition[];
}
export interface PrefabInstanceRecord {
    readonly instance: PrefabInstanceId;
    readonly prefab: PrefabId;
    readonly seed: number;
    readonly transform: PrefabTransform;
    readonly overrides: readonly PrefabOverride[];
}
export interface PrefabPartReference {
    readonly prefab: PrefabId;
    readonly role: string;
}
export interface PrefabDiagnostic {
    readonly code: PrefabDiagnosticCode;
    readonly path: string;
    readonly message: string;
}
export type PrefabValidationOutcome = {
    readonly status: 'valid';
} | {
    readonly status: 'invalid';
    readonly diagnostics: readonly PrefabDiagnostic[];
};
export type ManifestError = {
    readonly code: 'unsupportedSchema';
    readonly found: number;
    readonly supported: number;
} | {
    readonly code: 'unsupportedProtocol';
    readonly found: number;
    readonly supported: number;
} | {
    readonly code: 'duplicateArtifact';
    readonly path: string;
} | {
    readonly code: 'missingArtifact';
    readonly role: string;
    readonly path: string;
} | {
    readonly code: 'durableMissingHash';
    readonly path: string;
} | {
    readonly code: 'duplicateArtifactRole';
    readonly role: string;
} | {
    readonly code: 'artifactClassMismatch';
    readonly path: string;
    readonly expected: string;
    readonly found: string;
};
export interface ManifestValidationReport {
    readonly errors: readonly ManifestError[];
}
export type LoadStep = {
    readonly step: 'validateVersions';
    readonly bundleSchemaVersion: number;
    readonly protocolVersion: number;
} | {
    readonly step: 'loadAssetLock';
    readonly artifact: string;
    readonly assetCount: number;
} | {
    readonly step: 'loadSceneDocument';
    readonly artifact: string;
    readonly scene: SceneId;
} | {
    readonly step: 'generateTerrain';
    readonly seed: number;
    readonly version: number;
    readonly params: string;
} | {
    readonly step: 'applyVoxelEdits';
    readonly editLogs: readonly string[];
    readonly snapshots: readonly string[];
    readonly histories: readonly string[];
} | {
    readonly step: 'loadVoxelAnnotations';
    readonly artifacts: readonly string[];
} | {
    readonly step: 'bootstrapScene';
    readonly scene: SceneId;
    readonly runtimeSession: RuntimeSessionId;
} | {
    readonly step: 'restoreSessionState';
    readonly artifact: string;
} | {
    readonly step: 'validateFinalState';
};
export interface LoadPlan {
    readonly steps: readonly LoadStep[];
}
export type LoadPlanError = {
    readonly code: 'manifest';
    readonly error: ManifestError;
} | {
    readonly code: 'missingPrerequisiteArtifact';
    readonly role: string;
} | {
    readonly code: 'outOfOrder';
    readonly step: LoadStage;
    readonly after: LoadStage;
} | {
    readonly code: 'missingStage';
    readonly stage: LoadStage;
};
export interface CompactionSummary {
    readonly compactedEdits: number;
    readonly retainedEdits: number;
    readonly snapshotChunks: readonly string[];
}
export interface SaveSummary {
    readonly writes: readonly ArtifactEntry[];
    readonly compaction: CompactionSummary;
}
export interface GeneratorMismatch {
    readonly savedVersion: number;
    readonly currentVersion: number;
}
export interface EditConflict {
    readonly eventId: number;
    readonly coord: VoxelCoord;
    readonly oldGenerated: VoxelValue;
    readonly newGenerated: VoxelValue;
    readonly editValue: VoxelValue;
    readonly suggested: SuggestedAction;
}
export interface RegenConflictReport {
    readonly savedVersion: number;
    readonly newVersion: number;
    readonly conflicts: readonly EditConflict[];
    readonly replayedEdits: number;
    readonly stagingSessionHash: number;
}
//# sourceMappingURL=projectBundle.d.ts.map
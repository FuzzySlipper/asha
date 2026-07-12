import { type RuntimeBridge } from './bridge.js';
export type NativeOperationInvocation = (bridge: RuntimeBridge) => unknown;
export interface NativeOperationInvocationInputs {
    readonly collisionCamera: Parameters<RuntimeBridge['applyCollisionConstrainedCameraInput']>[0];
    readonly cameraInput: Parameters<RuntimeBridge['applyFirstPersonCameraInput']>[0];
    readonly cameraCreate: Parameters<RuntimeBridge['createCamera']>[0];
    readonly cameraMode: Parameters<RuntimeBridge['applyCameraModeCommand']>[0];
    readonly cameraNavigation: Parameters<RuntimeBridge['applyCameraNavigationInput']>[0];
    readonly gameRuleCatalog: Parameters<RuntimeBridge['validateGameRuleCatalog']>[0];
    readonly gameRuleRequest: Parameters<RuntimeBridge['submitGameRuleEffectIntent']>[0]['request'];
    readonly hashA: string;
    readonly voxelPlan: Parameters<RuntimeBridge['planVoxelConversion']>[0];
    readonly voxelSource: Parameters<RuntimeBridge['registerVoxelConversionSource']>[0];
    readonly voxelMeshAsset: Parameters<RuntimeBridge['registerVoxelConversionMeshAsset']>[0];
    readonly voxelMeshImport: Parameters<RuntimeBridge['importVoxelConversionMeshSource']>[0];
    readonly voxelPlanHash: string;
    readonly voxelPreviewHash: string;
    readonly voxelEvidence: Parameters<RuntimeBridge['exportVoxelConversionEvidence']>[0];
    readonly voxelModelInfo: Parameters<RuntimeBridge['readVoxelModelInfo']>[0];
    readonly voxelModelWindow: Parameters<RuntimeBridge['readVoxelModelWindow']>[0];
    readonly voxelExport: Parameters<RuntimeBridge['exportVoxelVolumeAsset']>[0];
    readonly voxelSave: Parameters<RuntimeBridge['saveVoxelVolumeAsset']>[0];
    readonly voxelPaletteUpdate: Parameters<RuntimeBridge['updateVoxelVolumeAssetPalette']>[0];
    readonly voxelAuthoring: Parameters<RuntimeBridge['initializeVoxelVolumeAuthoring']>[0];
    readonly voxelLoad: Parameters<RuntimeBridge['loadVoxelVolumeAsset']>[0];
    readonly voxelUnload: Parameters<RuntimeBridge['unloadVoxelVolumeAsset']>[0];
    readonly annotationValidation: Parameters<RuntimeBridge['validateVoxelAnnotationLayer']>[0];
    readonly annotationLoad: Parameters<RuntimeBridge['loadVoxelAnnotationLayer']>[0];
    readonly annotationQuery: Parameters<RuntimeBridge['readVoxelAnnotationQuery']>[0];
    readonly annotationEdit: Parameters<RuntimeBridge['applyVoxelAnnotationEdit']>[0];
    readonly annotationExport: Parameters<RuntimeBridge['exportVoxelAnnotationLayer']>[0];
    readonly historyRead: Parameters<RuntimeBridge['readVoxelEditHistory']>[0];
    readonly historyRevert: Parameters<RuntimeBridge['previewVoxelEditRevert']>[0];
    readonly historyUndo: Parameters<RuntimeBridge['undoVoxelEdit']>[0];
    readonly historyRedo: Parameters<RuntimeBridge['redoVoxelEdit']>[0];
    readonly materialPreview: Parameters<RuntimeBridge['readModelMaterialPreview']>[0];
    readonly inputConfigure: Parameters<RuntimeBridge['configureInputSession']>[0];
    readonly inputContextCommand: Parameters<RuntimeBridge['applyInputContextCommand']>[0];
    readonly rawInput: Parameters<RuntimeBridge['submitRawInput']>[0];
    readonly recordedInput: Parameters<RuntimeBridge['replayResolvedInputAction']>[0];
    readonly timeControlCommand: Parameters<RuntimeBridge['applyTimeControlCommand']>[0];
}
export declare function createNativeOperationInvocations(input: NativeOperationInvocationInputs): ReadonlyMap<string, NativeOperationInvocation>;
/**
 * Builds the invocation fixture against the generated bridge-manifest catalog.
 * Duplicate, missing, and non-manifest methods fail while the module loads, so
 * additive verbs cannot silently escape the real native conformance sequence.
 */
export declare function composeNativeOperationInvocations(entries: readonly (readonly [string, NativeOperationInvocation])[]): ReadonlyMap<string, NativeOperationInvocation>;
//# sourceMappingURL=native-operation-invocations.test-fixture.d.ts.map
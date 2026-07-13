import type * as Contracts from '@asha/contracts';
import type * as Session from '@asha/runtime-session';
import type * as Bridge from '../bridge.js';
export interface RuntimeInputPort {
    configureInputSession(input: Contracts.InputSessionConfigureRequest): Contracts.InputSessionSnapshot;
    applyInputContextCommand(input: Contracts.InputContextCommand): Contracts.InputContextChangeReceipt;
    submitRawInput(input: Contracts.RawInputSample): Contracts.InputResolutionReceipt;
    replayResolvedInputAction(input: Contracts.RecordedInputAction): Contracts.InputActionReplayReceipt;
    readInputContextState(): Contracts.InputContextStackState;
}
export interface RuntimeTimeSimulationPort {
    applyTimeControlCommand(input: Contracts.TimeControlCommand): Contracts.TimeControlReceipt;
    readTimeControlState(): Contracts.TimeControlState;
    stepSimulation(input: Bridge.StepInputEnvelope): Session.StepResult;
}
export interface RuntimeSceneEntityPort {
    readModelMaterialPreview(input: Contracts.ModelMaterialPreviewRequest): Contracts.ModelMaterialPreviewSnapshot;
    readSceneObjectSnapshot(): Contracts.SceneObjectSnapshot;
    applySceneObjectCommand(input: Contracts.SceneObjectCommandRequest): Contracts.SceneObjectCommandResult;
    applyEnemyDirectNavMovement(input: Session.EnemyDirectNavMovementRequest): Session.EnemyDirectNavMovementResult;
}
export interface RuntimeVoxelAssetBufferPort {
    submitCommands(input: Contracts.CommandBatch): Contracts.CommandResult;
    pickVoxel(input: Contracts.PickRay): Contracts.PickResult;
    selectVoxel(input: Contracts.ScreenPointToPickRayRequest): Contracts.VoxelSelectionSnapshot;
    readVoxelMeshEvidence(input: Bridge.VoxelMeshEvidenceRequest): Bridge.VoxelMeshEvidenceSnapshot;
    planVoxelConversion(input: Contracts.VoxelConversionPlanRequest): Contracts.VoxelConversionPlan;
    registerVoxelConversionSource(input: Contracts.VoxelConversionSourceRegistrationRequest): Contracts.VoxelConversionSourceRegistration;
    registerVoxelConversionMeshAsset(input: Contracts.VoxelConversionMeshAssetRegistrationRequest): Contracts.VoxelConversionSourceRegistration;
    importVoxelConversionMeshSource(input: Contracts.VoxelConversionMeshSourceImportRequest): Contracts.VoxelConversionMeshSourceImportReceipt;
    readVoxelConversionSourceMetadata(input: Contracts.VoxelConversionSourceMetadataRequest): Contracts.VoxelConversionSourceMetadataReadout;
    previewVoxelConversion(input: Contracts.VoxelConversionPreviewRequest): Contracts.VoxelConversionPreview;
    applyVoxelConversion(input: Contracts.VoxelConversionApplyRequest): Contracts.VoxelConversionReceipt;
    readVoxelModelInfo(input: Contracts.VoxelModelInfoRequest): Contracts.VoxelModelInfoReadout;
    readVoxelModelWindow(input: Contracts.VoxelModelWindowRequest): Contracts.VoxelModelWindowReadout;
    exportVoxelVolumeAsset(input: Contracts.VoxelVolumeAssetExportRequest): Contracts.VoxelVolumeAssetExportReceipt;
    saveVoxelVolumeAsset(input: Contracts.VoxelVolumeAssetSaveRequest): Contracts.VoxelVolumeAssetSaveReceipt;
    updateVoxelVolumeAssetPalette(input: Contracts.VoxelVolumeAssetPaletteUpdateRequest): Contracts.VoxelVolumeAssetPaletteUpdateReceipt;
    initializeVoxelVolumeAuthoring(input: Contracts.VoxelVolumeAuthoringInitializeRequest): Contracts.VoxelVolumeAuthoringInitializeReceipt;
    loadVoxelVolumeAsset(input: Contracts.VoxelVolumeAssetLoadRequest): Contracts.VoxelVolumeAssetLoadReceipt;
    unloadVoxelVolumeAsset(input: Contracts.VoxelVolumeAssetUnloadRequest): Contracts.VoxelVolumeAssetUnloadReceipt;
    validateVoxelAnnotationLayer(input: Contracts.VoxelAnnotationLayerValidationRequest): Contracts.VoxelAnnotationLayerValidationReport;
    loadVoxelAnnotationLayer(input: Contracts.VoxelAnnotationLayerLoadRequest): Contracts.VoxelAnnotationLayerLoadReceipt;
    readVoxelAnnotationQuery(input: Contracts.VoxelAnnotationQueryRequest): Contracts.VoxelAnnotationQueryReadout;
    applyVoxelAnnotationEdit(input: Contracts.VoxelAnnotationEditRequest): Contracts.VoxelAnnotationEditReceipt;
    exportVoxelAnnotationLayer(input: Contracts.VoxelAnnotationLayerExportRequest): Contracts.VoxelAnnotationLayerExportReceipt;
    readVoxelEditHistory(input: Contracts.VoxelEditHistoryReadRequest): Contracts.VoxelEditHistorySummary;
    previewVoxelEditRevert(input: Contracts.VoxelEditHistoryRevertRequest): Contracts.VoxelEditHistoryRevertReceipt;
    applyVoxelEditRevert(input: Contracts.VoxelEditHistoryRevertRequest): Contracts.VoxelEditHistoryRevertReceipt;
    undoVoxelEdit(input: Contracts.VoxelEditHistoryUndoRequest): Contracts.VoxelEditHistoryUndoReceipt;
    redoVoxelEdit(input: Contracts.VoxelEditHistoryRedoRequest): Contracts.VoxelEditHistoryRedoReceipt;
    getBuffer(input: Bridge.RuntimeBufferHandle): Bridge.RuntimeBufferView;
    releaseBuffer(input: Bridge.RuntimeBufferHandle): void;
}
export interface RuntimeCameraPort {
    applyCollisionConstrainedCameraInput(input: Contracts.CollisionConstrainedCameraInputEnvelope): Contracts.CameraCollisionSnapshot;
    createCamera(input: Contracts.CameraCreateRequest): Contracts.CameraSnapshot;
    applyCameraModeCommand(input: Contracts.CameraModeCommand): Contracts.CameraModeChangeReceipt;
    applyCameraNavigationInput(input: Contracts.CameraNavigationInputEnvelope): Contracts.CameraNavigationReceipt;
    readCameraControllerState(input: Contracts.CameraControllerReadRequest): Contracts.CameraControllerState;
    applyFirstPersonCameraInput(input: Contracts.FirstPersonCameraInputEnvelope): Contracts.CameraSnapshot;
    readCameraProjection(input: Contracts.CameraProjectionRequest): Contracts.CameraProjectionSnapshot;
}
export interface RuntimeGameplayPort {
    applyGeneratedTunnelToRuntimeWorld(input: Contracts.GeneratedTunnelRuntimeApplyRequest): Contracts.GeneratedTunnelRuntimeApplyReceipt;
    loadFpsRuntimeSession(input: Session.FpsRuntimeSessionLoadRequest): Session.FpsRuntimeSessionSnapshot;
    readFpsRuntimeSession(): Session.FpsRuntimeSessionSnapshot;
    applyFpsPrimaryFire(input: Session.FpsPrimaryFireRequest): Session.FpsPrimaryFireResult;
    invokeGameExtensionWeaponEffect(input: Session.GameExtensionWeaponEffectInvocationRequest): Session.GameExtensionWeaponEffectInvocationResult;
    validateGameRuleCatalog(input: Contracts.GameRuleCatalog): Session.GameRuleCatalogValidationReceipt;
    submitGameRuleEffectIntent(input: Session.GameRuleEffectIntentRequest): Contracts.GameRuleResolutionReceipt;
    readGameRuleRuntimeReadout(): Session.GameRuleRuntimeReadout;
    restartFpsRuntimeSession(input: Session.FpsRuntimeSessionRestartRequest): Session.FpsRuntimeSessionSnapshot;
    readFpsEncounterDirector(input: Session.FpsEncounterLifecycleInput): Session.FpsEncounterDirectorSnapshot;
    applyFpsEncounterTransition(input: Session.FpsEncounterTransitionRequest): Session.FpsEncounterTransitionResult;
}
export interface RuntimeProjectionPort {
    readRenderDiffs(input: Session.FrameCursor): Contracts.RenderFrameDiff;
    readProjectionFrame(input: Session.FrameCursor): Contracts.RuntimeProjectionFrame;
}
export interface RuntimeBundleLifecyclePort {
    initializeEngine(input: Bridge.EngineConfig): Session.EngineHandle;
    loadProjectBundle(input: Bridge.ProjectBundleLoadRequest): Bridge.CompositionStatus;
    saveProjectBundle(): Bridge.ProjectBundleSaveSummary;
    getProjectBundleCompositionStatus(): Bridge.CompositionStatus;
    unloadProjectBundle(): void;
}
export interface RuntimeReplayEvidencePort {
    exportVoxelConversionEvidence(input: readonly Contracts.VoxelConversionEvidenceRef[]): readonly Contracts.VoxelConversionEvidenceRef[];
    loadReplayFixture(input: Bridge.ReplayFixture): Bridge.ReplaySessionHandle;
    runReplayStep(input: Bridge.ReplaySessionHandle): Bridge.ReplayStepReport;
}
/** Bounded generated verbs only; no generic method-name dispatcher. */
export interface RuntimeBridge extends RuntimeInputPort, RuntimeTimeSimulationPort, RuntimeSceneEntityPort, RuntimeVoxelAssetBufferPort, RuntimeCameraPort, RuntimeGameplayPort, RuntimeProjectionPort, RuntimeBundleLifecyclePort, RuntimeReplayEvidencePort {
}
export interface RuntimeBridgePorts {
    readonly input: RuntimeInputPort;
    readonly timeSimulation: RuntimeTimeSimulationPort;
    readonly sceneEntities: RuntimeSceneEntityPort;
    readonly voxelAssetsBuffers: RuntimeVoxelAssetBufferPort;
    readonly camera: RuntimeCameraPort;
    readonly gameplay: RuntimeGameplayPort;
    readonly projection: RuntimeProjectionPort;
    readonly bundleLifecycle: RuntimeBundleLifecyclePort;
    readonly replayEvidence: RuntimeReplayEvidencePort;
}
export type RuntimeBridgePortId = keyof RuntimeBridgePorts;
export interface RuntimeBridgePortContract {
    readonly initialization: 'requiresEngine' | 'createsEngine';
    readonly projectBundle: 'retainedAcrossLoadUnload' | 'ownsLoadUnload';
    readonly snapshotHash: 'inputEvidence' | 'timeState' | 'sceneDocument' | 'voxelStateAndResources' | 'cameraProjection' | 'gameplaySessionAndReplay' | 'projectionFrame' | 'compositionStatus' | 'replayEvidence';
    readonly resourceLifetime: 'session' | 'frame' | 'mixedExplicitAndSession';
}
export declare const RUNTIME_BRIDGE_PORT_CONTRACTS: Readonly<Record<RuntimeBridgePortId, RuntimeBridgePortContract>>;
export declare function runtimeBridgePorts(bridge: RuntimeBridge): RuntimeBridgePorts;
//# sourceMappingURL=surfaces.d.ts.map
import { entityId } from '@asha/contracts';
import { frameCursor, } from './bridge.js';
import { MANIFEST_OPERATIONS } from './generated/operations.js';
import { fpsLoadRequest } from './native-fps-fixtures.test-fixture.js';
export function createNativeOperationInvocations(input) {
    return composeNativeOperationInvocations([
        ['initializeEngine', (bridge) => bridge.initializeEngine({ seed: 7 })],
        ['configureInputSession', (bridge) => bridge.configureInputSession(input.inputConfigure)],
        ['applyInputContextCommand', (bridge) => bridge.applyInputContextCommand(input.inputContextCommand)],
        ['submitRawInput', (bridge) => bridge.submitRawInput(input.rawInput)],
        ['replayResolvedInputAction', (bridge) => bridge.replayResolvedInputAction(input.recordedInput)],
        ['readInputContextState', (bridge) => bridge.readInputContextState()],
        ['applyTimeControlCommand', (bridge) => bridge.applyTimeControlCommand(input.timeControlCommand)],
        ['readTimeControlState', (bridge) => bridge.readTimeControlState()],
        ['stepSimulation', (bridge) => bridge.stepSimulation({ tick: 6 })],
        ['submitCommands', (bridge) => bridge.submitCommands({ commands: [] })],
        ['pickVoxel', (bridge) => bridge.pickVoxel({ grid: 1, origin: [0, 0, 0], direction: [1, 0, 0], maxDistance: 10 })],
        ['applyCollisionConstrainedCameraInput', (bridge) => bridge.applyCollisionConstrainedCameraInput(input.collisionCamera)],
        ['applyGeneratedTunnelToRuntimeWorld', (bridge) => bridge.applyGeneratedTunnelToRuntimeWorld({ preset: 'tiny-enclosed', seed: 17 })],
        ['selectVoxel', (bridge) => bridge.selectVoxel({
                camera: input.cameraInput.camera,
                grid: 1,
                viewport: null,
                screenPoint: { x: 0.5, y: 0.5, space: 'normalized_0_1' },
                maxDistance: 10,
            })],
        ['readVoxelMeshEvidence', (bridge) => bridge.readVoxelMeshEvidence({ grid: 1, chunks: [] })],
        ['loadFpsRuntimeSession', (bridge) => bridge.loadFpsRuntimeSession(fpsLoadRequest())],
        ['readFpsRuntimeSession', (bridge) => bridge.readFpsRuntimeSession()],
        ['applyFpsPrimaryFire', (bridge) => bridge.applyFpsPrimaryFire({ tick: 9, origin: [2.5, 1.5, 1.5], direction: [0, 0, 1] })],
        ['invokeGameExtensionWeaponEffect', (bridge) => bridge.invokeGameExtensionWeaponEffect({
                hook: {
                    moduleRef: {
                        moduleId: 'asha.reference.primary_fire_damage_modifier',
                        version: '0.1.0',
                        contractHash: 'sha256:asha-reference-primary-fire-damage-modifier-v0',
                    },
                    hookId: 'weapon.primary.damage_modifier',
                    requestId: 'request.native-fixture',
                    tick: 9,
                    source: entityId(101),
                    target: entityId(777),
                    baseDamage: 75,
                    rangeMillimeters: 16000,
                    tags: ['primary-fire'],
                    inputHash: input.hashA,
                },
                primaryFire: { tick: 9, origin: [2.5, 1.5, 1.5], direction: [0, 0, 1] },
            })],
        ['validateGameRuleCatalog', (bridge) => bridge.validateGameRuleCatalog(input.gameRuleCatalog)],
        ['submitGameRuleEffectIntent', (bridge) => bridge.submitGameRuleEffectIntent({
                catalog: input.gameRuleCatalog,
                request: input.gameRuleRequest,
            })],
        ['readGameRuleRuntimeReadout', (bridge) => bridge.readGameRuleRuntimeReadout()],
        ['restartFpsRuntimeSession', (bridge) => bridge.restartFpsRuntimeSession({ expectedEpoch: 1 })],
        ['readFpsEncounterDirector', (bridge) => bridge.readFpsEncounterDirector({
                outcomeKind: 'in_progress', terminal: false, enemyDead: false, playerDead: false,
                lifecycleHash: input.hashA,
            })],
        ['applyFpsEncounterTransition', (bridge) => bridge.applyFpsEncounterTransition({
                presetId: 'generated-tunnel-small-encounter',
                action: 'activate',
                lifecycle: {
                    outcomeKind: 'in_progress', terminal: false, enemyDead: false, playerDead: false,
                    lifecycleHash: input.hashA,
                },
            })],
        ['planVoxelConversion', (bridge) => bridge.planVoxelConversion(input.voxelPlan)],
        ['registerVoxelConversionSource', (bridge) => bridge.registerVoxelConversionSource(input.voxelSource)],
        ['registerVoxelConversionMeshAsset', (bridge) => bridge.registerVoxelConversionMeshAsset(input.voxelMeshAsset)],
        ['importVoxelConversionMeshSource', (bridge) => bridge.importVoxelConversionMeshSource(input.voxelMeshImport)],
        ['readVoxelConversionSourceMetadata', (bridge) => bridge.readVoxelConversionSourceMetadata({ source: input.voxelSource.source })],
        ['previewVoxelConversion', (bridge) => bridge.previewVoxelConversion({ planId: 'fnv1a64:0000000000000101', expectedPlanHash: input.voxelPlanHash })],
        ['applyVoxelConversion', (bridge) => bridge.applyVoxelConversion({
                planId: 'fnv1a64:0000000000000101',
                expectedPlanHash: input.voxelPlanHash,
                expectedPreviewHash: input.voxelPreviewHash,
            })],
        ['exportVoxelConversionEvidence', (bridge) => bridge.exportVoxelConversionEvidence(input.voxelEvidence)],
        ['readVoxelModelInfo', (bridge) => bridge.readVoxelModelInfo(input.voxelModelInfo)],
        ['readVoxelModelWindow', (bridge) => bridge.readVoxelModelWindow(input.voxelModelWindow)],
        ['exportVoxelVolumeAsset', (bridge) => bridge.exportVoxelVolumeAsset(input.voxelExport)],
        ['saveVoxelVolumeAsset', (bridge) => bridge.saveVoxelVolumeAsset(input.voxelSave)],
        ['updateVoxelVolumeAssetPalette', (bridge) => bridge.updateVoxelVolumeAssetPalette(input.voxelPaletteUpdate)],
        ['initializeVoxelVolumeAuthoring', (bridge) => bridge.initializeVoxelVolumeAuthoring(input.voxelAuthoring)],
        ['loadVoxelVolumeAsset', (bridge) => bridge.loadVoxelVolumeAsset(input.voxelLoad)],
        ['unloadVoxelVolumeAsset', (bridge) => bridge.unloadVoxelVolumeAsset(input.voxelUnload)],
        ['validateVoxelAnnotationLayer', (bridge) => bridge.validateVoxelAnnotationLayer(input.annotationValidation)],
        ['loadVoxelAnnotationLayer', (bridge) => bridge.loadVoxelAnnotationLayer(input.annotationLoad)],
        ['readVoxelAnnotationQuery', (bridge) => bridge.readVoxelAnnotationQuery(input.annotationQuery)],
        ['applyVoxelAnnotationEdit', (bridge) => bridge.applyVoxelAnnotationEdit(input.annotationEdit)],
        ['exportVoxelAnnotationLayer', (bridge) => bridge.exportVoxelAnnotationLayer(input.annotationExport)],
        ['readVoxelEditHistory', (bridge) => bridge.readVoxelEditHistory(input.historyRead)],
        ['previewVoxelEditRevert', (bridge) => bridge.previewVoxelEditRevert(input.historyRevert)],
        ['applyVoxelEditRevert', (bridge) => bridge.applyVoxelEditRevert({ ...input.historyRevert, mode: 'apply_revert' })],
        ['undoVoxelEdit', (bridge) => bridge.undoVoxelEdit(input.historyUndo)],
        ['redoVoxelEdit', (bridge) => bridge.redoVoxelEdit(input.historyRedo)],
        ['readModelMaterialPreview', (bridge) => bridge.readModelMaterialPreview(input.materialPreview)],
        ['readSceneObjectSnapshot', (bridge) => bridge.readSceneObjectSnapshot()],
        ['applySceneObjectCommand', (bridge) => bridge.applySceneObjectCommand({ expectedDocumentHash: 1, command: { kind: 'select', id: null } })],
        ['readRenderDiffs', (bridge) => bridge.readRenderDiffs(frameCursor(0))],
        ['readProjectionFrame', (bridge) => bridge.readProjectionFrame(frameCursor(0))],
        ['createCamera', (bridge) => bridge.createCamera(input.cameraCreate)],
        ['applyCameraModeCommand', (bridge) => bridge.applyCameraModeCommand(input.cameraMode)],
        ['applyCameraNavigationInput', (bridge) => bridge.applyCameraNavigationInput(input.cameraNavigation)],
        ['readCameraControllerState', (bridge) => bridge.readCameraControllerState({ camera: input.cameraMode.camera })],
        ['applyFirstPersonCameraInput', (bridge) => bridge.applyFirstPersonCameraInput(input.cameraInput)],
        ['applyEnemyDirectNavMovement', (bridge) => bridge.applyEnemyDirectNavMovement({
                entity: 777, seedPosition: [0, 0.5, -2.6], target: [0, 1.62, 1.25], maxStepUnits: 0.35,
            })],
        ['readCameraProjection', (bridge) => bridge.readCameraProjection({ camera: input.cameraInput.camera, viewport: null })],
        ['getBuffer', (bridge) => bridge.getBuffer(0)],
        ['releaseBuffer', (bridge) => bridge.releaseBuffer(0)],
        ['loadProjectBundle', (bridge) => bridge.loadProjectBundle({ bundleSchemaVersion: 1, protocolVersion: 1, sceneId: 1 })],
        ['saveProjectBundle', (bridge) => bridge.saveProjectBundle()],
        ['getProjectBundleCompositionStatus', (bridge) => bridge.getProjectBundleCompositionStatus()],
        ['unloadProjectBundle', (bridge) => bridge.unloadProjectBundle()],
        ['loadReplayFixture', (bridge) => bridge.loadReplayFixture({ name: 'x', steps: 1 })],
        ['runReplayStep', (bridge) => bridge.runReplayStep(0)],
    ]);
}
/**
 * Builds the invocation fixture against the generated bridge-manifest catalog.
 * Duplicate, missing, and non-manifest methods fail while the module loads, so
 * additive verbs cannot silently escape the real native conformance sequence.
 */
export function composeNativeOperationInvocations(entries) {
    const invocations = new Map(entries);
    if (invocations.size !== entries.length) {
        throw new Error('native operation invocation fixture contains a duplicate facade method');
    }
    const expected = new Set(MANIFEST_OPERATIONS.map((operation) => operation.facadeMethod));
    const unexpected = [...invocations.keys()].filter((method) => !expected.has(method));
    const missing = [...expected].filter((method) => !invocations.has(method));
    if (unexpected.length > 0 || missing.length > 0) {
        throw new Error(`native operation invocation fixture drifted; missing=${missing.join(',')} unexpected=${unexpected.join(',')}`);
    }
    return invocations;
}
//# sourceMappingURL=native-operation-invocations.test-fixture.js.map
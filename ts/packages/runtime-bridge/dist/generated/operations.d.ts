export type BridgeSurface = 'stable' | 'quarantined';
export type BridgeErrorFamily = 'not_initialized' | 'invalid_input' | 'unknown_handle' | 'buffer_expired' | 'native_unavailable' | 'voxel_conversion_unavailable' | 'unsupported_source_asset' | 'source_hash_mismatch' | 'invalid_material_map' | 'output_limit_exceeded' | 'stale_authority_snapshot' | 'conversion_replay_mismatch' | 'operation_unimplemented' | 'internal';
export interface BridgeOperation {
    readonly capability: string;
    readonly errors: string;
    readonly facadeMethod: string;
    readonly input: string;
    readonly manifestName: string;
    readonly nativeWired: boolean;
    readonly output: string;
    readonly surface: BridgeSurface;
}
declare const BRIDGE_OPERATION_DESCRIPTORS: readonly [{
    readonly capability: "bundle_lifecycle";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "initializeEngine";
    readonly input: "protocol_runtime::EngineConfig";
    readonly manifestName: "initialize_engine";
    readonly nativeWired: true;
    readonly output: "EngineHandle";
    readonly surface: "stable";
}, {
    readonly capability: "time_simulation";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "stepSimulation";
    readonly input: "protocol_runtime::StepInputEnvelope";
    readonly manifestName: "step_simulation";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::StepResult";
    readonly surface: "stable";
}, {
    readonly capability: "time_simulation";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyTimeControlCommand";
    readonly input: "protocol_time_control::TimeControlCommand";
    readonly manifestName: "apply_time_control_command";
    readonly nativeWired: true;
    readonly output: "protocol_time_control::TimeControlReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "time_simulation";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readTimeControlState";
    readonly input: "Unit";
    readonly manifestName: "read_time_control_state";
    readonly nativeWired: true;
    readonly output: "protocol_time_control::TimeControlState";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "submitCommands";
    readonly input: "protocol_voxel::CommandBatch";
    readonly manifestName: "submit_commands";
    readonly nativeWired: true;
    readonly output: "protocol_voxel::CommandResult";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "pickVoxel";
    readonly input: "protocol_voxel::PickRay";
    readonly manifestName: "pick_voxel";
    readonly nativeWired: true;
    readonly output: "protocol_voxel::PickResult";
    readonly surface: "stable";
}, {
    readonly capability: "camera";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyCollisionConstrainedCameraInput";
    readonly input: "protocol_view::CollisionConstrainedCameraInputEnvelope";
    readonly manifestName: "apply_collision_constrained_camera_input";
    readonly nativeWired: true;
    readonly output: "protocol_view::CameraCollisionSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyGeneratedTunnelToRuntimeWorld";
    readonly input: "protocol_view::GeneratedTunnelRuntimeApplyRequest";
    readonly manifestName: "apply_generated_tunnel_to_runtime_world";
    readonly nativeWired: true;
    readonly output: "protocol_view::GeneratedTunnelRuntimeApplyReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "selectVoxel";
    readonly input: "protocol_view::ScreenPointToPickRayRequest";
    readonly manifestName: "select_voxel";
    readonly nativeWired: true;
    readonly output: "protocol_view::VoxelSelectionSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readVoxelMeshEvidence";
    readonly input: "protocol_render::VoxelMeshEvidenceRequest";
    readonly manifestName: "read_voxel_mesh_evidence";
    readonly nativeWired: true;
    readonly output: "protocol_render::VoxelMeshEvidenceSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "planVoxelConversion";
    readonly input: "protocol_voxel_conversion::VoxelConversionPlanRequest";
    readonly manifestName: "plan_voxel_conversion";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelConversionPlan";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "registerVoxelConversionSource";
    readonly input: "protocol_voxel_conversion::VoxelConversionSourceRegistrationRequest";
    readonly manifestName: "register_voxel_conversion_source";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelConversionSourceRegistration";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "registerVoxelConversionMeshAsset";
    readonly input: "protocol_voxel_conversion::VoxelConversionMeshAssetRegistrationRequest";
    readonly manifestName: "register_voxel_conversion_mesh_asset";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelConversionSourceRegistration";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "importVoxelConversionMeshSource";
    readonly input: "protocol_voxel_conversion::VoxelConversionMeshSourceImportRequest";
    readonly manifestName: "import_voxel_conversion_mesh_source";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelConversionMeshSourceImportReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readVoxelConversionSourceMetadata";
    readonly input: "protocol_voxel_conversion::VoxelConversionSourceMetadataRequest";
    readonly manifestName: "read_voxel_conversion_source_metadata";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelConversionSourceMetadataReadout";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "previewVoxelConversion";
    readonly input: "protocol_voxel_conversion::VoxelConversionPreviewRequest";
    readonly manifestName: "preview_voxel_conversion";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelConversionPreview";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyVoxelConversion";
    readonly input: "protocol_voxel_conversion::VoxelConversionApplyRequest";
    readonly manifestName: "apply_voxel_conversion";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelConversionReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "replay_evidence";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "exportVoxelConversionEvidence";
    readonly input: "protocol_voxel_conversion::VoxelConversionEvidenceRef[]";
    readonly manifestName: "export_voxel_conversion_evidence";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelConversionEvidenceRef[]";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readVoxelModelInfo";
    readonly input: "protocol_voxel_conversion::VoxelModelInfoRequest";
    readonly manifestName: "read_voxel_model_info";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelModelInfoReadout";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readVoxelModelWindow";
    readonly input: "protocol_voxel_conversion::VoxelModelWindowRequest";
    readonly manifestName: "read_voxel_model_window";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_conversion::VoxelModelWindowReadout";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "exportVoxelVolumeAsset";
    readonly input: "protocol_voxel_asset::VoxelVolumeAssetExportRequest";
    readonly manifestName: "export_voxel_volume_asset";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_asset::VoxelVolumeAssetExportReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "saveVoxelVolumeAsset";
    readonly input: "protocol_voxel_asset::VoxelVolumeAssetSaveRequest";
    readonly manifestName: "save_voxel_volume_asset";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_asset::VoxelVolumeAssetSaveReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "updateVoxelVolumeAssetPalette";
    readonly input: "protocol_voxel_asset::VoxelVolumeAssetPaletteUpdateRequest";
    readonly manifestName: "update_voxel_volume_asset_palette";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_asset::VoxelVolumeAssetPaletteUpdateReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "initializeVoxelVolumeAuthoring";
    readonly input: "protocol_voxel_asset::VoxelVolumeAuthoringInitializeRequest";
    readonly manifestName: "initialize_voxel_volume_authoring";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_asset::VoxelVolumeAuthoringInitializeReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "loadVoxelVolumeAsset";
    readonly input: "protocol_voxel_asset::VoxelVolumeAssetLoadRequest";
    readonly manifestName: "load_voxel_volume_asset";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_asset::VoxelVolumeAssetLoadReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "unloadVoxelVolumeAsset";
    readonly input: "protocol_voxel_asset::VoxelVolumeAssetUnloadRequest";
    readonly manifestName: "unload_voxel_volume_asset";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_asset::VoxelVolumeAssetUnloadReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "validateVoxelAnnotationLayer";
    readonly input: "protocol_voxel_annotation::VoxelAnnotationLayerValidationRequest";
    readonly manifestName: "validate_voxel_annotation_layer";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_annotation::VoxelAnnotationLayerValidationReport";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "loadVoxelAnnotationLayer";
    readonly input: "protocol_voxel_annotation::VoxelAnnotationLayerLoadRequest";
    readonly manifestName: "load_voxel_annotation_layer";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_annotation::VoxelAnnotationLayerLoadReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readVoxelAnnotationQuery";
    readonly input: "protocol_voxel_annotation::VoxelAnnotationQueryRequest";
    readonly manifestName: "read_voxel_annotation_query";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_annotation::VoxelAnnotationQueryReadout";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyVoxelAnnotationEdit";
    readonly input: "protocol_voxel_annotation::VoxelAnnotationEditRequest";
    readonly manifestName: "apply_voxel_annotation_edit";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_annotation::VoxelAnnotationEditReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "exportVoxelAnnotationLayer";
    readonly input: "protocol_voxel_annotation::VoxelAnnotationLayerExportRequest";
    readonly manifestName: "export_voxel_annotation_layer";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_annotation::VoxelAnnotationLayerExportReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readVoxelEditHistory";
    readonly input: "protocol_voxel_edit_history::VoxelEditHistoryReadRequest";
    readonly manifestName: "read_voxel_edit_history";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_edit_history::VoxelEditHistorySummary";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "previewVoxelEditRevert";
    readonly input: "protocol_voxel_edit_history::VoxelEditHistoryRevertRequest";
    readonly manifestName: "preview_voxel_edit_revert";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_edit_history::VoxelEditHistoryRevertReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyVoxelEditRevert";
    readonly input: "protocol_voxel_edit_history::VoxelEditHistoryRevertRequest";
    readonly manifestName: "apply_voxel_edit_revert";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_edit_history::VoxelEditHistoryRevertReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "undoVoxelEdit";
    readonly input: "protocol_voxel_edit_history::VoxelEditHistoryUndoRequest";
    readonly manifestName: "undo_voxel_edit";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_edit_history::VoxelEditHistoryUndoReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "redoVoxelEdit";
    readonly input: "protocol_voxel_edit_history::VoxelEditHistoryRedoRequest";
    readonly manifestName: "redo_voxel_edit";
    readonly nativeWired: true;
    readonly output: "protocol_voxel_edit_history::VoxelEditHistoryRedoReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "loadFpsRuntimeSession";
    readonly input: "protocol_runtime::FpsRuntimeSessionLoadRequest";
    readonly manifestName: "load_fps_runtime_session";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::FpsRuntimeSessionSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readFpsRuntimeSession";
    readonly input: "Unit";
    readonly manifestName: "read_fps_runtime_session";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::FpsRuntimeSessionSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyFpsPrimaryFire";
    readonly input: "protocol_runtime::FpsPrimaryFireRequest";
    readonly manifestName: "apply_fps_primary_fire";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::FpsPrimaryFireResult";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "invokeGameExtensionWeaponEffect";
    readonly input: "protocol_runtime::GameExtensionWeaponEffectInvocationRequest";
    readonly manifestName: "invoke_game_extension_weapon_effect";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::GameExtensionWeaponEffectInvocationResult";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "validateGameRuleCatalog";
    readonly input: "protocol_game_rules::GameRuleCatalog";
    readonly manifestName: "validate_game_rule_catalog";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::GameRuleCatalogValidationReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "submitGameRuleEffectIntent";
    readonly input: "protocol_runtime::GameRuleEffectIntentRequest";
    readonly manifestName: "submit_game_rule_effect_intent";
    readonly nativeWired: true;
    readonly output: "protocol_game_rules::GameRuleResolutionReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readGameRuleRuntimeReadout";
    readonly input: "Unit";
    readonly manifestName: "read_game_rule_runtime_readout";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::GameRuleRuntimeReadout";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "restartFpsRuntimeSession";
    readonly input: "protocol_runtime::FpsRuntimeSessionRestartRequest";
    readonly manifestName: "restart_fps_runtime_session";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::FpsRuntimeSessionSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readFpsEncounterDirector";
    readonly input: "protocol_runtime::FpsEncounterLifecycleInput";
    readonly manifestName: "read_fps_encounter_director";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::FpsEncounterDirectorSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "gameplay";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyFpsEncounterTransition";
    readonly input: "protocol_runtime::FpsEncounterTransitionRequest";
    readonly manifestName: "apply_fps_encounter_transition";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::FpsEncounterTransitionResult";
    readonly surface: "stable";
}, {
    readonly capability: "projection";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readProjectionFrame";
    readonly input: "FrameCursor";
    readonly manifestName: "read_projection_frame";
    readonly nativeWired: true;
    readonly output: "protocol_presentation::RuntimeProjectionFrame";
    readonly surface: "stable";
}, {
    readonly capability: "projection";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readRenderDiffs";
    readonly input: "FrameCursor";
    readonly manifestName: "read_render_diffs";
    readonly nativeWired: true;
    readonly output: "protocol_render::RenderFrameDiffDescriptor";
    readonly surface: "stable";
}, {
    readonly capability: "scene_entities";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readModelMaterialPreview";
    readonly input: "protocol_render::ModelMaterialPreviewRequest";
    readonly manifestName: "read_model_material_preview";
    readonly nativeWired: true;
    readonly output: "protocol_render::ModelMaterialPreviewSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "scene_entities";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readSceneObjectSnapshot";
    readonly input: "Unit";
    readonly manifestName: "read_scene_object_snapshot";
    readonly nativeWired: true;
    readonly output: "protocol_scene::SceneObjectSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "scene_entities";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applySceneObjectCommand";
    readonly input: "protocol_scene::SceneObjectCommandRequest";
    readonly manifestName: "apply_scene_object_command";
    readonly nativeWired: true;
    readonly output: "protocol_scene::SceneObjectCommandResult";
    readonly surface: "stable";
}, {
    readonly capability: "input";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "configureInputSession";
    readonly input: "protocol_input::InputSessionConfigureRequest";
    readonly manifestName: "configure_input_session";
    readonly nativeWired: true;
    readonly output: "protocol_input::InputSessionSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "input";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyInputContextCommand";
    readonly input: "protocol_input::InputContextCommand";
    readonly manifestName: "apply_input_context_command";
    readonly nativeWired: true;
    readonly output: "protocol_input::InputContextChangeReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "input";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "submitRawInput";
    readonly input: "protocol_input::RawInputSample";
    readonly manifestName: "submit_raw_input";
    readonly nativeWired: true;
    readonly output: "protocol_input::InputResolutionReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "input";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "replayResolvedInputAction";
    readonly input: "protocol_input::RecordedInputAction";
    readonly manifestName: "replay_resolved_input_action";
    readonly nativeWired: true;
    readonly output: "protocol_input::InputActionReplayReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "input";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readInputContextState";
    readonly input: "Unit";
    readonly manifestName: "read_input_context_state";
    readonly nativeWired: true;
    readonly output: "protocol_input::InputContextStackState";
    readonly surface: "stable";
}, {
    readonly capability: "camera";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "createCamera";
    readonly input: "protocol_view::CameraCreateRequest";
    readonly manifestName: "create_camera";
    readonly nativeWired: true;
    readonly output: "protocol_view::CameraSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "camera";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyCameraModeCommand";
    readonly input: "protocol_view::CameraModeCommand";
    readonly manifestName: "apply_camera_mode_command";
    readonly nativeWired: true;
    readonly output: "protocol_view::CameraModeChangeReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "camera";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyCameraNavigationInput";
    readonly input: "protocol_view::CameraNavigationInputEnvelope";
    readonly manifestName: "apply_camera_navigation_input";
    readonly nativeWired: true;
    readonly output: "protocol_view::CameraNavigationReceipt";
    readonly surface: "stable";
}, {
    readonly capability: "camera";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readCameraControllerState";
    readonly input: "protocol_view::CameraControllerReadRequest";
    readonly manifestName: "read_camera_controller_state";
    readonly nativeWired: true;
    readonly output: "protocol_view::CameraControllerState";
    readonly surface: "stable";
}, {
    readonly capability: "camera";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyFirstPersonCameraInput";
    readonly input: "protocol_view::FirstPersonCameraInputEnvelope";
    readonly manifestName: "apply_first_person_camera_input";
    readonly nativeWired: true;
    readonly output: "protocol_view::CameraSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "scene_entities";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "applyEnemyDirectNavMovement";
    readonly input: "protocol_runtime::EnemyDirectNavMovementRequest";
    readonly manifestName: "apply_enemy_direct_nav_movement";
    readonly nativeWired: true;
    readonly output: "protocol_runtime::EnemyDirectNavMovementResult";
    readonly surface: "stable";
}, {
    readonly capability: "camera";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "readCameraProjection";
    readonly input: "protocol_view::CameraProjectionRequest";
    readonly manifestName: "read_camera_projection";
    readonly nativeWired: true;
    readonly output: "protocol_view::CameraProjectionSnapshot";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "getBuffer";
    readonly input: "RuntimeBufferHandle";
    readonly manifestName: "get_buffer";
    readonly nativeWired: true;
    readonly output: "RuntimeBufferView";
    readonly surface: "stable";
}, {
    readonly capability: "voxel_assets_buffers";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "releaseBuffer";
    readonly input: "RuntimeBufferHandle";
    readonly manifestName: "release_buffer";
    readonly nativeWired: true;
    readonly output: "Unit";
    readonly surface: "stable";
}, {
    readonly capability: "bundle_lifecycle";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "loadProjectBundle";
    readonly input: "protocol_project_bundle::ProjectBundleManifest";
    readonly manifestName: "load_project_bundle";
    readonly nativeWired: true;
    readonly output: "protocol_diagnostics::DiagnosticReportSet";
    readonly surface: "stable";
}, {
    readonly capability: "bundle_lifecycle";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "saveProjectBundle";
    readonly input: "Unit";
    readonly manifestName: "save_project_bundle";
    readonly nativeWired: true;
    readonly output: "protocol_project_bundle::SaveSummary";
    readonly surface: "stable";
}, {
    readonly capability: "bundle_lifecycle";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "getProjectBundleCompositionStatus";
    readonly input: "Unit";
    readonly manifestName: "get_project_bundle_composition_status";
    readonly nativeWired: true;
    readonly output: "protocol_diagnostics::DiagnosticReportSet";
    readonly surface: "stable";
}, {
    readonly capability: "bundle_lifecycle";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "unloadProjectBundle";
    readonly input: "Unit";
    readonly manifestName: "unload_project_bundle";
    readonly nativeWired: true;
    readonly output: "Unit";
    readonly surface: "stable";
}, {
    readonly capability: "replay_evidence";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "loadReplayFixture";
    readonly input: "protocol_replay::ReplayFixture";
    readonly manifestName: "load_replay_fixture";
    readonly nativeWired: false;
    readonly output: "ReplaySessionHandle";
    readonly surface: "quarantined";
}, {
    readonly capability: "replay_evidence";
    readonly errors: "RuntimeBridgeError";
    readonly facadeMethod: "runReplayStep";
    readonly input: "ReplaySessionHandle";
    readonly manifestName: "run_replay_step";
    readonly nativeWired: false;
    readonly output: "protocol_replay::ReplayStepReport";
    readonly surface: "quarantined";
}];
export type BridgeOperationDescriptor = (typeof BRIDGE_OPERATION_DESCRIPTORS)[number];
export declare const MANIFEST_OPERATIONS: readonly BridgeOperation[];
export declare const NATIVE_WIRED_OPERATIONS: ReadonlySet<string>;
export {};
//# sourceMappingURL=operations.d.ts.map
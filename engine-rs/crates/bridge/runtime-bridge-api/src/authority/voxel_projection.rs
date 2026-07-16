use super::*;

impl EngineBridge {
    pub(super) fn read_render_diffs_authority(
        &mut self,
        cursor: u64,
    ) -> BridgeResult<RenderFrameDiff> {
        self.require_initialized("read_render_diffs")?;
        if self.voxel.voxel.is_none() {
            return Err(RuntimeBridgeError::new(
                RuntimeBridgeErrorKind::NotInitialized,
                "read_render_diffs called before voxel authority was initialized",
            ));
        }
        self.drain_voxel_projection_frame(cursor)
    }

    pub(super) fn drain_voxel_projection_frame(
        &mut self,
        cursor: u64,
    ) -> BridgeResult<RenderFrameDiff> {
        let mut frame = std::mem::take(&mut self.projection.pending_voxel_frame);
        let Some(world) = self.voxel.voxel.as_mut() else {
            return Ok(frame);
        };
        let projected = self.projection.voxel_projector.project_dirty(world);
        frame.ops.extend(projected.ops);
        if !self.projection.voxel_projector.diagnostics().is_empty() {
            self.record_developer_console(DeveloperConsoleEmission {
                severity: DiagnosticSeverity::Error,
                category: DeveloperConsoleCategory::Resource,
                source: DeveloperConsoleSource::Projection,
                message: "one or more voxel chunks could not be projected".to_owned(),
                correlation: Some(format!("projection-cursor:{cursor}")),
                authority_tick: Some(self.time.authority_tick),
                detail: DeveloperConsoleDetail {
                    code: "voxel_projection_failed".to_owned(),
                    operation: Some("read_workspace_authoring_projection".to_owned()),
                    resource_kind: Some("voxel_chunk_mesh".to_owned()),
                    resource_id: None,
                    reason: Some("voxel mesh exceeded the supported projection limits".to_owned()),
                },
            });
        }
        Ok(frame)
    }
}

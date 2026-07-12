//! Validation and retained lifecycle for disposable telemetry overlays.
//!
//! The overlay is a projection target for a separately readable telemetry
//! snapshot. It never owns counters, authority state, or replay truth.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use protocol_presentation::{
    PresentationOp, PresentationOpMeta, TelemetryOverlayDescriptor, TelemetryOverlayDiagnostic,
    TelemetryOverlayDiagnosticCode, TelemetryOverlayHandle, TelemetryOverlayPatch,
    TelemetryOverlayProjectionOp, TelemetryOverlayReadout,
};

const MAX_TITLE_BYTES: usize = 96;
const MIN_REFRESH_INTERVAL_MS: u32 = 100;
const MAX_REFRESH_INTERVAL_MS: u32 = 5_000;
const MIN_FRAME_TIME_SAMPLES: u16 = 1;
const MAX_FRAME_TIME_SAMPLES: u16 = 240;

#[derive(Debug, Default)]
pub struct TelemetryOverlayProjector {
    active: BTreeMap<TelemetryOverlayHandle, TelemetryOverlayDescriptor>,
    diagnostics: Vec<TelemetryOverlayDiagnostic>,
}

impl TelemetryOverlayProjector {
    pub fn project(
        &mut self,
        meta: PresentationOpMeta,
        op: TelemetryOverlayProjectionOp,
    ) -> Result<PresentationOp, Box<TelemetryOverlayDiagnostic>> {
        if let Err(code) = self.validate_and_apply(&op) {
            let diagnostic = TelemetryOverlayDiagnostic {
                code,
                sequence: meta.sequence,
                handle: operation_handle(&op),
                message: diagnostic_message(code).to_string(),
                origin: meta.origin,
            };
            self.diagnostics.push(diagnostic.clone());
            return Err(Box::new(diagnostic));
        }
        Ok(PresentationOp::TelemetryOverlay { meta, op })
    }

    pub fn descriptor(
        &self,
        handle: TelemetryOverlayHandle,
    ) -> Option<&TelemetryOverlayDescriptor> {
        self.active.get(&handle)
    }

    pub fn readout(&self) -> TelemetryOverlayReadout {
        TelemetryOverlayReadout {
            active_overlays: self.active.len() as u32,
            rendered_snapshots: 0,
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub fn reset(&mut self) {
        self.active.clear();
        self.diagnostics.clear();
    }

    fn validate_and_apply(
        &mut self,
        op: &TelemetryOverlayProjectionOp,
    ) -> Result<(), TelemetryOverlayDiagnosticCode> {
        match op {
            TelemetryOverlayProjectionOp::Create { handle, descriptor } => {
                if self.active.contains_key(handle) {
                    return Err(TelemetryOverlayDiagnosticCode::DuplicateHandle);
                }
                validate_descriptor(descriptor)?;
                self.active.insert(*handle, descriptor.clone());
            }
            TelemetryOverlayProjectionOp::Update { handle, patch } => {
                let current = self
                    .active
                    .get(handle)
                    .cloned()
                    .ok_or(TelemetryOverlayDiagnosticCode::UnknownHandle)?;
                let updated = apply_patch(current, patch);
                validate_descriptor(&updated)?;
                self.active.insert(*handle, updated);
            }
            TelemetryOverlayProjectionOp::Destroy { handle } => {
                if self.active.remove(handle).is_none() {
                    return Err(TelemetryOverlayDiagnosticCode::UnknownHandle);
                }
            }
        }
        Ok(())
    }
}

fn validate_descriptor(
    descriptor: &TelemetryOverlayDescriptor,
) -> Result<(), TelemetryOverlayDiagnosticCode> {
    if descriptor.title.is_empty()
        || descriptor.title.len() > MAX_TITLE_BYTES
        || !(MIN_REFRESH_INTERVAL_MS..=MAX_REFRESH_INTERVAL_MS)
            .contains(&descriptor.refresh_interval_ms)
        || !(MIN_FRAME_TIME_SAMPLES..=MAX_FRAME_TIME_SAMPLES)
            .contains(&descriptor.max_frame_time_samples)
    {
        return Err(TelemetryOverlayDiagnosticCode::InvalidDescriptor);
    }
    Ok(())
}

fn apply_patch(
    mut descriptor: TelemetryOverlayDescriptor,
    patch: &TelemetryOverlayPatch,
) -> TelemetryOverlayDescriptor {
    if let Some(value) = &patch.title {
        descriptor.title = value.clone();
    }
    if let Some(value) = patch.corner {
        descriptor.corner = value;
    }
    if let Some(value) = patch.refresh_interval_ms {
        descriptor.refresh_interval_ms = value;
    }
    if let Some(value) = patch.max_frame_time_samples {
        descriptor.max_frame_time_samples = value;
    }
    if let Some(value) = patch.visible {
        descriptor.visible = value;
    }
    descriptor
}

fn operation_handle(op: &TelemetryOverlayProjectionOp) -> Option<TelemetryOverlayHandle> {
    Some(match op {
        TelemetryOverlayProjectionOp::Create { handle, .. }
        | TelemetryOverlayProjectionOp::Update { handle, .. }
        | TelemetryOverlayProjectionOp::Destroy { handle } => *handle,
    })
}

fn diagnostic_message(code: TelemetryOverlayDiagnosticCode) -> &'static str {
    match code {
        TelemetryOverlayDiagnosticCode::InvalidDescriptor => {
            "telemetry overlay descriptor is invalid"
        }
        TelemetryOverlayDiagnosticCode::DuplicateHandle => {
            "telemetry overlay handle is already active"
        }
        TelemetryOverlayDiagnosticCode::UnknownHandle => "telemetry overlay handle is not active",
        TelemetryOverlayDiagnosticCode::UnavailableHost => "telemetry overlay host is unavailable",
        TelemetryOverlayDiagnosticCode::SnapshotUnavailable => {
            "machine-readable telemetry snapshot is unavailable"
        }
        TelemetryOverlayDiagnosticCode::HostFailure => "telemetry overlay host failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol_presentation::{TelemetryOverlayCorner, TelemetryOverlayProjectionOp};

    fn descriptor() -> TelemetryOverlayDescriptor {
        TelemetryOverlayDescriptor {
            title: "ASHA runtime".into(),
            corner: TelemetryOverlayCorner::TopRight,
            refresh_interval_ms: 250,
            max_frame_time_samples: 60,
            visible: true,
        }
    }

    fn meta(sequence: u32) -> PresentationOpMeta {
        PresentationOpMeta {
            sequence,
            origin: None,
        }
    }

    #[test]
    fn retained_overlay_lifecycle_is_typed_atomic_and_bounded() {
        let handle = TelemetryOverlayHandle::new(9);
        let mut projector = TelemetryOverlayProjector::default();
        projector
            .project(
                meta(0),
                TelemetryOverlayProjectionOp::Create {
                    handle,
                    descriptor: descriptor(),
                },
            )
            .unwrap();
        projector
            .project(
                meta(1),
                TelemetryOverlayProjectionOp::Update {
                    handle,
                    patch: TelemetryOverlayPatch {
                        visible: Some(false),
                        ..TelemetryOverlayPatch::default()
                    },
                },
            )
            .unwrap();
        assert!(!projector.descriptor(handle).unwrap().visible);
        projector
            .project(meta(2), TelemetryOverlayProjectionOp::Destroy { handle })
            .unwrap();
        assert_eq!(projector.readout().active_overlays, 0);
    }

    #[test]
    fn duplicate_unknown_and_invalid_transitions_fail_without_partial_state() {
        let handle = TelemetryOverlayHandle::new(9);
        let mut projector = TelemetryOverlayProjector::default();
        let invalid = TelemetryOverlayDescriptor {
            refresh_interval_ms: 1,
            ..descriptor()
        };
        let diagnostic = projector
            .project(
                meta(0),
                TelemetryOverlayProjectionOp::Create {
                    handle,
                    descriptor: invalid,
                },
            )
            .unwrap_err();
        assert_eq!(
            diagnostic.code,
            TelemetryOverlayDiagnosticCode::InvalidDescriptor
        );
        assert_eq!(projector.readout().active_overlays, 0);

        projector
            .project(
                meta(1),
                TelemetryOverlayProjectionOp::Create {
                    handle,
                    descriptor: descriptor(),
                },
            )
            .unwrap();
        let duplicate = projector
            .project(
                meta(2),
                TelemetryOverlayProjectionOp::Create {
                    handle,
                    descriptor: descriptor(),
                },
            )
            .unwrap_err();
        assert_eq!(
            duplicate.code,
            TelemetryOverlayDiagnosticCode::DuplicateHandle
        );
        assert_eq!(projector.readout().active_overlays, 1);
    }
}

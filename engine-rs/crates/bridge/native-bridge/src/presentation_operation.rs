use napi_derive::napi;
use runtime_bridge_api::PresentationOp;

use super::{
    animation_projection::NativeAnimationProjectionOp, audio_projection::NativeAudioProjectionOp,
    billboard_projection::NativeBillboardProjectionOp,
    particle_projection::NativeParticleProjectionOp,
    telemetry_overlay_projection::NativeTelemetryOverlayProjectionOp, NativePresentationOpMeta,
};

#[napi(object)]
pub struct NativePresentationOp {
    pub domain: String,
    pub meta: NativePresentationOpMeta,
    pub audio_op: Option<NativeAudioProjectionOp>,
    pub billboard_op: Option<NativeBillboardProjectionOp>,
    pub particle_op: Option<NativeParticleProjectionOp>,
    pub telemetry_overlay_op: Option<NativeTelemetryOverlayProjectionOp>,
    pub animation_op: Option<NativeAnimationProjectionOp>,
}

impl From<PresentationOp> for NativePresentationOp {
    fn from(value: PresentationOp) -> Self {
        match value {
            PresentationOp::Audio { meta, op } => Self {
                domain: "audio".to_string(),
                meta: meta.into(),
                audio_op: Some(op.into()),
                billboard_op: None,
                particle_op: None,
                telemetry_overlay_op: None,
                animation_op: None,
            },
            PresentationOp::Billboard { meta, op } => Self {
                domain: "billboard".to_string(),
                meta: meta.into(),
                audio_op: None,
                billboard_op: Some(op.into()),
                particle_op: None,
                telemetry_overlay_op: None,
                animation_op: None,
            },
            PresentationOp::Particle { meta, op } => Self {
                domain: "particle".to_string(),
                meta: meta.into(),
                audio_op: None,
                billboard_op: None,
                particle_op: Some(op.into()),
                telemetry_overlay_op: None,
                animation_op: None,
            },
            PresentationOp::TelemetryOverlay { meta, op } => Self {
                domain: "telemetryOverlay".to_string(),
                meta: meta.into(),
                audio_op: None,
                billboard_op: None,
                particle_op: None,
                telemetry_overlay_op: Some(op.into()),
                animation_op: None,
            },
            PresentationOp::Animation { meta, op } => Self {
                domain: "animation".to_string(),
                meta: meta.into(),
                audio_op: None,
                billboard_op: None,
                particle_op: None,
                telemetry_overlay_op: None,
                animation_op: Some(op.into()),
            },
        }
    }
}

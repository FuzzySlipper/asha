use napi_derive::napi;
use runtime_bridge_api::{
    AudioBus, AudioEmitter, AudioProjectionOp, AudioSourceDescriptor, AudioSourcePatch,
};

#[napi(object)]
pub struct NativeAudioEmitter {
    pub kind: String,
    pub position: Option<Vec<f64>>,
    pub entity: Option<i64>,
    pub offset: Option<Vec<f64>>,
}

impl From<AudioEmitter> for NativeAudioEmitter {
    fn from(value: AudioEmitter) -> Self {
        match value {
            AudioEmitter::Global2d => Self {
                kind: "global2d".to_string(),
                position: None,
                entity: None,
                offset: None,
            },
            AudioEmitter::World3d { position } => Self {
                kind: "world3d".to_string(),
                position: Some(position.into_iter().map(f64::from).collect()),
                entity: None,
                offset: None,
            },
            AudioEmitter::EntityAttached { entity, offset } => Self {
                kind: "entityAttached".to_string(),
                position: None,
                entity: Some(entity as i64),
                offset: Some(offset.into_iter().map(f64::from).collect()),
            },
        }
    }
}

#[napi(object)]
pub struct NativeAudioClipRef {
    pub asset: String,
    pub content_hash: String,
}

#[napi(object)]
pub struct NativeAudioSourceDescriptor {
    pub clip: NativeAudioClipRef,
    pub bus: String,
    pub volume: f64,
    pub pitch: f64,
    pub looping: bool,
    pub spatial_blend: f64,
    pub attenuation: f64,
    pub pan: f64,
    pub emitter: NativeAudioEmitter,
}

fn native_audio_bus(value: AudioBus) -> String {
    match value {
        AudioBus::Sfx => "sfx",
        AudioBus::Ambient => "ambient",
        AudioBus::Ui => "ui",
    }
    .to_string()
}

impl From<AudioSourceDescriptor> for NativeAudioSourceDescriptor {
    fn from(value: AudioSourceDescriptor) -> Self {
        Self {
            clip: NativeAudioClipRef {
                asset: value.clip.asset,
                content_hash: value.clip.content_hash,
            },
            bus: native_audio_bus(value.bus),
            volume: f64::from(value.volume),
            pitch: f64::from(value.pitch),
            looping: value.looping,
            spatial_blend: f64::from(value.spatial_blend),
            attenuation: f64::from(value.attenuation),
            pan: f64::from(value.pan),
            emitter: value.emitter.into(),
        }
    }
}

#[napi(object)]
pub struct NativeAudioSourcePatch {
    pub volume: Option<f64>,
    pub pitch: Option<f64>,
    pub looping: Option<bool>,
    pub spatial_blend: Option<f64>,
    pub attenuation: Option<f64>,
    pub pan: Option<f64>,
    pub emitter: Option<NativeAudioEmitter>,
}

impl From<AudioSourcePatch> for NativeAudioSourcePatch {
    fn from(value: AudioSourcePatch) -> Self {
        Self {
            volume: value.volume.map(f64::from),
            pitch: value.pitch.map(f64::from),
            looping: value.looping,
            spatial_blend: value.spatial_blend.map(f64::from),
            attenuation: value.attenuation.map(f64::from),
            pan: value.pan.map(f64::from),
            emitter: value.emitter.map(NativeAudioEmitter::from),
        }
    }
}

#[napi(object)]
pub struct NativeAudioProjectionOp {
    pub op: String,
    pub signal_id: Option<String>,
    pub handle: Option<i64>,
    pub descriptor: Option<NativeAudioSourceDescriptor>,
    pub patch: Option<NativeAudioSourcePatch>,
}

impl From<AudioProjectionOp> for NativeAudioProjectionOp {
    fn from(value: AudioProjectionOp) -> Self {
        match value {
            AudioProjectionOp::Emit {
                signal_id,
                descriptor,
            } => Self {
                op: "emit".to_string(),
                signal_id: Some(signal_id),
                handle: None,
                descriptor: Some(descriptor.into()),
                patch: None,
            },
            AudioProjectionOp::Create { handle, descriptor } => Self {
                op: "create".to_string(),
                signal_id: None,
                handle: Some(handle.raw() as i64),
                descriptor: Some(descriptor.into()),
                patch: None,
            },
            AudioProjectionOp::Update { handle, patch } => Self {
                op: "update".to_string(),
                signal_id: None,
                handle: Some(handle.raw() as i64),
                descriptor: None,
                patch: Some(patch.into()),
            },
            AudioProjectionOp::Destroy { handle } => Self {
                op: "destroy".to_string(),
                signal_id: None,
                handle: Some(handle.raw() as i64),
                descriptor: None,
                patch: None,
            },
        }
    }
}

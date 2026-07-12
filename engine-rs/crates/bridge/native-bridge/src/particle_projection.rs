use napi_derive::napi;
use runtime_bridge_api::{
    ParticleAnchor, ParticleColorKey, ParticleEmitterDescriptor, ParticleEmitterPatch,
    ParticleProjectionOp, ParticleScalarKey, ParticleSpriteRef,
};

#[napi(object)]
pub struct NativeParticleAnchor {
    pub kind: String,
    pub position: Option<Vec<f64>>,
    pub entity: Option<i64>,
    pub offset: Option<Vec<f64>>,
}

impl From<ParticleAnchor> for NativeParticleAnchor {
    fn from(value: ParticleAnchor) -> Self {
        match value {
            ParticleAnchor::World { position } => Self {
                kind: "world".to_string(),
                position: Some(position.into_iter().map(f64::from).collect()),
                entity: None,
                offset: None,
            },
            ParticleAnchor::EntityAttached { entity, offset } => Self {
                kind: "entityAttached".to_string(),
                position: None,
                entity: Some(entity as i64),
                offset: Some(offset.into_iter().map(f64::from).collect()),
            },
        }
    }
}

#[napi(object)]
pub struct NativeParticleSpriteRef {
    pub asset: String,
    pub content_hash: String,
    pub frame_count: u32,
}

impl From<ParticleSpriteRef> for NativeParticleSpriteRef {
    fn from(value: ParticleSpriteRef) -> Self {
        Self {
            asset: value.asset,
            content_hash: value.content_hash,
            frame_count: u32::from(value.frame_count),
        }
    }
}

#[napi(object)]
pub struct NativeParticleScalarKey {
    pub age: f64,
    pub value: f64,
}

impl From<ParticleScalarKey> for NativeParticleScalarKey {
    fn from(value: ParticleScalarKey) -> Self {
        Self {
            age: f64::from(value.age),
            value: f64::from(value.value),
        }
    }
}

#[napi(object)]
pub struct NativeParticleColorKey {
    pub age: f64,
    pub color: Vec<f64>,
}

impl From<ParticleColorKey> for NativeParticleColorKey {
    fn from(value: ParticleColorKey) -> Self {
        Self {
            age: f64::from(value.age),
            color: value.color.into_iter().map(f64::from).collect(),
        }
    }
}

#[napi(object)]
pub struct NativeParticleEmitterDescriptor {
    pub anchor: NativeParticleAnchor,
    pub sprite: NativeParticleSpriteRef,
    pub rate_per_second: f64,
    pub burst_count: u32,
    pub lifetime_seconds: Vec<f64>,
    pub velocity_min: Vec<f64>,
    pub velocity_max: Vec<f64>,
    pub acceleration: Vec<f64>,
    pub size_curve: Vec<NativeParticleScalarKey>,
    pub color_curve: Vec<NativeParticleColorKey>,
    pub flipbook_frames_per_second: f64,
    pub seed: i64,
    pub max_particles: u32,
    pub visible: bool,
}

impl From<ParticleEmitterDescriptor> for NativeParticleEmitterDescriptor {
    fn from(value: ParticleEmitterDescriptor) -> Self {
        Self {
            anchor: value.anchor.into(),
            sprite: value.sprite.into(),
            rate_per_second: f64::from(value.rate_per_second),
            burst_count: value.burst_count,
            lifetime_seconds: value.lifetime_seconds.into_iter().map(f64::from).collect(),
            velocity_min: value.velocity_min.into_iter().map(f64::from).collect(),
            velocity_max: value.velocity_max.into_iter().map(f64::from).collect(),
            acceleration: value.acceleration.into_iter().map(f64::from).collect(),
            size_curve: value
                .size_curve
                .into_iter()
                .map(NativeParticleScalarKey::from)
                .collect(),
            color_curve: value
                .color_curve
                .into_iter()
                .map(NativeParticleColorKey::from)
                .collect(),
            flipbook_frames_per_second: f64::from(value.flipbook_frames_per_second),
            seed: value.seed as i64,
            max_particles: value.max_particles,
            visible: value.visible,
        }
    }
}

#[napi(object)]
pub struct NativeParticleEmitterPatch {
    pub anchor: Option<NativeParticleAnchor>,
    pub sprite: Option<NativeParticleSpriteRef>,
    pub rate_per_second: Option<f64>,
    pub burst_count: Option<u32>,
    pub lifetime_seconds: Option<Vec<f64>>,
    pub velocity_min: Option<Vec<f64>>,
    pub velocity_max: Option<Vec<f64>>,
    pub acceleration: Option<Vec<f64>>,
    pub size_curve: Option<Vec<NativeParticleScalarKey>>,
    pub color_curve: Option<Vec<NativeParticleColorKey>>,
    pub flipbook_frames_per_second: Option<f64>,
    pub max_particles: Option<u32>,
    pub visible: Option<bool>,
}

impl From<ParticleEmitterPatch> for NativeParticleEmitterPatch {
    fn from(value: ParticleEmitterPatch) -> Self {
        Self {
            anchor: value.anchor.map(NativeParticleAnchor::from),
            sprite: value.sprite.map(NativeParticleSpriteRef::from),
            rate_per_second: value.rate_per_second.map(f64::from),
            burst_count: value.burst_count,
            lifetime_seconds: value
                .lifetime_seconds
                .map(|range| range.into_iter().map(f64::from).collect()),
            velocity_min: value
                .velocity_min
                .map(|range| range.into_iter().map(f64::from).collect()),
            velocity_max: value
                .velocity_max
                .map(|range| range.into_iter().map(f64::from).collect()),
            acceleration: value
                .acceleration
                .map(|range| range.into_iter().map(f64::from).collect()),
            size_curve: value.size_curve.map(|keys| {
                keys.into_iter()
                    .map(NativeParticleScalarKey::from)
                    .collect()
            }),
            color_curve: value
                .color_curve
                .map(|keys| keys.into_iter().map(NativeParticleColorKey::from).collect()),
            flipbook_frames_per_second: value.flipbook_frames_per_second.map(f64::from),
            max_particles: value.max_particles,
            visible: value.visible,
        }
    }
}

#[napi(object)]
pub struct NativeParticleProjectionOp {
    pub op: String,
    pub signal_id: Option<String>,
    pub handle: Option<i64>,
    pub descriptor: Option<NativeParticleEmitterDescriptor>,
    pub patch: Option<NativeParticleEmitterPatch>,
}

impl From<ParticleProjectionOp> for NativeParticleProjectionOp {
    fn from(value: ParticleProjectionOp) -> Self {
        match value {
            ParticleProjectionOp::Emit {
                signal_id,
                descriptor,
            } => Self {
                op: "emit".to_string(),
                signal_id: Some(signal_id),
                handle: None,
                descriptor: Some(descriptor.into()),
                patch: None,
            },
            ParticleProjectionOp::Create { handle, descriptor } => Self {
                op: "create".to_string(),
                signal_id: None,
                handle: Some(handle.raw() as i64),
                descriptor: Some(descriptor.into()),
                patch: None,
            },
            ParticleProjectionOp::Update { handle, patch } => Self {
                op: "update".to_string(),
                signal_id: None,
                handle: Some(handle.raw() as i64),
                descriptor: None,
                patch: Some(patch.into()),
            },
            ParticleProjectionOp::Destroy { handle } => Self {
                op: "destroy".to_string(),
                signal_id: None,
                handle: Some(handle.raw() as i64),
                descriptor: None,
                patch: None,
            },
        }
    }
}

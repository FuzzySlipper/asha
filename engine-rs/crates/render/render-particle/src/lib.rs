//! Validation and retained lifecycle for disposable particle projection.
//!
//! Rust chooses and validates effect descriptors. Per-particle simulation and
//! billboard realization belong to the renderer host and never become Session
//! authority or replay truth.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use core_assets::{AssetId, AssetKind};
use core_catalog::Catalog;
use protocol_presentation::{
    ParticleAnchor, ParticleColorKey, ParticleEmitterDescriptor, ParticleEmitterHandle,
    ParticleEmitterPatch, ParticleProjectionDiagnostic, ParticleProjectionDiagnosticCode,
    ParticleProjectionOp, ParticleProjectionReadout, ParticleScalarKey, PresentationOp,
    PresentationOpMeta,
};

const MAX_CURVE_KEYS: usize = 8;
const JSON_SAFE_U64_MAX: u64 = (1_u64 << 53) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleProjectionLimits {
    pub max_active_emitters: u32,
    pub max_particles_per_emitter: u32,
    pub max_reserved_particles: u32,
}

impl Default for ParticleProjectionLimits {
    fn default() -> Self {
        Self {
            max_active_emitters: 64,
            max_particles_per_emitter: 1_024,
            max_reserved_particles: 4_096,
        }
    }
}

#[derive(Debug)]
pub struct ParticleProjector {
    catalog: Catalog,
    limits: ParticleProjectionLimits,
    active: BTreeMap<ParticleEmitterHandle, ParticleEmitterDescriptor>,
    seen_signals: BTreeSet<String>,
    emitted_bursts: u64,
    diagnostics: Vec<ParticleProjectionDiagnostic>,
}

impl ParticleProjector {
    pub fn new(catalog: &Catalog, limits: ParticleProjectionLimits) -> Self {
        Self {
            catalog: catalog.clone(),
            limits,
            active: BTreeMap::new(),
            seen_signals: BTreeSet::new(),
            emitted_bursts: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn project(
        &mut self,
        meta: PresentationOpMeta,
        op: ParticleProjectionOp,
    ) -> Result<PresentationOp, Box<ParticleProjectionDiagnostic>> {
        if let Err(code) = self.validate_and_apply(&op) {
            let diagnostic = ParticleProjectionDiagnostic {
                code,
                sequence: meta.sequence,
                handle: operation_handle(&op),
                message: diagnostic_message(code).to_owned(),
                origin: meta.origin,
            };
            self.diagnostics.push(diagnostic.clone());
            return Err(Box::new(diagnostic));
        }
        Ok(PresentationOp::Particle { meta, op })
    }

    pub fn descriptor(&self, handle: ParticleEmitterHandle) -> Option<&ParticleEmitterDescriptor> {
        self.active.get(&handle)
    }

    pub fn readout(&self) -> ParticleProjectionReadout {
        ParticleProjectionReadout {
            active_emitters: self.active.len() as u32,
            active_particles: 0,
            loaded_sprites: 0,
            emitted_bursts: self.emitted_bursts,
            dropped_particles: 0,
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub fn reset(&mut self) {
        self.active.clear();
        self.seen_signals.clear();
        self.emitted_bursts = 0;
        self.diagnostics.clear();
    }

    fn validate_and_apply(
        &mut self,
        op: &ParticleProjectionOp,
    ) -> Result<(), ParticleProjectionDiagnosticCode> {
        match op {
            ParticleProjectionOp::Emit {
                signal_id,
                descriptor,
            } => {
                if signal_id.is_empty() || descriptor.burst_count == 0 {
                    return Err(ParticleProjectionDiagnosticCode::InvalidDescriptor);
                }
                self.validate_descriptor(descriptor)?;
                if !self.seen_signals.insert(signal_id.clone()) {
                    return Err(ParticleProjectionDiagnosticCode::DuplicateSignal);
                }
                self.emitted_bursts = self.emitted_bursts.saturating_add(1);
            }
            ParticleProjectionOp::Create { handle, descriptor } => {
                if self.active.contains_key(handle) {
                    return Err(ParticleProjectionDiagnosticCode::DuplicateHandle);
                }
                self.validate_descriptor(descriptor)?;
                if descriptor.rate_per_second <= 0.0
                    || self.active.len() as u32 >= self.limits.max_active_emitters
                    || self
                        .reserved_particles()
                        .saturating_add(descriptor.max_particles)
                        > self.limits.max_reserved_particles
                {
                    return Err(ParticleProjectionDiagnosticCode::BudgetExceeded);
                }
                self.active.insert(*handle, descriptor.clone());
            }
            ParticleProjectionOp::Update { handle, patch } => {
                let current = self
                    .active
                    .get(handle)
                    .cloned()
                    .ok_or(ParticleProjectionDiagnosticCode::UnknownHandle)?;
                let updated = apply_patch(current.clone(), patch);
                self.validate_descriptor(&updated)?;
                if updated.rate_per_second <= 0.0
                    || self
                        .reserved_particles()
                        .saturating_sub(current.max_particles)
                        .saturating_add(updated.max_particles)
                        > self.limits.max_reserved_particles
                {
                    return Err(ParticleProjectionDiagnosticCode::BudgetExceeded);
                }
                self.active.insert(*handle, updated);
            }
            ParticleProjectionOp::Destroy { handle } => {
                if self.active.remove(handle).is_none() {
                    return Err(ParticleProjectionDiagnosticCode::UnknownHandle);
                }
            }
        }
        Ok(())
    }

    fn validate_descriptor(
        &self,
        descriptor: &ParticleEmitterDescriptor,
    ) -> Result<(), ParticleProjectionDiagnosticCode> {
        if !anchor_is_finite(&descriptor.anchor)
            || !in_range(descriptor.rate_per_second, 0.0, 10_000.0)
            || descriptor.burst_count > self.limits.max_particles_per_emitter
            || descriptor.max_particles == 0
            || descriptor.max_particles > self.limits.max_particles_per_emitter
            || !ordered_positive_range(descriptor.lifetime_seconds, 0.01, 60.0)
            || !ordered_vec3(descriptor.velocity_min, descriptor.velocity_max)
            || !finite_vec3(descriptor.acceleration)
            || !in_range(descriptor.flipbook_frames_per_second, 0.0, 120.0)
            || descriptor.burst_count > descriptor.max_particles
            || descriptor.seed > JSON_SAFE_U64_MAX
            || !validate_scalar_curve(&descriptor.size_curve)
            || !validate_color_curve(&descriptor.color_curve)
        {
            return Err(ParticleProjectionDiagnosticCode::InvalidDescriptor);
        }
        if descriptor.sprite.frame_count == 0
            || (descriptor.sprite.frame_count > 1 && descriptor.flipbook_frames_per_second <= 0.0)
        {
            return Err(ParticleProjectionDiagnosticCode::InvalidDescriptor);
        }
        self.validate_sprite(descriptor)
    }

    fn validate_sprite(
        &self,
        descriptor: &ParticleEmitterDescriptor,
    ) -> Result<(), ParticleProjectionDiagnosticCode> {
        let asset = AssetId::parse(&descriptor.sprite.asset)
            .map_err(|_| ParticleProjectionDiagnosticCode::AssetKindMismatch)?;
        let expected = if descriptor.sprite.frame_count == 1 {
            AssetKind::Sprite
        } else {
            AssetKind::SpriteSheet
        };
        if asset.kind() != expected {
            return Err(ParticleProjectionDiagnosticCode::AssetKindMismatch);
        }
        let entry = self
            .catalog
            .get(&asset)
            .ok_or(ParticleProjectionDiagnosticCode::AssetMissing)?;
        let hash = entry
            .hash
            .as_ref()
            .ok_or(ParticleProjectionDiagnosticCode::ContentHashMismatch)?;
        if hash.as_str() != descriptor.sprite.content_hash {
            return Err(ParticleProjectionDiagnosticCode::ContentHashMismatch);
        }
        Ok(())
    }

    fn reserved_particles(&self) -> u32 {
        self.active.values().fold(0_u32, |total, descriptor| {
            total.saturating_add(descriptor.max_particles)
        })
    }
}

fn apply_patch(
    mut descriptor: ParticleEmitterDescriptor,
    patch: &ParticleEmitterPatch,
) -> ParticleEmitterDescriptor {
    if let Some(value) = &patch.anchor {
        descriptor.anchor = value.clone();
    }
    if let Some(value) = &patch.sprite {
        descriptor.sprite = value.clone();
    }
    if let Some(value) = patch.rate_per_second {
        descriptor.rate_per_second = value;
    }
    if let Some(value) = patch.burst_count {
        descriptor.burst_count = value;
    }
    if let Some(value) = patch.lifetime_seconds {
        descriptor.lifetime_seconds = value;
    }
    if let Some(value) = patch.velocity_min {
        descriptor.velocity_min = value;
    }
    if let Some(value) = patch.velocity_max {
        descriptor.velocity_max = value;
    }
    if let Some(value) = patch.acceleration {
        descriptor.acceleration = value;
    }
    if let Some(value) = &patch.size_curve {
        descriptor.size_curve.clone_from(value);
    }
    if let Some(value) = &patch.color_curve {
        descriptor.color_curve.clone_from(value);
    }
    if let Some(value) = patch.flipbook_frames_per_second {
        descriptor.flipbook_frames_per_second = value;
    }
    if let Some(value) = patch.max_particles {
        descriptor.max_particles = value;
    }
    if let Some(value) = patch.visible {
        descriptor.visible = value;
    }
    descriptor
}

fn validate_scalar_curve(keys: &[ParticleScalarKey]) -> bool {
    curve_ages(keys.iter().map(|key| key.age))
        && keys
            .iter()
            .all(|key| key.value.is_finite() && key.value >= 0.0)
}

fn validate_color_curve(keys: &[ParticleColorKey]) -> bool {
    curve_ages(keys.iter().map(|key| key.age))
        && keys
            .iter()
            .all(|key| key.color.into_iter().all(|value| in_range(value, 0.0, 1.0)))
}

fn curve_ages(ages: impl Iterator<Item = f32>) -> bool {
    let values = ages.collect::<Vec<_>>();
    values.len() >= 2
        && values.len() <= MAX_CURVE_KEYS
        && values.first() == Some(&0.0)
        && values.last() == Some(&1.0)
        && values
            .windows(2)
            .all(|pair| pair[0].is_finite() && pair[0] < pair[1])
}

fn anchor_is_finite(anchor: &ParticleAnchor) -> bool {
    match anchor {
        ParticleAnchor::World { position }
        | ParticleAnchor::EntityAttached {
            offset: position, ..
        } => finite_vec3(*position),
    }
}

fn finite_vec3(value: [f32; 3]) -> bool {
    value.into_iter().all(f32::is_finite)
}

fn ordered_vec3(min: [f32; 3], max: [f32; 3]) -> bool {
    finite_vec3(min) && finite_vec3(max) && min.into_iter().zip(max).all(|(low, high)| low <= high)
}

fn ordered_positive_range(value: [f32; 2], min: f32, max: f32) -> bool {
    in_range(value[0], min, max) && in_range(value[1], min, max) && value[0] <= value[1]
}

fn in_range(value: f32, min: f32, max: f32) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}

fn operation_handle(op: &ParticleProjectionOp) -> Option<ParticleEmitterHandle> {
    match op {
        ParticleProjectionOp::Emit { .. } => None,
        ParticleProjectionOp::Create { handle, .. }
        | ParticleProjectionOp::Update { handle, .. }
        | ParticleProjectionOp::Destroy { handle } => Some(*handle),
    }
}

fn diagnostic_message(code: ParticleProjectionDiagnosticCode) -> &'static str {
    match code {
        ParticleProjectionDiagnosticCode::InvalidDescriptor => "particle descriptor is invalid",
        ParticleProjectionDiagnosticCode::AssetMissing => {
            "particle sprite is absent from the catalog"
        }
        ParticleProjectionDiagnosticCode::AssetKindMismatch => {
            "particle sprite kind does not match its frame count"
        }
        ParticleProjectionDiagnosticCode::ContentHashMismatch => {
            "particle sprite hash does not match the catalog"
        }
        ParticleProjectionDiagnosticCode::DuplicateSignal => {
            "particle burst signal was already projected"
        }
        ParticleProjectionDiagnosticCode::DuplicateHandle => {
            "particle emitter handle is already active"
        }
        ParticleProjectionDiagnosticCode::UnknownHandle => "particle emitter handle is not active",
        ParticleProjectionDiagnosticCode::AnchorMissing => "particle entity anchor is unavailable",
        ParticleProjectionDiagnosticCode::BudgetExceeded => {
            "particle projection budget is exhausted"
        }
        ParticleProjectionDiagnosticCode::UnavailableHost => "particle host is unavailable",
        ParticleProjectionDiagnosticCode::SpriteLoadFailed => "particle sprite failed to load",
        ParticleProjectionDiagnosticCode::HostFailure => "particle host operation failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_assets::{AssetHash, AssetId};
    use core_catalog::CatalogEntry;
    use protocol_presentation::{ParticleColorKey, ParticleScalarKey, ParticleSpriteRef};

    fn catalog() -> Catalog {
        Catalog::from_entries(vec![CatalogEntry::new(
            AssetId::parse("sprite-sheet/sparks").unwrap(),
            1,
        )
        .with_hash(AssetHash::parse("aabb").unwrap())])
    }

    fn descriptor() -> ParticleEmitterDescriptor {
        ParticleEmitterDescriptor {
            anchor: ParticleAnchor::World {
                position: [1.0, 2.0, 3.0],
            },
            sprite: ParticleSpriteRef {
                asset: "sprite-sheet/sparks".into(),
                content_hash: "aabb".into(),
                frame_count: 4,
            },
            rate_per_second: 12.0,
            burst_count: 8,
            lifetime_seconds: [0.2, 0.6],
            velocity_min: [-1.0, 1.0, -1.0],
            velocity_max: [1.0, 3.0, 1.0],
            acceleration: [0.0, -4.0, 0.0],
            size_curve: vec![
                ParticleScalarKey {
                    age: 0.0,
                    value: 0.2,
                },
                ParticleScalarKey {
                    age: 1.0,
                    value: 0.0,
                },
            ],
            color_curve: vec![
                ParticleColorKey {
                    age: 0.0,
                    color: [1.0, 0.8, 0.2, 1.0],
                },
                ParticleColorKey {
                    age: 1.0,
                    color: [1.0, 0.2, 0.0, 0.0],
                },
            ],
            flipbook_frames_per_second: 16.0,
            seed: 44,
            max_particles: 64,
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
    fn burst_and_retained_lifecycle_are_catalog_validated_and_budgeted() {
        let mut projector = ParticleProjector::new(&catalog(), ParticleProjectionLimits::default());
        projector
            .project(
                meta(0),
                ParticleProjectionOp::Emit {
                    signal_id: "impact:1".into(),
                    descriptor: descriptor(),
                },
            )
            .unwrap();
        let handle = ParticleEmitterHandle::new(4);
        projector
            .project(
                meta(1),
                ParticleProjectionOp::Create {
                    handle,
                    descriptor: descriptor(),
                },
            )
            .unwrap();
        projector
            .project(
                meta(2),
                ParticleProjectionOp::Update {
                    handle,
                    patch: ParticleEmitterPatch {
                        rate_per_second: Some(20.0),
                        visible: Some(false),
                        ..ParticleEmitterPatch::default()
                    },
                },
            )
            .unwrap();
        assert_eq!(projector.readout().active_emitters, 1);
        assert_eq!(projector.readout().emitted_bursts, 1);
        assert_eq!(projector.descriptor(handle).unwrap().rate_per_second, 20.0);
        projector
            .project(meta(3), ParticleProjectionOp::Destroy { handle })
            .unwrap();
        assert_eq!(projector.readout().active_emitters, 0);
    }

    #[test]
    fn invalid_curve_duplicate_signal_and_budget_fail_without_partial_state() {
        let mut projector = ParticleProjector::new(
            &catalog(),
            ParticleProjectionLimits {
                max_active_emitters: 1,
                max_particles_per_emitter: 64,
                max_reserved_particles: 64,
            },
        );
        let mut invalid = descriptor();
        invalid.size_curve[1].age = 0.0;
        assert_eq!(
            projector
                .project(
                    meta(0),
                    ParticleProjectionOp::Emit {
                        signal_id: "invalid".into(),
                        descriptor: invalid,
                    },
                )
                .unwrap_err()
                .code,
            ParticleProjectionDiagnosticCode::InvalidDescriptor
        );
        projector
            .project(
                meta(1),
                ParticleProjectionOp::Emit {
                    signal_id: "impact:1".into(),
                    descriptor: descriptor(),
                },
            )
            .unwrap();
        assert_eq!(
            projector
                .project(
                    meta(2),
                    ParticleProjectionOp::Emit {
                        signal_id: "impact:1".into(),
                        descriptor: descriptor(),
                    },
                )
                .unwrap_err()
                .code,
            ParticleProjectionDiagnosticCode::DuplicateSignal
        );
        projector
            .project(
                meta(3),
                ParticleProjectionOp::Create {
                    handle: ParticleEmitterHandle::new(1),
                    descriptor: descriptor(),
                },
            )
            .unwrap();
        assert_eq!(
            projector
                .project(
                    meta(4),
                    ParticleProjectionOp::Create {
                        handle: ParticleEmitterHandle::new(2),
                        descriptor: descriptor(),
                    },
                )
                .unwrap_err()
                .code,
            ParticleProjectionDiagnosticCode::BudgetExceeded
        );
        assert_eq!(projector.readout().active_emitters, 1);
    }
}

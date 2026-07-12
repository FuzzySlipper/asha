//! Rust validation and retained lifecycle for audio presentation operations.
//!
//! This crate is projection authority only: it validates catalog identity and
//! descriptor shape, allocates no gameplay state, and emits disposable G1
//! presentation operations for a host to realize.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use core_assets::{AssetId, AssetKind};
use core_catalog::Catalog;
use protocol_presentation::{
    AudioEmitter, AudioHandle, AudioProjectionDiagnostic, AudioProjectionDiagnosticCode,
    AudioProjectionOp, AudioProjectionReadout, AudioSourceDescriptor, AudioSourcePatch,
    PresentationOp, PresentationOpMeta,
};

#[derive(Debug)]
pub struct AudioProjector {
    catalog: Catalog,
    active: BTreeMap<AudioHandle, AudioSourceDescriptor>,
    seen_signals: BTreeSet<String>,
    emitted_signals: u64,
    diagnostics: Vec<AudioProjectionDiagnostic>,
}

impl AudioProjector {
    pub fn new(catalog: &Catalog) -> Self {
        Self {
            catalog: catalog.clone(),
            active: BTreeMap::new(),
            seen_signals: BTreeSet::new(),
            emitted_signals: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn project(
        &mut self,
        meta: PresentationOpMeta,
        op: AudioProjectionOp,
    ) -> Result<PresentationOp, Box<AudioProjectionDiagnostic>> {
        if let Err(code) = self.validate_and_apply(&op) {
            let diagnostic = AudioProjectionDiagnostic {
                code,
                sequence: meta.sequence,
                handle: operation_handle(&op),
                message: diagnostic_message(code).to_string(),
                origin: meta.origin,
            };
            self.diagnostics.push(diagnostic.clone());
            return Err(Box::new(diagnostic));
        }
        Ok(PresentationOp::Audio { meta, op })
    }

    pub fn descriptor(&self, handle: AudioHandle) -> Option<&AudioSourceDescriptor> {
        self.active.get(&handle)
    }

    pub fn readout(&self) -> AudioProjectionReadout {
        AudioProjectionReadout {
            active_sources: self.active.len() as u32,
            cached_clips: 0,
            emitted_signals: self.emitted_signals,
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub fn reset(&mut self) {
        self.active.clear();
        self.seen_signals.clear();
        self.emitted_signals = 0;
        self.diagnostics.clear();
    }

    fn validate_and_apply(
        &mut self,
        op: &AudioProjectionOp,
    ) -> Result<(), AudioProjectionDiagnosticCode> {
        match op {
            AudioProjectionOp::Emit {
                signal_id,
                descriptor,
            } => {
                if signal_id.is_empty() {
                    return Err(AudioProjectionDiagnosticCode::InvalidDescriptor);
                }
                self.validate_descriptor(descriptor)?;
                if !self.seen_signals.insert(signal_id.clone()) {
                    return Err(AudioProjectionDiagnosticCode::DuplicateSignal);
                }
                self.emitted_signals = self.emitted_signals.saturating_add(1);
            }
            AudioProjectionOp::Create { handle, descriptor } => {
                if self.active.contains_key(handle) {
                    return Err(AudioProjectionDiagnosticCode::DuplicateHandle);
                }
                self.validate_descriptor(descriptor)?;
                self.active.insert(*handle, descriptor.clone());
            }
            AudioProjectionOp::Update { handle, patch } => {
                let current = self
                    .active
                    .get(handle)
                    .cloned()
                    .ok_or(AudioProjectionDiagnosticCode::UnknownHandle)?;
                let updated = apply_patch(current, patch);
                self.validate_descriptor(&updated)?;
                self.active.insert(*handle, updated);
            }
            AudioProjectionOp::Destroy { handle } => {
                if self.active.remove(handle).is_none() {
                    return Err(AudioProjectionDiagnosticCode::UnknownHandle);
                }
            }
        }
        Ok(())
    }

    fn validate_descriptor(
        &self,
        descriptor: &AudioSourceDescriptor,
    ) -> Result<(), AudioProjectionDiagnosticCode> {
        if !in_range(descriptor.volume, 0.0, 1.0)
            || !in_range(descriptor.pitch, 0.25, 4.0)
            || !in_range(descriptor.spatial_blend, 0.0, 1.0)
            || !in_range(descriptor.pan, -1.0, 1.0)
            || !descriptor.attenuation.is_finite()
            || descriptor.attenuation <= 0.0
            || !emitter_is_finite(&descriptor.emitter)
        {
            return Err(AudioProjectionDiagnosticCode::InvalidDescriptor);
        }

        let asset = AssetId::parse(&descriptor.clip.asset)
            .map_err(|_| AudioProjectionDiagnosticCode::AssetKindMismatch)?;
        if asset.kind() != AssetKind::AudioClip {
            return Err(AudioProjectionDiagnosticCode::AssetKindMismatch);
        }
        let entry = self
            .catalog
            .get(&asset)
            .ok_or(AudioProjectionDiagnosticCode::AssetMissing)?;
        let hash = entry
            .hash
            .as_ref()
            .ok_or(AudioProjectionDiagnosticCode::ContentHashMismatch)?;
        if hash.as_str() != descriptor.clip.content_hash {
            return Err(AudioProjectionDiagnosticCode::ContentHashMismatch);
        }
        Ok(())
    }
}

fn apply_patch(
    mut descriptor: AudioSourceDescriptor,
    patch: &AudioSourcePatch,
) -> AudioSourceDescriptor {
    if let Some(value) = patch.volume {
        descriptor.volume = value;
    }
    if let Some(value) = patch.pitch {
        descriptor.pitch = value;
    }
    if let Some(value) = patch.looping {
        descriptor.looping = value;
    }
    if let Some(value) = patch.spatial_blend {
        descriptor.spatial_blend = value;
    }
    if let Some(value) = patch.attenuation {
        descriptor.attenuation = value;
    }
    if let Some(value) = patch.pan {
        descriptor.pan = value;
    }
    if let Some(value) = &patch.emitter {
        descriptor.emitter = value.clone();
    }
    descriptor
}

fn in_range(value: f32, min: f32, max: f32) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}

fn emitter_is_finite(emitter: &AudioEmitter) -> bool {
    match emitter {
        AudioEmitter::Global2d => true,
        AudioEmitter::World3d { position }
        | AudioEmitter::EntityAttached {
            offset: position, ..
        } => position.iter().all(|value| value.is_finite()),
    }
}

fn operation_handle(op: &AudioProjectionOp) -> Option<AudioHandle> {
    match op {
        AudioProjectionOp::Emit { .. } => None,
        AudioProjectionOp::Create { handle, .. }
        | AudioProjectionOp::Update { handle, .. }
        | AudioProjectionOp::Destroy { handle } => Some(*handle),
    }
}

fn diagnostic_message(code: AudioProjectionDiagnosticCode) -> &'static str {
    match code {
        AudioProjectionDiagnosticCode::InvalidDescriptor => "audio descriptor is invalid",
        AudioProjectionDiagnosticCode::AssetMissing => "audio clip is absent from the catalog",
        AudioProjectionDiagnosticCode::AssetKindMismatch => {
            "audio clip reference does not name an audio asset"
        }
        AudioProjectionDiagnosticCode::ContentHashMismatch => {
            "audio clip content hash does not match the catalog"
        }
        AudioProjectionDiagnosticCode::DuplicateSignal => {
            "audio one-shot signal id was already projected"
        }
        AudioProjectionDiagnosticCode::DuplicateHandle => "audio handle is already active",
        AudioProjectionDiagnosticCode::UnknownHandle => "audio handle is not active",
        AudioProjectionDiagnosticCode::UnavailableHost => "audio host is unavailable",
        AudioProjectionDiagnosticCode::AudioContextBlocked => "audio context start was blocked",
        AudioProjectionDiagnosticCode::DecodeFailed => "audio clip decoding failed",
        AudioProjectionDiagnosticCode::HostFailure => "audio host operation failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_assets::{AssetHash, AssetId};
    use core_catalog::CatalogEntry;
    use protocol_presentation::{
        AudioBus, AudioClipRef, PresentationOriginKind, PresentationOriginRef,
    };

    fn catalog() -> Catalog {
        Catalog::from_entries(vec![CatalogEntry::new(
            AssetId::parse("audio/fixture-pulse").expect("valid audio id"),
            1,
        )
        .with_hash(AssetHash::parse("aabb").expect("valid hash"))])
    }

    fn descriptor() -> AudioSourceDescriptor {
        AudioSourceDescriptor {
            clip: AudioClipRef {
                asset: "audio/fixture-pulse".to_string(),
                content_hash: "aabb".to_string(),
            },
            bus: AudioBus::Sfx,
            volume: 0.8,
            pitch: 1.0,
            looping: true,
            spatial_blend: 1.0,
            attenuation: 12.0,
            pan: 0.0,
            emitter: AudioEmitter::World3d {
                position: [1.0, 2.0, 3.0],
            },
        }
    }

    fn meta(sequence: u32) -> PresentationOpMeta {
        PresentationOpMeta {
            sequence,
            origin: Some(PresentationOriginRef {
                kind: PresentationOriginKind::OwnerFact,
                id: "combat.primary-fire.accepted:41".to_string(),
                authority_tick: 41,
                causation_id: Some("command:fire:9".to_string()),
                correlation_id: Some("encounter:fixture".to_string()),
            }),
        }
    }

    #[test]
    fn catalog_validated_emit_create_update_destroy_are_ordered_and_retained() {
        let catalog = catalog();
        let mut projector = AudioProjector::new(&catalog);
        projector
            .project(
                meta(0),
                AudioProjectionOp::Emit {
                    signal_id: "shot:41".to_string(),
                    descriptor: AudioSourceDescriptor {
                        looping: false,
                        ..descriptor()
                    },
                },
            )
            .expect("one-shot emits");
        projector
            .project(
                meta(1),
                AudioProjectionOp::Create {
                    handle: AudioHandle::new(1),
                    descriptor: descriptor(),
                },
            )
            .expect("retained source creates");
        projector
            .project(
                meta(2),
                AudioProjectionOp::Update {
                    handle: AudioHandle::new(1),
                    patch: AudioSourcePatch {
                        volume: Some(0.25),
                        pan: Some(-0.5),
                        ..AudioSourcePatch::default()
                    },
                },
            )
            .expect("retained source updates");
        assert_eq!(
            projector
                .descriptor(AudioHandle::new(1))
                .expect("source remains")
                .volume,
            0.25
        );
        projector
            .project(
                meta(3),
                AudioProjectionOp::Destroy {
                    handle: AudioHandle::new(1),
                },
            )
            .expect("retained source destroys");
        assert_eq!(projector.readout().active_sources, 0);
        assert_eq!(projector.readout().emitted_signals, 1);

        let diagnostic = projector
            .project(
                meta(4),
                AudioProjectionOp::Emit {
                    signal_id: "shot:41".to_string(),
                    descriptor: AudioSourceDescriptor {
                        looping: false,
                        ..descriptor()
                    },
                },
            )
            .expect_err("one-shot signal ids are unique within a Session generation");
        assert_eq!(
            diagnostic.code,
            AudioProjectionDiagnosticCode::DuplicateSignal
        );
    }

    #[test]
    fn invalid_asset_descriptor_and_handle_transitions_fail_without_partial_state() {
        let catalog = catalog();
        let mut projector = AudioProjector::new(&catalog);
        let mut bad_hash = descriptor();
        bad_hash.clip.content_hash = "ccdd".to_string();
        let diagnostic = projector
            .project(
                meta(0),
                AudioProjectionOp::Create {
                    handle: AudioHandle::new(1),
                    descriptor: bad_hash,
                },
            )
            .expect_err("hash drift rejects");
        assert_eq!(
            diagnostic.code,
            AudioProjectionDiagnosticCode::ContentHashMismatch
        );
        assert_eq!(projector.readout().active_sources, 0);

        let diagnostic = projector
            .project(
                meta(1),
                AudioProjectionOp::Destroy {
                    handle: AudioHandle::new(1),
                },
            )
            .expect_err("unknown destroy rejects");
        assert_eq!(
            diagnostic.code,
            AudioProjectionDiagnosticCode::UnknownHandle
        );
        assert_eq!(projector.readout().diagnostics.len(), 2);
    }
}

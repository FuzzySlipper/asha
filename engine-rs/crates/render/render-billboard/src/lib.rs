//! Validation and retained lifecycle for world-space billboard projection.
//!
//! Billboard state is disposable presentation state. This projector validates
//! the closed G1 contract and catalog references without granting a renderer or
//! downstream host authority over entity state.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use core_assets::{AssetId, AssetKind};
use core_catalog::Catalog;
use protocol_presentation::{
    BillboardAnchor, BillboardContent, BillboardDescriptor, BillboardFontRef, BillboardHandle,
    BillboardPatch, BillboardProjectionDiagnostic, BillboardProjectionDiagnosticCode,
    BillboardProjectionOp, BillboardProjectionReadout, BillboardTemplateArgument, PresentationOp,
    PresentationOpMeta,
};

const MAX_TEXT_BYTES: usize = 256;
const MAX_KEY_BYTES: usize = 128;
const MAX_ARGUMENTS: usize = 8;

#[derive(Debug)]
pub struct BillboardProjector {
    catalog: Catalog,
    active: BTreeMap<BillboardHandle, BillboardDescriptor>,
    diagnostics: Vec<BillboardProjectionDiagnostic>,
}

impl BillboardProjector {
    pub fn new(catalog: &Catalog) -> Self {
        Self {
            catalog: catalog.clone(),
            active: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn project(
        &mut self,
        meta: PresentationOpMeta,
        op: BillboardProjectionOp,
    ) -> Result<PresentationOp, Box<BillboardProjectionDiagnostic>> {
        if let Err(code) = self.validate_and_apply(&op) {
            let diagnostic = BillboardProjectionDiagnostic {
                code,
                sequence: meta.sequence,
                handle: operation_handle(&op),
                message: diagnostic_message(code).to_string(),
                origin: meta.origin,
            };
            self.diagnostics.push(diagnostic.clone());
            return Err(Box::new(diagnostic));
        }
        Ok(PresentationOp::Billboard { meta, op })
    }

    pub fn descriptor(&self, handle: BillboardHandle) -> Option<&BillboardDescriptor> {
        self.active.get(&handle)
    }

    pub fn readout(&self) -> BillboardProjectionReadout {
        BillboardProjectionReadout {
            active_billboards: self.active.len() as u32,
            loaded_fonts: 0,
            loaded_icons: 0,
            culled_billboards: 0,
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub fn reset(&mut self) {
        self.active.clear();
        self.diagnostics.clear();
    }

    fn validate_and_apply(
        &mut self,
        op: &BillboardProjectionOp,
    ) -> Result<(), BillboardProjectionDiagnosticCode> {
        match op {
            BillboardProjectionOp::Create { handle, descriptor } => {
                if self.active.contains_key(handle) {
                    return Err(BillboardProjectionDiagnosticCode::DuplicateHandle);
                }
                self.validate_descriptor(descriptor)?;
                self.active.insert(*handle, descriptor.clone());
            }
            BillboardProjectionOp::Update { handle, patch } => {
                let current = self
                    .active
                    .get(handle)
                    .cloned()
                    .ok_or(BillboardProjectionDiagnosticCode::UnknownHandle)?;
                let updated = apply_patch(current, patch);
                self.validate_descriptor(&updated)?;
                self.active.insert(*handle, updated);
            }
            BillboardProjectionOp::Destroy { handle } => {
                if self.active.remove(handle).is_none() {
                    return Err(BillboardProjectionDiagnosticCode::UnknownHandle);
                }
            }
        }
        Ok(())
    }

    fn validate_descriptor(
        &self,
        descriptor: &BillboardDescriptor,
    ) -> Result<(), BillboardProjectionDiagnosticCode> {
        if !anchor_is_finite(&descriptor.anchor)
            || !in_range(descriptor.height_pixels, 8.0, 256.0)
            || !descriptor.max_distance.is_finite()
            || descriptor.max_distance <= 0.0
            || descriptor.max_distance > 10_000.0
            || !color_is_valid(descriptor.color)
            || !color_is_valid(descriptor.background)
        {
            return Err(BillboardProjectionDiagnosticCode::InvalidDescriptor);
        }
        self.validate_content(&descriptor.content)?;
        self.validate_font(&descriptor.font)
    }

    fn validate_content(
        &self,
        content: &BillboardContent,
    ) -> Result<(), BillboardProjectionDiagnosticCode> {
        match content {
            BillboardContent::Text {
                localization_key,
                fallback_text,
                arguments,
            } => {
                validate_key(localization_key)?;
                validate_text(fallback_text)?;
                validate_arguments(arguments)
            }
            BillboardContent::Value {
                label_key,
                fallback_label,
                value,
                unit_key,
                fallback_unit,
            } => {
                validate_key(label_key)?;
                validate_text(fallback_label)?;
                validate_text(value)?;
                validate_optional_key(unit_key)?;
                validate_optional_text(fallback_unit)
            }
            BillboardContent::Icon {
                texture,
                alt_key,
                fallback_alt,
            } => {
                validate_key(alt_key)?;
                validate_text(fallback_alt)?;
                self.validate_asset(&texture.asset, &texture.content_hash, AssetKind::Texture)
            }
        }
    }

    fn validate_font(
        &self,
        font: &BillboardFontRef,
    ) -> Result<(), BillboardProjectionDiagnosticCode> {
        match font {
            BillboardFontRef::System { family } => validate_text(family),
            BillboardFontRef::Asset {
                asset,
                content_hash,
                family,
            } => {
                validate_text(family)?;
                self.validate_asset(asset, content_hash, AssetKind::Font)
            }
        }
    }

    fn validate_asset(
        &self,
        raw_asset: &str,
        content_hash: &str,
        expected_kind: AssetKind,
    ) -> Result<(), BillboardProjectionDiagnosticCode> {
        let asset = AssetId::parse(raw_asset)
            .map_err(|_| BillboardProjectionDiagnosticCode::AssetKindMismatch)?;
        if asset.kind() != expected_kind {
            return Err(BillboardProjectionDiagnosticCode::AssetKindMismatch);
        }
        let entry = self
            .catalog
            .get(&asset)
            .ok_or(BillboardProjectionDiagnosticCode::AssetMissing)?;
        let hash = entry
            .hash
            .as_ref()
            .ok_or(BillboardProjectionDiagnosticCode::ContentHashMismatch)?;
        if hash.as_str() != content_hash {
            return Err(BillboardProjectionDiagnosticCode::ContentHashMismatch);
        }
        Ok(())
    }
}

fn apply_patch(mut descriptor: BillboardDescriptor, patch: &BillboardPatch) -> BillboardDescriptor {
    if let Some(value) = &patch.anchor {
        descriptor.anchor = value.clone();
    }
    if let Some(value) = &patch.content {
        descriptor.content = value.clone();
    }
    if let Some(value) = &patch.font {
        descriptor.font = value.clone();
    }
    if let Some(value) = patch.height_pixels {
        descriptor.height_pixels = value;
    }
    if let Some(value) = patch.color {
        descriptor.color = value;
    }
    if let Some(value) = patch.background {
        descriptor.background = value;
    }
    if let Some(value) = patch.max_distance {
        descriptor.max_distance = value;
    }
    if let Some(value) = patch.layer {
        descriptor.layer = value;
    }
    if let Some(value) = patch.visible {
        descriptor.visible = value;
    }
    descriptor
}

fn anchor_is_finite(anchor: &BillboardAnchor) -> bool {
    match anchor {
        BillboardAnchor::World { position }
        | BillboardAnchor::EntityAttached {
            offset: position, ..
        } => position.iter().all(|value| value.is_finite()),
    }
}

fn color_is_valid(color: [f32; 4]) -> bool {
    color.into_iter().all(|value| in_range(value, 0.0, 1.0))
}

fn in_range(value: f32, min: f32, max: f32) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}

fn validate_key(value: &str) -> Result<(), BillboardProjectionDiagnosticCode> {
    if value.is_empty() || value.len() > MAX_KEY_BYTES {
        return Err(BillboardProjectionDiagnosticCode::InvalidDescriptor);
    }
    Ok(())
}

fn validate_text(value: &str) -> Result<(), BillboardProjectionDiagnosticCode> {
    if value.is_empty() || value.len() > MAX_TEXT_BYTES {
        return Err(BillboardProjectionDiagnosticCode::InvalidDescriptor);
    }
    Ok(())
}

fn validate_optional_key(value: &Option<String>) -> Result<(), BillboardProjectionDiagnosticCode> {
    value.as_deref().map_or(Ok(()), validate_key)
}

fn validate_optional_text(value: &Option<String>) -> Result<(), BillboardProjectionDiagnosticCode> {
    value.as_deref().map_or(Ok(()), validate_text)
}

fn validate_arguments(
    arguments: &[BillboardTemplateArgument],
) -> Result<(), BillboardProjectionDiagnosticCode> {
    if arguments.len() > MAX_ARGUMENTS {
        return Err(BillboardProjectionDiagnosticCode::InvalidDescriptor);
    }
    let mut names = BTreeSet::new();
    for argument in arguments {
        validate_key(&argument.name)?;
        validate_text(&argument.value)?;
        if !names.insert(argument.name.as_str()) {
            return Err(BillboardProjectionDiagnosticCode::InvalidDescriptor);
        }
    }
    Ok(())
}

fn operation_handle(op: &BillboardProjectionOp) -> Option<BillboardHandle> {
    match op {
        BillboardProjectionOp::Create { handle, .. }
        | BillboardProjectionOp::Update { handle, .. }
        | BillboardProjectionOp::Destroy { handle } => Some(*handle),
    }
}

fn diagnostic_message(code: BillboardProjectionDiagnosticCode) -> &'static str {
    match code {
        BillboardProjectionDiagnosticCode::InvalidDescriptor => "billboard descriptor is invalid",
        BillboardProjectionDiagnosticCode::AssetMissing => {
            "billboard asset is absent from the catalog"
        }
        BillboardProjectionDiagnosticCode::AssetKindMismatch => {
            "billboard asset kind does not match its role"
        }
        BillboardProjectionDiagnosticCode::ContentHashMismatch => {
            "billboard asset hash does not match the catalog"
        }
        BillboardProjectionDiagnosticCode::DuplicateHandle => "billboard handle is already active",
        BillboardProjectionDiagnosticCode::UnknownHandle => "billboard handle is not active",
        BillboardProjectionDiagnosticCode::AnchorMissing => {
            "billboard entity anchor is unavailable"
        }
        BillboardProjectionDiagnosticCode::UnavailableHost => "billboard host is unavailable",
        BillboardProjectionDiagnosticCode::FontLoadFailed => "billboard font failed to load",
        BillboardProjectionDiagnosticCode::IconLoadFailed => "billboard icon failed to load",
        BillboardProjectionDiagnosticCode::HostFailure => "billboard host operation failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_assets::{AssetHash, AssetId};
    use core_catalog::CatalogEntry;
    use protocol_presentation::{
        BillboardLayer, BillboardTextureRef, PresentationOriginKind, PresentationOriginRef,
    };

    fn catalog() -> Catalog {
        Catalog::from_entries(vec![
            CatalogEntry::new(AssetId::parse("font/ui-sans").unwrap(), 1)
                .with_hash(AssetHash::parse("aabb").unwrap()),
            CatalogEntry::new(AssetId::parse("texture/alert").unwrap(), 1)
                .with_hash(AssetHash::parse("ccdd").unwrap()),
        ])
    }

    fn descriptor() -> BillboardDescriptor {
        BillboardDescriptor {
            anchor: BillboardAnchor::EntityAttached {
                entity: 42,
                offset: [0.0, 1.8, 0.0],
            },
            content: BillboardContent::Value {
                label_key: "enemy.health".into(),
                fallback_label: "Health".into(),
                value: "80/100".into(),
                unit_key: None,
                fallback_unit: None,
            },
            font: BillboardFontRef::Asset {
                asset: "font/ui-sans".into(),
                content_hash: "aabb".into(),
                family: "Asha UI".into(),
            },
            height_pixels: 24.0,
            color: [1.0, 1.0, 1.0, 1.0],
            background: [0.0, 0.0, 0.0, 0.7],
            max_distance: 40.0,
            layer: BillboardLayer::Occluded,
            visible: true,
        }
    }

    fn meta(sequence: u32) -> PresentationOpMeta {
        PresentationOpMeta {
            sequence,
            origin: Some(PresentationOriginRef {
                kind: PresentationOriginKind::CapabilityState,
                id: "health:42".into(),
                authority_tick: 7,
                causation_id: None,
                correlation_id: Some("session:1".into()),
            }),
        }
    }

    #[test]
    fn create_update_destroy_validates_retained_billboard_lifecycle() {
        let mut projector = BillboardProjector::new(&catalog());
        let handle = BillboardHandle::new(1);
        projector
            .project(
                meta(0),
                BillboardProjectionOp::Create {
                    handle,
                    descriptor: descriptor(),
                },
            )
            .unwrap();
        projector
            .project(
                meta(1),
                BillboardProjectionOp::Update {
                    handle,
                    patch: BillboardPatch {
                        content: Some(BillboardContent::Text {
                            localization_key: "enemy.defeated".into(),
                            fallback_text: "Defeated".into(),
                            arguments: Vec::new(),
                        }),
                        ..BillboardPatch::default()
                    },
                },
            )
            .unwrap();
        assert_eq!(projector.readout().active_billboards, 1);
        projector
            .project(meta(2), BillboardProjectionOp::Destroy { handle })
            .unwrap();
        assert_eq!(projector.readout().active_billboards, 0);
    }

    #[test]
    fn assets_and_descriptor_bounds_fail_closed_without_partial_update() {
        let mut projector = BillboardProjector::new(&catalog());
        let handle = BillboardHandle::new(1);
        projector
            .project(
                meta(0),
                BillboardProjectionOp::Create {
                    handle,
                    descriptor: descriptor(),
                },
            )
            .unwrap();
        let error = projector
            .project(
                meta(1),
                BillboardProjectionOp::Update {
                    handle,
                    patch: BillboardPatch {
                        font: Some(BillboardFontRef::Asset {
                            asset: "font/ui-sans".into(),
                            content_hash: "wrong".into(),
                            family: "Asha UI".into(),
                        }),
                        ..BillboardPatch::default()
                    },
                },
            )
            .unwrap_err();
        assert_eq!(
            error.code,
            BillboardProjectionDiagnosticCode::ContentHashMismatch
        );
        assert_eq!(projector.descriptor(handle), Some(&descriptor()));

        let duplicate = projector
            .project(
                meta(2),
                BillboardProjectionOp::Create {
                    handle,
                    descriptor: descriptor(),
                },
            )
            .unwrap_err();
        assert_eq!(
            duplicate.code,
            BillboardProjectionDiagnosticCode::DuplicateHandle
        );
    }

    #[test]
    fn icon_texture_and_reset_are_catalog_validated_and_disposable() {
        let mut descriptor = descriptor();
        descriptor.content = BillboardContent::Icon {
            texture: BillboardTextureRef {
                asset: "texture/alert".into(),
                content_hash: "ccdd".into(),
            },
            alt_key: "warning".into(),
            fallback_alt: "Warning".into(),
        };
        let mut projector = BillboardProjector::new(&catalog());
        projector
            .project(
                meta(0),
                BillboardProjectionOp::Create {
                    handle: BillboardHandle::new(2),
                    descriptor,
                },
            )
            .unwrap();
        projector.reset();
        assert_eq!(projector.readout().active_billboards, 0);
    }
}

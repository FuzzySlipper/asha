use napi_derive::napi;
use runtime_bridge_api::{
    BillboardAnchor, BillboardContent, BillboardDescriptor, BillboardFontRef, BillboardLayer,
    BillboardPatch, BillboardProjectionOp, BillboardTemplateArgument, BillboardTextureRef,
};

#[napi(object)]
pub struct NativeBillboardAnchor {
    pub kind: String,
    pub position: Option<Vec<f64>>,
    pub entity: Option<i64>,
    pub offset: Option<Vec<f64>>,
}

impl From<BillboardAnchor> for NativeBillboardAnchor {
    fn from(value: BillboardAnchor) -> Self {
        match value {
            BillboardAnchor::World { position } => Self {
                kind: "world".to_string(),
                position: Some(position.into_iter().map(f64::from).collect()),
                entity: None,
                offset: None,
            },
            BillboardAnchor::EntityAttached { entity, offset } => Self {
                kind: "entityAttached".to_string(),
                position: None,
                entity: Some(entity as i64),
                offset: Some(offset.into_iter().map(f64::from).collect()),
            },
        }
    }
}

#[napi(object)]
pub struct NativeBillboardTemplateArgument {
    pub name: String,
    pub value: String,
}

impl From<BillboardTemplateArgument> for NativeBillboardTemplateArgument {
    fn from(value: BillboardTemplateArgument) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

#[napi(object)]
pub struct NativeBillboardTextureRef {
    pub asset: String,
    pub content_hash: String,
}

impl From<BillboardTextureRef> for NativeBillboardTextureRef {
    fn from(value: BillboardTextureRef) -> Self {
        Self {
            asset: value.asset,
            content_hash: value.content_hash,
        }
    }
}

#[napi(object)]
pub struct NativeBillboardContent {
    pub kind: String,
    pub localization_key: Option<String>,
    pub fallback_text: Option<String>,
    pub arguments: Option<Vec<NativeBillboardTemplateArgument>>,
    pub label_key: Option<String>,
    pub fallback_label: Option<String>,
    pub value: Option<String>,
    pub unit_key: Option<String>,
    pub fallback_unit: Option<String>,
    pub texture: Option<NativeBillboardTextureRef>,
    pub alt_key: Option<String>,
    pub fallback_alt: Option<String>,
}

impl From<BillboardContent> for NativeBillboardContent {
    fn from(value: BillboardContent) -> Self {
        let mut native = Self {
            kind: String::new(),
            localization_key: None,
            fallback_text: None,
            arguments: None,
            label_key: None,
            fallback_label: None,
            value: None,
            unit_key: None,
            fallback_unit: None,
            texture: None,
            alt_key: None,
            fallback_alt: None,
        };
        match value {
            BillboardContent::Text {
                localization_key,
                fallback_text,
                arguments,
            } => {
                native.kind = "text".to_string();
                native.localization_key = Some(localization_key);
                native.fallback_text = Some(fallback_text);
                native.arguments = Some(
                    arguments
                        .into_iter()
                        .map(NativeBillboardTemplateArgument::from)
                        .collect(),
                );
            }
            BillboardContent::Value {
                label_key,
                fallback_label,
                value,
                unit_key,
                fallback_unit,
            } => {
                native.kind = "value".to_string();
                native.label_key = Some(label_key);
                native.fallback_label = Some(fallback_label);
                native.value = Some(value);
                native.unit_key = unit_key;
                native.fallback_unit = fallback_unit;
            }
            BillboardContent::Icon {
                texture,
                alt_key,
                fallback_alt,
            } => {
                native.kind = "icon".to_string();
                native.texture = Some(texture.into());
                native.alt_key = Some(alt_key);
                native.fallback_alt = Some(fallback_alt);
            }
        }
        native
    }
}

#[napi(object)]
pub struct NativeBillboardFontRef {
    pub kind: String,
    pub family: String,
    pub asset: Option<String>,
    pub content_hash: Option<String>,
}

impl From<BillboardFontRef> for NativeBillboardFontRef {
    fn from(value: BillboardFontRef) -> Self {
        match value {
            BillboardFontRef::System { family } => Self {
                kind: "system".to_string(),
                family,
                asset: None,
                content_hash: None,
            },
            BillboardFontRef::Asset {
                asset,
                content_hash,
                family,
            } => Self {
                kind: "asset".to_string(),
                family,
                asset: Some(asset),
                content_hash: Some(content_hash),
            },
        }
    }
}

#[napi(object)]
pub struct NativeBillboardDescriptor {
    pub anchor: NativeBillboardAnchor,
    pub content: NativeBillboardContent,
    pub font: NativeBillboardFontRef,
    pub height_pixels: f64,
    pub color: Vec<f64>,
    pub background: Vec<f64>,
    pub max_distance: f64,
    pub layer: String,
    pub visible: bool,
}

fn native_billboard_layer(value: BillboardLayer) -> String {
    match value {
        BillboardLayer::AlwaysOnTop => "alwaysOnTop",
        BillboardLayer::DepthTested => "depthTested",
        BillboardLayer::Occluded => "occluded",
    }
    .to_string()
}

impl From<BillboardDescriptor> for NativeBillboardDescriptor {
    fn from(value: BillboardDescriptor) -> Self {
        Self {
            anchor: value.anchor.into(),
            content: value.content.into(),
            font: value.font.into(),
            height_pixels: f64::from(value.height_pixels),
            color: value.color.into_iter().map(f64::from).collect(),
            background: value.background.into_iter().map(f64::from).collect(),
            max_distance: f64::from(value.max_distance),
            layer: native_billboard_layer(value.layer),
            visible: value.visible,
        }
    }
}

#[napi(object)]
pub struct NativeBillboardPatch {
    pub anchor: Option<NativeBillboardAnchor>,
    pub content: Option<NativeBillboardContent>,
    pub font: Option<NativeBillboardFontRef>,
    pub height_pixels: Option<f64>,
    pub color: Option<Vec<f64>>,
    pub background: Option<Vec<f64>>,
    pub max_distance: Option<f64>,
    pub layer: Option<String>,
    pub visible: Option<bool>,
}

impl From<BillboardPatch> for NativeBillboardPatch {
    fn from(value: BillboardPatch) -> Self {
        Self {
            anchor: value.anchor.map(NativeBillboardAnchor::from),
            content: value.content.map(NativeBillboardContent::from),
            font: value.font.map(NativeBillboardFontRef::from),
            height_pixels: value.height_pixels.map(f64::from),
            color: value
                .color
                .map(|color| color.into_iter().map(f64::from).collect()),
            background: value
                .background
                .map(|color| color.into_iter().map(f64::from).collect()),
            max_distance: value.max_distance.map(f64::from),
            layer: value.layer.map(native_billboard_layer),
            visible: value.visible,
        }
    }
}

#[napi(object)]
pub struct NativeBillboardProjectionOp {
    pub op: String,
    pub handle: i64,
    pub descriptor: Option<NativeBillboardDescriptor>,
    pub patch: Option<NativeBillboardPatch>,
}

impl From<BillboardProjectionOp> for NativeBillboardProjectionOp {
    fn from(value: BillboardProjectionOp) -> Self {
        match value {
            BillboardProjectionOp::Create { handle, descriptor } => Self {
                op: "create".to_string(),
                handle: handle.raw() as i64,
                descriptor: Some(descriptor.into()),
                patch: None,
            },
            BillboardProjectionOp::Update { handle, patch } => Self {
                op: "update".to_string(),
                handle: handle.raw() as i64,
                descriptor: None,
                patch: Some(patch.into()),
            },
            BillboardProjectionOp::Destroy { handle } => Self {
                op: "destroy".to_string(),
                handle: handle.raw() as i64,
                descriptor: None,
                patch: None,
            },
        }
    }
}

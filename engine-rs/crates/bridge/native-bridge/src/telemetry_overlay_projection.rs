use napi_derive::napi;
use runtime_bridge_api::{
    TelemetryOverlayCorner, TelemetryOverlayDescriptor, TelemetryOverlayPatch,
    TelemetryOverlayProjectionOp,
};

fn corner(value: TelemetryOverlayCorner) -> String {
    match value {
        TelemetryOverlayCorner::TopLeft => "topLeft",
        TelemetryOverlayCorner::TopRight => "topRight",
        TelemetryOverlayCorner::BottomLeft => "bottomLeft",
        TelemetryOverlayCorner::BottomRight => "bottomRight",
    }
    .to_string()
}

#[napi(object)]
pub struct NativeTelemetryOverlayDescriptor {
    pub title: String,
    pub corner: String,
    pub refresh_interval_ms: u32,
    pub max_frame_time_samples: u32,
    pub visible: bool,
}

impl From<TelemetryOverlayDescriptor> for NativeTelemetryOverlayDescriptor {
    fn from(value: TelemetryOverlayDescriptor) -> Self {
        Self {
            title: value.title,
            corner: corner(value.corner),
            refresh_interval_ms: value.refresh_interval_ms,
            max_frame_time_samples: u32::from(value.max_frame_time_samples),
            visible: value.visible,
        }
    }
}

#[napi(object)]
pub struct NativeTelemetryOverlayPatch {
    pub title: Option<String>,
    pub corner: Option<String>,
    pub refresh_interval_ms: Option<u32>,
    pub max_frame_time_samples: Option<u32>,
    pub visible: Option<bool>,
}

impl From<TelemetryOverlayPatch> for NativeTelemetryOverlayPatch {
    fn from(value: TelemetryOverlayPatch) -> Self {
        Self {
            title: value.title,
            corner: value.corner.map(corner),
            refresh_interval_ms: value.refresh_interval_ms,
            max_frame_time_samples: value.max_frame_time_samples.map(u32::from),
            visible: value.visible,
        }
    }
}

#[napi(object)]
pub struct NativeTelemetryOverlayProjectionOp {
    pub op: String,
    pub handle: i64,
    pub descriptor: Option<NativeTelemetryOverlayDescriptor>,
    pub patch: Option<NativeTelemetryOverlayPatch>,
}

impl From<TelemetryOverlayProjectionOp> for NativeTelemetryOverlayProjectionOp {
    fn from(value: TelemetryOverlayProjectionOp) -> Self {
        match value {
            TelemetryOverlayProjectionOp::Create { handle, descriptor } => Self {
                op: "create".to_string(),
                handle: handle.raw() as i64,
                descriptor: Some(descriptor.into()),
                patch: None,
            },
            TelemetryOverlayProjectionOp::Update { handle, patch } => Self {
                op: "update".to_string(),
                handle: handle.raw() as i64,
                descriptor: None,
                patch: Some(patch.into()),
            },
            TelemetryOverlayProjectionOp::Destroy { handle } => Self {
                op: "destroy".to_string(),
                handle: handle.raw() as i64,
                descriptor: None,
                patch: None,
            },
        }
    }
}

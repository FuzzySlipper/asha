//! Public camera/view DTOs for ASHA runtime view/projection evidence.
//!
//! # Lane
//!
//! `contract-steward` — owns the border shape for deterministic camera input,
//! pose snapshots, and projection evidence. This crate is pure protocol data: it
//! has no renderer behavior, no gameplay/player-controller semantics, and no
//! access to authority state.
//!
//! # Border ownership
//!
//! A [`CameraHandle`] names bridge-owned runtime view state scoped to an
//! initialized runtime session. It is not a pointer, a renderer object, or a
//! `StateStore` handle. Consumers propose bounded first-person camera input and
//! read deterministic pose/projection snapshots through manifest-backed runtime
//! bridge operations.
//!
//! # Matrix convention
//!
//! Projection snapshots use column-major 4×4 matrices, matching WebGL/Three.js
//! upload order. The generated TypeScript contract documents the same
//! convention so consumers can compare hashes or matrices without guessing.
//!
//! # Forbidden convenience logic
//!
//! Do not add movement integration, projection math, renderer adapters,
//! collision, sprint/crouch/head-bob, or product/game vocabulary here. Those
//! behaviors belong in runtime bridge implementation tasks, not the protocol
//! border.

#![forbid(unsafe_code)]

/// Opaque bridge-owned camera handle for runtime view/projection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CameraHandle(pub u64);

impl CameraHandle {
    #[inline]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Camera pose in world units/degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPose {
    pub position: [f32; 3],
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
}

/// Orthogonal basis vectors derived from a camera pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraBasis {
    pub forward: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
}

/// Perspective projection parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveProjection {
    pub fov_y_degrees: f32,
    pub near: f32,
    pub far: f32,
}

/// Pixel viewport dimensions for projection evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportSize {
    pub width: u32,
    pub height: u32,
}

/// Request to create a bridge-owned runtime view camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraCreateRequest {
    pub initial_pose: CameraPose,
    pub projection: PerspectiveProjection,
    pub viewport: ViewportSize,
}

/// Bounded first-person input for deterministic camera movement evidence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FirstPersonCameraInput {
    pub move_forward: f32,
    pub move_right: f32,
    pub move_up: f32,
    pub yaw_delta_degrees: f32,
    pub pitch_delta_degrees: f32,
    pub dt_seconds: f32,
    pub move_speed_units_per_second: f32,
}

/// One camera input proposal for a specific deterministic tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FirstPersonCameraInputEnvelope {
    pub camera: CameraHandle,
    pub input: FirstPersonCameraInput,
    pub tick: u64,
}

/// Request to read current projection evidence for a camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraProjectionRequest {
    pub camera: CameraHandle,
    pub viewport: Option<ViewportSize>,
}

/// Camera pose/basis snapshot after create or input application.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraSnapshot {
    pub camera: CameraHandle,
    pub tick: u64,
    pub pose: CameraPose,
    pub basis: CameraBasis,
    pub projection: PerspectiveProjection,
    pub viewport: ViewportSize,
}

/// Camera pose plus deterministic projection matrices.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraProjectionSnapshot {
    pub camera: CameraHandle,
    pub tick: u64,
    pub pose: CameraPose,
    pub basis: CameraBasis,
    pub projection: PerspectiveProjection,
    pub viewport: ViewportSize,
    /// Column-major 4×4 view matrix.
    pub view_matrix: [f32; 16],
    /// Column-major 4×4 projection matrix.
    pub projection_matrix: [f32; 16],
    /// Column-major 4×4 view-projection matrix.
    pub view_projection_matrix: [f32; 16],
    pub projection_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_handle_is_opaque_u64_newtype() {
        let handle = CameraHandle::new(42);
        assert_eq!(handle.raw(), 42);
    }

    #[test]
    fn camera_snapshot_carries_only_protocol_data() {
        let camera = CameraHandle::new(7);
        let pose = CameraPose {
            position: [0.0, 1.6, 0.0],
            yaw_degrees: 0.0,
            pitch_degrees: 0.0,
        };
        let basis = CameraBasis {
            forward: [0.0, 0.0, -1.0],
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
        };
        let projection = PerspectiveProjection {
            fov_y_degrees: 60.0,
            near: 0.1,
            far: 1000.0,
        };
        let viewport = ViewportSize {
            width: 1280,
            height: 720,
        };

        let snapshot = CameraSnapshot {
            camera,
            tick: 1,
            pose,
            basis,
            projection,
            viewport,
        };

        assert_eq!(snapshot.camera, camera);
        assert_eq!(snapshot.pose.position, [0.0, 1.6, 0.0]);
        assert_eq!(snapshot.viewport.width, 1280);
    }
}

//! Initial transform components for scene nodes.
//!
//! A scene document owns **initial** transforms only; authority owns runtime
//! transforms after bootstrap (scene-capability-01, "Transform ownership"). The
//! types here reuse [`core_math::Vec3`] rather than naked `{x,y,z}` so the
//! coordinate foundation stays shared, and add a small [`Quat`] for rotation
//! (`core-math` deliberately excludes quaternions).

use core_math::Vec3;

/// A rotation quaternion in `(x, y, z, w)` order — matching the render border's
/// `rotation` tuple so a later projection is a straight copy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    /// The identity rotation `(0, 0, 0, 1)`.
    pub const IDENTITY: Quat = Quat {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Squared norm; used to reject a degenerate (zero) rotation.
    pub fn norm_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    fn all_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }
}

impl Default for Quat {
    fn default() -> Self {
        Quat::IDENTITY
    }
}

/// A scene node's initial position, rotation, and scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl SceneTransform {
    /// The identity transform: origin, no rotation, unit scale.
    pub const IDENTITY: SceneTransform = SceneTransform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    pub const fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Compose one local authored transform beneath a parent transform. This is
    /// the canonical scene hierarchy rule used when stored local placement is
    /// seeded into flat runtime transform authority.
    pub fn compose(self, local: SceneTransform) -> SceneTransform {
        let scaled = Vec3::new(
            local.translation.x * self.scale.x,
            local.translation.y * self.scale.y,
            local.translation.z * self.scale.z,
        );
        let rotated = rotate_vector(self.rotation, scaled);
        SceneTransform {
            translation: self.translation + rotated,
            rotation: multiply_quat(self.rotation, local.rotation),
            scale: Vec3::new(
                self.scale.x * local.scale.x,
                self.scale.y * local.scale.y,
                self.scale.z * local.scale.z,
            ),
        }
    }

    /// Validate transform components: all finite, rotation non-degenerate, and
    /// no zero scale axis (a zero scale collapses geometry and breaks inverse
    /// transforms downstream).
    pub fn validate(&self) -> Result<(), TransformInvalid> {
        if !vec3_finite(self.translation) {
            return Err(TransformInvalid::NonFiniteTranslation);
        }
        if !self.rotation.all_finite() {
            return Err(TransformInvalid::NonFiniteRotation);
        }
        if self.rotation.norm_squared() <= f32::EPSILON {
            return Err(TransformInvalid::DegenerateRotation);
        }
        if !vec3_finite(self.scale) {
            return Err(TransformInvalid::NonFiniteScale);
        }
        if self.scale.x == 0.0 || self.scale.y == 0.0 || self.scale.z == 0.0 {
            return Err(TransformInvalid::ZeroScaleAxis);
        }
        Ok(())
    }
}

fn multiply_quat(a: Quat, b: Quat) -> Quat {
    Quat::new(
        a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    )
}

fn rotate_vector(rotation: Quat, vector: Vec3) -> Vec3 {
    let inverse_length = rotation.norm_squared().sqrt().recip();
    let q = Quat::new(
        rotation.x * inverse_length,
        rotation.y * inverse_length,
        rotation.z * inverse_length,
        rotation.w * inverse_length,
    );
    let v = Quat::new(vector.x, vector.y, vector.z, 0.0);
    let conjugate = Quat::new(-q.x, -q.y, -q.z, q.w);
    let rotated = multiply_quat(multiply_quat(q, v), conjugate);
    Vec3::new(rotated.x, rotated.y, rotated.z)
}

impl Default for SceneTransform {
    fn default() -> Self {
        SceneTransform::IDENTITY
    }
}

fn vec3_finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

/// Why a [`SceneTransform`] failed validation. Classified so a future protocol
/// diagnostic can report the exact axis of failure rather than a generic error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformInvalid {
    /// A translation component was NaN or infinite.
    NonFiniteTranslation,
    /// A rotation component was NaN or infinite.
    NonFiniteRotation,
    /// The rotation quaternion was (near) zero and cannot be normalized.
    DegenerateRotation,
    /// A scale component was NaN or infinite.
    NonFiniteScale,
    /// A scale axis was exactly zero.
    ZeroScaleAxis,
}

impl TransformInvalid {
    /// A short, stable label for diagnostics/serialization.
    pub fn label(self) -> &'static str {
        match self {
            TransformInvalid::NonFiniteTranslation => "non-finite-translation",
            TransformInvalid::NonFiniteRotation => "non-finite-rotation",
            TransformInvalid::DegenerateRotation => "degenerate-rotation",
            TransformInvalid::NonFiniteScale => "non-finite-scale",
            TransformInvalid::ZeroScaleAxis => "zero-scale-axis",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_valid() {
        assert_eq!(SceneTransform::IDENTITY.validate(), Ok(()));
        assert_eq!(SceneTransform::default(), SceneTransform::IDENTITY);
    }

    #[test]
    fn rejects_non_finite_and_zero_scale() {
        let mut t = SceneTransform::IDENTITY;
        t.translation = Vec3::new(f32::NAN, 0.0, 0.0);
        assert_eq!(t.validate(), Err(TransformInvalid::NonFiniteTranslation));

        let mut t = SceneTransform::IDENTITY;
        t.scale = Vec3::new(1.0, 0.0, 1.0);
        assert_eq!(t.validate(), Err(TransformInvalid::ZeroScaleAxis));

        let mut t = SceneTransform::IDENTITY;
        t.rotation = Quat::new(0.0, 0.0, 0.0, 0.0);
        assert_eq!(t.validate(), Err(TransformInvalid::DegenerateRotation));

        let mut t = SceneTransform::IDENTITY;
        t.scale = Vec3::new(1.0, f32::INFINITY, 1.0);
        assert_eq!(t.validate(), Err(TransformInvalid::NonFiniteScale));
    }
}

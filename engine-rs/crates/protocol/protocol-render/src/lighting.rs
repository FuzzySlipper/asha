//! Renderer-neutral ordinary scene light descriptors and numeric validation.

/// Renderer-neutral shadow request. A backend may degrade `Requested` to
/// disabled when its surface has no shadow-map support, but it must expose that
/// degradation in projection diagnostics rather than silently changing intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LightShadowIntent {
    #[default]
    Disabled,
    Requested,
}

/// Ordinary scene lights expressed without renderer objects or property bags.
///
/// Colours are linear RGB in `0.0..=1.0`; intensity is non-negative. Direction
/// vectors point from the light toward the illuminated scene and are normalized
/// by the renderer adapter. Point/spot range is in scene units (`None` means no
/// explicit cutoff), decay is a non-negative distance exponent, spot angles are
/// radians, and penumbra is `0.0..=1.0`.
#[derive(Debug, Clone, PartialEq)]
pub enum LightDescriptor {
    Ambient {
        color: [f32; 3],
        intensity: f32,
        enabled: bool,
        shadow_intent: LightShadowIntent,
    },
    Directional {
        color: [f32; 3],
        intensity: f32,
        enabled: bool,
        direction: [f32; 3],
        shadow_intent: LightShadowIntent,
    },
    Point {
        color: [f32; 3],
        intensity: f32,
        enabled: bool,
        position: [f32; 3],
        range: Option<f32>,
        decay: f32,
        shadow_intent: LightShadowIntent,
    },
    Spot {
        color: [f32; 3],
        intensity: f32,
        enabled: bool,
        position: [f32; 3],
        direction: [f32; 3],
        range: Option<f32>,
        decay: f32,
        outer_angle_radians: f32,
        penumbra: f32,
        shadow_intent: LightShadowIntent,
    },
}

/// Classified malformed light descriptor. This is Rust validation detail, not
/// a wire DTO; generated decoders reject structural mistakes before this layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightDescriptorError {
    InvalidColor,
    InvalidIntensity,
    InvalidPosition,
    InvalidDirection,
    InvalidRange,
    InvalidDecay,
    InvalidSpotAngle,
    InvalidPenumbra,
}

impl LightDescriptor {
    /// Validate numeric invariants before a descriptor reaches any renderer.
    pub fn validate(&self) -> Result<(), LightDescriptorError> {
        let (color, intensity) = match self {
            Self::Ambient {
                color, intensity, ..
            }
            | Self::Directional {
                color, intensity, ..
            }
            | Self::Point {
                color, intensity, ..
            }
            | Self::Spot {
                color, intensity, ..
            } => (color, *intensity),
        };
        if !color
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        {
            return Err(LightDescriptorError::InvalidColor);
        }
        if !intensity.is_finite() || intensity < 0.0 {
            return Err(LightDescriptorError::InvalidIntensity);
        }

        match self {
            Self::Ambient { .. } => Ok(()),
            Self::Directional { direction, .. } => validate_direction(*direction),
            Self::Point {
                position,
                range,
                decay,
                ..
            } => {
                validate_position(*position)?;
                validate_range_and_decay(*range, *decay)
            }
            Self::Spot {
                position,
                direction,
                range,
                decay,
                outer_angle_radians,
                penumbra,
                ..
            } => {
                validate_position(*position)?;
                validate_direction(*direction)?;
                validate_range_and_decay(*range, *decay)?;
                if !outer_angle_radians.is_finite()
                    || *outer_angle_radians <= 0.0
                    || *outer_angle_radians > std::f32::consts::FRAC_PI_2
                {
                    return Err(LightDescriptorError::InvalidSpotAngle);
                }
                if !penumbra.is_finite() || !(0.0..=1.0).contains(penumbra) {
                    return Err(LightDescriptorError::InvalidPenumbra);
                }
                Ok(())
            }
        }
    }

    pub const fn shadow_intent(&self) -> LightShadowIntent {
        match self {
            Self::Ambient { shadow_intent, .. }
            | Self::Directional { shadow_intent, .. }
            | Self::Point { shadow_intent, .. }
            | Self::Spot { shadow_intent, .. } => *shadow_intent,
        }
    }
}

fn validate_position(position: [f32; 3]) -> Result<(), LightDescriptorError> {
    position
        .iter()
        .all(|value| value.is_finite())
        .then_some(())
        .ok_or(LightDescriptorError::InvalidPosition)
}

fn validate_direction(direction: [f32; 3]) -> Result<(), LightDescriptorError> {
    if !direction.iter().all(|value| value.is_finite()) {
        return Err(LightDescriptorError::InvalidDirection);
    }
    let length_squared = direction.iter().map(|value| value * value).sum::<f32>();
    (length_squared > f32::EPSILON)
        .then_some(())
        .ok_or(LightDescriptorError::InvalidDirection)
}

fn validate_range_and_decay(range: Option<f32>, decay: f32) -> Result<(), LightDescriptorError> {
    if range.is_some_and(|value| !value.is_finite() || value <= 0.0) {
        return Err(LightDescriptorError::InvalidRange);
    }
    if !decay.is_finite() || decay < 0.0 {
        return Err(LightDescriptorError::InvalidDecay);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_light_kinds_validate_with_documented_units() {
        let lights = [
            LightDescriptor::Ambient {
                color: [0.2, 0.3, 0.4],
                intensity: 0.5,
                enabled: true,
                shadow_intent: LightShadowIntent::Disabled,
            },
            LightDescriptor::Directional {
                color: [1.0, 0.9, 0.8],
                intensity: 2.0,
                enabled: true,
                direction: [-1.0, -2.0, -1.0],
                shadow_intent: LightShadowIntent::Requested,
            },
            LightDescriptor::Point {
                color: [1.0, 0.4, 0.2],
                intensity: 4.0,
                enabled: true,
                position: [2.0, 3.0, 4.0],
                range: Some(12.0),
                decay: 2.0,
                shadow_intent: LightShadowIntent::Disabled,
            },
            LightDescriptor::Spot {
                color: [0.4, 0.6, 1.0],
                intensity: 6.0,
                enabled: true,
                position: [0.0, 8.0, 0.0],
                direction: [0.0, -1.0, 0.0],
                range: Some(20.0),
                decay: 2.0,
                outer_angle_radians: 0.7,
                penumbra: 0.25,
                shadow_intent: LightShadowIntent::Requested,
            },
        ];
        assert!(lights.iter().all(|light| light.validate().is_ok()));
    }

    #[test]
    fn malformed_light_numbers_are_classified() {
        let invalid_direction = LightDescriptor::Directional {
            color: [1.0; 3],
            intensity: 1.0,
            enabled: true,
            direction: [0.0; 3],
            shadow_intent: LightShadowIntent::Disabled,
        };
        assert_eq!(
            invalid_direction.validate(),
            Err(LightDescriptorError::InvalidDirection)
        );
        let invalid_cone = LightDescriptor::Spot {
            color: [1.0; 3],
            intensity: 1.0,
            enabled: true,
            position: [0.0; 3],
            direction: [0.0, -1.0, 0.0],
            range: Some(5.0),
            decay: 2.0,
            outer_angle_radians: std::f32::consts::PI,
            penumbra: 0.0,
            shadow_intent: LightShadowIntent::Disabled,
        };
        assert_eq!(
            invalid_cone.validate(),
            Err(LightDescriptorError::InvalidSpotAngle)
        );
    }
}

//! Public spatial-grid and editor-grid projection descriptors.
//!
//! These DTOs carry one explicit right-handed Y-up convention across project
//! settings, editor tooling, and renderer backends. They describe projection
//! intent only; authoritative world↔cell arithmetic remains in `core-space`.

use serde::{Deserialize, Serialize};

/// The one world coordinate system supported by current ASHA project data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpatialGridCoordinateSystem {
    RightHandedYUp,
}

/// Axis-aligned spatial grid with an explicit minimum-corner origin and cell size.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialGridSpec {
    pub coordinate_system: SpatialGridCoordinateSystem,
    pub origin: [f64; 3],
    pub spacing: [f64; 3],
}

/// Which pair of world axes an editor grid visualizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorGridPlane {
    Xz,
    Xy,
    Yz,
}

/// How transform translation proposals align to the active spatial grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpatialGridSnapAnchor {
    Boundary,
    CellCenter,
}

/// Renderer-neutral appearance and distance policy for an editor grid.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EditorGridStyle {
    pub minor_color: [f32; 4],
    pub major_color: [f32; 4],
    pub x_axis_color: [f32; 4],
    pub y_axis_color: [f32; 4],
    pub z_axis_color: [f32; 4],
    pub major_line_every: u32,
    pub opacity: f32,
    pub fade_start: f64,
    pub fade_end: f64,
}

/// Complete public grid projection intent consumed by renderer hosts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EditorGridDescriptor {
    pub visible: bool,
    pub grid: SpatialGridSpec,
    pub plane: EditorGridPlane,
    pub snap_anchor: SpatialGridSnapAnchor,
    pub style: EditorGridStyle,
}

/// Camera-derived world bounds covered by the current procedural grid projection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EditorGridBounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

/// Backend readout for the currently realized procedural editor grid.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EditorGridProjectionReadout {
    pub descriptor: EditorGridDescriptor,
    pub bounds: Option<EditorGridBounds>,
    pub minor_line_step: u32,
    pub rendered_line_count: u32,
}

/// Classified malformed editor-grid descriptor; validation detail, not wire data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorGridDescriptorError {
    UnsupportedCoordinateSystem,
    InvalidOrigin,
    InvalidSpacing,
    InvalidColor,
    InvalidMajorLineCadence,
    InvalidOpacity,
    InvalidFadeRange,
}

impl EditorGridDescriptor {
    pub fn validate(&self) -> Result<(), EditorGridDescriptorError> {
        if self.grid.coordinate_system != SpatialGridCoordinateSystem::RightHandedYUp {
            return Err(EditorGridDescriptorError::UnsupportedCoordinateSystem);
        }
        if !self.grid.origin.iter().all(|value| value.is_finite()) {
            return Err(EditorGridDescriptorError::InvalidOrigin);
        }
        if !self
            .grid
            .spacing
            .iter()
            .all(|value| value.is_finite() && *value > 0.0)
        {
            return Err(EditorGridDescriptorError::InvalidSpacing);
        }
        let colors = [
            self.style.minor_color,
            self.style.major_color,
            self.style.x_axis_color,
            self.style.y_axis_color,
            self.style.z_axis_color,
        ];
        if !colors
            .iter()
            .flatten()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        {
            return Err(EditorGridDescriptorError::InvalidColor);
        }
        if self.style.major_line_every == 0 {
            return Err(EditorGridDescriptorError::InvalidMajorLineCadence);
        }
        if !self.style.opacity.is_finite() || !(0.0..=1.0).contains(&self.style.opacity) {
            return Err(EditorGridDescriptorError::InvalidOpacity);
        }
        if !self.style.fade_start.is_finite()
            || !self.style.fade_end.is_finite()
            || self.style.fade_start < 0.0
            || self.style.fade_end <= self.style.fade_start
        {
            return Err(EditorGridDescriptorError::InvalidFadeRange);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_validation_enforces_y_up_and_numeric_bounds() {
        let descriptor = EditorGridDescriptor {
            visible: true,
            grid: SpatialGridSpec {
                coordinate_system: SpatialGridCoordinateSystem::RightHandedYUp,
                origin: [0.0; 3],
                spacing: [1.0; 3],
            },
            plane: EditorGridPlane::Xz,
            snap_anchor: SpatialGridSnapAnchor::Boundary,
            style: EditorGridStyle {
                minor_color: [0.1, 0.1, 0.1, 0.4],
                major_color: [0.2, 0.2, 0.2, 0.8],
                x_axis_color: [1.0, 0.0, 0.0, 1.0],
                y_axis_color: [0.0, 1.0, 0.0, 1.0],
                z_axis_color: [0.0, 0.0, 1.0, 1.0],
                major_line_every: 4,
                opacity: 1.0,
                fade_start: 12.0,
                fade_end: 48.0,
            },
        };
        assert_eq!(descriptor.validate(), Ok(()));
        assert_eq!(
            EditorGridDescriptor {
                grid: SpatialGridSpec {
                    spacing: [1.0, 0.0, 1.0],
                    ..descriptor.grid
                },
                ..descriptor
            }
            .validate(),
            Err(EditorGridDescriptorError::InvalidSpacing)
        );
        assert_eq!(
            EditorGridDescriptor {
                style: EditorGridStyle {
                    fade_end: 12.0,
                    ..descriptor.style
                },
                ..descriptor
            }
            .validate(),
            Err(EditorGridDescriptorError::InvalidFadeRange)
        );
    }
}

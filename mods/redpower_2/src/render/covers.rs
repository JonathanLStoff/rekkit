//! Microblock Sub-Part Model Tessellation & Bounding Boxes (`CoverRenderer` / `RenderCovers`).
//!
//! Handles all 29 microblock sub-parts:
//! - Parts 0..6: 6 face plates (Down, Up, North, South, West, East).
//! - Parts 6..14: 8 corner cubes.
//! - Parts 14..26: 12 edge strips.
//! - Parts 26..29: 3 centered structural columns / posts (Y, Z, X).
//!
//! Also handles 1/8th through 7/8th thickness slices and hollow face plate cutouts.

use crate::core::spatial::Vector3;
use crate::render::RenderContext;

/// Total number of microblock sub-parts per multipart tile.
pub const TOTAL_SUBPARTS: usize = 29;

/// Axis-aligned bounding box defined by minimum [x1, y1, z1] and maximum [x2, y2, z2].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: Vector3,
    pub max: Vector3,
}

impl BoundingBox {
    pub fn new(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> Self {
        Self {
            min: Vector3::new(x1.min(x2), y1.min(y2), z1.min(z2)),
            max: Vector3::new(x1.max(x2), y1.max(y2), z1.max(z2)),
        }
    }

    /// Check if ray from `origin` with `dir` intersects this box. Returns hit distance `t`.
    pub fn intersect_ray(&self, origin: Vector3, dir: Vector3) -> Option<f64> {
        let mut tmin = f64::NEG_INFINITY;
        let mut tmax = f64::INFINITY;

        // X axis
        if dir.x.abs() > 1e-9 {
            let tx1 = (self.min.x - origin.x) / dir.x;
            let tx2 = (self.max.x - origin.x) / dir.x;
            tmin = tmin.max(tx1.min(tx2));
            tmax = tmax.min(tx1.max(tx2));
        } else if origin.x < self.min.x || origin.x > self.max.x {
            return None;
        }

        // Y axis
        if dir.y.abs() > 1e-9 {
            let ty1 = (self.min.y - origin.y) / dir.y;
            let ty2 = (self.max.y - origin.y) / dir.y;
            tmin = tmin.max(ty1.min(ty2));
            tmax = tmax.min(ty1.max(ty2));
        } else if origin.y < self.min.y || origin.y > self.max.y {
            return None;
        }

        // Z axis
        if dir.z.abs() > 1e-9 {
            let tz1 = (self.min.z - origin.z) / dir.z;
            let tz2 = (self.max.z - origin.z) / dir.z;
            tmin = tmin.max(tz1.min(tz2));
            tmax = tmax.min(tz1.max(tz2));
        } else if origin.z < self.min.z || origin.z > self.max.z {
            return None;
        }

        if tmax >= tmin && tmax >= 0.0 {
            Some(if tmin >= 0.0 { tmin } else { tmax })
        } else {
            None
        }
    }
}

/// Compute the base bounding box for subpart `part` (0..29) with thickness `th` (0.0..1.0).
pub fn subpart_box(part: usize, th: f64) -> BoundingBox {
    let th = th.clamp(0.0, 1.0);
    match part {
        // Faces (0..6)
        0 => BoundingBox::new(0.0, 0.0, 0.0, 1.0, th, 1.0), // Down (Y-)
        1 => BoundingBox::new(0.0, 1.0 - th, 0.0, 1.0, 1.0, 1.0), // Up (Y+)
        2 => BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, th), // North (Z-)
        3 => BoundingBox::new(0.0, 0.0, 1.0 - th, 1.0, 1.0, 1.0), // South (Z+)
        4 => BoundingBox::new(0.0, 0.0, 0.0, th, 1.0, 1.0), // West (X-)
        5 => BoundingBox::new(1.0 - th, 0.0, 0.0, 1.0, 1.0, 1.0), // East (X+)

        // Corners (6..14)
        6 => BoundingBox::new(0.0, 0.0, 0.0, th, th, th),
        7 => BoundingBox::new(0.0, 0.0, 1.0 - th, th, th, 1.0),
        8 => BoundingBox::new(1.0 - th, 0.0, 0.0, 1.0, th, th),
        9 => BoundingBox::new(1.0 - th, 0.0, 1.0 - th, 1.0, th, 1.0),
        10 => BoundingBox::new(0.0, 1.0 - th, 0.0, th, 1.0, th),
        11 => BoundingBox::new(0.0, 1.0 - th, 1.0 - th, th, 1.0, 1.0),
        12 => BoundingBox::new(1.0 - th, 1.0 - th, 0.0, 1.0, 1.0, th),
        13 => BoundingBox::new(1.0 - th, 1.0 - th, 1.0 - th, 1.0, 1.0, 1.0),

        // Strips (14..26)
        14 => BoundingBox::new(0.0, 0.0, 0.0, 1.0, th, th),
        15 => BoundingBox::new(0.0, 0.0, 1.0 - th, 1.0, th, 1.0),
        16 => BoundingBox::new(0.0, 0.0, 0.0, th, th, 1.0),
        17 => BoundingBox::new(1.0 - th, 0.0, 0.0, 1.0, th, 1.0),
        18 => BoundingBox::new(0.0, 0.0, 0.0, th, 1.0, th),
        19 => BoundingBox::new(0.0, 0.0, 1.0 - th, th, 1.0, 1.0),
        20 => BoundingBox::new(1.0 - th, 0.0, 0.0, 1.0, 1.0, th),
        21 => BoundingBox::new(1.0 - th, 0.0, 1.0 - th, 1.0, 1.0, 1.0),
        22 => BoundingBox::new(0.0, 1.0 - th, 0.0, 1.0, 1.0, th),
        23 => BoundingBox::new(0.0, 1.0 - th, 1.0 - th, 1.0, 1.0, 1.0),
        24 => BoundingBox::new(0.0, 1.0 - th, 0.0, th, 1.0, 1.0),
        25 => BoundingBox::new(1.0 - th, 1.0 - th, 0.0, 1.0, 1.0, 1.0),

        // Centered posts / columns (26..29)
        26 => BoundingBox::new(0.5 - th, 0.0, 0.5 - th, 0.5 + th, 1.0, 0.5 + th),
        27 => BoundingBox::new(0.5 - th, 0.5 - th, 0.0, 0.5 + th, 0.5 + th, 1.0),
        28 => BoundingBox::new(0.0, 0.5 - th, 0.5 - th, 1.0, 0.5 + th, 0.5 + th),

        _ => BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
    }
}

/// Compute the 4 sub-boxes of a hollow face plate (leaving a 0.25..0.75 centered aperture).
pub fn hollow_face_boxes(part: usize, th: f64) -> [BoundingBox; 4] {
    let th = th.clamp(0.0, 1.0);
    match part {
        // Y- or Y+
        0 | 1 => {
            let (y1, y2) = if part == 0 {
                (0.0, th)
            } else {
                (1.0 - th, 1.0)
            };
            [
                BoundingBox::new(0.0, y1, 0.0, 0.25, y2, 1.0),
                BoundingBox::new(0.75, y1, 0.0, 1.0, y2, 1.0),
                BoundingBox::new(0.25, y1, 0.0, 0.75, y2, 0.25),
                BoundingBox::new(0.25, y1, 0.75, 0.75, y2, 1.0),
            ]
        }
        // Z- or Z+
        2 | 3 => {
            let (z1, z2) = if part == 2 {
                (0.0, th)
            } else {
                (1.0 - th, 1.0)
            };
            [
                BoundingBox::new(0.0, 0.0, z1, 0.25, 1.0, z2),
                BoundingBox::new(0.75, 0.0, z1, 1.0, 1.0, z2),
                BoundingBox::new(0.25, 0.0, z1, 0.75, 0.25, z2),
                BoundingBox::new(0.25, 0.75, z1, 0.75, 1.0, z2),
            ]
        }
        // X- or X+
        _ => {
            let (x1, x2) = if part == 4 {
                (0.0, th)
            } else {
                (1.0 - th, 1.0)
            };
            [
                BoundingBox::new(x1, 0.0, 0.0, x2, 1.0, 0.25),
                BoundingBox::new(x1, 0.0, 0.75, x2, 1.0, 1.0),
                BoundingBox::new(x1, 0.0, 0.25, x2, 0.25, 0.75),
                BoundingBox::new(x1, 0.75, 0.25, x2, 1.0, 0.75),
            ]
        }
    }
}

/// Renderer constructing geometries for solid and hollow microblock covers.
pub struct CoverRenderer;

impl CoverRenderer {
    /// Tessellate a subpart into `RenderContext`.
    pub fn render_part(
        context: &mut RenderContext,
        part: usize,
        th: f64,
        is_hollow: bool,
        texture_index: i32,
    ) {
        context.set_tex_all(texture_index);
        if is_hollow && part < 6 {
            for b in &hollow_face_boxes(part, th) {
                context.set_size(b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z);
                context.calc_bounds();
                context.do_mapping_box(0x3F);
            }
        } else {
            let b = subpart_box(part, th);
            context.set_size(b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z);
            context.calc_bounds();
            context.do_mapping_box(0x3F);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_29_subparts_bounds() {
        let th = 0.125; // 1/8th cover thickness
        for part in 0..TOTAL_SUBPARTS {
            let b = subpart_box(part, th);
            assert!(b.min.x <= b.max.x);
            assert!(b.min.y <= b.max.y);
            assert!(b.min.z <= b.max.z);
            assert!(b.min.x >= 0.0 && b.max.x <= 1.0);
            assert!(b.min.y >= 0.0 && b.max.y <= 1.0);
            assert!(b.min.z >= 0.0 && b.max.z <= 1.0);
        }
    }

    #[test]
    fn test_hollow_cutout_aperture() {
        let th = 0.125;
        for face in 0..6 {
            let boxes = hollow_face_boxes(face, th);
            assert_eq!(boxes.len(), 4);
            // Verify center (0.5, 0.5) inside the face is NOT contained in any of the 4 boxes
            let center_hit = match face {
                0 | 1 => boxes
                    .iter()
                    .any(|b| b.min.x < 0.5 && b.max.x > 0.5 && b.min.z < 0.5 && b.max.z > 0.5),
                2 | 3 => boxes
                    .iter()
                    .any(|b| b.min.x < 0.5 && b.max.x > 0.5 && b.min.y < 0.5 && b.max.y > 0.5),
                _ => boxes
                    .iter()
                    .any(|b| b.min.z < 0.5 && b.max.z > 0.5 && b.min.y < 0.5 && b.max.y > 0.5),
            };
            assert!(
                !center_hit,
                "Hollow face {} must have an open center hole",
                face
            );
        }
    }
}

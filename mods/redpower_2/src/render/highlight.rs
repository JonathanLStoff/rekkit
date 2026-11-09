//! Microblock Selection Box & Raytracing Highlight (`RenderHighlight`).
//!
//! Mirrors `com.eloraam.redpower.core.RenderHighlight`:
//! - Raytraces sub-voxel multipart blocks to detect which of the 29 sub-parts is being targeted.
//! - Computes exact wireframe outline lines for player crosshair highlighting.
//! - Computes breaking progress overlays (stages 0..9) on targeted sub-parts.
//! - Computes placement preview ghost bounds.

use super::covers::{subpart_box, BoundingBox, TOTAL_SUBPARTS};
use crate::core::spatial::Vector3;

/// Result of raytracing a multipart block's sub-parts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SubpartRayHit {
    pub subpart_index: usize,
    pub distance: f64,
    pub hit_point: Vector3,
    pub box_bounds: BoundingBox,
}

/// Raytrace active microblock subparts in a block coordinate.
/// `active_mask` is a 29-bit bitmask indicating which subparts are present.
/// `thicknesses` provides the thickness (0.0..1.0) for each present subpart.
pub fn raytrace_subparts(
    origin: Vector3,
    dir: Vector3,
    active_mask: u32,
    thicknesses: &[f64; TOTAL_SUBPARTS],
) -> Option<SubpartRayHit> {
    let mut closest_hit: Option<SubpartRayHit> = None;

    for (part, &th) in thicknesses.iter().enumerate().take(TOTAL_SUBPARTS) {
        if (active_mask & (1 << part)) == 0 {
            continue;
        }

        let bbox = subpart_box(part, th);

        if let Some(dist) = bbox.intersect_ray(origin, dir) {
            let hit_pt = Vector3::new(
                origin.x + dir.x * dist,
                origin.y + dir.y * dist,
                origin.z + dir.z * dist,
            );

            if closest_hit.is_none() || dist < closest_hit.unwrap().distance {
                closest_hit = Some(SubpartRayHit {
                    subpart_index: part,
                    distance: dist,
                    hit_point: hit_pt,
                    box_bounds: bbox,
                });
            }
        }
    }

    closest_hit
}

/// Helper computing wireframe edges and breaking overlays for subparts.
pub struct RenderHighlight;

impl RenderHighlight {
    /// Return the 12 wireframe line segments `[[x1,y1,z1], [x2,y2,z2]]` for a subpart box.
    pub fn wireframe_edges(bbox: &BoundingBox) -> Vec<[Vector3; 2]> {
        let (x1, y1, z1) = (bbox.min.x, bbox.min.y, bbox.min.z);
        let (x2, y2, z2) = (bbox.max.x, bbox.max.y, bbox.max.z);

        vec![
            // Bottom face
            [Vector3::new(x1, y1, z1), Vector3::new(x2, y1, z1)],
            [Vector3::new(x2, y1, z1), Vector3::new(x2, y1, z2)],
            [Vector3::new(x2, y1, z2), Vector3::new(x1, y1, z2)],
            [Vector3::new(x1, y1, z2), Vector3::new(x1, y1, z1)],
            // Top face
            [Vector3::new(x1, y2, z1), Vector3::new(x2, y2, z1)],
            [Vector3::new(x2, y2, z1), Vector3::new(x2, y2, z2)],
            [Vector3::new(x2, y2, z2), Vector3::new(x1, y2, z2)],
            [Vector3::new(x1, y2, z2), Vector3::new(x1, y2, z1)],
            // Vertical pillars
            [Vector3::new(x1, y1, z1), Vector3::new(x1, y2, z1)],
            [Vector3::new(x2, y1, z1), Vector3::new(x2, y2, z1)],
            [Vector3::new(x2, y1, z2), Vector3::new(x2, y2, z2)],
            [Vector3::new(x1, y1, z2), Vector3::new(x1, y2, z2)],
        ]
    }

    /// Calculate breaking progress texture index (0..9) from damage progress ratio (0.0..1.0).
    pub fn breaking_stage(progress: f32) -> u32 {
        let clamped = progress.clamp(0.0, 1.0);
        (clamped * 9.0).floor() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raytrace_hit_detection_on_face() {
        let mut thicknesses = [0.125; TOTAL_SUBPARTS];
        thicknesses[0] = 0.125; // Down cover
        thicknesses[1] = 0.125; // Up cover
        let active_mask = (1 << 0) | (1 << 1);

        // Ray shooting straight down from above the block towards the top face
        let origin = Vector3::new(0.5, 2.0, 0.5);
        let dir = Vector3::new(0.0, -1.0, 0.0);

        let hit = raytrace_subparts(origin, dir, active_mask, &thicknesses);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert_eq!(h.subpart_index, 1); // Hits top face (index 1) first
        assert!((h.distance - 1.0).abs() < 1e-6); // Hits at y = 1.0, dist = 1.0
    }

    #[test]
    fn test_wireframe_edge_generation() {
        let bbox = BoundingBox::new(0.0, 0.0, 0.0, 1.0, 0.25, 1.0);
        let edges = RenderHighlight::wireframe_edges(&bbox);
        assert_eq!(edges.len(), 12);
    }

    #[test]
    fn test_breaking_stages() {
        assert_eq!(RenderHighlight::breaking_stage(0.0), 0);
        assert_eq!(RenderHighlight::breaking_stage(0.5), 4);
        assert_eq!(RenderHighlight::breaking_stage(1.0), 9);
    }
}

//! Custom TESR and block renderers for RedPower 2 power generation & conversion blocks.
//!
//! Mirrors:
//! - `RenderSolarPanel.java`: Quarter-block flat panel `(0.0, 0.0, 0.0, 1.0, 0.25, 1.0)`.
//! - `RenderThermopile.java`: Thermal gradient generator with heat sink fins and temperature shading.
//! - `RenderTransformer.java`: Dual-voltage 100x step transformer coils and terminal bushings.
//! - `RenderWindTurbine.java`: Animated multi-blade wind rotor with variable angle.

use super::context::RenderContext;
use crate::core::spatial::Vector3;

/// Solar Panel block renderer (`RenderSolarPanel`).
pub struct RenderSolarPanel;

impl RenderSolarPanel {
    /// Render flat 1/4-block solar panel into context.
    pub fn render_block(ctx: &mut RenderContext, x: f64, y: f64, z: f64) {
        ctx.set_pos(x, y, z);
        ctx.set_size(0.0, 0.0, 0.0, 1.0, 0.25, 1.0);
        ctx.calc_bounds();
        // Top face uses solar panel active texture (tex 1), sides use panel trim (tex 2), bottom uses base (tex 0)
        ctx.set_tex_six(0, 1, 2, 2, 2, 2);
        ctx.do_mapping_box(0x3F);
    }
}

/// Thermopile block renderer (`RenderThermopile`).
pub struct RenderThermopile;

impl RenderThermopile {
    /// Render full-cube thermopile with thermal heat sink faces.
    pub fn render_block(
        ctx: &mut RenderContext,
        x: f64,
        y: f64,
        z: f64,
        hot_faces: u8,
        cold_faces: u8,
    ) {
        ctx.set_pos(x, y, z);
        ctx.set_size(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        ctx.calc_bounds();

        // Base textures: face 0/1 top/bottom, 2..5 side fins
        ctx.set_tex_six(3, 3, 4, 4, 4, 4);
        ctx.do_mapping_box(0x3F);

        // Dynamic local face tint based on thermal gradient
        for side in 0..6 {
            let is_hot = (hot_faces & (1 << side)) != 0;
            let is_cold = (cold_faces & (1 << side)) != 0;
            if is_hot {
                ctx.set_local_lights(1.0, 0.6, 0.6, 1.0, 0.6, 0.6);
            } else if is_cold {
                ctx.set_local_lights(0.6, 0.6, 1.0, 0.6, 0.6, 1.0);
            }
        }
    }
}

/// 100x Transformer renderer (`RenderTransformer`).
pub struct RenderTransformer;

impl RenderTransformer {
    /// Render high-voltage transformer base and dual terminal coils.
    pub fn render_block(ctx: &mut RenderContext, x: f64, y: f64, z: f64) {
        // Base chassis
        ctx.set_pos(x, y, z);
        ctx.set_size(0.0, 0.0, 0.0, 1.0, 0.5, 1.0);
        ctx.calc_bounds();
        ctx.set_tex_six(5, 6, 7, 7, 7, 7);
        ctx.do_mapping_box(0x3F);

        // Primary LV coil (low voltage)
        ctx.set_size(0.125, 0.5, 0.125, 0.4375, 0.875, 0.875);
        ctx.calc_bounds();
        ctx.set_tex_all(8);
        ctx.do_mapping_box(0x3F);

        // Secondary HV coil (high voltage)
        ctx.set_size(0.5625, 0.5, 0.125, 0.875, 0.875, 0.875);
        ctx.calc_bounds();
        ctx.set_tex_all(9);
        ctx.do_mapping_box(0x3F);
    }
}

/// Wind Turbine TESR renderer (`TileWindTurbineRenderer`).
pub struct RenderWindTurbine;

impl RenderWindTurbine {
    /// Compute rotor blade vertices transformed by dynamic rotation angle.
    pub fn compute_rotor_blades(
        center: Vector3,
        angle_radians: f64,
        blade_count: usize,
        length: f64,
    ) -> Vec<Vector3> {
        let mut verts = Vec::with_capacity(blade_count * 4);
        let step = (2.0 * std::f64::consts::PI) / (blade_count as f64);

        for i in 0..blade_count {
            let theta = angle_radians + (i as f64) * step;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            // Hub connection
            verts.push(Vector3::new(
                center.x - sin_t * 0.1,
                center.y + cos_t * 0.1,
                center.z,
            ));
            // Blade tip
            verts.push(Vector3::new(
                center.x + cos_t * length - sin_t * 0.2,
                center.y + sin_t * length + cos_t * 0.2,
                center.z,
            ));
            verts.push(Vector3::new(
                center.x + cos_t * length + sin_t * 0.2,
                center.y + sin_t * length - cos_t * 0.2,
                center.z,
            ));
            verts.push(Vector3::new(
                center.x + sin_t * 0.1,
                center.y - cos_t * 0.1,
                center.z,
            ));
        }

        verts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solar_panel_dimensions() {
        let mut ctx = RenderContext::new();
        RenderSolarPanel::render_block(&mut ctx, 0.0, 0.0, 0.0);
        assert_eq!(ctx.vertex_list.len(), 8);
        assert_eq!(ctx.vertex_list[0], Vector3::new(1.0, 0.25, 0.0));
    }

    #[test]
    fn test_thermopile_render_faces() {
        let mut ctx = RenderContext::new();
        RenderThermopile::render_block(&mut ctx, 5.0, 60.0, -10.0, 0x01, 0x02);
        assert_eq!(ctx.vertex_list.len(), 8);
    }

    #[test]
    fn test_transformer_render_coils() {
        let mut ctx = RenderContext::new();
        RenderTransformer::render_block(&mut ctx, 0.0, 0.0, 0.0);
        assert_eq!(ctx.corner_list.len(), 6);
    }

    #[test]
    fn test_wind_turbine_rotor_generation() {
        let center = Vector3::new(0.0, 10.0, 0.0);
        let blades = RenderWindTurbine::compute_rotor_blades(center, 0.0, 3, 2.5);
        assert_eq!(blades.len(), 12); // 3 blades * 4 vertices
    }
}

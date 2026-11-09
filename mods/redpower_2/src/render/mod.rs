//! RedPower 2 sub-voxel transformation, procedural meshes, and model rendering engine.

pub mod context;
pub mod covers;
pub mod highlight;
pub mod lib;
pub mod machine_power_renders;
pub mod model;

pub use context::{blend_light, RenderContext, TEX_ROT_TABLE};
pub use lib::{
    generate_pointer, generate_special_lever, registry, registry_mut, RenderCustomBlock,
    RenderListEntry, RenderRegistry, RenderVertex, TextureAnimFX,
};
pub use machine_power_renders::{
    RenderSolarPanel, RenderThermopile, RenderTransformer, RenderWindTurbine,
};
pub use model::{ParseError, RenderModel, TexVertex};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Quat;
    use crate::core::spatial::Vector3;

    #[test]
    fn test_vertex_transformation_and_box_corners() {
        let mut ctx = RenderContext::new();
        ctx.set_pos(10.0, 20.0, 30.0);
        ctx.set_size(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        ctx.calc_bounds();

        assert_eq!(ctx.vertex_list.len(), 8);
        // Without rotation and offset, box corners span 10.0..11.0, 20.0..21.0, 30.0..31.0
        assert_eq!(ctx.vertex_list[0], Vector3::new(11.0, 21.0, 30.0));
        assert_eq!(ctx.vertex_list[1], Vector3::new(10.0, 21.0, 30.0));
        assert_eq!(ctx.vertex_list[6], Vector3::new(10.0, 20.0, 31.0));
    }

    #[test]
    fn test_uv_mapping_and_exact_coordinates() {
        let mut ctx = RenderContext::new();
        ctx.exact_texture_coordinates = true;
        ctx.set_tex_all(17); // tile 17 -> tx = (17&15)*0.0625 = 1*0.0625, ty = (17>>4)*0.0625 = 1*0.0625
        ctx.set_size(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        ctx.do_mapping_box(0x3F);

        // Face 0: bottom face (y-)
        let face0 = &ctx.corner_list[0];
        assert_eq!(face0.len(), 4);
        assert!((face0[0].u - 0.0625).abs() < 1e-6);
        assert!((face0[0].v - 0.0625).abs() < 1e-6);
    }

    #[test]
    fn test_texture_rotation_table_lookups() {
        let mut ctx = RenderContext::new();
        ctx.set_tex_six(0, 1, 2, 3, 4, 5);
        ctx.orient_texture_new(1); // Down = 0, Rot = 1
        assert_eq!(ctx.tex_index, [0, 1, 4, 5, 3, 2]);

        ctx.set_tex_six(0, 1, 2, 3, 4, 5);
        ctx.orient_texture_new(4); // Down = 1, Rot = 0
        assert_eq!(ctx.tex_index, [1, 0, 2, 3, 5, 4]);
    }

    #[test]
    fn test_model_scaling_and_binding() {
        let mut model = RenderModel::default();
        model.vertex_list.push(Vector3::new(1.0, 2.0, 3.0));
        model.vertex_list.push(Vector3::new(4.0, 5.0, 6.0));
        model.tex_list.push([
            TexVertex::new(0, 0.0, 0.0),
            TexVertex::new(1, 1.0, 0.0),
            TexVertex::new(1, 1.0, 1.0),
            TexVertex::new(0, 0.0, 1.0),
        ]);

        model.scale(2.0);
        assert_eq!(model.vertex_list[0], Vector3::new(2.0, 4.0, 6.0));
        assert_eq!(model.vertex_list[1], Vector3::new(8.0, 10.0, 12.0));

        let mut ctx = RenderContext::new();
        ctx.set_pos(1.0, 1.0, 1.0);
        ctx.bind_model(&model);
        assert_eq!(ctx.vertex_list[0], Vector3::new(3.0, 5.0, 7.0));
        assert_eq!(ctx.vertex_list[1], Vector3::new(9.0, 11.0, 13.0));
    }

    #[test]
    fn test_procedural_pointer_and_lever_generation() {
        let lever_verts = generate_special_lever(Vector3::ZERO, Quat::IDENTITY, 0);
        assert_eq!(lever_verts.len(), 24);

        let pointer_verts = generate_pointer(Vector3::ZERO, Quat::IDENTITY);
        assert_eq!(pointer_verts.len(), 24);
        assert_eq!(pointer_verts[0].r, 0.9);
        assert_eq!(pointer_verts[8].r, 0.6);
    }

    #[test]
    fn test_texture_anim_fx_tick_and_rgb_conversion() {
        let rgb_data = vec![0xFF112233u32, 0x88445566u32];
        let bytes = TextureAnimFX::convert_rgb(&rgb_data);
        assert_eq!(bytes, vec![0x11, 0x22, 0x33, 0xFF, 0x44, 0x55, 0x66, 0x88]);

        let mut anim = TextureAnimFX::new(2, vec![vec![0xFF000000], vec![0xFFFFFFFF]]);
        assert_eq!(anim.index, 0);
        anim.tick();
        assert_eq!(anim.index, 0);
        anim.tick();
        assert_eq!(anim.index, 1);
    }

    #[test]
    fn test_blend_light_and_ao_smooth() {
        let blended = blend_light(0x00FF00FF, 0x00FF00FF, 0x00FF00FF, 0x00FF00FF);
        assert_eq!(blended, 0x00FF00FF);

        let mut ctx = RenderContext::new();
        ctx.set_size(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        ctx.do_light_smooth(0x3F);
        assert_eq!(ctx.corner_list.len(), 6);
    }
}

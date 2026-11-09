//! Sub-voxel matrix transformation, texture rotation, and smooth ambient occlusion lighting.
//!
//! Direct port of `com.eloraam.redpower.core.RenderContext`.

use super::lib::RenderVertex;
use crate::core::math::{orient_matrix, Matrix3};
use crate::core::model_loader::{RenderModel, TexVertex};
use crate::core::spatial::Vector3;

/// Standard texture rotation and mapping table for all 24 orientations + 2 extensions.
///
/// Ports `RenderContext.texRotTable`.
pub const TEX_ROT_TABLE: [[i32; 9]; 26] = [
    [0, 1, 2, 3, 4, 5, 0, 112347, 0],
    [0, 1, 4, 5, 3, 2, 45, 112320, 27],
    [0, 1, 3, 2, 5, 4, 27, 112347, 0],
    [0, 1, 5, 4, 2, 3, 54, 112320, 27],
    [1, 0, 2, 3, 5, 4, 112347, 112347, 0],
    [1, 0, 4, 5, 2, 3, 112374, 112320, 27],
    [1, 0, 3, 2, 4, 5, 112320, 112347, 0],
    [1, 0, 5, 4, 3, 2, 112365, 112320, 27],
    [4, 5, 0, 1, 2, 3, 217134, 1728, 110619],
    [3, 2, 0, 1, 4, 5, 220014, 0, 112347],
    [5, 4, 0, 1, 3, 2, 218862, 1728, 110619],
    [2, 3, 0, 1, 5, 4, 220590, 0, 112347],
    [4, 5, 1, 0, 3, 2, 188469, 1728, 110619],
    [3, 2, 1, 0, 5, 4, 191349, 0, 112347],
    [5, 4, 1, 0, 2, 3, 190197, 1728, 110619],
    [2, 3, 1, 0, 4, 5, 191925, 0, 112347],
    [4, 5, 3, 2, 0, 1, 2944, 110619, 1728],
    [3, 2, 5, 4, 0, 1, 187264, 27, 112320],
    [5, 4, 2, 3, 0, 1, 113536, 110619, 1728],
    [2, 3, 4, 5, 0, 1, 224128, 27, 112320],
    [4, 5, 2, 3, 1, 0, 3419, 110619, 1728],
    [3, 2, 4, 5, 1, 0, 187739, 27, 112320],
    [5, 4, 3, 2, 1, 0, 114011, 110619, 1728],
    [2, 3, 5, 4, 1, 0, 224603, 27, 112320],
    // Padding entries matching Java array length 26
    [0, 1, 2, 3, 4, 5, 0, 0, 0],
    [0, 1, 2, 3, 4, 5, 0, 0, 0],
];

/// Blend 4 packed light values matching `RenderContext.blendLight`.
pub fn blend_light(i: i32, mut j: i32, mut k: i32, mut l: i32) -> i32 {
    if j == 0 {
        j = i;
    }
    if k == 0 {
        k = i;
    }
    if l == 0 {
        l = i;
    }
    ((i + j + k + l) >> 2) & 0x00FF00FF
}

/// Dynamic sub-voxel geometry transformation and lighting context.
///
/// Directly ports `com.eloraam.redpower.core.RenderContext`.
#[derive(Debug, Clone)]
pub struct RenderContext {
    pub basis: Matrix3,
    pub local_offset: Vector3,
    pub global_origin: Vector3,
    pub box_size1: Vector3,
    pub box_size2: Vector3,
    pub bound_model: Option<RenderModel>,
    pub vertex_list: Vec<Vector3>,
    vertex_list_box: [Vector3; 8],
    pub corner_list: Vec<[TexVertex; 4]>,
    corner_list_box: [[TexVertex; 4]; 6],
    pub tex_binds: Option<[Option<String>; 6]>,
    pub tex_index_box: [i32; 6],
    pub tex_index: [i32; 6],
    pub tex_index_list: Option<Vec<[i32; 6]>>,
    pub lock_texture: bool,
    pub exact_texture_coordinates: bool,
    pub tex_flags: i32,
    pub use_normal: bool,
    pub force_flat: bool,
    pub tint_r: f32,
    pub tint_g: f32,
    pub tint_b: f32,
    pub tint_a: f32,
    pub light_local: [f32; 6],
    pub bright_local: [i32; 6],
    pub light_global: [[[i32; 3]; 3]; 3],
    pub ao_global: [[[f32; 3]; 3]; 3],
    pub light_flat: [f32; 6],
    pub glob_trans: i32,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderContext {
    pub fn new() -> Self {
        let vertex_list_box = [Vector3::ZERO; 8];
        let mut corner_list_box = [[TexVertex::default(); 4]; 6];
        let vtxl: [[i32; 4]; 6] = [
            [7, 6, 5, 4],
            [0, 1, 2, 3],
            [0, 4, 5, 1],
            [2, 6, 7, 3],
            [1, 5, 6, 2],
            [3, 7, 4, 0],
        ];

        for var4 in 0..6 {
            for j in 0..4 {
                corner_list_box[var4][j] = TexVertex::default();
                corner_list_box[var4][j].vtx = vtxl[var4][j];
            }
        }

        let mut ctx = Self {
            basis: Matrix3::IDENTITY,
            local_offset: Vector3::ZERO,
            global_origin: Vector3::ZERO,
            box_size1: Vector3::ZERO,
            box_size2: Vector3::ZERO,
            bound_model: None,
            vertex_list: vertex_list_box.to_vec(),
            vertex_list_box,
            corner_list: corner_list_box.to_vec(),
            corner_list_box,
            tex_binds: None,
            tex_index_box: [0; 6],
            tex_index: [0; 6],
            tex_index_list: None,
            lock_texture: false,
            exact_texture_coordinates: false,
            tex_flags: 0,
            use_normal: false,
            force_flat: false,
            tint_r: 1.0,
            tint_g: 1.0,
            tint_b: 1.0,
            tint_a: 1.0,
            light_local: [1.0; 6],
            bright_local: [15728880; 6],
            light_global: [[[0; 3]; 3]; 3],
            ao_global: [[[1.0; 3]; 3]; 3],
            light_flat: [1.0; 6],
            glob_trans: 0,
        };
        ctx.set_defaults();
        ctx
    }

    pub fn set_defaults(&mut self) {
        self.local_offset = Vector3::ZERO;
        self.set_orientation(0, 0);
        self.tex_flags = 0;
        self.tint_r = 1.0;
        self.tint_g = 1.0;
        self.tint_b = 1.0;
        self.tint_a = 1.0;
        self.set_local_lights_scalar(1.0);
        self.set_brightness(15728880);
    }

    pub fn set_pos(&mut self, x: f64, y: f64, z: f64) {
        self.global_origin = Vector3::new(x, y, z);
    }

    pub fn set_pos_vec(&mut self, v: Vector3) {
        self.global_origin = v;
    }

    pub fn set_rel_pos(&mut self, x: f64, y: f64, z: f64) {
        self.local_offset = Vector3::new(x, y, z);
    }

    pub fn set_rel_pos_vec(&mut self, v: Vector3) {
        self.local_offset = v;
    }

    pub fn set_orientation(&mut self, down: i32, rot: i32) {
        self.basis = orient_matrix(down.max(0) as usize, rot.max(0) as usize);
    }

    pub fn set_size(&mut self, tx: f64, ty: f64, tz: f64, bx: f64, by: f64, bz: f64) {
        self.box_size1 = Vector3::new(tx, ty, tz);
        self.box_size2 = Vector3::new(bx, by, bz);
    }

    pub fn set_tex_flags(&mut self, fl: i32) {
        self.tex_flags = fl;
    }

    pub fn set_tex_six(&mut self, a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) {
        if !self.lock_texture {
            self.tex_index_box = [a, b, c, d, e, f];
            self.tex_index = self.tex_index_box;
        }
    }

    pub fn set_tex_all(&mut self, a: i32) {
        if !self.lock_texture {
            self.tex_index_box = [a; 6];
            self.tex_index = self.tex_index_box;
        }
    }

    pub fn set_tex_array(&mut self, a: [i32; 6]) {
        if !self.lock_texture {
            self.tex_index_box = a;
            self.tex_index = a;
        }
    }

    pub fn set_tex_num(&mut self, num: usize, tex: i32) {
        if num < 6 {
            self.tex_index[num] = tex;
            self.tex_index_box[num] = tex;
        }
    }

    pub fn set_tint(&mut self, r: f32, g: f32, b: f32) {
        self.tint_r = r;
        self.tint_g = g;
        self.tint_b = b;
    }

    pub fn set_tint_hex(&mut self, tc: i32) {
        self.tint_r = ((tc >> 16) & 0xFF) as f32 / 255.0;
        self.tint_g = ((tc >> 8) & 0xFF) as f32 / 255.0;
        self.tint_b = (tc & 0xFF) as f32 / 255.0;
    }

    pub fn set_alpha(&mut self, a: f32) {
        self.tint_a = a;
    }

    pub fn set_local_lights(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.light_local = [a, b, c, d, e, f];
    }

    pub fn set_local_lights_scalar(&mut self, a: f32) {
        self.light_local = [a; 6];
    }

    pub fn set_brightness(&mut self, a: i32) {
        self.bright_local = [a; 6];
    }

    pub fn setup_box(&mut self) {
        self.vertex_list_box[0] =
            Vector3::new(self.box_size2.x, self.box_size2.y, self.box_size1.z);
        self.vertex_list_box[1] =
            Vector3::new(self.box_size1.x, self.box_size2.y, self.box_size1.z);
        self.vertex_list_box[2] =
            Vector3::new(self.box_size1.x, self.box_size2.y, self.box_size2.z);
        self.vertex_list_box[3] =
            Vector3::new(self.box_size2.x, self.box_size2.y, self.box_size2.z);
        self.vertex_list_box[4] =
            Vector3::new(self.box_size2.x, self.box_size1.y, self.box_size1.z);
        self.vertex_list_box[5] =
            Vector3::new(self.box_size1.x, self.box_size1.y, self.box_size1.z);
        self.vertex_list_box[6] =
            Vector3::new(self.box_size1.x, self.box_size1.y, self.box_size2.z);
        self.vertex_list_box[7] =
            Vector3::new(self.box_size2.x, self.box_size1.y, self.box_size2.z);
        self.vertex_list = self.vertex_list_box.to_vec();
    }

    pub fn transform_rotate(&mut self) {
        for v in &mut self.vertex_list {
            v.add_coords(
                self.local_offset.x - 0.5,
                self.local_offset.y - 0.5,
                self.local_offset.z - 0.5,
            );
            self.basis.rotate(v);
            v.add_coords(
                self.global_origin.x + 0.5,
                self.global_origin.y + 0.5,
                self.global_origin.z + 0.5,
            );
        }
    }

    pub fn transform(&mut self) {
        for v in &mut self.vertex_list {
            v.add_vec(self.local_offset);
            v.add_vec(self.global_origin);
        }
    }

    pub fn calc_bounds(&mut self) {
        self.setup_box();
        self.transform_rotate();
    }

    pub fn calc_bounds_global(&mut self) {
        self.setup_box();
        self.transform();
    }

    pub fn set_side_uv(&mut self, s: usize, mut u1: f64, mut u2: f64, mut v1: f64, mut v2: f64) {
        if !self.exact_texture_coordinates {
            u1 += 0.001;
            v1 += 0.001;
            u2 -= 0.001;
            v2 -= 0.001;
        }

        u1 *= 0.0625;
        u2 *= 0.0625;
        v1 *= 0.0625;
        v2 *= 0.0625;

        let txl = self.tex_flags >> (s * 3);
        if (txl & 1) > 0 {
            u1 = 0.0625 - u1;
            u2 = 0.0625 - u2;
        }
        if (txl & 2) > 0 {
            v1 = 0.0625 - v1;
            v2 = 0.0625 - v2;
        }

        let ti = self.tex_index[s];
        let tx = (ti & 15) as f64 * 0.0625;
        let ty = (ti >> 4) as f64 * 0.0625;

        if (txl & 4) > 0 {
            u1 += ty;
            u2 += ty;
            v1 += tx;
            v2 += tx;
            self.corner_list_box[s][0].set_uv(v1, u1);
            self.corner_list_box[s][1].set_uv(v2, u1);
            self.corner_list_box[s][2].set_uv(v2, u2);
            self.corner_list_box[s][3].set_uv(v1, u2);
        } else {
            u1 += tx;
            u2 += tx;
            v1 += ty;
            v2 += ty;
            self.corner_list_box[s][0].set_uv(u1, v1);
            self.corner_list_box[s][1].set_uv(u1, v2);
            self.corner_list_box[s][2].set_uv(u2, v2);
            self.corner_list_box[s][3].set_uv(u2, v1);
        }
        self.corner_list = self.corner_list_box.to_vec();
    }

    pub fn do_mapping_box(&mut self, sides: i32) {
        if (sides & 3) > 0 {
            let v1 = 1.0 - self.box_size2.x;
            let v2 = 1.0 - self.box_size1.x;
            if (sides & 1) > 0 {
                let u1 = 1.0 - self.box_size2.z;
                let u2 = 1.0 - self.box_size1.z;
                self.set_side_uv(0, u1, u2, v1, v2);
            }
            if (sides & 2) > 0 {
                let u1 = self.box_size1.z;
                let u2 = self.box_size2.z;
                self.set_side_uv(1, u1, u2, v1, v2);
            }
        }

        if (sides & 60) != 0 {
            let v1 = 1.0 - self.box_size2.y;
            let v2 = 1.0 - self.box_size1.y;
            if (sides & 4) > 0 {
                let u1 = 1.0 - self.box_size2.x;
                let u2 = 1.0 - self.box_size1.x;
                self.set_side_uv(2, u1, u2, v1, v2);
            }
            if (sides & 8) > 0 {
                let u1 = self.box_size1.x;
                let u2 = self.box_size2.x;
                self.set_side_uv(3, u1, u2, v1, v2);
            }
            if (sides & 16) > 0 {
                let u1 = self.box_size1.z;
                let u2 = self.box_size2.z;
                self.set_side_uv(4, u1, u2, v1, v2);
            }
            if (sides & 32) > 0 {
                let u1 = 1.0 - self.box_size2.z;
                let u2 = 1.0 - self.box_size1.z;
                self.set_side_uv(5, u1, u2, v1, v2);
            }
        }
        self.corner_list = self.corner_list_box.to_vec();
    }

    fn swaptex(&mut self, a: usize, b: usize) {
        self.tex_index_box.swap(a, b);
        self.tex_index = self.tex_index_box;
    }

    pub fn orient_textures(&mut self, down: i32) {
        match down {
            0 => {}
            1 => {
                self.swaptex(0, 1);
                self.swaptex(4, 5);
                self.tex_flags = 112347;
            }
            2 => {
                self.swaptex(0, 2);
                self.swaptex(1, 3);
                self.swaptex(0, 4);
                self.swaptex(1, 5);
                self.tex_flags = 217134;
            }
            3 => {
                self.swaptex(0, 3);
                self.swaptex(1, 2);
                self.swaptex(0, 4);
                self.swaptex(1, 5);
                self.tex_flags = 188469;
            }
            4 => {
                self.swaptex(0, 4);
                self.swaptex(1, 5);
                self.swaptex(2, 3);
                self.tex_flags = 2944;
            }
            5 => {
                self.swaptex(0, 5);
                self.swaptex(1, 4);
                self.swaptex(0, 1);
                self.tex_flags = 3419;
            }
            _ => {}
        }
    }

    pub fn orient_texture_rot(&mut self, down: i32, rot: i32) {
        let mut r = if rot > 1 {
            if rot == 2 {
                3
            } else {
                6
            }
        } else if rot == 0 {
            0
        } else {
            5
        };
        r |= r << 3;

        match down {
            0 => self.tex_flags = r,
            1 => {
                self.swaptex(0, 1);
                self.swaptex(4, 5);
                self.tex_flags = 112347 ^ r;
            }
            2 => {
                self.swaptex(0, 2);
                self.swaptex(1, 3);
                self.swaptex(0, 4);
                self.swaptex(1, 5);
                self.tex_flags = 217134 ^ (r << 6);
            }
            3 => {
                self.swaptex(0, 3);
                self.swaptex(1, 2);
                self.swaptex(0, 4);
                self.swaptex(1, 5);
                self.tex_flags = 188469 ^ (r << 6);
            }
            4 => {
                self.swaptex(0, 4);
                self.swaptex(1, 5);
                self.swaptex(2, 3);
                self.tex_flags = 2944 ^ (r << 12);
            }
            5 => {
                self.swaptex(0, 5);
                self.swaptex(1, 4);
                self.swaptex(0, 1);
                self.tex_flags = 3419 ^ (r << 12);
            }
            _ => {}
        }
    }

    fn swaptexfl(&mut self, a: usize, b: usize) {
        self.tex_index_box.swap(a, b);
        self.tex_index = self.tex_index_box;
        let ash = a * 3;
        let bsh = b * 3;
        let f1 = (self.tex_flags >> ash) & 7;
        let f2 = (self.tex_flags >> bsh) & 7;
        self.tex_flags &= !(7 << ash | 7 << bsh);
        self.tex_flags |= (f1 << bsh) | (f2 << ash);
    }

    pub fn rotate_textures(&mut self, rot: i32) {
        let mut r = if rot > 1 {
            if rot == 2 {
                3
            } else {
                6
            }
        } else if rot == 0 {
            0
        } else {
            5
        };
        r |= r << 3;
        self.tex_flags ^= r;
        match rot {
            1 => {
                self.swaptexfl(2, 4);
                self.swaptexfl(3, 4);
                self.swaptexfl(3, 5);
            }
            2 => {
                self.swaptexfl(2, 3);
                self.swaptexfl(4, 5);
            }
            3 => {
                self.swaptexfl(2, 5);
                self.swaptexfl(3, 5);
                self.swaptexfl(3, 4);
            }
            _ => {}
        }
    }

    pub fn orient_texture_fl(&mut self, down: i32) {
        match down {
            0 => {}
            1 => {
                self.swaptexfl(0, 1);
                self.swaptexfl(4, 5);
                self.tex_flags ^= 112347;
            }
            2 => {
                self.swaptexfl(0, 2);
                self.swaptexfl(1, 3);
                self.swaptexfl(0, 4);
                self.swaptexfl(1, 5);
                self.tex_flags ^= 217134;
            }
            3 => {
                self.swaptexfl(0, 3);
                self.swaptexfl(1, 2);
                self.swaptexfl(0, 4);
                self.swaptexfl(1, 5);
                self.tex_flags ^= 188469;
            }
            4 => {
                self.swaptexfl(0, 4);
                self.swaptexfl(1, 5);
                self.swaptexfl(2, 3);
                self.tex_flags ^= 2944;
            }
            5 => {
                self.swaptexfl(0, 5);
                self.swaptexfl(1, 4);
                self.swaptexfl(0, 1);
                self.tex_flags ^= 3419;
            }
            _ => {}
        }
    }

    pub fn orient_texture_new(&mut self, rv: usize) {
        if rv >= TEX_ROT_TABLE.len() {
            return;
        }
        let tex_src = self.tex_index_box;
        let rot = TEX_ROT_TABLE[rv];
        let mut tfo = 0;

        for (i, &rot_val) in rot.iter().take(6).enumerate() {
            let r_idx = rot_val as usize;
            self.tex_index_box[i] = tex_src[r_idx];
            tfo |= ((self.tex_flags >> (r_idx * 3)) & 7) << (i * 3);
        }
        self.tex_index = self.tex_index_box;

        let t2 = ((tfo & 37449) << 1) | ((tfo & 74898) >> 1);
        self.tex_flags = rot[6] ^ (tfo & rot[7]) ^ (t2 & rot[8]);
    }

    pub fn flip_textures(&mut self) {
        self.swaptex(0, 1);
        self.swaptex(2, 3);
        self.swaptex(4, 5);
    }

    pub fn do_light_local(&mut self, sides: i32) {
        for i in 0..self.corner_list.len() {
            if (sides & (1 << i)) != 0 {
                let c = &mut self.corner_list[i][0];
                c.r = self.light_local[i] * self.tint_r;
                c.g = self.light_local[i] * self.tint_g;
                c.b = self.light_local[i] * self.tint_b;
                c.brtex = self.bright_local[i];
            }
        }
    }

    pub fn light_smooth_face(&mut self, fn_idx: usize) {
        let mut ff = 0;
        if self.box_size1.y > 0.0 {
            ff |= 1;
        }
        if self.box_size2.y < 1.0 {
            ff |= 2;
        }
        if self.box_size1.z > 0.0 {
            ff |= 4;
        }
        if self.box_size2.z < 1.0 {
            ff |= 8;
        }
        if self.box_size1.x > 0.0 {
            ff |= 16;
        }
        if self.box_size2.x < 1.0 {
            ff |= 32;
        }

        let mut gf1 = self.ao_global[1][1][1];
        let mut gf2 = gf1;
        let mut gf3 = gf1;
        let mut gf4 = gf1;

        let mut gl1 = self.light_global[1][1][1];
        let mut gl2 = gl1;
        let mut gl3 = gl1;
        let mut gl4 = gl1;

        match fn_idx {
            0 => {
                if (ff & 61) <= 0 {
                    let mut ao1 = self.ao_global[0][0][1];
                    let mut ao2 = ao1;
                    let mut ao3 = self.ao_global[2][0][1];
                    let mut ao4 = ao3;
                    let mut lv1 = self.light_global[0][0][1];
                    let mut lv2 = lv1;
                    let mut lv3 = self.light_global[2][0][1];
                    let mut lv4 = lv3;

                    if (self.glob_trans & 5) > 0 {
                        ao1 = self.ao_global[0][0][0];
                        lv1 = self.light_global[0][0][0];
                    }
                    if (self.glob_trans & 6) > 0 {
                        ao2 = self.ao_global[0][0][2];
                        lv2 = self.light_global[0][0][2];
                    }
                    if (self.glob_trans & 9) > 0 {
                        ao3 = self.ao_global[2][0][0];
                        lv3 = self.light_global[2][0][0];
                    }
                    if (self.glob_trans & 10) > 0 {
                        ao4 = self.ao_global[2][0][2];
                        lv4 = self.light_global[2][0][2];
                    }

                    gf3 = 0.25
                        * (self.ao_global[1][0][1]
                            + self.ao_global[1][0][0]
                            + self.ao_global[0][0][1]
                            + ao1);
                    gf4 = 0.25
                        * (self.ao_global[1][0][1]
                            + self.ao_global[1][0][0]
                            + self.ao_global[2][0][1]
                            + ao3);
                    gf2 = 0.25
                        * (self.ao_global[1][0][1]
                            + self.ao_global[1][0][2]
                            + self.ao_global[0][0][1]
                            + ao2);
                    gf1 = 0.25
                        * (self.ao_global[1][0][1]
                            + self.ao_global[1][0][2]
                            + self.ao_global[2][0][1]
                            + ao4);

                    gl3 = blend_light(
                        self.light_global[1][0][1],
                        self.light_global[1][0][0],
                        self.light_global[0][0][1],
                        lv1,
                    );
                    gl4 = blend_light(
                        self.light_global[1][0][1],
                        self.light_global[1][0][0],
                        self.light_global[2][0][1],
                        lv3,
                    );
                    gl2 = blend_light(
                        self.light_global[1][0][1],
                        self.light_global[1][0][2],
                        self.light_global[0][0][1],
                        lv2,
                    );
                    gl1 = blend_light(
                        self.light_global[1][0][1],
                        self.light_global[1][0][2],
                        self.light_global[2][0][1],
                        lv4,
                    );
                }
            }
            1 => {
                if (ff & 62) <= 0 {
                    let mut var24 = self.ao_global[0][2][1];
                    let mut var29 = var24;
                    let mut var34 = self.ao_global[2][2][1];
                    let mut var39 = var34;
                    let mut var44 = self.light_global[0][2][1];
                    let mut var49 = var44;
                    let mut var54 = self.light_global[2][2][1];
                    let mut var59 = var54;

                    if (self.glob_trans & 1280) > 0 {
                        var24 = self.ao_global[0][2][0];
                        var44 = self.light_global[0][2][0];
                    }
                    if (self.glob_trans & 1536) > 0 {
                        var29 = self.ao_global[0][2][2];
                        var49 = self.light_global[0][2][2];
                    }
                    if (self.glob_trans & 2304) > 0 {
                        var34 = self.ao_global[2][2][0];
                        var54 = self.light_global[2][2][0];
                    }
                    if (self.glob_trans & 2560) > 0 {
                        var39 = self.ao_global[2][2][2];
                        var59 = self.light_global[2][2][2];
                    }

                    gf2 = 0.25
                        * (self.ao_global[1][2][1]
                            + self.ao_global[1][2][0]
                            + self.ao_global[0][2][1]
                            + var24);
                    gf1 = 0.25
                        * (self.ao_global[1][2][1]
                            + self.ao_global[1][2][0]
                            + self.ao_global[2][2][1]
                            + var34);
                    gf3 = 0.25
                        * (self.ao_global[1][2][1]
                            + self.ao_global[1][2][2]
                            + self.ao_global[0][2][1]
                            + var29);
                    gf4 = 0.25
                        * (self.ao_global[1][2][1]
                            + self.ao_global[1][2][2]
                            + self.ao_global[2][2][1]
                            + var39);

                    gl2 = blend_light(
                        self.light_global[1][2][1],
                        self.light_global[1][2][0],
                        self.light_global[0][2][1],
                        var44,
                    );
                    gl1 = blend_light(
                        self.light_global[1][2][1],
                        self.light_global[1][2][0],
                        self.light_global[2][2][1],
                        var54,
                    );
                    gl3 = blend_light(
                        self.light_global[1][2][1],
                        self.light_global[1][2][2],
                        self.light_global[0][2][1],
                        var49,
                    );
                    gl4 = blend_light(
                        self.light_global[1][2][1],
                        self.light_global[1][2][2],
                        self.light_global[2][2][1],
                        var59,
                    );
                }
            }
            2 => {
                if (ff & 55) <= 0 {
                    let mut var23 = self.ao_global[0][1][0];
                    let mut var28 = var23;
                    let mut var33 = self.ao_global[2][1][0];
                    let mut var38 = var33;
                    let mut var43 = self.light_global[0][1][0];
                    let mut var48 = var43;
                    let mut var53 = self.light_global[2][1][0];
                    let mut var58 = var53;

                    if (self.glob_trans & 17) > 0 {
                        var23 = self.ao_global[0][0][0];
                        var43 = self.light_global[0][0][0];
                    }
                    if (self.glob_trans & 272) > 0 {
                        var28 = self.ao_global[0][2][0];
                        var48 = self.light_global[0][2][0];
                    }
                    if (self.glob_trans & 65) > 0 {
                        var33 = self.ao_global[2][0][0];
                        var53 = self.light_global[2][0][0];
                    }
                    if (self.glob_trans & 320) > 0 {
                        var38 = self.ao_global[2][2][0];
                        var58 = self.light_global[2][2][0];
                    }

                    gf3 = 0.25
                        * (self.ao_global[1][1][0]
                            + self.ao_global[1][0][0]
                            + self.ao_global[0][1][0]
                            + var23);
                    gf4 = 0.25
                        * (self.ao_global[1][1][0]
                            + self.ao_global[1][2][0]
                            + self.ao_global[0][1][0]
                            + var28);
                    gf2 = 0.25
                        * (self.ao_global[1][1][0]
                            + self.ao_global[1][0][0]
                            + self.ao_global[2][1][0]
                            + var33);
                    gf1 = 0.25
                        * (self.ao_global[1][1][0]
                            + self.ao_global[1][2][0]
                            + self.ao_global[2][1][0]
                            + var38);

                    gl3 = blend_light(
                        self.light_global[1][1][0],
                        self.light_global[1][0][0],
                        self.light_global[0][1][0],
                        var43,
                    );
                    gl4 = blend_light(
                        self.light_global[1][1][0],
                        self.light_global[1][2][0],
                        self.light_global[0][1][0],
                        var48,
                    );
                    gl2 = blend_light(
                        self.light_global[1][1][0],
                        self.light_global[1][0][0],
                        self.light_global[2][1][0],
                        var53,
                    );
                    gl1 = blend_light(
                        self.light_global[1][1][0],
                        self.light_global[1][2][0],
                        self.light_global[2][1][0],
                        var58,
                    );
                }
            }
            3 => {
                if (ff & 59) <= 0 {
                    let mut var22 = self.ao_global[0][1][2];
                    let mut var27 = var22;
                    let mut var32 = self.ao_global[2][1][2];
                    let mut var37 = var32;
                    let mut var42 = self.light_global[0][1][2];
                    let mut var47 = var42;
                    let mut var52 = self.light_global[2][1][2];
                    let mut var57 = var52;

                    if (self.glob_trans & 34) > 0 {
                        var22 = self.ao_global[0][0][2];
                        var42 = self.light_global[0][0][2];
                    }
                    if (self.glob_trans & 544) > 0 {
                        var27 = self.ao_global[0][2][2];
                        var47 = self.light_global[0][2][2];
                    }
                    if (self.glob_trans & 130) > 0 {
                        var32 = self.ao_global[2][0][2];
                        var52 = self.light_global[2][0][2];
                    }
                    if (self.glob_trans & 640) > 0 {
                        var37 = self.ao_global[2][2][2];
                        var57 = self.light_global[2][2][2];
                    }

                    gf2 = 0.25
                        * (self.ao_global[1][1][2]
                            + self.ao_global[1][0][2]
                            + self.ao_global[0][1][2]
                            + var22);
                    gf1 = 0.25
                        * (self.ao_global[1][1][2]
                            + self.ao_global[1][2][2]
                            + self.ao_global[0][1][2]
                            + var32);
                    gf3 = 0.25
                        * (self.ao_global[1][1][2]
                            + self.ao_global[1][0][2]
                            + self.ao_global[2][1][2]
                            + var27);
                    gf4 = 0.25
                        * (self.ao_global[1][1][2]
                            + self.ao_global[1][2][2]
                            + self.ao_global[2][1][2]
                            + var37);

                    gl2 = blend_light(
                        self.light_global[1][1][2],
                        self.light_global[1][0][2],
                        self.light_global[0][1][2],
                        var42,
                    );
                    gl1 = blend_light(
                        self.light_global[1][1][2],
                        self.light_global[1][2][2],
                        self.light_global[0][1][2],
                        var47,
                    );
                    gl3 = blend_light(
                        self.light_global[1][1][2],
                        self.light_global[1][0][2],
                        self.light_global[2][1][2],
                        var52,
                    );
                    gl4 = blend_light(
                        self.light_global[1][1][2],
                        self.light_global[1][2][2],
                        self.light_global[2][1][2],
                        var57,
                    );
                }
            }
            4 => {
                if (ff & 31) <= 0 {
                    let mut var21 = self.ao_global[0][1][0];
                    let mut var26 = var21;
                    let mut var31 = self.ao_global[0][1][2];
                    let mut var36 = var31;
                    let mut var41 = self.light_global[0][1][0];
                    let mut var46 = var41;
                    let mut var51 = self.light_global[0][1][2];
                    let mut var56 = var51;

                    if (self.glob_trans & 20) > 0 {
                        var21 = self.ao_global[0][0][0];
                        var41 = self.light_global[0][0][0];
                    }
                    if (self.glob_trans & 1040) > 0 {
                        var26 = self.ao_global[0][2][0];
                        var46 = self.light_global[0][2][0];
                    }
                    if (self.glob_trans & 36) > 0 {
                        var31 = self.ao_global[0][0][2];
                        var51 = self.light_global[0][0][2];
                    }
                    if (self.glob_trans & 1056) > 0 {
                        var36 = self.ao_global[0][2][2];
                        var56 = self.light_global[0][2][2];
                    }

                    gf2 = 0.25
                        * (self.ao_global[0][1][1]
                            + self.ao_global[0][0][1]
                            + self.ao_global[0][1][0]
                            + var21);
                    gf1 = 0.25
                        * (self.ao_global[0][1][1]
                            + self.ao_global[0][2][1]
                            + self.ao_global[0][1][0]
                            + var26);
                    gf3 = 0.25
                        * (self.ao_global[0][1][1]
                            + self.ao_global[0][0][1]
                            + self.ao_global[0][1][2]
                            + var31);
                    gf4 = 0.25
                        * (self.ao_global[0][1][1]
                            + self.ao_global[0][2][1]
                            + self.ao_global[0][1][2]
                            + var36);

                    gl2 = blend_light(
                        self.light_global[0][1][1],
                        self.light_global[0][0][1],
                        self.light_global[0][1][0],
                        var41,
                    );
                    gl1 = blend_light(
                        self.light_global[0][1][1],
                        self.light_global[0][2][1],
                        self.light_global[0][1][0],
                        var46,
                    );
                    gl3 = blend_light(
                        self.light_global[0][1][1],
                        self.light_global[0][0][1],
                        self.light_global[0][1][2],
                        var51,
                    );
                    gl4 = blend_light(
                        self.light_global[0][1][1],
                        self.light_global[0][2][1],
                        self.light_global[0][1][2],
                        var56,
                    );
                }
            }
            _ => {
                if (ff & 47) <= 0 {
                    let mut ao1 = self.ao_global[2][1][0];
                    let mut ao2 = ao1;
                    let mut ao3 = self.ao_global[2][1][2];
                    let mut ao4 = ao3;
                    let mut lv1 = self.light_global[2][1][0];
                    let mut lv2 = lv1;
                    let mut lv3 = self.light_global[2][1][2];
                    let mut lv4 = lv3;

                    if (self.glob_trans & 72) > 0 {
                        ao1 = self.ao_global[2][0][0];
                        lv1 = self.light_global[2][0][0];
                    }
                    if (self.glob_trans & 2112) > 0 {
                        ao2 = self.ao_global[2][2][0];
                        lv2 = self.light_global[2][2][0];
                    }
                    if (self.glob_trans & 136) > 0 {
                        ao3 = self.ao_global[2][0][2];
                        lv3 = self.light_global[2][0][2];
                    }
                    if (self.glob_trans & 2176) > 0 {
                        ao4 = self.ao_global[2][2][2];
                        lv4 = self.light_global[2][2][2];
                    }

                    gf3 = 0.25
                        * (self.ao_global[2][1][1]
                            + self.ao_global[2][0][1]
                            + self.ao_global[2][1][0]
                            + ao1);
                    gf4 = 0.25
                        * (self.ao_global[2][1][1]
                            + self.ao_global[2][2][1]
                            + self.ao_global[2][1][0]
                            + ao2);
                    gf2 = 0.25
                        * (self.ao_global[2][1][1]
                            + self.ao_global[2][0][1]
                            + self.ao_global[2][1][2]
                            + ao3);
                    gf1 = 0.25
                        * (self.ao_global[2][1][1]
                            + self.ao_global[2][2][1]
                            + self.ao_global[2][1][2]
                            + ao4);

                    gl3 = blend_light(
                        self.light_global[2][1][1],
                        self.light_global[2][0][1],
                        self.light_global[2][1][0],
                        lv1,
                    );
                    gl4 = blend_light(
                        self.light_global[2][1][1],
                        self.light_global[2][2][1],
                        self.light_global[2][1][0],
                        lv2,
                    );
                    gl2 = blend_light(
                        self.light_global[2][1][1],
                        self.light_global[2][0][1],
                        self.light_global[2][1][2],
                        lv3,
                    );
                    gl1 = blend_light(
                        self.light_global[2][1][1],
                        self.light_global[2][2][1],
                        self.light_global[2][1][2],
                        lv4,
                    );
                }
            }
        }

        let gfs = [gf1, gf2, gf3, gf4];
        let gls = [gl1, gl2, gl3, gl4];
        for j in 0..4 {
            let c = &mut self.corner_list[fn_idx][j];
            let fc = self.light_local[fn_idx] * gfs[j];
            c.r = fc * self.tint_r;
            c.g = fc * self.tint_g;
            c.b = fc * self.tint_b;
            c.brtex = gls[j];
        }
    }

    pub fn do_light_smooth(&mut self, sides: i32) {
        for i in 0..6 {
            if (sides & (1 << i)) != 0 {
                self.light_smooth_face(i);
            }
        }
    }

    pub fn do_light_flat(&mut self, sides: i32) {
        for i in 0..self.corner_list.len() {
            if (sides & (1 << i)) != 0 {
                let c = &mut self.corner_list[i][0];
                c.r = self.light_flat[i] * self.light_local[i] * self.tint_r;
                c.g = self.light_flat[i] * self.light_local[i] * self.tint_g;
                c.b = self.light_flat[i] * self.light_local[i] * self.tint_b;
                c.brtex = self.bright_local[i];
            }
        }
    }

    /// Calculate normal vector for face `i` matching Java `useNormal` logic:
    /// `(v1 - v0) x (v2 - v0)` normalized.
    pub fn calculate_face_normal(&self, face_idx: usize) -> Option<Vector3> {
        let quad = self.corner_list.get(face_idx)?;
        let v0 = self.vertex_list.get(quad[0].vtx as usize)?;
        let v1 = self.vertex_list.get(quad[1].vtx as usize)?;
        let v2 = self.vertex_list.get(quad[2].vtx as usize)?;

        let d1 = *v1 - *v0;
        let d2 = *v2 - *v0;
        let mut normal = d1.cross(&d2);
        normal.normalize();
        Some(normal)
    }

    /// Produce rendered vertices for all active faces.
    pub fn build_face_vertices(&self, sides: i32) -> Vec<RenderVertex> {
        let mut vertices = Vec::new();
        for i in 0..self.corner_list.len() {
            if (sides & (1 << i)) != 0 {
                let normal = if self.use_normal {
                    self.calculate_face_normal(i).unwrap_or(Vector3::UNIT_Y)
                } else {
                    Vector3::ZERO
                };
                let _ = normal;

                for j in 0..4 {
                    let c = &self.corner_list[i][j];
                    if let Some(&v) = self.vertex_list.get(c.vtx as usize) {
                        let mut rv = RenderVertex::new(v, c.u, c.v);
                        rv.r = c.r * self.tint_r;
                        rv.g = c.g * self.tint_g;
                        rv.b = c.b * self.tint_b;
                        rv.a = self.tint_a;
                        rv.brightness = c.brtex;
                        vertices.push(rv);
                    }
                }
            }
        }
        vertices
    }

    pub fn bind_model(&mut self, model: &RenderModel) {
        let mut vertices = Vec::with_capacity(model.vertex_list.len());
        for v in &model.vertex_list {
            let mut v_rot = *v;
            self.basis.rotate(&mut v_rot);
            v_rot.add_vec(self.global_origin);
            vertices.push(v_rot);
        }
        self.vertex_list = vertices;
        self.corner_list = model.tex_list.clone();
        self.bound_model = Some(model.clone());
    }

    pub fn bind_model_offset(&mut self, model: &RenderModel, ofx: f64, ofy: f64, ofz: f64) {
        let mut vertices = Vec::with_capacity(model.vertex_list.len());
        for v in &model.vertex_list {
            let mut v_rot = *v;
            v_rot.add_coords(
                self.local_offset.x - ofx,
                self.local_offset.y - ofy,
                self.local_offset.z - ofz,
            );
            self.basis.rotate(&mut v_rot);
            v_rot.add_coords(ofx, ofy, ofz);
            v_rot.add_vec(self.global_origin);
            vertices.push(v_rot);
        }
        self.vertex_list = vertices;
        self.corner_list = model.tex_list.clone();
        self.bound_model = Some(model.clone());
    }
}

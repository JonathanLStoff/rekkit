//! Procedural block and dynamic model rendering helpers, custom block renderer registry,
//! and texture animation frame conversions.
//!
//! Direct port of:
//! - `com.eloraam.redpower.core.RenderLib`
//! - `com.eloraam.redpower.core.TextureAnimFX`
//! - `com.eloraam.redpower.core.RenderCustomBlock`

use crate::core::math::Quat;
use crate::core::spatial::Vector3;
use std::sync::RwLock;

/// Procedural vertex buffer entry emitted during tessellation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderVertex {
    pub pos: Vector3,
    pub u: f64,
    pub v: f64,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub brightness: i32,
}

impl RenderVertex {
    pub const fn new(pos: Vector3, u: f64, v: f64) -> Self {
        Self {
            pos,
            u,
            v,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
            brightness: 0,
        }
    }
}

/// Abstract custom block renderer contract matching `com.eloraam.redpower.core.RenderCustomBlock`.
pub trait RenderCustomBlock: Send + Sync {
    fn render_world_block(&self, x: i32, y: i32, z: i32, metadata: i32) -> bool {
        let _ = (x, y, z, metadata);
        false
    }

    fn render_inv_block(&self, metadata: i32) {
        let _ = metadata;
    }
}

/// Entry in the custom block renderer registry.
#[derive(Default)]
pub struct RenderListEntry {
    pub meta_renders: [Option<std::sync::Arc<dyn RenderCustomBlock>>; 16],
    pub default_render: Option<std::sync::Arc<dyn RenderCustomBlock>>,
    pub shift: u32,
}

impl RenderListEntry {
    pub fn new(shift: u32) -> Self {
        Self {
            meta_renders: Default::default(),
            default_render: None,
            shift,
        }
    }

    pub fn map_damage_value(&self, dmg: i32) -> usize {
        (dmg >> self.shift) as usize
    }
}

/// Custom block renderer registry (matches `RenderLib.renderers[4096]`).
pub struct RenderRegistry {
    entries: Vec<Option<RenderListEntry>>,
}

impl Default for RenderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderRegistry {
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(4096);
        for _ in 0..4096 {
            entries.push(None);
        }
        Self { entries }
    }

    pub fn set_renderer(
        &mut self,
        block_id: usize,
        renderer: std::sync::Arc<dyn RenderCustomBlock>,
    ) {
        if block_id >= self.entries.len() {
            return;
        }
        let entry = self.entries[block_id].get_or_insert_with(|| RenderListEntry::new(0));
        for r in &mut entry.meta_renders {
            *r = Some(std::sync::Arc::clone(&renderer));
        }
    }

    pub fn set_meta_renderer(
        &mut self,
        block_id: usize,
        md: usize,
        renderer: std::sync::Arc<dyn RenderCustomBlock>,
    ) {
        if block_id >= self.entries.len() || md >= 16 {
            return;
        }
        let entry = self.entries[block_id].get_or_insert_with(|| RenderListEntry::new(0));
        entry.meta_renders[md] = Some(renderer);
    }

    pub fn set_shifted_renderer(
        &mut self,
        block_id: usize,
        md: usize,
        shift: u32,
        renderer: std::sync::Arc<dyn RenderCustomBlock>,
    ) {
        if block_id >= self.entries.len() || md >= 16 {
            return;
        }
        let entry = self.entries[block_id].get_or_insert_with(|| RenderListEntry::new(shift));
        entry.shift = shift;
        entry.meta_renders[md] = Some(renderer);
    }

    pub fn set_default_renderer(
        &mut self,
        block_id: usize,
        shift: u32,
        renderer: std::sync::Arc<dyn RenderCustomBlock>,
    ) {
        if block_id >= self.entries.len() {
            return;
        }
        let entry = self.entries[block_id].get_or_insert_with(|| RenderListEntry::new(shift));
        entry.shift = shift;
        for slot in &mut entry.meta_renders {
            if slot.is_none() {
                *slot = Some(std::sync::Arc::clone(&renderer));
            }
        }
        entry.default_render = Some(renderer);
    }

    pub fn get_renderer(
        &self,
        block_id: usize,
        md: usize,
    ) -> Option<std::sync::Arc<dyn RenderCustomBlock>> {
        let entry = self.entries.get(block_id)?.as_ref()?;
        entry.meta_renders.get(md).and_then(Option::clone)
    }

    pub fn get_inv_renderer(
        &self,
        block_id: usize,
        md: i32,
    ) -> Option<std::sync::Arc<dyn RenderCustomBlock>> {
        let entry = self.entries.get(block_id)?.as_ref()?;
        let mdv = entry.map_damage_value(md);
        if mdv > 15 {
            entry.default_render.clone()
        } else {
            entry.meta_renders.get(mdv).and_then(Option::clone)
        }
    }
}

static GLOBAL_RENDER_REGISTRY: RwLock<Option<RenderRegistry>> = RwLock::new(None);

pub fn registry() -> std::sync::RwLockReadGuard<'static, Option<RenderRegistry>> {
    let read = GLOBAL_RENDER_REGISTRY.read().unwrap();
    if read.is_none() {
        drop(read);
        let mut write = GLOBAL_RENDER_REGISTRY.write().unwrap();
        if write.is_none() {
            *write = Some(RenderRegistry::new());
        }
        drop(write);
        GLOBAL_RENDER_REGISTRY.read().unwrap()
    } else {
        read
    }
}

pub fn registry_mut() -> std::sync::RwLockWriteGuard<'static, Option<RenderRegistry>> {
    let mut write = GLOBAL_RENDER_REGISTRY.write().unwrap();
    if write.is_none() {
        *write = Some(RenderRegistry::new());
    }
    write
}

/// Dynamic lever geometry calculation matching `RenderLib.renderSpecialLever`.
///
/// Returns the 24 vertices (6 quads) of the lever in world space with UV mappings.
pub fn generate_special_lever(pos: Vector3, rot: Quat, tex: i32) -> Vec<RenderVertex> {
    let k1 = (tex & 15) << 4;
    let l1 = tex & 240;
    let f8 = 0.0625f64;
    let f9 = 0.0625f64;
    let f10 = 0.375f64;

    let mut pl = [
        Vector3::new(-f8, 0.0, -f9),
        Vector3::new(f8, 0.0, -f9),
        Vector3::new(f8, 0.0, f9),
        Vector3::new(-f8, 0.0, f9),
        Vector3::new(-f8, f10, -f9),
        Vector3::new(f8, f10, -f9),
        Vector3::new(f8, f10, f9),
        Vector3::new(-f8, f10, f9),
    ];

    for p in &mut pl {
        rot.rotate(p);
        p.add_coords(pos.x + 0.5, pos.y + 0.5, pos.z + 0.5);
    }

    let mut vertices = Vec::with_capacity(24);

    let u1_base = (k1 + 7) as f64 / 256.0;
    let u2_base = (k1 as f64 + 9.0 - 0.01) / 256.0;
    let v1_base = (l1 + 6) as f64 / 256.0;
    let v2_base = (l1 as f64 + 8.0 - 0.01) / 256.0;

    // Quad 1: bottom/top faces
    vertices.push(RenderVertex::new(pl[0], u1_base, v2_base));
    vertices.push(RenderVertex::new(pl[1], u2_base, v2_base));
    vertices.push(RenderVertex::new(pl[2], u2_base, v1_base));
    vertices.push(RenderVertex::new(pl[3], u1_base, v1_base));

    vertices.push(RenderVertex::new(pl[7], u1_base, v2_base));
    vertices.push(RenderVertex::new(pl[6], u2_base, v2_base));
    vertices.push(RenderVertex::new(pl[5], u2_base, v1_base));
    vertices.push(RenderVertex::new(pl[4], u1_base, v1_base));

    let u1_side = (k1 + 7) as f64 / 256.0;
    let u2_side = (k1 as f64 + 9.0 - 0.01) / 256.0;
    let v1_side = (l1 + 6) as f64 / 256.0;
    let v2_side = (l1 as f64 + 12.0 - 0.01) / 256.0;

    // Quad 2-5: side faces
    vertices.push(RenderVertex::new(pl[1], u1_side, v2_side));
    vertices.push(RenderVertex::new(pl[0], u2_side, v2_side));
    vertices.push(RenderVertex::new(pl[4], u2_side, v1_side));
    vertices.push(RenderVertex::new(pl[5], u1_side, v1_side));

    vertices.push(RenderVertex::new(pl[2], u1_side, v2_side));
    vertices.push(RenderVertex::new(pl[1], u2_side, v2_side));
    vertices.push(RenderVertex::new(pl[5], u2_side, v1_side));
    vertices.push(RenderVertex::new(pl[6], u1_side, v1_side));

    vertices.push(RenderVertex::new(pl[3], u1_side, v2_side));
    vertices.push(RenderVertex::new(pl[2], u2_side, v2_side));
    vertices.push(RenderVertex::new(pl[6], u2_side, v1_side));
    vertices.push(RenderVertex::new(pl[7], u1_side, v1_side));

    vertices.push(RenderVertex::new(pl[0], u1_side, v2_side));
    vertices.push(RenderVertex::new(pl[3], u2_side, v2_side));
    vertices.push(RenderVertex::new(pl[7], u2_side, v1_side));
    vertices.push(RenderVertex::new(pl[4], u1_side, v1_side));

    vertices
}

/// Dynamic timer / compass pointer geometry matching `RenderLib.renderPointer`.
///
/// Returns the vertices and shading attributes for the indicator needle.
pub fn generate_pointer(pos: Vector3, rot: Quat) -> Vec<RenderVertex> {
    let u = 0.390625f64;
    let v = 0.015625f64;
    let uvd = 0.0312109375f64;
    let uvd2 = 0.0077734375f64;

    let mut pl = [
        Vector3::new(0.4, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.2),
        Vector3::new(-0.2, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -0.2),
        Vector3::new(0.4, 0.1, 0.0),
        Vector3::new(0.0, 0.1, 0.2),
        Vector3::new(-0.2, 0.1, 0.0),
        Vector3::new(0.0, 0.1, -0.2),
    ];

    for p in &mut pl {
        rot.rotate(p);
        p.add_vec(pos);
    }

    let mut vertices = Vec::with_capacity(24);

    let mut add_v = |vtx: Vector3, uv_u: f64, uv_v: f64, tint: f32| {
        let mut vert = RenderVertex::new(vtx, uv_u, uv_v);
        vert.r = tint;
        vert.g = tint;
        vert.b = tint;
        vertices.push(vert);
    };

    // Shaded at 0.9F
    add_v(pl[0], u, v, 0.9);
    add_v(pl[1], u + uvd, v, 0.9);
    add_v(pl[2], u + uvd, v + uvd, 0.9);
    add_v(pl[3], u, v + uvd, 0.9);

    add_v(pl[4], u, v, 0.9);
    add_v(pl[7], u, v + uvd, 0.9);
    add_v(pl[6], u + uvd, v + uvd, 0.9);
    add_v(pl[5], u + uvd, v, 0.9);

    // Shaded at 0.6F
    add_v(pl[0], u + uvd2, v, 0.6);
    add_v(pl[4], u, v, 0.6);
    add_v(pl[5], u, v + uvd, 0.6);
    add_v(pl[1], u + uvd2, v + uvd, 0.6);

    add_v(pl[0], u, v + uvd2, 0.6);
    add_v(pl[3], u + uvd, v + uvd2, 0.6);
    add_v(pl[7], u + uvd, v, 0.6);
    add_v(pl[4], u, v, 0.6);

    add_v(pl[2], u + uvd, v + uvd - uvd2, 0.6);
    add_v(pl[6], u + uvd, v + uvd, 0.6);
    add_v(pl[7], u, v + uvd, 0.6);
    add_v(pl[3], u, v + uvd - uvd2, 0.6);

    add_v(pl[2], u + uvd, v + uvd - uvd2, 0.6);
    add_v(pl[1], u, v + uvd - uvd2, 0.6);
    add_v(pl[5], u, v + uvd, 0.6);
    add_v(pl[6], u + uvd, v + uvd, 0.6);

    vertices
}

/// Frame animation helper converting ARGB/RGB int arrays to raw byte channels.
///
/// Direct port of `com.eloraam.redpower.core.TextureAnimFX`.
pub struct TextureAnimFX {
    pub tick_rate: u32,
    pub tick: u32,
    pub index: usize,
    pub texture_frames: Vec<Vec<u8>>,
}

impl TextureAnimFX {
    pub fn new(tick_rate: u32, frames: Vec<Vec<u32>>) -> Self {
        let texture_frames = frames
            .into_iter()
            .map(|frame| Self::convert_rgb(&frame))
            .collect();
        Self {
            tick_rate,
            tick: 0,
            index: 0,
            texture_frames,
        }
    }

    /// Convert packed ARGB32 integer pixels into 4-byte RGBA/raw bytes matching Java `convertRGB`.
    ///
    /// Java code:
    /// ```java
    /// tr[i * 4] = (byte)(rgb[i] >> 16);
    /// tr[i * 4 + 1] = (byte)(rgb[i] >> 8);
    /// tr[i * 4 + 2] = (byte)rgb[i];
    /// tr[i * 4 + 3] = (byte)(rgb[i] >> 24);
    /// ```
    pub fn convert_rgb(rgb: &[u32]) -> Vec<u8> {
        let mut tr = vec![0u8; rgb.len() * 4];
        for (i, &val) in rgb.iter().enumerate() {
            tr[i * 4] = (val >> 16) as u8;
            tr[i * 4 + 1] = (val >> 8) as u8;
            tr[i * 4 + 2] = val as u8;
            tr[i * 4 + 3] = (val >> 24) as u8;
        }
        tr
    }

    /// Advance one tick; returns current frame byte slice.
    pub fn tick(&mut self) -> Option<&[u8]> {
        if self.texture_frames.is_empty() {
            return None;
        }
        self.tick += 1;
        if self.tick >= self.tick_rate {
            self.tick = 0;
            self.index = (self.index + 1) % self.texture_frames.len();
        }
        Some(&self.texture_frames[self.index])
    }

    pub fn current_frame(&self) -> Option<&[u8]> {
        self.texture_frames.get(self.index).map(|f| f.as_slice())
    }
}

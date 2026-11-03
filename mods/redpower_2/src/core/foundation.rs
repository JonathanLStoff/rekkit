//! Core runtime foundation: orientation math (`MathLib`, `Matrix3`, `Quat`), floppy disk image
//! utilities (`DiskLib`), recipe matching (`CraftLib`, `OreStack`) and the VLC packet descriptors
//! (`PacketVLC`, `Packet211TileDesc`, `Packet212GuiEvent`).
//!
//! Ports of `eloraam.core.{MathLib, Matrix3, Quat, DiskLib, CraftLib, PacketVLC,
//! Packet211TileDesc, Packet212GuiEvent}`. Pure logic, no engine calls.

use super::spatial::Vector3;
use rekkit_core::inventory::ItemStack;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------------------------
// Quat
// ---------------------------------------------------------------------------------------------

/// Quaternion `s + xi + yj + zk`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub s: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(s: f64, x: f64, y: f64, z: f64) -> Self {
        Self { s, x, y, z }
    }

    /// Rotation of `a` radians around the axis `(xi, yi, zi)` (axis is not normalised).
    pub fn around_axis(xi: f64, yi: f64, zi: f64, a: f64) -> Self {
        let h = a * 0.5;
        let sn = h.sin();
        Self::new(h.cos(), xi * sn, yi * sn, zi * sn)
    }

    /// `self = self * q`.
    pub fn multiply(&mut self, q: &Self) {
        let ts = self.s * q.s - self.x * q.x - self.y * q.y - self.z * q.z;
        let tx = self.s * q.x + self.x * q.s - self.y * q.z + self.z * q.y;
        let ty = self.s * q.y + self.x * q.z + self.y * q.s - self.z * q.x;
        let tz = self.s * q.z - self.x * q.y + self.y * q.x + self.z * q.s;
        *self = Self::new(ts, tx, ty, tz);
    }

    /// Original `rightMultiply` (the product with the opposite handedness of the cross term).
    pub fn right_multiply(&mut self, q: &Self) {
        let ts = self.s * q.s - self.x * q.x - self.y * q.y - self.z * q.z;
        let tx = self.s * q.x + self.x * q.s + self.y * q.z - self.z * q.y;
        let ty = self.s * q.y - self.x * q.z + self.y * q.s + self.z * q.x;
        let tz = self.s * q.z + self.x * q.y - self.y * q.x + self.z * q.s;
        *self = Self::new(ts, tx, ty, tz);
    }

    pub fn mag(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.s * self.s).sqrt()
    }

    pub fn normalize(&mut self) {
        let d = self.mag();
        if d != 0.0 {
            let d = 1.0 / d;
            self.x *= d;
            self.y *= d;
            self.z *= d;
            self.s *= d;
        }
    }

    /// Rotate `v` in place (`q v q^-1` as written by the original).
    pub fn rotate(&self, v: &mut Vector3) {
        let ts = -self.x * v.x - self.y * v.y - self.z * v.z;
        let tx = self.s * v.x + self.y * v.z - self.z * v.y;
        let ty = self.s * v.y - self.x * v.z + self.z * v.x;
        let tz = self.s * v.z + self.x * v.y - self.y * v.x;
        v.x = tx * self.s - ts * self.x - ty * self.z + tz * self.y;
        v.y = ty * self.s - ts * self.y + tx * self.z - tz * self.x;
        v.z = tz * self.s - ts * self.z - tx * self.y + ty * self.x;
    }
}

// ---------------------------------------------------------------------------------------------
// Matrix3
// ---------------------------------------------------------------------------------------------

/// 3x3 rotation matrix, row-major (`xx xy xz / yx yy yz / zx zy zz`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Matrix3 {
    pub xx: f64,
    pub xy: f64,
    pub xz: f64,
    pub yx: f64,
    pub yy: f64,
    pub yz: f64,
    pub zx: f64,
    pub zy: f64,
    pub zz: f64,
}

impl Matrix3 {
    pub const IDENTITY: Self = Self {
        xx: 1.0,
        xy: 0.0,
        xz: 0.0,
        yx: 0.0,
        yy: 1.0,
        yz: 0.0,
        zx: 0.0,
        zy: 0.0,
        zz: 1.0,
    };

    /// Matrix of the (unit) quaternion `q`.
    pub fn from_quat(q: &Quat) -> Self {
        Self {
            xx: q.s * q.s + q.x * q.x - q.z * q.z - q.y * q.y,
            xy: 2.0 * (q.s * q.z + q.x * q.y),
            xz: 2.0 * (q.x * q.z - q.s * q.y),
            yx: 2.0 * (q.x * q.y - q.s * q.z),
            yy: q.s * q.s + q.y * q.y - q.z * q.z - q.x * q.x,
            yz: 2.0 * (q.s * q.x + q.y * q.z),
            zx: 2.0 * (q.s * q.y + q.x * q.z),
            zy: 2.0 * (q.y * q.z - q.s * q.x),
            zz: q.s * q.s + q.z * q.z - q.y * q.y - q.x * q.x,
        }
    }

    /// `self * m` as a new matrix.
    pub fn multiply(&self, m: &Self) -> Self {
        Self {
            xx: self.xx * m.xx + self.xy * m.yx + self.xz * m.zx,
            xy: self.xx * m.xy + self.xy * m.yy + self.xz * m.zy,
            xz: self.xx * m.xz + self.xy * m.yz + self.xz * m.zz,
            yx: self.yx * m.xx + self.yy * m.yx + self.yz * m.zx,
            yy: self.yx * m.xy + self.yy * m.yy + self.yz * m.zy,
            yz: self.yx * m.xz + self.yy * m.yz + self.yz * m.zz,
            zx: self.zx * m.xx + self.zy * m.yx + self.zz * m.zx,
            zy: self.zx * m.xy + self.zy * m.yy + self.zz * m.zy,
            zz: self.zx * m.xz + self.zy * m.yz + self.zz * m.zz,
        }
    }

    /// Row `n` (0 = x, 1 = y, otherwise z) as a vector.
    pub fn basis_vector(&self, n: usize) -> Vector3 {
        match n {
            0 => Vector3::new(self.xx, self.xy, self.xz),
            1 => Vector3::new(self.yx, self.yy, self.yz),
            _ => Vector3::new(self.zx, self.zy, self.zz),
        }
    }

    pub fn det(&self) -> f64 {
        self.xx * (self.yy * self.zz - self.yz * self.zy)
            - self.xy * (self.yx * self.zz - self.yz * self.zx)
            + self.xz * (self.yx * self.zy - self.yy * self.zx)
    }

    /// Rotate `v` in place by the transpose-product used by the original (`M^T v`).
    pub fn rotate(&self, v: &mut Vector3) {
        let tx = self.xx * v.x + self.yx * v.y + self.zx * v.z;
        let ty = self.xy * v.x + self.yy * v.y + self.zy * v.z;
        let tz = self.xz * v.x + self.yz * v.y + self.zz * v.z;
        v.x = tx;
        v.y = ty;
        v.z = tz;
    }
}

// ---------------------------------------------------------------------------------------------
// MathLib
// ---------------------------------------------------------------------------------------------

static ORIENT_QUATS: OnceLock<[Quat; 24]> = OnceLock::new();
static ORIENT_MATRICES: OnceLock<[Matrix3; 24]> = OnceLock::new();

fn orient_quats() -> &'static [Quat; 24] {
    ORIENT_QUATS.get_or_init(|| {
        let mut list = [Quat::IDENTITY; 24];
        let q2 = Quat::around_axis(1.0, 0.0, 0.0, PI);
        for j in 0..4 {
            let q1 = Quat::around_axis(0.0, 1.0, 0.0, (-PI / 2.0) * j as f64);
            list[j] = q1;
            let mut flipped = q1;
            flipped.multiply(&q2);
            list[j + 4] = flipped;
        }
        for i in 0..4usize {
            let k = ((i >> 1) | (i << 1)) & 3;
            let mut q2 = Quat::around_axis(0.0, 0.0, 1.0, PI / 2.0);
            q2.multiply(&Quat::around_axis(
                0.0,
                1.0,
                0.0,
                (PI / 2.0) * (k as f64 + 1.0),
            ));
            for j in 0..4 {
                let mut q1 = list[j];
                q1.multiply(&q2);
                list[8 + 4 * i + j] = q1;
            }
        }
        list
    })
}

/// Matrix for the block orientation `(down, rot)`; `down` in 0..6, `rot` in 0..4.
pub fn orient_matrix(down: usize, rot: usize) -> Matrix3 {
    let list = ORIENT_MATRICES.get_or_init(|| {
        let q = orient_quats();
        let mut m = [Matrix3::IDENTITY; 24];
        for (i, slot) in m.iter_mut().enumerate() {
            *slot = Matrix3::from_quat(&q[i]);
        }
        m
    });
    list[down * 4 + rot]
}

/// Quaternion for the block orientation `(down, rot)`.
pub fn orient_quat(down: usize, rot: usize) -> Quat {
    orient_quats()[down * 4 + rot]
}

// ---------------------------------------------------------------------------------------------
// DiskLib
// ---------------------------------------------------------------------------------------------

/// Bytes in one floppy sector.
pub const DISK_SECTOR_SIZE: usize = 128;
/// Sectors on a floppy (256 KB).
pub const DISK_SECTORS: usize = 2048;
/// Total image size in bytes.
pub const DISK_IMAGE_SIZE: usize = DISK_SECTOR_SIZE * DISK_SECTORS;

/// 16 lowercase hex digits drawn from `next_nibble` (a source of values in `0..16`).
pub fn generate_serial_number(mut next_nibble: impl FnMut() -> u32) -> String {
    (0..16)
        .map(|_| format!("{:x}", next_nibble() & 15))
        .collect()
}

/// File name of a disk image: `disk_<serno>.img`.
pub fn disk_file_name(serno: &str) -> String {
    format!("disk_{serno}.img")
}

/// Relative save directory for disk images (under the world save dir).
pub const DISK_SAVE_SUBDIR: &str = "redpower";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskError {
    SectorOutOfRange(usize),
    BadLength(usize),
}

/// In-memory floppy image of 2048 sectors of 128 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskImage {
    data: Vec<u8>,
}

impl Default for DiskImage {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskImage {
    pub fn new() -> Self {
        Self {
            data: vec![0; DISK_IMAGE_SIZE],
        }
    }

    /// Load an image from bytes; shorter input is zero padded, longer input is rejected.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DiskError> {
        if bytes.len() > DISK_IMAGE_SIZE {
            return Err(DiskError::BadLength(bytes.len()));
        }
        let mut data = bytes.to_vec();
        data.resize(DISK_IMAGE_SIZE, 0);
        Ok(Self { data })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn read_sector(&self, sector: usize) -> Result<&[u8], DiskError> {
        if sector >= DISK_SECTORS {
            return Err(DiskError::SectorOutOfRange(sector));
        }
        let o = sector * DISK_SECTOR_SIZE;
        Ok(&self.data[o..o + DISK_SECTOR_SIZE])
    }

    pub fn write_sector(&mut self, sector: usize, bytes: &[u8]) -> Result<(), DiskError> {
        if sector >= DISK_SECTORS {
            return Err(DiskError::SectorOutOfRange(sector));
        }
        if bytes.len() != DISK_SECTOR_SIZE {
            return Err(DiskError::BadLength(bytes.len()));
        }
        let o = sector * DISK_SECTOR_SIZE;
        self.data[o..o + DISK_SECTOR_SIZE].copy_from_slice(bytes);
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------
// OreStack / CraftLib
// ---------------------------------------------------------------------------------------------

/// Damage value that matches any damage.
pub const WILDCARD_DAMAGE: u16 = 32767;

/// Ore dictionary: ore name -> set of `(item_id, damage)` members.
#[derive(Debug, Default, Clone)]
pub struct OreDictionary {
    entries: HashMap<String, HashSet<(String, u16)>>,
}

impl OreDictionary {
    pub fn register(&mut self, name: &str, item_id: &str, damage: u16) {
        self.entries
            .entry(name.to_string())
            .or_default()
            .insert((item_id.to_string(), damage));
    }

    pub fn contains(&self, name: &str, stack: &ItemStack) -> bool {
        self.entries.get(name).is_some_and(|s| {
            s.iter().any(|(id, d)| {
                *id == stack.item_id && (*d == WILDCARD_DAMAGE || *d == stack.damage)
            })
        })
    }
}

/// A recipe ingredient: either a concrete item or an ore-dictionary name, with a required count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OreStack {
    Item {
        item_id: String,
        damage: u16,
        count: u32,
    },
    Ore {
        name: String,
        count: u32,
    },
}

impl OreStack {
    pub fn item(item_id: &str, damage: u16, count: u32) -> Self {
        Self::Item {
            item_id: item_id.to_string(),
            damage,
            count,
        }
    }

    pub fn ore(name: &str, count: u32) -> Self {
        Self::Ore {
            name: name.to_string(),
            count,
        }
    }

    pub fn count(&self) -> u32 {
        match self {
            Self::Item { count, .. } | Self::Ore { count, .. } => *count,
        }
    }

    /// Whether `stack` is the kind of item this ingredient names (count is not compared).
    pub fn matches(&self, stack: &ItemStack, ores: &OreDictionary) -> bool {
        match self {
            Self::Item {
                item_id, damage, ..
            } => {
                *item_id == stack.item_id
                    && (*damage == WILDCARD_DAMAGE
                        || stack.damage == WILDCARD_DAMAGE
                        || *damage == stack.damage)
            }
            Self::Ore { name, .. } => ores.contains(name, stack),
        }
    }
}

/// One alloy furnace recipe: ingredients and the result.
#[derive(Debug, Clone)]
pub struct AlloyRecipe {
    pub inputs: Vec<OreStack>,
    pub output: ItemStack,
}

/// Recipe registry shared by the machines (`CraftLib`).
#[derive(Debug, Default, Clone)]
pub struct CraftLib {
    pub alloy_recipes: Vec<AlloyRecipe>,
    pub damage_on_craft: HashSet<String>,
    /// Tool item id -> container item id left behind when it is consumed.
    pub damage_container: HashMap<String, String>,
    pub ores: OreDictionary,
}

impl CraftLib {
    pub fn add_alloy_result(&mut self, output: ItemStack, inputs: Vec<OreStack>) {
        self.alloy_recipes.push(AlloyRecipe { inputs, output });
    }

    pub fn add_damage_on_craft(&mut self, tool: &str) {
        self.damage_on_craft.insert(tool.to_string());
    }

    pub fn add_damage_container(&mut self, tool: &str, container: &str) {
        self.damage_on_craft.insert(tool.to_string());
        self.damage_container
            .insert(tool.to_string(), container.to_string());
    }

    /// Registers the alloy furnace recipes (`CraftLib` recipe table, spec 7.1).
    pub fn register_default_alloy_recipes(&mut self) {
        let it = |id: &str, n: u32| OreStack::item(id, WILDCARD_DAMAGE, n);
        let out = |id: &str, n: u32| ItemStack {
            item_id: id.to_string(),
            count: n,
            damage: 0,
            max_stack_size: 64,
        };
        let rp = |n: &str| format!("redpower_2:{n}");
        self.add_alloy_result(
            out(&rp("red_alloy_ingot"), 1),
            vec![it("minecraft:redstone", 4), it(&rp("copper_ingot"), 1)],
        );
        self.add_alloy_result(
            out(&rp("red_alloy_ingot"), 1),
            vec![it("minecraft:redstone", 4), it("minecraft:iron_ingot", 1)],
        );
        self.add_alloy_result(
            out(&rp("blue_alloy_ingot"), 1),
            vec![it(&rp("nikolite"), 4), it(&rp("silver_ingot"), 1)],
        );
        self.add_alloy_result(
            out(&rp("brass_ingot"), 4),
            vec![it(&rp("copper_ingot"), 3), it(&rp("tin_ingot"), 1)],
        );
        self.add_alloy_result(
            out(&rp("tinplate"), 4),
            vec![it("minecraft:iron_ingot", 2), it(&rp("tin_ingot"), 1)],
        );
        for fuel in ["minecraft:coal", "minecraft:charcoal"] {
            self.add_alloy_result(
                out(&rp("silicon_boule"), 1),
                vec![it(fuel, 8), it("minecraft:sand", 8)],
            );
        }
        self.add_alloy_result(
            out(&rp("blue_doped_wafer"), 1),
            vec![it(&rp("silicon_wafer"), 1), it(&rp("nikolite"), 4)],
        );
        self.add_alloy_result(
            out(&rp("red_doped_wafer"), 1),
            vec![it(&rp("silicon_wafer"), 1), it("minecraft:redstone", 4)],
        );
        // Recycling: (input, count, output, yield)
        let mut recycle: Vec<(String, u32, &str, u32)> = vec![
            ("minecraft:iron_bars".into(), 8, "minecraft:iron_ingot", 3),
            ("minecraft:minecart".into(), 1, "minecraft:iron_ingot", 5),
            ("minecraft:iron_door".into(), 1, "minecraft:iron_ingot", 6),
            ("minecraft:cauldron".into(), 1, "minecraft:iron_ingot", 7),
            ("minecraft:bucket".into(), 1, "minecraft:iron_ingot", 3),
            ("minecraft:shears".into(), 1, "minecraft:iron_ingot", 2),
            (rp("iron_nugget"), 9, "minecraft:iron_ingot", 1),
            (rp("fine_copper_wire"), 1, "redpower_2:copper_ingot", 1),
            (rp("fine_iron_wire"), 1, "minecraft:iron_ingot", 1),
        ];
        let gear = [
            ("sword", 2),
            ("pickaxe", 3),
            ("axe", 3),
            ("shovel", 1),
            ("hoe", 2),
            ("helmet", 5),
            ("chestplate", 8),
            ("leggings", 7),
            ("boots", 4),
        ];
        for (mat, ingot) in [
            ("iron", "minecraft:iron_ingot"),
            ("golden", "minecraft:gold_ingot"),
        ] {
            for (kind, n) in gear {
                recycle.push((format!("minecraft:{mat}_{kind}"), 1, ingot, n));
            }
        }
        for (input, cnt, output, n) in recycle {
            self.add_alloy_result(out(output, n), vec![it(&input, cnt)]);
        }
    }

    /// First recipe whose every ingredient is covered by `input[ofs..len]`. With `decr`, the
    /// ingredients are removed from the slots (a consumed slot with a container item turns into
    /// one container item, otherwise it empties).
    pub fn get_alloy_result(
        &self,
        input: &mut [Option<ItemStack>],
        ofs: usize,
        len: usize,
        decr: bool,
    ) -> Option<ItemStack> {
        let len = len.min(input.len());
        'recipes: for recipe in &self.alloy_recipes {
            for ing in &recipe.inputs {
                let mut rc = i64::from(ing.count());
                for slot in input.iter().take(len).skip(ofs).flatten() {
                    if ing.matches(slot, &self.ores) {
                        rc -= i64::from(slot.count);
                    }
                    if rc <= 0 {
                        break;
                    }
                }
                if rc > 0 {
                    continue 'recipes;
                }
            }
            if decr {
                for ing in &recipe.inputs {
                    let mut rc = i64::from(ing.count());
                    for cell in input.iter_mut().take(len).skip(ofs) {
                        let Some(slot) = cell.as_mut() else { continue };
                        if !ing.matches(slot, &self.ores) {
                            continue;
                        }
                        rc -= i64::from(slot.count);
                        if rc < 0 {
                            slot.count = (-rc) as u32;
                        } else if let (OreStack::Item { .. }, Some(c)) =
                            (ing, self.damage_container.get(&slot.item_id))
                        {
                            // Only concrete-item ingredients leave a container behind
                            // (the original's ore branch just empties the slot).
                            let mut left = slot.clone();
                            left.item_id = c.clone();
                            left.count = 1;
                            left.damage = 0;
                            *cell = Some(left);
                        } else {
                            *cell = None;
                        }
                        if rc <= 0 {
                            break;
                        }
                    }
                }
            }
            return Some(recipe.output.clone());
        }
        None
    }
}

// ---------------------------------------------------------------------------------------------
// PacketVLC and the two packet descriptors
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketError {
    NotEnoughData,
    BadVlc,
    TooBig,
}

pub fn write_uvlc(out: &mut Vec<u8>, mut l: u64) {
    loop {
        let mut b = (l & 127) as u8;
        l >>= 7;
        if l != 0 {
            b |= 128;
        }
        out.push(b);
        if l == 0 {
            break;
        }
    }
}

/// Signed VLC: sign in the low bit, magnitude above it.
pub fn write_vlc(out: &mut Vec<u8>, l: i64) {
    let v = if l >= 0 {
        (l as u64) << 1
    } else {
        (l.unsigned_abs() << 1) | 1
    };
    write_uvlc(out, v);
}

/// Cursor over a byte slice with the VLC readers.
#[derive(Debug, Clone)]
pub struct PacketReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PacketReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn byte(&mut self) -> Result<u8, PacketError> {
        let b = *self.data.get(self.pos).ok_or(PacketError::NotEnoughData)?;
        self.pos += 1;
        Ok(b)
    }

    pub fn uvlc(&mut self) -> Result<u64, PacketError> {
        let mut tr = 0u64;
        let mut sc = 0u32;
        loop {
            let b = self.byte()?;
            // The original shifts an int; beyond 64 bits the value is rejected.
            tr |= u64::from(b & 127).checked_shl(sc).unwrap_or(0);
            if b & 128 == 0 {
                return Ok(tr);
            }
            sc += 7;
            if sc > 64 {
                return Err(PacketError::BadVlc);
            }
        }
    }

    pub fn vlc(&mut self) -> Result<i64, PacketError> {
        let v = self.uvlc()?;
        Ok(if v & 1 == 0 {
            (v >> 1) as i64
        } else {
            -((v >> 1) as i64)
        })
    }

    pub fn byte_array(&mut self) -> Result<Vec<u8>, PacketError> {
        let n = self.uvlc()? as usize;
        let end = self.pos.checked_add(n).ok_or(PacketError::NotEnoughData)?;
        let s = self
            .data
            .get(self.pos..end)
            .ok_or(PacketError::NotEnoughData)?;
        self.pos = end;
        Ok(s.to_vec())
    }
}

/// Maximum body length of either packet.
pub const MAX_PACKET_BODY: usize = 65535;

/// Tile description packet (id 211): sub id, block position, body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Packet211TileDesc {
    pub sub_id: u8,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub body: Vec<u8>,
}

impl Packet211TileDesc {
    pub const PACKET_ID: u8 = 211;

    pub fn add_byte(&mut self, b: u8) {
        self.body.push(b);
    }
    pub fn add_uvlc(&mut self, l: u64) {
        write_uvlc(&mut self.body, l);
    }
    pub fn add_vlc(&mut self, l: i64) {
        write_vlc(&mut self.body, l);
    }
    pub fn add_byte_array(&mut self, ba: &[u8]) {
        self.add_uvlc(ba.len() as u64);
        self.body.extend_from_slice(ba);
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![self.sub_id];
        write_vlc(&mut out, i64::from(self.x));
        write_vlc(&mut out, i64::from(self.y));
        write_vlc(&mut out, i64::from(self.z));
        write_uvlc(&mut out, self.body.len() as u64);
        out.extend_from_slice(&self.body);
        out
    }

    pub fn decode(data: &[u8]) -> Result<Self, PacketError> {
        let mut r = PacketReader::new(data);
        let sub_id = r.byte()?;
        let x = r.vlc()? as i32;
        let y = r.vlc()? as i32;
        let z = r.vlc()? as i32;
        let l = r.uvlc()? as usize;
        if l > MAX_PACKET_BODY {
            return Err(PacketError::TooBig);
        }
        let body = data
            .get(r.pos..r.pos + l)
            .ok_or(PacketError::NotEnoughData)?
            .to_vec();
        Ok(Self {
            sub_id,
            x,
            y,
            z,
            body,
        })
    }
}

/// GUI event packet (id 212): window id, event id, body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Packet212GuiEvent {
    pub window_id: u8,
    pub event_id: u32,
    pub body: Vec<u8>,
}

impl Packet212GuiEvent {
    pub const PACKET_ID: u8 = 212;

    pub fn add_byte(&mut self, b: u8) {
        self.body.push(b);
    }
    pub fn add_uvlc(&mut self, l: u64) {
        write_uvlc(&mut self.body, l);
    }
    pub fn add_vlc(&mut self, l: i64) {
        write_vlc(&mut self.body, l);
    }
    pub fn add_byte_array(&mut self, ba: &[u8]) {
        self.add_uvlc(ba.len() as u64);
        self.body.extend_from_slice(ba);
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![self.window_id];
        write_uvlc(&mut out, u64::from(self.event_id));
        write_uvlc(&mut out, self.body.len() as u64);
        out.extend_from_slice(&self.body);
        out
    }

    pub fn decode(data: &[u8]) -> Result<Self, PacketError> {
        let mut r = PacketReader::new(data);
        let window_id = r.byte()?;
        let event_id = r.uvlc()? as u32;
        let l = r.uvlc()? as usize;
        if l > MAX_PACKET_BODY {
            return Err(PacketError::TooBig);
        }
        let body = data
            .get(r.pos..r.pos + l)
            .ok_or(PacketError::NotEnoughData)?
            .to_vec();
        Ok(Self {
            window_id,
            event_id,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(id: &str, n: u32) -> ItemStack {
        ItemStack {
            item_id: id.into(),
            count: n,
            damage: 0,
            max_stack_size: 64,
        }
    }

    #[test]
    fn orientations_are_distinct_rotations() {
        let mut seen = Vec::new();
        for d in 0..6 {
            for r in 0..4 {
                let m = orient_matrix(d, r);
                assert!((m.det() - 1.0).abs() < 1e-9);
                let key: Vec<i64> = [m.xx, m.xy, m.xz, m.yx, m.yy, m.yz, m.zx, m.zy, m.zz]
                    .iter()
                    .map(|v| v.round() as i64)
                    .collect();
                assert!(!seen.contains(&key), "duplicate orientation {d},{r}");
                seen.push(key);
            }
        }
        assert_eq!(seen.len(), 24);
        assert_eq!(orient_matrix(0, 0), Matrix3::from_quat(&Quat::IDENTITY));
    }

    #[test]
    fn quat_and_matrix_agree() {
        let q = orient_quat(2, 1);
        let m = orient_matrix(2, 1);
        let mut a = Vector3::new(1.0, 2.0, 3.0);
        let mut b = a;
        q.rotate(&mut a);
        m.rotate(&mut b);
        assert!((a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9 && (a.z - b.z).abs() < 1e-9);
    }

    #[test]
    fn matrix_multiply_and_basis() {
        let m = orient_matrix(1, 2);
        let p = m.multiply(&Matrix3::IDENTITY);
        assert_eq!(p, m);
        let rot = Quat::around_axis(0.0, 1.0, 0.0, PI / 2.0);
        let mut v = Vector3::new(1.0, 0.0, 0.0);
        rot.rotate(&mut v);
        assert!((v.x).abs() < 1e-9 && (v.z + 1.0).abs() < 1e-9);
        assert_eq!(
            Matrix3::IDENTITY.basis_vector(1),
            Vector3::new(0.0, 1.0, 0.0)
        );
    }

    #[test]
    fn disk_sectors() {
        let mut d = DiskImage::new();
        assert_eq!(d.as_bytes().len(), 262_144);
        let s = [7u8; DISK_SECTOR_SIZE];
        d.write_sector(2047, &s).unwrap();
        assert_eq!(d.read_sector(2047).unwrap(), &s[..]);
        assert_eq!(d.read_sector(0).unwrap()[0], 0);
        assert_eq!(
            d.write_sector(2048, &s),
            Err(DiskError::SectorOutOfRange(2048))
        );
        assert_eq!(d.write_sector(0, &[1, 2]), Err(DiskError::BadLength(2)));
        let mut n = 0;
        let sn = generate_serial_number(|| {
            n += 1;
            n
        });
        assert_eq!(sn, "123456789abcdef0");
        assert_eq!(disk_file_name(&sn), "disk_123456789abcdef0.img");
    }

    #[test]
    fn alloy_matching_and_decrement() {
        let mut c = CraftLib::default();
        c.ores.register("ingotCopper", "redpower_2:ingot_copper", 0);
        c.add_alloy_result(
            stack("redpower_2:ingot_brass", 4),
            vec![
                OreStack::ore("ingotCopper", 3),
                OreStack::item("redpower_2:ingot_zinc", 0, 1),
            ],
        );
        let mut inv = vec![
            Some(stack("redpower_2:ingot_copper", 2)),
            Some(stack("redpower_2:ingot_copper", 2)),
            Some(stack("redpower_2:ingot_zinc", 1)),
            None,
        ];
        assert!(c.get_alloy_result(&mut inv, 0, 2, false).is_none());
        let out = c.get_alloy_result(&mut inv, 0, 4, false).unwrap();
        assert_eq!(out.count, 4);
        assert_eq!(inv[0].as_ref().unwrap().count, 2);
        c.get_alloy_result(&mut inv, 0, 4, true).unwrap();
        assert!(inv[0].is_none());
        assert_eq!(inv[1].as_ref().unwrap().count, 1);
        assert!(inv[2].is_none());
    }

    #[test]
    fn ore_ingredient_leaves_no_container() {
        let mut c = CraftLib::default();
        c.add_damage_container("dust", "box");
        c.ores.register("dustX", "dust", 0);
        c.add_alloy_result(stack("out", 1), vec![OreStack::ore("dustX", 1)]);
        let mut inv = vec![Some(stack("dust", 1))];
        c.get_alloy_result(&mut inv, 0, 1, true).unwrap();
        assert!(inv[0].is_none());
    }

    #[test]
    fn default_alloy_recipes() {
        let mut c = CraftLib::default();
        c.register_default_alloy_recipes();
        let mut inv = vec![
            Some(stack("redpower_2:copper_ingot", 3)),
            Some(stack("redpower_2:tin_ingot", 2)),
        ];
        let o = c.get_alloy_result(&mut inv, 0, 2, true).unwrap();
        assert_eq!((o.item_id.as_str(), o.count), ("redpower_2:brass_ingot", 4));
        assert!(inv[0].is_none());
        assert_eq!(inv[1].as_ref().unwrap().count, 1);
        let mut inv = vec![Some(stack("minecraft:iron_door", 1))];
        assert_eq!(c.get_alloy_result(&mut inv, 0, 1, false).unwrap().count, 6);
    }

    #[test]
    fn damage_container_left_behind() {
        let mut c = CraftLib::default();
        c.add_damage_container("tool", "box");
        assert!(c.damage_on_craft.contains("tool"));
        c.add_alloy_result(stack("out", 1), vec![OreStack::item("tool", 0, 1)]);
        let mut inv = vec![Some(stack("tool", 1))];
        c.get_alloy_result(&mut inv, 0, 1, true).unwrap();
        assert_eq!(inv[0].as_ref().unwrap().item_id, "box");
    }

    #[test]
    fn vlc_round_trip() {
        for v in [
            0i64,
            1,
            -1,
            63,
            64,
            -64,
            100_000,
            -100_000,
            i32::MAX as i64,
            i32::MIN as i64 + 1,
        ] {
            let mut b = Vec::new();
            write_vlc(&mut b, v);
            assert_eq!(PacketReader::new(&b).vlc().unwrap(), v);
        }
        let mut b = Vec::new();
        write_uvlc(&mut b, 300);
        assert_eq!(b, vec![0xAC, 0x02]);
    }

    #[test]
    fn packets_round_trip() {
        let mut p = Packet211TileDesc {
            sub_id: 3,
            x: -12,
            y: 64,
            z: 1000,
            body: vec![],
        };
        p.add_byte(9);
        p.add_byte_array(&[1, 2, 3]);
        let d = Packet211TileDesc::decode(&p.encode()).unwrap();
        assert_eq!(d, p);
        assert_eq!(
            Packet211TileDesc::decode(&p.encode()[..4]),
            Err(PacketError::NotEnoughData)
        );

        let mut g = Packet212GuiEvent {
            window_id: 5,
            event_id: 300,
            body: vec![],
        };
        g.add_vlc(-5);
        let d = Packet212GuiEvent::decode(&g.encode()).unwrap();
        assert_eq!(d, g);
        assert_eq!(PacketReader::new(&d.body).vlc().unwrap(), -5);
    }
}

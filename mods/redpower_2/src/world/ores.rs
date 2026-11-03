//! Ores and mineral storage blocks.
//!
//! Java reference: `eloraam/world/{BlockCustomOre,BlockStorage,WorldGenCustomOre}.java`
//! and `RedPowerWorld#GenerateSurface`. Pure logic: randomness is passed in as closures.

/// The eight ore variants of `BlockCustomOre` (meta 0..=7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ore {
    Ruby,
    GreenSapphire,
    Sapphire,
    Silver,
    Tin,
    Copper,
    Tungsten,
    Nikolite,
}

pub const ALL_ORES: [Ore; 8] = [
    Ore::Ruby,
    Ore::GreenSapphire,
    Ore::Sapphire,
    Ore::Silver,
    Ore::Tin,
    Ore::Copper,
    Ore::Tungsten,
    Ore::Nikolite,
];

/// Ore hardness (`BlockCustomOre#getHardness`) and blast resistance.
pub const ORE_HARDNESS: f32 = 3.0;
pub const ORE_RESISTANCE: f32 = 5.0;
/// Storage block hardness and resistance (`BlockStorage` ctor).
pub const STORAGE_HARDNESS: f32 = 5.0;
pub const STORAGE_RESISTANCE: f32 = 10.0;

/// Per-ore world generation: clusters per chunk, vein size, Y range `0..y_range`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OreGen {
    pub clusters: u32,
    pub vein_size: u32,
    pub y_range: u32,
}

impl Ore {
    pub fn meta(self) -> u8 {
        self as u8
    }
    pub fn from_meta(meta: u8) -> Option<Self> {
        ALL_ORES.get(meta as usize).copied()
    }
    pub fn block_id(self) -> &'static str {
        match self {
            Self::Ruby => "redpower_2:ore_ruby",
            Self::GreenSapphire => "redpower_2:ore_green_sapphire",
            Self::Sapphire => "redpower_2:ore_sapphire",
            Self::Silver => "redpower_2:ore_silver",
            Self::Tin => "redpower_2:ore_tin",
            Self::Copper => "redpower_2:ore_copper",
            Self::Tungsten => "redpower_2:ore_tungsten",
            Self::Nikolite => "redpower_2:ore_nikolite",
        }
    }
    /// Item dropped, or `None` when the ore block drops itself.
    pub fn drop_item(self) -> Option<&'static str> {
        match self {
            Self::Ruby => Some("redpower_2:ruby"),
            Self::GreenSapphire => Some("redpower_2:green_sapphire"),
            Self::Sapphire => Some("redpower_2:sapphire"),
            Self::Nikolite => Some("redpower_2:nikolite"),
            _ => None,
        }
    }
    /// Smelting result for ores that smelt.
    pub fn smelts_to(self) -> Option<&'static str> {
        match self {
            Self::Silver => Some("redpower_2:silver_ingot"),
            Self::Tin => Some("redpower_2:tin_ingot"),
            Self::Copper => Some("redpower_2:copper_ingot"),
            _ => None,
        }
    }
    /// Pickaxe harvest level (0 wood, 1 stone, 2 iron).
    pub fn harvest_level(self) -> u8 {
        match self {
            Self::Silver => 1,
            Self::Tin | Self::Copper => 0,
            _ => 2,
        }
    }
    pub fn gen(self) -> OreGen {
        let (clusters, vein_size, y_range) = match self {
            Self::Ruby | Self::GreenSapphire | Self::Sapphire => (2, 7, 48),
            Self::Silver => (4, 8, 32),
            Self::Tin => (10, 8, 48),
            Self::Copper => (20, 8, 64),
            Self::Tungsten => (1, 4, 16),
            Self::Nikolite => (4, 10, 16),
        };
        OreGen {
            clusters,
            vein_size,
            y_range,
        }
    }
    /// Experience dropped when mined, as an inclusive range (spec section 5).
    pub fn xp_range(self) -> (u32, u32) {
        match self {
            Self::Ruby | Self::GreenSapphire | Self::Sapphire => (3, 7),
            Self::Nikolite => (1, 5),
            _ => (0, 0),
        }
    }
    /// Drop count (`BlockCustomOre#quantityDropped`); `rng(n)` returns `0..n`.
    pub fn quantity_dropped(self, fortune: u32, rng: &mut dyn FnMut(u32) -> u32) -> u32 {
        match self {
            Self::Nikolite => 4 + rng(2) + rng(fortune + 1),
            Self::Ruby | Self::GreenSapphire | Self::Sapphire => {
                let b = rng(fortune + 2) as i64 - 1;
                b.max(0) as u32 + 1
            }
            _ => 1,
        }
    }
}

/// The three gem storage blocks of `BlockStorage` (meta 0..=2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageBlock {
    Ruby,
    GreenSapphire,
    Sapphire,
}

impl StorageBlock {
    pub fn block_id(self) -> &'static str {
        match self {
            Self::Ruby => "redpower_2:block_ruby",
            Self::GreenSapphire => "redpower_2:block_green_sapphire",
            Self::Sapphire => "redpower_2:block_sapphire",
        }
    }
    pub fn gem_id(self) -> &'static str {
        match self {
            Self::Ruby => "redpower_2:ruby",
            Self::GreenSapphire => "redpower_2:green_sapphire",
            Self::Sapphire => "redpower_2:sapphire",
        }
    }
}

/// Block-compress recipe: 9 gems -> 1 block; and 1 block -> 9 gems.
pub const GEMS_PER_BLOCK: u32 = 9;

/// Which stone positions an ore vein replaces: a position is placed only inside stone.
/// Returns the block positions of one vein (`WorldGenCustomOre#a`), starting at `(x, y, z)`.
/// `rng_f` yields a float in `[0,1)`, `rng_n(n)` an int in `0..n`.
pub fn vein_positions(
    size: u32,
    x: i32,
    y: i32,
    z: i32,
    rng_f: &mut dyn FnMut() -> f64,
    rng_n: &mut dyn FnMut(u32) -> u32,
) -> Vec<(i32, i32, i32)> {
    let n = size as f64;
    let f = rng_f() as f32 * std::f32::consts::PI;
    let (sin, cos) = (f.sin() as f64, f.cos() as f64);
    let d = x as f64 + 8.0 + sin * n / 8.0;
    let d1 = x as f64 + 8.0 - sin * n / 8.0;
    let d2 = z as f64 + 8.0 + cos * n / 8.0;
    let d3 = z as f64 + 8.0 - cos * n / 8.0;
    let d4 = y as f64 + rng_n(3) as f64 + 2.0;
    let d5 = y as f64 + rng_n(3) as f64 + 2.0;
    let mut out = Vec::new();
    for l in 0..=size {
        let t = l as f64 / n;
        let (d6, d7, d8) = (d + (d1 - d) * t, d4 + (d5 - d4) * t, d2 + (d3 - d2) * t);
        let d9 = rng_f() * n / 16.0;
        let d10 = ((l as f32 * std::f32::consts::PI / size as f32).sin() as f64 + 1.0) * d9 + 1.0;
        let (h, lo) = (d10 / 2.0, d10 / 2.0);
        let (x0, x1) = ((d6 - h).floor() as i32, (d6 + h).floor() as i32);
        let (y0, y1) = ((d7 - lo).floor() as i32, (d7 + lo).floor() as i32);
        let (z0, z1) = ((d8 - h).floor() as i32, (d8 + h).floor() as i32);
        for px in x0..=x1 {
            let a = (px as f64 + 0.5 - d6) / h;
            if a * a >= 1.0 {
                continue;
            }
            for py in y0..=y1 {
                let b = (py as f64 + 0.5 - d7) / lo;
                if a * a + b * b >= 1.0 {
                    continue;
                }
                for pz in z0..=z1 {
                    let c = (pz as f64 + 0.5 - d8) / h;
                    if a * a + b * b + c * c < 1.0 {
                        out.push((px, py, pz));
                    }
                }
            }
        }
    }
    out
}

/// Origins `(x, y, z)` of every vein attempted for `ore` in a chunk at block origin
/// `(chunk_x, chunk_z)` (`RedPowerWorld#GenerateSurface`).
pub fn vein_origins(
    ore: Ore,
    chunk_x: i32,
    chunk_z: i32,
    rng: &mut dyn FnMut(u32) -> u32,
) -> Vec<(i32, i32, i32)> {
    let g = ore.gen();
    (0..g.clusters)
        .map(|_| {
            let x = chunk_x + rng(16) as i32;
            let y = rng(g.y_range) as i32;
            let z = chunk_z + rng(16) as i32;
            (x, y, z)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metas_roundtrip() {
        for (i, o) in ALL_ORES.iter().enumerate() {
            assert_eq!(Ore::from_meta(i as u8), Some(*o));
        }
        assert_eq!(Ore::from_meta(8), None);
    }

    #[test]
    fn drops() {
        let mut zero = |_n: u32| 0;
        assert_eq!(Ore::Copper.quantity_dropped(3, &mut zero), 1);
        assert_eq!(Ore::Ruby.quantity_dropped(0, &mut zero), 1);
        assert_eq!(Ore::Nikolite.quantity_dropped(0, &mut zero), 4);
        let mut max = |n: u32| n - 1;
        assert_eq!(Ore::Nikolite.quantity_dropped(2, &mut max), 4 + 1 + 2);
        assert_eq!(Ore::Sapphire.quantity_dropped(2, &mut max), 3);
    }

    #[test]
    fn gen_table() {
        assert_eq!(
            Ore::Copper.gen(),
            OreGen {
                clusters: 20,
                vein_size: 8,
                y_range: 64
            }
        );
        assert_eq!(Ore::Tungsten.gen().vein_size, 4);
        assert_eq!(Ore::Silver.harvest_level(), 1);
    }

    #[test]
    fn vein_is_nonempty_and_near_origin() {
        let mut s = 12345u64;
        let mut next = move || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let mut s2 = 99u64;
        let mut rf = move || {
            s2 ^= s2 << 13;
            s2 ^= s2 >> 7;
            s2 ^= s2 << 17;
            (s2 >> 11) as f64 / (1u64 << 53) as f64
        };
        let mut rn = move |n: u32| (next() % n as u64) as u32;
        let v = vein_positions(8, 0, 20, 0, &mut rf, &mut rn);
        assert!(!v.is_empty());
        assert!(v
            .iter()
            .all(|p| p.0.abs() < 20 && p.2.abs() < 20 && (p.1 - 20).abs() < 10));
    }
}

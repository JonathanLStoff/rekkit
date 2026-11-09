//! World generation handler and terrain feature placement (`spec §5`).
//!
//! Java reference: `eloraam/world/WorldGenHandler.java`.
//!
//! Features per chunk:
//! - 8 custom ore veins (Ruby, Green Sapphire, Sapphire, Silver, Tin, Copper, Tungsten, Nikolite).
//! - 4 Marble strata pockets (Y 32..63).
//! - 1/256 chance of active Basalt Volcano with lava chamber.
//! - Biome-dependent Indigo flower patches.
//! - Biome-dependent Jungle Rubber Trees (6/chunk in Jungle biomes).

use crate::ores::ALL_ORES;
use crate::stone::{marble_fill, BlockClass};
use crate::trees::{indigo_patches_for_biome, rubber_trees_for_biome, tree_height};

/// Placed block record in world generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenBlock {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_id: &'static str,
    pub meta: u8,
}

/// Simple deterministic RNG for chunk feature population.
pub struct ChunkRng {
    state: u64,
}

impl ChunkRng {
    pub fn new(chunk_x: i32, chunk_z: i32, world_seed: u64) -> Self {
        let n = (chunk_x as u64).wrapping_mul(31) ^ (chunk_z as u64) ^ world_seed;
        Self {
            state: n.wrapping_add(0x9E3779B97F4A7C15),
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    pub fn next_range(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            0
        } else {
            self.next_u32() % bound
        }
    }
}

/// Generates all RedPower 2 ores and features within a 16x16 chunk.
pub fn generate_chunk(chunk_x: i32, chunk_z: i32, biome: &str, world_seed: u64) -> Vec<GenBlock> {
    let mut rng = ChunkRng::new(chunk_x, chunk_z, world_seed);
    let mut blocks = Vec::new();
    let base_x = chunk_x * 16;
    let base_z = chunk_z * 16;

    // 1. Ore veins
    for ore in ALL_ORES {
        let gen = ore.gen();
        for _ in 0..gen.clusters {
            let x = base_x + rng.next_range(16) as i32;
            let y = rng.next_range(gen.y_range) as i32;
            let z = base_z + rng.next_range(16) as i32;

            // Simple cluster footprint around (x, y, z)
            let cluster_size = gen.vein_size.min(10);
            for i in 0..cluster_size {
                let ox = x + (i as i32 % 3) - 1;
                let oy = (y + (i as i32 / 3)).clamp(1, 255);
                let oz = z + ((i as i32 + 1) % 3) - 1;

                blocks.push(GenBlock {
                    x: ox,
                    y: oy,
                    z: oz,
                    block_id: ore.block_id(),
                    meta: ore.meta(),
                });
            }
        }
    }

    // 2. Marble pockets (4 pockets/chunk, Y 32..63)
    for _ in 0..4 {
        let x = base_x + rng.next_range(16) as i32;
        let y = 32 + rng.next_range(32) as i32;
        let z = base_z + rng.next_range(16) as i32;

        let pocket = marble_fill((x, y, z), 16, |_bx, by, _bz| {
            if by > y {
                BlockClass::Stone
            } else {
                BlockClass::Air
            }
        });

        if let Some(positions) = pocket {
            for (px, py, pz) in positions {
                blocks.push(GenBlock {
                    x: px,
                    y: py,
                    z: pz,
                    block_id: "redpower_2:marble",
                    meta: 0,
                });
            }
        }
    }

    // 3. Basalt Volcanoes (1 in 256 chance per chunk)
    if rng.next_range(256) == 0 {
        let vx = base_x + 8;
        let vz = base_z + 8;
        let v_base_y = 60;
        let v_height = 20;

        for dy in 0..v_height {
            let radius = (v_height - dy) / 2 + 2;
            let y = v_base_y + dy;
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let dist_sq = dx * dx + dz * dz;
                    if dist_sq <= 1 {
                        // Magma conduit core
                        blocks.push(GenBlock {
                            x: vx + dx,
                            y,
                            z: vz + dz,
                            block_id: "minecraft:lava",
                            meta: 0,
                        });
                    } else if dist_sq <= radius * radius {
                        // Basalt cone wall
                        blocks.push(GenBlock {
                            x: vx + dx,
                            y,
                            z: vz + dz,
                            block_id: "redpower_2:basalt",
                            meta: 1,
                        });
                    }
                }
            }
        }
    }

    // 4. Wild Indigo Flowers
    let flower_patches = indigo_patches_for_biome(biome);
    for _ in 0..flower_patches {
        let fx = base_x + rng.next_range(16) as i32;
        let fz = base_z + rng.next_range(16) as i32;
        blocks.push(GenBlock {
            x: fx,
            y: 64,
            z: fz,
            block_id: "redpower_2:indigo_flower",
            meta: 0,
        });
    }

    // 5. Jungle Rubber Trees
    let rubber_trees = rubber_trees_for_biome(biome);
    for _ in 0..rubber_trees {
        let tx = base_x + rng.next_range(16) as i32;
        let tz = base_z + rng.next_range(16) as i32;
        let th = tree_height(rng.next_range(6));

        // 3x3 trunk base
        for ty in 0..7 {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    blocks.push(GenBlock {
                        x: tx + dx,
                        y: 64 + ty,
                        z: tz + dz,
                        block_id: "redpower_2:rubber_log",
                        meta: 1,
                    });
                }
            }
        }

        // Tree crown
        for cy in 7..th {
            blocks.push(GenBlock {
                x: tx,
                y: 64 + cy,
                z: tz,
                block_id: "redpower_2:rubber_log",
                meta: 1,
            });
            for dx in -2..=2 {
                for dz in -2..=2 {
                    if dx != 0 || dz != 0 {
                        blocks.push(GenBlock {
                            x: tx + dx,
                            y: 64 + cy,
                            z: tz + dz,
                            block_id: "redpower_2:rubber_leaves",
                            meta: 0,
                        });
                    }
                }
            }
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_ore_and_marble_generation() {
        let blocks = generate_chunk(0, 0, "plains", 42);
        assert!(!blocks.is_empty());

        let has_ruby = blocks.iter().any(|b| b.block_id == "redpower_2:ore_ruby");
        let has_copper = blocks.iter().any(|b| b.block_id == "redpower_2:ore_copper");
        let has_marble = blocks.iter().any(|b| b.block_id == "redpower_2:marble");
        let has_indigo = blocks
            .iter()
            .any(|b| b.block_id == "redpower_2:indigo_flower");

        assert!(has_ruby);
        assert!(has_copper);
        assert!(has_marble);
        assert!(has_indigo);
    }

    #[test]
    fn test_jungle_rubber_tree_generation() {
        let jungle_blocks = generate_chunk(1, 1, "jungle", 1234);
        let has_rubber_logs = jungle_blocks
            .iter()
            .any(|b| b.block_id == "redpower_2:rubber_log");
        let has_rubber_leaves = jungle_blocks
            .iter()
            .any(|b| b.block_id == "redpower_2:rubber_leaves");

        assert!(has_rubber_logs);
        assert!(has_rubber_leaves);

        let plains_blocks = generate_chunk(1, 1, "plains", 1234);
        assert!(!plains_blocks
            .iter()
            .any(|b| b.block_id == "redpower_2:rubber_log"));
    }
}

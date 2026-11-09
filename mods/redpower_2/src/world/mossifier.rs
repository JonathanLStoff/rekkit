//! RedPower 2 mossifier and brick cracking mechanics.
//!
//! Java reference: `eloraam/world/{BlockBrickMossifier,BlockCobbleMossifier}.java`.
//!
//! Mechanics:
//! - Brick cracking (`crackBrick`): When stone bricks are adjacent to both water and lava
//!   across their 6 faces, a 50% chance cracks the bricks (`minecraft:cracked_stone_bricks`).
//! - Moss spreading (`spreadMoss`): In shaded/damp conditions (block above is covered, not seeing sky),
//!   adjacent cobblestone and stone bricks touching water have a 50% chance to turn into their
//!   mossy variants (`minecraft:mossy_cobblestone` and `minecraft:mossy_stone_bricks`).

/// Block IDs involved in mossifier and brick cracking transformations.
pub const COBBLESTONE: &str = "minecraft:cobblestone";
pub const MOSSY_COBBLESTONE: &str = "minecraft:mossy_cobblestone";
pub const STONE_BRICKS: &str = "minecraft:stone_bricks";
pub const MOSSY_STONE_BRICKS: &str = "minecraft:mossy_stone_bricks";
pub const CRACKED_STONE_BRICKS: &str = "minecraft:cracked_stone_bricks";
pub const WATER: &str = "minecraft:water";
pub const FLOWING_WATER: &str = "minecraft:flowing_water";
pub const LAVA: &str = "minecraft:lava";
pub const FLOWING_LAVA: &str = "minecraft:flowing_lava";
pub const AIR: &str = "minecraft:air";

/// Checks if a block is water.
pub fn is_water(id: &str) -> bool {
    let stripped = id.strip_prefix("minecraft:").unwrap_or(id);
    stripped == "water" || stripped == "flowing_water"
}

/// Checks if a block is lava.
pub fn is_lava(id: &str) -> bool {
    let stripped = id.strip_prefix("minecraft:").unwrap_or(id);
    stripped == "lava" || stripped == "flowing_lava"
}

/// Checks if a block is air.
pub fn is_air(id: &str) -> bool {
    let stripped = id.strip_prefix("minecraft:").unwrap_or(id);
    stripped == "air" || stripped == "cave_air" || stripped == "void_air"
}

/// Checks if adjacent 6 blocks contain both water and lava, cracking stone bricks.
///
/// Java: `BlockBrickMossifier.crackBrick`
pub fn try_crack_brick(
    get_block: impl Fn(i32, i32, i32) -> &'static str,
    pos: (i32, i32, i32),
    rng_coin_flip: bool,
) -> Option<&'static str> {
    let (x, y, z) = pos;
    let faces = [
        (x - 1, y, z),
        (x + 1, y, z),
        (x, y - 1, z),
        (x, y + 1, z),
        (x, y, z - 1),
        (x, y, z + 1),
    ];

    let mut has_water = false;
    let mut has_lava = false;

    for (fx, fy, fz) in faces {
        let b = get_block(fx, fy, fz);
        if is_water(b) {
            has_water = true;
        } else if is_lava(b) {
            has_lava = true;
        }
    }

    if has_water && has_lava && rng_coin_flip {
        Some(CRACKED_STONE_BRICKS)
    } else {
        None
    }
}

/// Result of a moss spread attempt onto an adjacent position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MossSpreadTarget {
    pub pos: (i32, i32, i32),
    pub new_block_id: &'static str,
}

/// Attempts to spread moss from a mossy block `(x, y, z)` to 4 horizontal neighbors.
///
/// Java: `BlockBrickMossifier.spreadMoss` and `BlockCobbleMossifier.b`
///
/// Requirements:
/// - Block above source is solid/opaque or not seeing direct sunlight.
/// - Neighbor is cobblestone or stone bricks.
/// - Block above neighbor is air and does not see direct sunlight.
/// - Neighbor is horizontally adjacent to water.
/// - 50% roll passes (`rng_coin_flip == true`).
pub fn try_spread_moss(
    get_block: impl Fn(i32, i32, i32) -> &'static str,
    sees_sky: impl Fn(i32, i32, i32) -> bool,
    is_solid_above_source: bool,
    pos: (i32, i32, i32),
    rng_coin_flip: impl Fn() -> bool,
) -> Vec<MossSpreadTarget> {
    let (x, y, z) = pos;

    // Source must be shaded / covered
    if !is_solid_above_source || sees_sky(x, y + 1, z) {
        return Vec::new();
    }

    let horizontal_neighbors = [
        (x, y, z - 1), // North
        (x, y, z + 1), // South
        (x - 1, y, z), // West
        (x + 1, y, z), // East
    ];

    let mut results = Vec::new();

    for (nx, ny, nz) in horizontal_neighbors {
        let n_block = get_block(nx, ny, nz);
        let n_stripped = n_block.strip_prefix("minecraft:").unwrap_or(n_block);

        let target_block = match n_stripped {
            "cobblestone" => MOSSY_COBBLESTONE,
            "stone_bricks" => MOSSY_STONE_BRICKS,
            _ => continue,
        };

        // Block directly above neighbor must be air and not see direct sky
        let above_neighbor = get_block(nx, ny + 1, nz);
        if !is_air(above_neighbor) || sees_sky(nx, ny + 1, nz) {
            continue;
        }

        // Check if neighbor is horizontally adjacent to water
        let neighbor_faces = [
            (nx, ny, nz - 1),
            (nx, ny, nz + 1),
            (nx - 1, ny, nz),
            (nx + 1, ny, nz),
        ];

        let has_water = neighbor_faces
            .iter()
            .any(|&(wx, wy, wz)| is_water(get_block(wx, wy, wz)));

        if has_water && rng_coin_flip() {
            results.push(MossSpreadTarget {
                pos: (nx, ny, nz),
                new_block_id: target_block,
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crack_brick_with_water_and_lava() {
        let world = |x: i32, _y: i32, _z: i32| {
            if x == 1 {
                WATER
            } else if x == -1 {
                LAVA
            } else {
                AIR
            }
        };

        // Lava + water + true roll -> cracked
        let res = try_crack_brick(world, (0, 10, 0), true);
        assert_eq!(res, Some(CRACKED_STONE_BRICKS));

        // False roll -> None
        let res_fail = try_crack_brick(world, (0, 10, 0), false);
        assert_eq!(res_fail, None);

        // Missing lava -> None
        let world_no_lava = |x: i32, _y: i32, _z: i32| {
            if x == 1 {
                WATER
            } else {
                AIR
            }
        };
        assert_eq!(try_crack_brick(world_no_lava, (0, 10, 0), true), None);
    }

    #[test]
    fn test_spread_moss_to_cobblestone_and_bricks() {
        let world = |x: i32, y: i32, z: i32| {
            if y == 11 {
                AIR
            } else if (x, y, z) == (0, 10, -1) {
                COBBLESTONE
            } else if (x, y, z) == (0, 10, 1) {
                STONE_BRICKS
            } else if (x, y, z) == (0, 10, -2) || (x, y, z) == (0, 10, 2) {
                WATER
            } else {
                AIR
            }
        };

        let sees_sky = |_x, _y, _z| false;

        let targets = try_spread_moss(world, sees_sky, true, (0, 10, 0), || true);
        assert_eq!(targets.len(), 2);

        assert!(targets
            .iter()
            .any(|t| t.pos == (0, 10, -1) && t.new_block_id == MOSSY_COBBLESTONE));
        assert!(targets
            .iter()
            .any(|t| t.pos == (0, 10, 1) && t.new_block_id == MOSSY_STONE_BRICKS));
    }

    #[test]
    fn test_spread_moss_fails_in_sunlight_or_uncovered() {
        let world = |x: i32, y: i32, z: i32| {
            if (x, y, z) == (0, 10, -1) {
                COBBLESTONE
            } else if (x, y, z) == (0, 10, -2) {
                WATER
            } else {
                AIR
            }
        };

        // If source is uncovered (not solid above) -> fails
        let targets1 = try_spread_moss(world, |_x, _y, _z| false, false, (0, 10, 0), || true);
        assert!(targets1.is_empty());

        // If neighbor sees direct sunlight -> fails
        let targets2 = try_spread_moss(world, |_x, _y, _z| true, true, (0, 10, 0), || true);
        assert!(targets2.is_empty());
    }
}

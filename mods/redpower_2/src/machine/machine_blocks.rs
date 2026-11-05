//! RedPower 2 Machine Blocks (`BlockMachine` meta 0..15).
//!
//! Mirrors:
//! - `com.eloraam.redpower.machine.BlockMachine`:
//!   - Metadata dispatch (meta 0..15): Deployer (0), Block Breaker (1), Transposer (2),
//!     Filter (3), Item Detector (4), Sorting Machine (5), Battery Box (6), Frame Motor (7),
//!     Retriever (8), Wind Turbine (9), Regulator (10), Thermopile (11), Igniter (12),
//!     Assembler (13), Ejector (14), Relay (15).
//!   - `getBlockTextureFromSideAndMetadata`: texture mapping for sides and machine states.
//!   - `onBlockPlacedBy`: 6-direction facing orientation based on player placement geometry.
//!   - Harvest properties: hardness 2.0F, material rock, requires pickaxe, self drops.
//!   - `isFireSource`: Igniter (meta 12) provides fire source on top when active and facing down.
//! - `com.eloraam.redpower.machine.ItemRenderMachine`:
//!   - `renderItem`: handles inventory rendering for machines and dynamic battery box charge level (meta 6).
//! - `com.eloraam.redpower.machine.EntityPlayerFake`:
//!   - `canHarvestBlock`: checks if a block with given hardness can be harvested (hardness >= 0.0).
//!   - Simulated right-click deploy, block harvesting, and coordinate/pitch/yaw tracking.

use super::base::{orient_textures, MachineFlags, MachineKind, TileMachine};
use crate::ItemStack;
use rekkit_core::grid::BlockPos;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};

// ============================================================================
// 1. BlockMachine Harvest Properties & Metadata Registration
// ============================================================================

/// Machine block hardness constant mirroring `BlockMachine:11` (`this.c(2.0F)`).
pub const BLOCK_MACHINE_HARDNESS: f32 = 2.0;

/// Machine block blast resistance constant.
pub const BLOCK_MACHINE_RESISTANCE: f32 = 10.0;

/// Machine harvest tool requirement (Pickaxe for rock material `agi.e`).
pub const BLOCK_MACHINE_TOOL: &str = "minecraft:mineable/pickaxe";

/// Full configuration and registry descriptor for a machine block variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachineBlockDef {
    pub kind: MachineKind,
    pub metadata: u8,
    pub block_id: &'static str,
    pub unlocalized_name: &'static str,
    pub tile_entity_id: &'static str,
    pub has_gui: bool,
    pub slot_count: usize,
    pub hardness: f32,
    pub explosion_resistance: f32,
}

impl MachineBlockDef {
    pub const fn of(kind: MachineKind) -> Self {
        let meta = kind.to_meta();
        let tile_entity_id = match kind {
            MachineKind::Deployer => "RPDeploy",
            MachineKind::BlockBreaker => "RPBreaker",
            MachineKind::Transposer => "RPTranspose",
            MachineKind::Filter => "RPFilter",
            MachineKind::ItemDetector => "RPItemDet",
            MachineKind::SortingMachine => "RPSorter",
            MachineKind::BatteryBox => "RPBatBox",
            MachineKind::FrameMotor => "RPMotor",
            MachineKind::Retriever => "RPRetrieve",
            MachineKind::WindTurbine => "RPWind",
            MachineKind::Regulator => "RPRegulate",
            MachineKind::Thermopile => "RPThermo",
            MachineKind::Igniter => "RPIgnite",
            MachineKind::Assembler => "RPAssemble",
            MachineKind::Ejector => "RPEject",
            MachineKind::Relay => "RPRelay",
        };

        Self {
            kind,
            metadata: meta,
            block_id: kind.block_id(),
            unlocalized_name: kind.item_name(),
            tile_entity_id,
            has_gui: kind.has_gui(),
            slot_count: kind.default_slot_count(),
            hardness: BLOCK_MACHINE_HARDNESS,
            explosion_resistance: BLOCK_MACHINE_RESISTANCE,
        }
    }
}

/// Registry table of all 16 `BlockMachine` metadata configurations.
pub struct BlockMachineRegistry;

impl BlockMachineRegistry {
    /// Return the definition for a given metadata `0..=15`.
    pub fn get_by_metadata(metadata: u8) -> Option<MachineBlockDef> {
        MachineKind::from_meta(metadata).map(MachineBlockDef::of)
    }

    /// Return the definition for a canonical block ID string.
    pub fn get_by_block_id(block_id: &str) -> Option<MachineBlockDef> {
        MachineKind::from_block_id(block_id).map(MachineBlockDef::of)
    }

    /// Get harvest drops for a machine block: drops 1 of itself with its metadata.
    pub fn get_block_dropped(metadata: u8, _fortune: u32) -> Option<ItemStack> {
        let kind = MachineKind::from_meta(metadata)?;
        Some(ItemStack::new(kind.block_id(), 1))
    }

    /// Replicate `BlockMachine.isFireSource(world, x, y, z, md, face)`:
    /// Returns true only for Igniter (`md == 12`) when active and facing `Down` (rotation 0,
    /// so the front face and fire are on the top face).
    pub fn is_fire_source(
        metadata: u8,
        rotation: Direction,
        active: bool,
        face: Direction,
    ) -> bool {
        if metadata != 12 {
            return false;
        }
        // TileIgniter:64 `return this.Rotation != 0 ? false : this.Active;`
        rotation == Direction::Down && active && face == Direction::Up
    }
}

// ============================================================================
// 2. Texture Dispatch (getBlockTextureFromSideAndMetadata)
// ============================================================================

/// Return the texture sprite index for a machine side and state, mirroring
/// `BlockMachine#getBlockTextureFromSideAndMetadata` and `RenderMachine#renderWorldBlock`.
///
/// `side` is the world direction of the face being rendered.
/// `rotation` is the internal facing direction (the rear face) stored in `TileMachine.rotation`.
pub fn get_block_texture_from_side_and_metadata(
    side: Direction,
    metadata: u8,
    rotation: Direction,
    flags: MachineFlags,
) -> Option<u32> {
    let kind = MachineKind::from_meta(metadata)?;
    let unrotated =
        kind.unrotated_texture_indices(flags.active, flags.powered, flags.delay, flags.charged);
    let oriented = orient_textures(rotation, unrotated);
    let side_idx = match side {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    };
    Some(oriented[side_idx])
}

// ============================================================================
// 3. ItemRenderMachine Rendering Helper
// ============================================================================

/// Rendering helper for inventory item models mirroring `ItemRenderMachine` and `RenderMachine.renderInvBlock`.
pub struct ItemRenderMachine;

impl ItemRenderMachine {
    /// Calculate battery box dynamic charge texture index for `damage == 6`
    /// from `ItemRenderMachine.renderItem` (`129 + bat * 8 / 6000`).
    pub fn battery_box_texture_index(charge_level: u32) -> u32 {
        let clamped = charge_level.min(6000);
        129 + (clamped * 8 / 6000)
    }

    /// Return the 6 face texture indices `[bottom, top, north, south, west, east]`
    /// for inventory display, exactly matching `RenderMachine.renderInvBlock`.
    pub fn inventory_texture_indices(
        metadata: u8,
        battery_charge: Option<u32>,
    ) -> Option<[u32; 6]> {
        match metadata {
            0 => Some([48, 53, 56, 56, 55, 55]),       // Deployer
            1 => Some([58, 57, 61, 61, 61, 61]),       // Block Breaker
            2 => Some([58, 57, 59, 59, 59, 59]),       // Transposer
            4 => Some([102, 103, 98, 98, 96, 96]),     // Item Detector
            5 => Some([113, 112, 117, 117, 117, 117]), // Sorting Machine
            6 => {
                let tx = Self::battery_box_texture_index(battery_charge.unwrap_or(0));
                Some([84, 128, tx, tx, tx, tx]) // Battery Box
            }
            8 => Some([124, 125, 120, 120, 120, 120]), // Retriever
            10 => Some([102, 103, 106, 106, 104, 104]), // Regulator
            12 => Some([48, 164, 167, 167, 166, 166]), // Igniter
            13 => Some([172, 168, 171, 171, 170, 170]), // Assembler
            14 => Some([58, 89, 91, 91, 90, 90]),      // Ejector
            15 => Some([58, 89, 93, 93, 90, 90]),      // Relay
            _ => {
                let kind = MachineKind::from_meta(metadata)?;
                Some(kind.unrotated_texture_indices(false, false, false, false))
            }
        }
    }
}

// ============================================================================
// 4. EntityPlayerFake Simulation Helper
// ============================================================================

/// Dummy item ID used to fill fake player inventory slots 9..35 mirroring `EntityPlayerFake:9`.
pub const FAKE_PLAYER_DUMMY_ITEM: &str = "minecraft:stick";

/// Simulation helper mirroring `com.eloraam.redpower.machine.EntityPlayerFake`
/// and `TileDeployBase.initPlayer`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityPlayerFake {
    pub pos: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub held_item: Option<ItemStack>,
    pub inventory: Vec<Option<ItemStack>>,
}

impl Default for EntityPlayerFake {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityPlayerFake {
    /// Create a new fake player with dummy item stacks filling inventory slots 9..35,
    /// replicating Java `EntityPlayerFake` constructor lines 8-10.
    pub fn new() -> Self {
        let mut inv: Vec<Option<ItemStack>> = vec![None; 36];
        for item in inv.iter_mut().skip(9).take(27) {
            *item = Some(ItemStack::new(FAKE_PLAYER_DUMMY_ITEM, 1));
        }
        Self {
            pos: [0.0, 0.0, 0.0],
            yaw: 0.0,
            pitch: 0.0,
            held_item: None,
            inventory: inv,
        }
    }

    /// Check if a block can be harvested by the fake player mirroring
    /// `EntityPlayerFake#canHarvestBlock`.
    /// Returns true when `hardness >= 0.0` (bedrock and unbreakable blocks have negative hardness).
    pub fn can_harvest_block(hardness: f32) -> bool {
        hardness >= 0.0
    }

    /// Configure fake player position, yaw and pitch for a machine at `block_pos` facing `rotation`,
    /// exactly mirroring `TileDeployBase.initPlayer` lines 46-90.
    pub fn init_for_machine(&mut self, block_pos: BlockPos, rotation: Direction) {
        let mut x = block_pos.0 as f64 + 0.5;
        let mut y = block_pos.1 as f64 - 1.1;
        let mut z = block_pos.2 as f64 + 0.5;
        let pitch: f32;
        let yaw: f32;

        match rotation {
            Direction::Down => {
                pitch = -90.0;
                yaw = 0.0;
                y += 0.51;
            }
            Direction::Up => {
                pitch = 90.0;
                yaw = 0.0;
                y -= 0.51;
            }
            Direction::North => {
                pitch = 0.0;
                yaw = 0.0;
                z += 0.51;
            }
            Direction::South => {
                pitch = 0.0;
                yaw = 180.0;
                z -= 0.51;
            }
            Direction::West => {
                pitch = 0.0;
                yaw = 270.0;
                x += 0.51;
            }
            Direction::East => {
                pitch = 0.0;
                yaw = 90.0;
                x -= 0.51;
            }
        }

        self.pos = [x, y, z];
        self.yaw = yaw;
        self.pitch = pitch;
    }

    /// Simulate breaking an adjacent block in front (`rotation.opposite()`).
    /// Returns the drops if harvestable, or None if unbreakable.
    pub fn simulate_break_block(
        &self,
        hardness: f32,
        drops: impl IntoIterator<Item = ItemStack>,
    ) -> Option<Vec<ItemStack>> {
        if !Self::can_harvest_block(hardness) {
            return None;
        }
        Some(drops.into_iter().collect())
    }

    /// Simulate deploying / using the currently held item on an adjacent block or entity.
    /// Decrements the held stack count on success.
    pub fn simulate_use_held_item<F>(&mut self, mut use_action: F) -> bool
    where
        F: FnMut(&mut ItemStack) -> bool,
    {
        if let Some(stack) = self.held_item.as_mut() {
            if stack.count == 0 {
                return false;
            }
            if use_action(stack) {
                if stack.count == 0 {
                    self.held_item = None;
                }
                return true;
            }
        }
        false
    }
}

// ============================================================================
// 5. Facing Helpers & Placement
// ============================================================================

/// Replicate `BlockMachine.onBlockPlacedBy`: calculates machine facing direction
/// when placed by a player in the world.
pub fn on_block_placed_by(
    player_pos: [f64; 3],
    player_yaw: f32,
    player_eye_offset: f64,
    block_pos: [i32; 3],
) -> Direction {
    TileMachine::calculate_facing(player_pos, player_yaw, player_eye_offset, block_pos)
}

/// Compute the front acting direction for a machine given its stored `rotation`
/// (which records the rear face; front is `rotation.opposite()` / `rotation ^ 1`).
pub fn machine_front_direction(rotation: Direction) -> Direction {
    rotation.opposite()
}

/// Compute the rear connection direction for a machine given its stored `rotation`.
pub fn machine_rear_direction(rotation: Direction) -> Direction {
    rotation
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_16_machine_definitions() {
        for meta in 0..16u8 {
            let def = BlockMachineRegistry::get_by_metadata(meta).expect("metadata must exist");
            assert_eq!(def.metadata, meta);
            assert_eq!(def.hardness, BLOCK_MACHINE_HARDNESS);
            assert_eq!(def.explosion_resistance, BLOCK_MACHINE_RESISTANCE);
            assert_eq!(
                BlockMachineRegistry::get_by_block_id(def.block_id),
                Some(def.clone())
            );

            // Self drop check
            let drop = BlockMachineRegistry::get_block_dropped(meta, 0).unwrap();
            assert_eq!(drop.item_id, def.block_id);
            assert_eq!(drop.count, 1);
        }
    }

    #[test]
    fn test_targeted_five_machines() {
        // Deployer (0)
        let dep = BlockMachineRegistry::get_by_metadata(0).unwrap();
        assert_eq!(dep.block_id, "redpower_2:deployer");
        assert_eq!(dep.slot_count, 9);
        assert!(dep.has_gui);

        // Block Breaker (1)
        let bb = BlockMachineRegistry::get_by_metadata(1).unwrap();
        assert_eq!(bb.block_id, "redpower_2:block_breaker");
        assert_eq!(bb.slot_count, 0);
        assert!(!bb.has_gui);

        // Transposer (2)
        let tr = BlockMachineRegistry::get_by_metadata(2).unwrap();
        assert_eq!(tr.block_id, "redpower_2:transposer");
        assert_eq!(tr.slot_count, 0);
        assert!(!tr.has_gui);

        // Igniter (12)
        let ig = BlockMachineRegistry::get_by_metadata(12).unwrap();
        assert_eq!(ig.block_id, "redpower_2:igniter");
        assert_eq!(ig.slot_count, 0);
        assert!(!ig.has_gui);

        // Relay (15)
        let rel = BlockMachineRegistry::get_by_metadata(15).unwrap();
        assert_eq!(rel.block_id, "redpower_2:relay");
        assert_eq!(rel.slot_count, 9);
        assert!(rel.has_gui);
    }

    #[test]
    fn test_facing_orientations_and_front_rear() {
        for dir in [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            assert_eq!(machine_rear_direction(dir), dir);
            assert_eq!(machine_front_direction(dir), dir.opposite());
        }

        // Test on_block_placed_by
        let block_pos = [0, 64, 0];
        let p_above = [0.5, 68.0, 0.5];
        assert_eq!(
            on_block_placed_by(p_above, 0.0, 1.62, block_pos),
            Direction::Down
        );

        let p_below = [0.5, 62.0, 0.5];
        assert_eq!(
            on_block_placed_by(p_below, 0.0, 1.62, block_pos),
            Direction::Up
        );

        let p_level = [0.5, 64.5, 0.5];
        assert_eq!(
            on_block_placed_by(p_level, 0.0, 1.62, block_pos),
            Direction::South
        );
        assert_eq!(
            on_block_placed_by(p_level, 90.0, 1.62, block_pos),
            Direction::West
        );
        assert_eq!(
            on_block_placed_by(p_level, 180.0, 1.62, block_pos),
            Direction::North
        );
        assert_eq!(
            on_block_placed_by(p_level, 270.0, 1.62, block_pos),
            Direction::East
        );
    }

    #[test]
    fn test_fire_source_igniter_only() {
        // Non-igniter machines never provide fire source
        for meta in 0..16u8 {
            if meta != 12 {
                assert!(!BlockMachineRegistry::is_fire_source(
                    meta,
                    Direction::Down,
                    true,
                    Direction::Up
                ));
            }
        }

        // Igniter: true only when active, rotation == Down, face == Up
        assert!(BlockMachineRegistry::is_fire_source(
            12,
            Direction::Down,
            true,
            Direction::Up
        ));
        assert!(!BlockMachineRegistry::is_fire_source(
            12,
            Direction::Down,
            false,
            Direction::Up
        ));
        assert!(!BlockMachineRegistry::is_fire_source(
            12,
            Direction::North,
            true,
            Direction::Up
        ));
        assert!(!BlockMachineRegistry::is_fire_source(
            12,
            Direction::Down,
            true,
            Direction::North
        ));
    }

    #[test]
    fn test_texture_dispatch_deployer_and_breaker() {
        let flags = MachineFlags::NONE;

        // Deployer (0) facing Down (0):
        // Bottom is front (48), Top is top (53), Sides are side (55)
        let tex_bottom =
            get_block_texture_from_side_and_metadata(Direction::Down, 0, Direction::Down, flags);
        assert_eq!(tex_bottom, Some(48));

        let tex_top =
            get_block_texture_from_side_and_metadata(Direction::Up, 0, Direction::Down, flags);
        assert_eq!(tex_top, Some(53));

        // When active, top changes from 53 to 54
        let mut active_flags = MachineFlags::NONE;
        active_flags.active = true;
        let tex_top_active = get_block_texture_from_side_and_metadata(
            Direction::Up,
            0,
            Direction::Down,
            active_flags,
        );
        assert_eq!(tex_top_active, Some(54));
    }

    #[test]
    fn test_item_render_machine_and_battery_box() {
        // Battery box charge formulas: 129 + bat * 8 / 6000
        assert_eq!(ItemRenderMachine::battery_box_texture_index(0), 129);
        assert_eq!(ItemRenderMachine::battery_box_texture_index(3000), 133);
        assert_eq!(ItemRenderMachine::battery_box_texture_index(6000), 137);

        // Inventory textures for deployer (0), breaker (1), transposer (2), igniter (12), relay (15)
        assert_eq!(
            ItemRenderMachine::inventory_texture_indices(0, None),
            Some([48, 53, 56, 56, 55, 55])
        );
        assert_eq!(
            ItemRenderMachine::inventory_texture_indices(1, None),
            Some([58, 57, 61, 61, 61, 61])
        );
        assert_eq!(
            ItemRenderMachine::inventory_texture_indices(2, None),
            Some([58, 57, 59, 59, 59, 59])
        );
        assert_eq!(
            ItemRenderMachine::inventory_texture_indices(12, None),
            Some([48, 164, 167, 167, 166, 166])
        );
        assert_eq!(
            ItemRenderMachine::inventory_texture_indices(15, None),
            Some([58, 89, 93, 93, 90, 90])
        );
    }

    #[test]
    fn test_fake_player_simulation_helper() {
        let mut fake = EntityPlayerFake::new();
        // Check dummy item slots 9..35
        for i in 0..9 {
            assert!(fake.inventory[i].is_none());
        }
        for i in 9..36 {
            assert_eq!(
                fake.inventory[i].as_ref().map(|s| s.item_id.as_str()),
                Some(FAKE_PLAYER_DUMMY_ITEM)
            );
        }

        // Harvesting bedrock (hardness -1.0) is disallowed
        assert!(!EntityPlayerFake::can_harvest_block(-1.0));
        assert!(EntityPlayerFake::can_harvest_block(0.0));
        assert!(EntityPlayerFake::can_harvest_block(2.0));
        assert!(fake
            .simulate_break_block(-1.0, vec![ItemStack::new("minecraft:bedrock", 1)])
            .is_none());

        let stone_drops =
            fake.simulate_break_block(1.5, vec![ItemStack::new("minecraft:cobblestone", 1)]);
        assert_eq!(stone_drops.unwrap()[0].item_id, "minecraft:cobblestone");

        // Pitch & yaw setup for machine
        fake.init_for_machine(BlockPos(10, 64, 10), Direction::North);
        assert_eq!(fake.pitch, 0.0);
        assert_eq!(fake.yaw, 0.0);
        assert_eq!(fake.pos, [10.5, 62.9, 11.01]);

        fake.init_for_machine(BlockPos(10, 64, 10), Direction::Down);
        assert_eq!(fake.pitch, -90.0);
        assert_eq!(fake.yaw, 0.0);

        // Held item usage
        fake.held_item = Some(ItemStack::new("minecraft:wheat_seeds", 3));
        let used = fake.simulate_use_held_item(|item| {
            item.count -= 1;
            true
        });
        assert!(used);
        assert_eq!(fake.held_item.as_ref().unwrap().count, 2);
    }
}

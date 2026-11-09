//! RedPower 2 Machine Panel blocks (`BlockMachinePanel` meta 0..=4 and `ItemMachinePanel`).
//!
//! Mirrors:
//! - `com.eloraam.redpower.machine.BlockMachinePanel`:
//!   - Metadata 0: Solar Panel (`redpower_2:solar_panel`)
//!   - Metadata 1: Pump (`redpower_2:pump`)
//!   - Metadata 2: Accelerator (`redpower_2:accelerator`)
//!   - Metadata 3: Grate (`redpower_2:grate`)
//!   - Metadata 4: Transformer (`redpower_2:transformer`)
//!   - Material rock (`agi.e`), hardness 2.0F (`this.c(2.0F)`), creative tab Machine (`CreativeExtraTabs.tabMachine`)
//! - `com.eloraam.redpower.machine.ItemMachinePanel`:
//!   - Placement logic (`ItemMachinePanel#a`), orientation calculation (`TileMachinePanel#getFacing` / `onBlockPlaced`),
//!     and placement surface attachment validation (e.g. Solar Panel requiring solid support underneath:
//!     `ist.j() == 0 && !world.isBlockSolidOnSide(x, y - 1, z, ForgeDirection.UP)`).
//! - Shapes and Hitboxes:
//!   - Solar Panel: 1/4 slab height `[0.0, 0.0, 0.0, 1.0, 0.25, 1.0]`.
//!   - Pump, Accelerator, Grate, Transformer: full block `[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]`.

use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};

/// Machine panel block hardness constant mirroring `BlockMachinePanel:10` (`this.c(2.0F)`).
pub const BLOCK_MACHINE_PANEL_HARDNESS: f32 = 2.0;

/// Machine panel blast resistance.
pub const BLOCK_MACHINE_PANEL_RESISTANCE: f32 = 10.0;

/// Machine panel harvest tool requirement.
pub const BLOCK_MACHINE_PANEL_TOOL: &str = "minecraft:mineable/pickaxe";

/// Enumeration of all Machine Panel kinds (meta 0..=4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MachinePanelKind {
    /// Meta 0: Solar Panel (1/4 slab height, blutricity generator in daylight)
    SolarPanel = 0,
    /// Meta 1: Fluid Pump (moves fluids through pipes consuming blutricity)
    Pump = 1,
    /// Meta 2: Magtube Accelerator (accelerates items into magtubes consuming blutricity)
    Accelerator = 2,
    /// Meta 3: Fluid Grate (sucks/pours fluid blocks to/from world)
    Grate = 3,
    /// Meta 4: Transformer (steps voltage between 100V low voltage and 10000V high voltage)
    Transformer = 4,
}

impl MachinePanelKind {
    /// Convert metadata (0..=4) to `MachinePanelKind`.
    pub const fn from_meta(meta: u8) -> Option<Self> {
        match meta {
            0 => Some(Self::SolarPanel),
            1 => Some(Self::Pump),
            2 => Some(Self::Accelerator),
            3 => Some(Self::Grate),
            4 => Some(Self::Transformer),
            _ => None,
        }
    }

    /// Return metadata `0..=4`.
    pub const fn to_meta(self) -> u8 {
        self as u8
    }

    /// Return the canonical block ID string.
    pub const fn block_id(self) -> &'static str {
        match self {
            Self::SolarPanel => "redpower_2:solar_panel",
            Self::Pump => "redpower_2:pump",
            Self::Accelerator => "redpower_2:accelerator",
            Self::Grate => "redpower_2:grate",
            Self::Transformer => "redpower_2:transformer",
        }
    }

    /// Parse from block ID string.
    pub fn from_block_id(id: &str) -> Option<Self> {
        match id {
            "redpower_2:solar_panel" | "solar_panel" => Some(Self::SolarPanel),
            "redpower_2:pump" | "pump" => Some(Self::Pump),
            "redpower_2:accelerator" | "accelerator" => Some(Self::Accelerator),
            "redpower_2:grate" | "grate" => Some(Self::Grate),
            "redpower_2:transformer" | "transformer" => Some(Self::Transformer),
            _ => None,
        }
    }

    /// Return the unlocalized name.
    pub const fn unlocalized_name(self) -> &'static str {
        match self {
            Self::SolarPanel => "tile.rppanel.solar",
            Self::Pump => "tile.rppanel.pump",
            Self::Accelerator => "tile.rppanel.accel",
            Self::Grate => "tile.rppanel.grate",
            Self::Transformer => "tile.rppanel.transformer",
        }
    }

    /// Return the display name.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::SolarPanel => "Solar Panel",
            Self::Pump => "Pump",
            Self::Accelerator => "Accelerator",
            Self::Grate => "Grate",
            Self::Transformer => "Transformer",
        }
    }

    /// Return whether this machine panel is a 1/4 slab (Solar Panel).
    pub const fn is_quarter_slab(self) -> bool {
        matches!(self, Self::SolarPanel)
    }

    /// Return the AABB bounding box `[min_x, min_y, min_z, max_x, max_y, max_z]`.
    pub const fn bounding_box(self) -> [f64; 6] {
        match self {
            Self::SolarPanel => [0.0, 0.0, 0.0, 1.0, 0.25, 1.0],
            Self::Pump | Self::Accelerator | Self::Grate | Self::Transformer => {
                [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]
            }
        }
    }

    /// Check if placement is allowed based on support requirements.
    ///
    /// Solar Panel (meta 0) requires solid top surface on the block below:
    /// `ist.j() == 0 && !world.isBlockSolidOnSide(x, y - 1, z, ForgeDirection.UP) -> false`
    pub fn can_place_at(self, solid_below: bool) -> bool {
        if self == Self::SolarPanel {
            solid_below
        } else {
            true
        }
    }

    /// Calculate block placement orientation given player yaw, pitch, and relative position.
    ///
    /// Mirrors original Java `onBlockPlaced` implementations:
    /// - Solar Panel: `(floor(yaw * 4 / 360 + 0.5)) & 3`
    /// - Pump: `(floor(yaw * 4 / 360 + 2.5)) & 3`
    /// - Accelerator / Grate: `getFacing(ent)`
    /// - Transformer: 6-directional side + rot
    pub fn calculate_placement_rotation(
        self,
        yaw: f32,
        pitch: f32,
        player_offset: (f64, f64, f64),
        side_clicked: u8,
    ) -> u8 {
        match self {
            Self::SolarPanel => {
                let yawrx = ((yaw * 4.0 / 360.0 + 0.5).floor() as i32) & 3;
                yawrx as u8
            }
            Self::Pump => {
                let rot = ((yaw * 4.0 / 360.0 + 2.5).floor() as i32) & 3;
                rot as u8
            }
            Self::Accelerator | Self::Grate => get_facing(yaw, pitch, player_offset),
            Self::Transformer => {
                let down = (side_clicked ^ 1) as i32;
                let yaw_quad = ((yaw / 90.0 + 0.5).floor() as i32) & 3;
                let pitch_quad = (pitch / 90.0 + 0.5).floor() as i32;
                let rot = match down {
                    0 => yaw_quad,
                    1 => yaw_quad ^ ((yaw_quad & 1) << 1),
                    2 => {
                        if (yaw_quad & 1) > 0 {
                            if pitch_quad > 0 {
                                2
                            } else {
                                0
                            }
                        } else {
                            (1 - yaw_quad) & 3
                        }
                    }
                    3 => {
                        if (yaw_quad & 1) > 0 {
                            if pitch_quad > 0 {
                                2
                            } else {
                                0
                            }
                        } else {
                            (yaw_quad - 1) & 3
                        }
                    }
                    4 => {
                        if (yaw_quad & 1) == 0 {
                            if pitch_quad > 0 {
                                2
                            } else {
                                0
                            }
                        } else {
                            (yaw_quad - 2) & 3
                        }
                    }
                    5 => {
                        if (yaw_quad & 1) == 0 {
                            if pitch_quad > 0 {
                                2
                            } else {
                                0
                            }
                        } else {
                            (2 - yaw_quad) & 3
                        }
                    }
                    _ => 0,
                };
                ((down << 2) | (rot & 3)) as u8
            }
        }
    }
}

/// Compute 6-way facing direction mirroring `TileMachinePanel#getFacing(EntityLiving ent)`.
///
/// Java source:
/// ```java
/// int yawrx = (int)Math.floor(ent.rotationYaw * 4.0F / 360.0F + 0.5) & 3;
/// if (Math.abs(ent.posX - this.xCoord) < 2.0 && Math.abs(ent.posZ - this.zCoord) < 2.0) {
///     double p = ent.posY + 1.82 - ent.yOffset - this.yCoord;
///     if (p > 2.0) return 0;
///     if (p < 0.0) return 1;
/// }
/// switch (yawrx) {
///     case 0: return 3; // North
///     case 1: return 4; // East
///     case 2: return 2; // South
///     default: return 5; // West
/// }
/// ```
pub fn get_facing(yaw: f32, _pitch: f32, player_offset: (f64, f64, f64)) -> u8 {
    let (dx, dy, dz) = player_offset;
    if dx.abs() < 2.0 && dz.abs() < 2.0 {
        if dy > 2.0 {
            return 0; // Down
        }
        if dy < 0.0 {
            return 1; // Up
        }
    }

    let yawrx = ((yaw * 4.0 / 360.0 + 0.5).floor() as i32) & 3;
    match yawrx {
        0 => 3, // North
        1 => 4, // East
        2 => 2, // South
        _ => 5, // West
    }
}

/// Helper mapping an orientation index to a `rekkit_core::side::Direction`.
pub fn orientation_to_direction(side: u8) -> Direction {
    match side {
        0 => Direction::Down,
        1 => Direction::Up,
        2 => Direction::North,
        3 => Direction::South,
        4 => Direction::West,
        5 => Direction::East,
        _ => Direction::North,
    }
}

/// Helper mapping a `Direction` to orientation index `0..=5`.
pub fn direction_to_orientation(dir: Direction) -> u8 {
    match dir {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_panel_kinds_and_meta() {
        for meta in 0..=4 {
            let kind = MachinePanelKind::from_meta(meta).expect("valid meta");
            assert_eq!(kind.to_meta(), meta);
            assert_eq!(MachinePanelKind::from_block_id(kind.block_id()), Some(kind));
        }
        assert!(MachinePanelKind::from_meta(5).is_none());
    }

    #[test]
    fn test_solar_panel_shape_and_placement() {
        let solar = MachinePanelKind::SolarPanel;
        assert!(solar.is_quarter_slab());
        assert_eq!(solar.bounding_box(), [0.0, 0.0, 0.0, 1.0, 0.25, 1.0]);

        // Support check: Solar Panel requires solid surface below
        assert!(!solar.can_place_at(false));
        assert!(solar.can_place_at(true));

        // Other panels do not require solid surface below
        assert!(MachinePanelKind::Pump.can_place_at(false));
        assert!(MachinePanelKind::Accelerator.can_place_at(false));
        assert!(MachinePanelKind::Grate.can_place_at(false));
        assert!(MachinePanelKind::Transformer.can_place_at(false));
    }

    #[test]
    fn test_full_block_shapes() {
        for kind in [
            MachinePanelKind::Pump,
            MachinePanelKind::Accelerator,
            MachinePanelKind::Grate,
            MachinePanelKind::Transformer,
        ] {
            assert!(!kind.is_quarter_slab());
            assert_eq!(kind.bounding_box(), [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        }
    }

    #[test]
    fn test_get_facing_vertical_and_horizontal() {
        // Looking down from above: dy > 2.0 -> Down (0)
        let facing_down = get_facing(0.0, 90.0, (0.0, 2.5, 0.0));
        assert_eq!(facing_down, 0);

        // Looking up from below: dy < 0.0 -> Up (1)
        let facing_up = get_facing(0.0, -90.0, (0.0, -0.5, 0.0));
        assert_eq!(facing_up, 1);

        // Horizontal tests (far away vertically so horizontal branch triggers)
        // yaw ~ 0 -> North (3)
        assert_eq!(get_facing(0.0, 0.0, (5.0, 1.0, 5.0)), 3);
        // yaw ~ 90 -> East (4)
        assert_eq!(get_facing(90.0, 0.0, (5.0, 1.0, 5.0)), 4);
        // yaw ~ 180 -> South (2)
        assert_eq!(get_facing(180.0, 0.0, (5.0, 1.0, 5.0)), 2);
        // yaw ~ 270 -> West (5)
        assert_eq!(get_facing(270.0, 0.0, (5.0, 1.0, 5.0)), 5);
    }

    #[test]
    fn test_placement_rotation_for_each_machine_panel() {
        // Solar panel rotation
        let rot_solar = MachinePanelKind::SolarPanel.calculate_placement_rotation(
            90.0,
            0.0,
            (0.0, 0.0, 0.0),
            1,
        );
        assert_eq!(rot_solar, 1);

        // Pump rotation (+2.5 yaw offset)
        let rot_pump =
            MachinePanelKind::Pump.calculate_placement_rotation(0.0, 0.0, (0.0, 0.0, 0.0), 1);
        assert_eq!(rot_pump, 2);

        // Grate / Accelerator facing
        let rot_accel = MachinePanelKind::Accelerator.calculate_placement_rotation(
            0.0,
            0.0,
            (5.0, 1.0, 5.0),
            1,
        );
        assert_eq!(rot_accel, 3);

        // Transformer placement
        let rot_trans = MachinePanelKind::Transformer.calculate_placement_rotation(
            0.0,
            0.0,
            (0.0, 0.0, 0.0),
            1,
        );
        // side 1 ^ 1 = 0 (down = 0), yaw 0 -> rot = 0
        assert_eq!(rot_trans, 0);
    }
}

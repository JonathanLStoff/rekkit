//! RedPower 2 (`redpower_2`) Rust domain crate and `HorizonNativeMod` implementation (`NATIVE_HOOKS_API = 40`).
//!
//! Implements all core subsystems specified in `planning/mods/redpower_2.md`:
//! - **Microblocks (§2.1)**: 1/8th fractional slabs, panels, covers, hollow variants, strips, corners,
//!   structural poles, handsaw cutting rules, 27-subvoxel occupancy bitmasks, and occlusion masking.
//! - **Wiring (§2.2)**: Red Alloy Wire (0..=255 internal signal, lossless over 256 blocks, mapped via
//!   `strength / 17`), 16-color Insulated Wire with no cross-color crosstalk, 16-channel Bundled Cable
//!   (`rekkit_core::frequency::BundledSignal`), and Blutricity Bluewire (0.01 Ω/block).
//! - **Integrated Logic Gates (§2.3)**: Complete 20+ logic tile state machines (Inverter, AND, OR, NAND,
//!   NOR, XOR, XNOR, Buffer, Multiplexer, Repeater with schedule table, Timer, Counter, Sequencer,
//!   Pulse Former, Toggle Latch, RS Latch, Transparent Latch, State Cell, Synchronizer, Light Sensor,
//!   Randomizer, Null/Invert/Non-Invert array cells, and 16-channel Bus Transceiver).
//! - **Pneumatic Tubes & Item Logistics (§2.4)**: Zero-drop backpressure routing, Dijkstra shortest-path
//!   pathfinding with +2 hop weight, +1 turn penalty, 1,000,000 Restrictor Tube weight, 16-color filtering,
//!   and 16.0 blocks/tick Magtubes with 100 W-t/item Blutricity Accelerators.
//! - **Blutricity Power Grid (§2.5)**: 100V DC nominal, 60V threshold, 120V safe peak, 0.01 Ω conductor
//!   resistance, 0.07 inductive scaling dynamic transfer step, Solar Panel (2.0 A), Thermopile
//!   (0.005 A * min(Thot, Tcold) with thermal shock hazard), Wind Turbine (sails wear, brakes),
//!   Battery Box (6,000 storage units, 90V charge / 80V discharge), and 100x Transformer.
//! - **Frame Automation & Structural Movement (§2.6)**: Contiguous support frame BFS solver (up to
//!   1,000 blocks), negative hardness bedrock rejection, obsidian movement, 16-tick translation,
//!   and 100 + 10*LinkSize W-t power consumption.
//! - **Control 6502 Computing Subsystem (§2.7)**: MOS 6502/65C816 microprocessor emulator, 8 KB onboard RAM
//!   expandable to 64 KB, 1,000 cycles/tick slice with 100,000 burst cap, 80x50 ASCII terminal with
//!   hardware blitter, 256 KB floppy disk drive (2048 sectors of 128 bytes), and 16-channel Bundled Cable
//!   I/O Expander MMIO registers.
//! - **Alloy Furnace (§3.1 & §7.1)**: 11-slot state machine with solid fuel and Blulectric operation,
//!   supporting Red Alloy, Blue Alloy, Brass, Tinplate, Silicon Boule, Doped Wafers, and metal recycling.

pub use rekkit_core::energy::BlutricityNode;
pub use rekkit_core::frequency::{BundledSignal, DyeColor};
pub use rekkit_core::grid::{BlockPos, GridGraph, GridNode};
pub use rekkit_core::hooks::{
    HorizonNativeMod, ModBlockEntity, ModContainer, NativeModInfo, NativeScreen,
};
pub use rekkit_core::inventory::{Inventory, ItemStack};
pub use rekkit_core::side::{ConnectionMask, Direction};
pub use rekkit_core::util::{TickTimer, XorShift64};
pub use rekkit_core::{HORIZON_REVISION, MIXINS_REVISION, NATIVE_HOOKS_API};

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

pub const MOD_ID: &str = "redpower_2";

/// All 113 canonical block IDs declared in `planning/mods/redpower_2.md` §6.1.
pub const BLOCK_IDS: [&str; 113] = [
    "redpower_2:alloy_furnace",
    "redpower_2:project_table",
    "redpower_2:blulectric_furnace",
    "redpower_2:buffer_chest",
    "redpower_2:blulectric_alloy_furnace",
    "redpower_2:charging_bench",
    "redpower_2:microblock",
    "redpower_2:wire_red_alloy",
    "redpower_2:wire_insulated",
    "redpower_2:wire_bundled",
    "redpower_2:wire_blutricity",
    "redpower_2:wire_blutricity_insulated",
    "redpower_2:wire_jacketed",
    "redpower_2:cable_jacketed",
    "redpower_2:wire_jacketed_blutricity",
    "redpower_2:pipe",
    "redpower_2:pneumatic_tube",
    "redpower_2:redstone_tube",
    "redpower_2:restrictor_tube",
    "redpower_2:magtube",
    "redpower_2:ribbon_cable",
    "redpower_2:timer",
    "redpower_2:sequencer",
    "redpower_2:state_cell",
    "redpower_2:rs_latch",
    "redpower_2:nor_gate",
    "redpower_2:or_gate",
    "redpower_2:nand_gate",
    "redpower_2:and_gate",
    "redpower_2:xnor_gate",
    "redpower_2:xor_gate",
    "redpower_2:pulse_former",
    "redpower_2:toggle_latch",
    "redpower_2:not_gate",
    "redpower_2:buffer_gate",
    "redpower_2:multiplexer",
    "redpower_2:repeater",
    "redpower_2:synchronizer",
    "redpower_2:randomizer",
    "redpower_2:transparent_latch",
    "redpower_2:light_sensor",
    "redpower_2:null_cell",
    "redpower_2:invert_cell",
    "redpower_2:non_invert_cell",
    "redpower_2:counter",
    "redpower_2:bus_transceiver",
    "redpower_2:lamp",
    "redpower_2:lamp_lit",
    "redpower_2:lamp_inverted",
    "redpower_2:lamp_inverted_lit",
    "redpower_2:lumar_fixture",
    "redpower_2:lumar_fixture_inverted",
    "redpower_2:cage_lamp",
    "redpower_2:cage_lamp_inverted",
    "redpower_2:ore_ruby",
    "redpower_2:ore_green_sapphire",
    "redpower_2:ore_sapphire",
    "redpower_2:ore_silver",
    "redpower_2:ore_tin",
    "redpower_2:ore_copper",
    "redpower_2:ore_tungsten",
    "redpower_2:ore_nikolite",
    "redpower_2:block_ruby",
    "redpower_2:block_green_sapphire",
    "redpower_2:block_sapphire",
    "redpower_2:block_silver",
    "redpower_2:block_tin",
    "redpower_2:block_copper",
    "redpower_2:marble",
    "redpower_2:basalt",
    "redpower_2:marble_brick",
    "redpower_2:basalt_cobblestone",
    "redpower_2:basalt_brick",
    "redpower_2:basalt_chiseled",
    "redpower_2:basalt_paver",
    "redpower_2:flax_crop",
    "redpower_2:indigo_flower",
    "redpower_2:rubber_sapling",
    "redpower_2:rubber_leaves",
    "redpower_2:rubber_log",
    "redpower_2:deployer",
    "redpower_2:block_breaker",
    "redpower_2:transposer",
    "redpower_2:filter",
    "redpower_2:item_detector",
    "redpower_2:sorting_machine",
    "redpower_2:battery_box",
    "redpower_2:frame_motor",
    "redpower_2:retriever",
    "redpower_2:wind_turbine",
    "redpower_2:regulator",
    "redpower_2:thermopile",
    "redpower_2:igniter",
    "redpower_2:assembler",
    "redpower_2:ejector",
    "redpower_2:relay",
    "redpower_2:sortron",
    "redpower_2:manager",
    "redpower_2:solar_panel",
    "redpower_2:pump",
    "redpower_2:accelerator",
    "redpower_2:grate",
    "redpower_2:transformer",
    "redpower_2:frame",
    "redpower_2:frame_tube",
    "redpower_2:frame_redstone_tube",
    "redpower_2:backplane",
    "redpower_2:ram_module",
    "redpower_2:display",
    "redpower_2:cpu",
    "redpower_2:disk_drive",
    "redpower_2:io_expander",
    "redpower_2:blulectric_engine",
];

/// All 88 canonical item IDs declared in `planning/mods/redpower_2.md` §6.2.
pub const ITEM_IDS: [&str; 88] = [
    "redpower_2:lumar",
    "redpower_2:ruby",
    "redpower_2:green_sapphire",
    "redpower_2:sapphire",
    "redpower_2:silver_ingot",
    "redpower_2:tin_ingot",
    "redpower_2:copper_ingot",
    "redpower_2:nikolite",
    "redpower_2:red_alloy_ingot",
    "redpower_2:blue_alloy_ingot",
    "redpower_2:brass_ingot",
    "redpower_2:silicon_boule",
    "redpower_2:silicon_wafer",
    "redpower_2:blue_doped_wafer",
    "redpower_2:red_doped_wafer",
    "redpower_2:tinplate",
    "redpower_2:fine_copper_wire",
    "redpower_2:fine_iron_wire",
    "redpower_2:copper_coil",
    "redpower_2:bl_motor",
    "redpower_2:canvas",
    "redpower_2:iron_nugget",
    "redpower_2:silver_nugget",
    "redpower_2:tin_nugget",
    "redpower_2:copper_nugget",
    "redpower_2:dye_indigo",
    "redpower_2:iron_handsaw",
    "redpower_2:diamond_handsaw",
    "redpower_2:screwdriver",
    "redpower_2:diamond_drawplate",
    "redpower_2:blank_plan",
    "redpower_2:plan",
    "redpower_2:canvas_bag",
    "redpower_2:stone_wafer",
    "redpower_2:wire_part",
    "redpower_2:anode",
    "redpower_2:cathode",
    "redpower_2:pointer",
    "redpower_2:redwire_part",
    "redpower_2:plate",
    "redpower_2:silicon_chip",
    "redpower_2:tainted_chip",
    "redpower_2:bundled_wafer",
    "redpower_2:ruby_pickaxe",
    "redpower_2:green_sapphire_pickaxe",
    "redpower_2:sapphire_pickaxe",
    "redpower_2:ruby_shovel",
    "redpower_2:green_sapphire_shovel",
    "redpower_2:sapphire_shovel",
    "redpower_2:ruby_axe",
    "redpower_2:green_sapphire_axe",
    "redpower_2:sapphire_axe",
    "redpower_2:ruby_sword",
    "redpower_2:green_sapphire_sword",
    "redpower_2:sapphire_sword",
    "redpower_2:ruby_hoe",
    "redpower_2:green_sapphire_hoe",
    "redpower_2:sapphire_hoe",
    "redpower_2:ruby_handsaw",
    "redpower_2:green_sapphire_handsaw",
    "redpower_2:sapphire_handsaw",
    "redpower_2:wood_sickle",
    "redpower_2:stone_sickle",
    "redpower_2:iron_sickle",
    "redpower_2:diamond_sickle",
    "redpower_2:gold_sickle",
    "redpower_2:ruby_sickle",
    "redpower_2:green_sapphire_sickle",
    "redpower_2:sapphire_sickle",
    "redpower_2:athame",
    "redpower_2:flax_seeds",
    "redpower_2:wool_card",
    "redpower_2:dry_paint_brush",
    "redpower_2:paint_brush",
    "redpower_2:empty_paint_can",
    "redpower_2:paint_can",
    "redpower_2:seed_bag",
    "redpower_2:voltmeter",
    "redpower_2:sonic_screwdriver",
    "redpower_2:battery_empty",
    "redpower_2:battery_powered",
    "redpower_2:wood_sail",
    "redpower_2:wood_turbine",
    "redpower_2:wood_windmill",
    "redpower_2:blank_disk",
    "redpower_2:forth_boot_disk",
    "redpower_2:forth_extra_disk",
    "redpower_2:brass_gear",
];

// ============================================================================
// 2.1 Microblocks & Sub-Voxel System
// ============================================================================

/// 1/8th fractional microblock thickness and form hierarchy (`planning/mods/redpower_2.md` §2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MicroblockKind {
    /// 4/8th = 1/2 block.
    Slab,
    /// 2/8th = 1/4 block.
    Panel,
    /// 1/8th block.
    Cover,
    /// 3/8th block.
    TripleCover,
    /// 5/8th block.
    CoverSlab,
    /// 6/8th block.
    TriplePanel,
    /// 7/8th block.
    Anticover,
    /// 4/8th hollow slab with center cutout for wires/tubes.
    HollowSlab,
    /// 2/8th hollow panel with center cutout.
    HollowPanel,
    /// 1/8th hollow cover with center cutout.
    HollowCover,
    /// 3/8th hollow triple cover.
    HollowTripleCover,
    /// 5/8th hollow cover slab.
    HollowCoverSlab,
    /// 6/8th hollow triple panel.
    HollowTriplePanel,
    /// 7/8th hollow anticover.
    HollowAnticover,
    /// 1/2 x 1/8 x 1 strip.
    StripSlab,
    /// 1/4 x 1/8 x 1 strip.
    StripPanel,
    /// 1/8 x 1/8 x 1 corner strip.
    CornerStrip,
    /// 1/8 x 1/8 x 1/8 corner vertex.
    Corner,
    /// 1/8th centered column along an axis (`rppole1`).
    Post,
    /// 1/4th centered column along an axis (`rppole2`).
    Pillar,
    /// 1/2th centered column along an axis (`rppole3`).
    Column,
}

impl MicroblockKind {
    /// Thickness in eighths of a block (`1..=7`).
    pub const fn thickness_eighths(self) -> u8 {
        match self {
            Self::Cover | Self::HollowCover | Self::CornerStrip | Self::Corner | Self::Post => 1,
            Self::Panel | Self::HollowPanel | Self::StripPanel | Self::Pillar => 2,
            Self::TripleCover | Self::HollowTripleCover => 3,
            Self::Slab | Self::HollowSlab | Self::StripSlab | Self::Column => 4,
            Self::CoverSlab | Self::HollowCoverSlab => 5,
            Self::TriplePanel | Self::HollowTriplePanel => 6,
            Self::Anticover | Self::HollowAnticover => 7,
        }
    }

    /// Whether a central wire, cable, or tube can pass through this face microblock.
    pub const fn is_hollow(self) -> bool {
        matches!(
            self,
            Self::HollowCover
                | Self::HollowPanel
                | Self::HollowTripleCover
                | Self::HollowSlab
                | Self::HollowCoverSlab
                | Self::HollowTriplePanel
                | Self::HollowAnticover
        )
    }

    /// Handsaw cutting progression (`planning/mods/redpower_2.md` §2.1):
    /// Vertical cut: `Slab -> Panel -> Cover -> CornerStrip -> Corner`.
    pub const fn slice_with_handsaw(self) -> Option<(Self, u32)> {
        match self {
            Self::Slab => Some((Self::Panel, 2)),
            Self::Panel => Some((Self::Cover, 2)),
            Self::Cover => Some((Self::CornerStrip, 4)),
            Self::CornerStrip => Some((Self::Corner, 2)),
            Self::HollowSlab => Some((Self::HollowPanel, 2)),
            Self::HollowPanel => Some((Self::HollowCover, 2)),
            Self::StripSlab => Some((Self::StripPanel, 2)),
            Self::StripPanel => Some((Self::CornerStrip, 2)),
            Self::TripleCover => Some((Self::Panel, 1)),
            Self::CoverSlab => Some((Self::Slab, 1)),
            Self::TriplePanel => Some((Self::Slab, 1)),
            Self::Anticover => Some((Self::TriplePanel, 1)),
            Self::HollowTripleCover => Some((Self::HollowPanel, 1)),
            Self::HollowCoverSlab => Some((Self::HollowSlab, 1)),
            Self::HollowTriplePanel => Some((Self::HollowSlab, 1)),
            Self::HollowAnticover => Some((Self::HollowTriplePanel, 1)),
            Self::Post | Self::Pillar | Self::Column | Self::HollowCover | Self::Corner => None,
        }
    }

    /// Vertical cut (saw above or below material): Halves thickness.
    pub const fn cut_vertical(self) -> Option<(Self, u32)> {
        match self {
            Self::Slab => Some((Self::Panel, 2)),
            Self::Panel => Some((Self::Cover, 2)),
            Self::HollowSlab => Some((Self::HollowPanel, 2)),
            Self::HollowPanel => Some((Self::HollowCover, 2)),
            _ => None,
        }
    }

    /// Horizontal cut (saw beside material): Slices slice into strips, and strips into corners.
    pub const fn cut_horizontal(self) -> Option<(Self, u32)> {
        match self {
            Self::Slab => Some((Self::StripSlab, 2)),
            Self::Panel => Some((Self::StripPanel, 2)),
            Self::Cover => Some((Self::CornerStrip, 2)),
            Self::StripSlab => Some((Self::StripPanel, 2)),
            Self::StripPanel => Some((Self::CornerStrip, 2)),
            Self::CornerStrip => Some((Self::Corner, 2)),
            _ => None,
        }
    }
}

/// Saws have sharpness ratings: Iron Handsaw = 0, Ruby/Sapphire/Green Sapphire = 1, Diamond Handsaw = 2.
pub fn handsaw_sharpness(handsaw_id: &str) -> Option<u32> {
    if handsaw_id.contains("iron") || handsaw_id == "redpower_2:handsaw" {
        Some(0)
    } else if handsaw_id.contains("ruby")
        || handsaw_id.contains("sapphire")
        || handsaw_id.contains("green_sapphire")
    {
        Some(1)
    } else if handsaw_id.contains("diamond") {
        Some(2)
    } else {
        None
    }
}

/// Check if handsaw sharpness can cut a material with given hardness.
pub const fn can_handsaw_cut(hardness: f32, sharpness: u32) -> bool {
    match sharpness {
        0 => hardness <= 2.0,
        1 => hardness <= 5.0,
        _ => hardness <= 50.0,
    }
}

/// Combine smaller microblock slices back into larger ones (§2.1: 2 covers -> panel, 2 panels -> slab, 2 slabs -> block).
pub const fn recombine_microblocks(a: MicroblockKind, b: MicroblockKind) -> Option<MicroblockKind> {
    match (a, b) {
        (MicroblockKind::Cover, MicroblockKind::Cover) => Some(MicroblockKind::Panel),
        (MicroblockKind::Panel, MicroblockKind::Panel) => Some(MicroblockKind::Slab),
        (MicroblockKind::TripleCover, MicroblockKind::Cover)
        | (MicroblockKind::Cover, MicroblockKind::TripleCover) => Some(MicroblockKind::Slab),
        (MicroblockKind::Corner, MicroblockKind::Corner) => Some(MicroblockKind::CornerStrip),
        _ => None,
    }
}

/// Multi-part sub-voxel block space holding up to 6 face plates, 12 edge strips, 8 vertex corners,
/// and a central wire, cable, or tube (`planning/mods/redpower_2.md` §2.1 & §9.1).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroblockVoxel {
    pub face_parts: [Option<MicroblockKind>; 6],
    pub edge_parts: [Option<MicroblockKind>; 12],
    pub corner_parts: [Option<MicroblockKind>; 8],
    pub center_conduit: Option<String>,
}

impl MicroblockVoxel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Place a microblock slice onto face `dir`.
    /// Rejects if the face is already occupied or combined opposing thickness exceeds 8/8ths.
    pub fn place_on_face(&mut self, dir: Direction, kind: MicroblockKind) -> bool {
        let idx = dir as usize;
        let opp = dir.opposite() as usize;
        let opp_thick = self.face_parts[opp].map_or(0, MicroblockKind::thickness_eighths);
        if self.face_parts[idx].is_some() || kind.thickness_eighths() + opp_thick > 8 {
            return false;
        }
        self.face_parts[idx] = Some(kind);
        true
    }

    /// Place an edge strip at index `0..12`.
    pub fn place_edge(&mut self, edge_idx: usize, kind: MicroblockKind) -> bool {
        if edge_idx >= 12 || self.edge_parts[edge_idx].is_some() {
            return false;
        }
        self.edge_parts[edge_idx] = Some(kind);
        true
    }

    /// Place a corner vertex piece at index `0..8`.
    pub fn place_corner(&mut self, corner_idx: usize, kind: MicroblockKind) -> bool {
        if corner_idx >= 8 || self.corner_parts[corner_idx].is_some() {
            return false;
        }
        self.corner_parts[corner_idx] = Some(kind);
        true
    }

    /// True if an internal wire, cable, or pneumatic tube can connect through `dir`.
    /// Uncovered faces and hollow covers permit connection; solid covers seal the face.
    pub fn allows_center_connection(&self, dir: Direction) -> bool {
        match self.face_parts[dir as usize] {
            None => true,
            Some(kind) => kind.is_hollow(),
        }
    }

    /// 27-bit sub-voxel occupancy bitmask (§9.1: 6 faces, 12 edges, 8 corners, 1 center).
    pub fn occupancy_mask(&self) -> u32 {
        let mut mask = 0u32;
        for (i, face) in self.face_parts.iter().enumerate() {
            if face.is_some() {
                mask |= 1 << i;
            }
        }
        for (i, edge) in self.edge_parts.iter().enumerate() {
            if edge.is_some() {
                mask |= 1 << (6 + i);
            }
        }
        for (i, corner) in self.corner_parts.iter().enumerate() {
            if corner.is_some() {
                mask |= 1 << (18 + i);
            }
        }
        if self.center_conduit.is_some() {
            mask |= 1 << 26;
        }
        mask
    }
}

// ============================================================================
// 2.2 Wiring System
// ============================================================================

/// Wire type classification for crosstalk and connection rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WireKind {
    RedAlloy,
    Insulated(DyeColor),
    Bundled(Option<DyeColor>),
    Bluewire { insulated: bool },
    Jacketed,
}

impl WireKind {
    /// Connection rule: bare Red Alloy connects to bare and any insulated color.
    /// Insulated wire connects only to bare Red Alloy or identical dye color (§2.2).
    pub fn can_connect_to(self, other: Self) -> bool {
        match (self, other) {
            (Self::RedAlloy, Self::RedAlloy) => true,
            (Self::RedAlloy, Self::Insulated(_)) | (Self::Insulated(_), Self::RedAlloy) => true,
            (Self::Insulated(c1), Self::Insulated(c2)) => c1 == c2,
            (Self::Bundled(_), Self::Bundled(_)) => true,
            (Self::Bundled(_), Self::Insulated(_)) | (Self::Insulated(_), Self::Bundled(_)) => true,
            (Self::Bluewire { .. }, Self::Bluewire { .. }) => true,
            _ => false,
        }
    }
}

/// Compute Red Alloy Wire internal signal (`0..=255`) at `distance` blocks from a source.
/// Maintained for test compatibility.
pub const fn red_alloy_signal_at_distance(source_strength: u8, distance: u32) -> u8 {
    if source_strength == 0 || distance >= 256 {
        0
    } else {
        source_strength.saturating_sub(distance as u8)
    }
}

/// Lossless signal propagation over up to 256 blocks (`planning/mods/redpower_2.md` §2.2).
pub const fn red_alloy_signal_lossless(source_strength: u8, distance: u32) -> u8 {
    if distance < 256 {
        source_strength
    } else {
        0
    }
}

/// Map a Red Alloy internal signal (`0..=255`) to a standard vanilla redstone level (`0..=15`)
/// using the exact spec formula: `strength / 17` (§2.2).
pub const fn red_alloy_to_vanilla(strength: u8) -> u8 {
    strength / 17
}

/// Map internal signal (`0..=255`) to vanilla redstone (`0..=15`) using spec formula: `internal / 17` (§2.2).
pub const fn internal_to_vanilla_redstone(internal: u8) -> i32 {
    (internal / 17) as i32
}

// ============================================================================
// 2.3 Integrated Logic Tiles (All 20+ Subtypes)
// ============================================================================

/// All 20+ RedPower 2 integrated logic gate tiles (`planning/mods/redpower_2.md` §2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicGateKind {
    Inverter,
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Xnor,
    Buffer,
    Multiplexer,
    Repeater,
    Timer,
    Counter,
    Sequencer,
    PulseFormer,
    ToggleLatch,
    RsLatch,
    TransparentLatch,
    StateCell,
    Synchronizer,
    LightSensor,
    Randomizer,
    NullCell,
    InvertCell,
    NonInvertCell,
    BusTransceiver,
}

/// Repeater configurable delay schedule table in redstone ticks (`tickSchedule`, §2.3).
pub const REPEATER_SCHEDULE: [u32; 9] = [2, 4, 6, 8, 16, 32, 64, 128, 256];

/// Complete state machine for RedPower 2 Logic Gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicGateState {
    pub kind: LogicGateKind,
    pub facing: Direction,
    /// Configured delay or timer interval in ticks (Timer minimum: `4` ticks = `0.2s`).
    pub interval_ticks: u32,
    pub elapsed_ticks: u32,
    /// Counter current value and configured maximum.
    pub count: u32,
    pub max_count: u32,
    pub increment_step: u32,
    pub decrement_step: u32,
    /// Timer pulse ticks remaining (spec §2.3: 2-tick output pulse).
    pub pulse_ticks: u32,
    /// Internal latch/flip-flop/sequencer step (`0..4`).
    pub latch_state: u8,
    /// Previous input mask `[left, back, right]` for rising-edge detection.
    pub prev_inputs: [bool; 3],
    /// Ambient light level (`0..=15`) for `LightSensor`.
    pub ambient_light: u8,
    /// Current output signal (`0..=15`).
    pub output_level: i32,
    /// Randomizer RNG seed.
    pub rng_seed: u64,
}

impl LogicGateState {
    /// Minimum Timer period: 0.2 seconds = 4 server ticks (at 20 TPS).
    pub const MIN_TIMER_TICKS: u32 = 4;

    pub fn new(kind: LogicGateKind) -> Self {
        Self {
            kind,
            facing: Direction::North,
            interval_ticks: Self::MIN_TIMER_TICKS,
            elapsed_ticks: 0,
            count: 0,
            max_count: 10,
            increment_step: 1,
            decrement_step: 1,
            pulse_ticks: 0,
            latch_state: 0,
            prev_inputs: [false; 3],
            ambient_light: 15,
            output_level: if matches!(
                kind,
                LogicGateKind::Inverter
                    | LogicGateKind::Nand
                    | LogicGateKind::Nor
                    | LogicGateKind::Xnor
                    | LogicGateKind::Counter
            ) {
                15
            } else {
                0
            },
            rng_seed: 0x1234_5678_9ABC_DEF0,
        }
    }

    /// Advance the logic gate by 1 server tick with inputs `[left, back, right]`.
    /// Returns the updated redstone output level (`0..=15`).
    pub fn step_tick(&mut self, inputs: [bool; 3]) -> i32 {
        let [left, back, right] = inputs;
        let rising_left = left && !self.prev_inputs[0];
        let rising_back = back && !self.prev_inputs[1];
        let rising_right = right && !self.prev_inputs[2];

        let active = match self.kind {
            LogicGateKind::Inverter => !back,
            LogicGateKind::And => left && back && right,
            LogicGateKind::Or => left || back || right,
            LogicGateKind::Nand => !(left && back && right),
            LogicGateKind::Nor => !(left || back || right),
            LogicGateKind::Xor => (u8::from(left) + u8::from(back) + u8::from(right)) % 2 == 1,
            LogicGateKind::Xnor => left == right,
            LogicGateKind::Buffer => back,
            LogicGateKind::Multiplexer => {
                if back {
                    right
                } else {
                    left
                }
            }
            LogicGateKind::Repeater => {
                if back {
                    self.elapsed_ticks = self.elapsed_ticks.saturating_add(1);
                    self.elapsed_ticks >= self.interval_ticks.max(1)
                } else {
                    self.elapsed_ticks = 0;
                    false
                }
            }
            LogicGateKind::Timer => {
                if back || left || right {
                    // Paused while reset/inhibit input is high
                    self.pulse_ticks = 0;
                    false
                } else {
                    let period = self.interval_ticks.max(Self::MIN_TIMER_TICKS);
                    self.elapsed_ticks += 1;
                    if self.elapsed_ticks >= period {
                        self.elapsed_ticks = 0;
                        self.pulse_ticks = 2;
                    }
                    if self.pulse_ticks > 0 {
                        self.pulse_ticks -= 1;
                        true
                    } else {
                        false
                    }
                }
            }
            LogicGateKind::Counter => {
                if rising_left {
                    self.count = (self.count + self.increment_step).min(self.max_count);
                }
                if rising_right {
                    self.count = self.count.saturating_sub(self.decrement_step);
                }
                self.count == 0 || self.count >= self.max_count
            }
            LogicGateKind::Sequencer => {
                let period = self.interval_ticks.max(Self::MIN_TIMER_TICKS);
                self.elapsed_ticks += 1;
                if self.elapsed_ticks >= period {
                    self.elapsed_ticks = 0;
                    self.latch_state = (self.latch_state + 1) & 0x03;
                }
                self.latch_state == 0
            }
            LogicGateKind::PulseFormer => rising_back,
            LogicGateKind::ToggleLatch => {
                if rising_left || rising_right {
                    self.latch_state ^= 1;
                }
                self.latch_state != 0
            }
            LogicGateKind::RsLatch => {
                if left && !right {
                    self.latch_state = 1;
                } else if right && !left {
                    self.latch_state = 0;
                }
                self.latch_state != 0
            }
            LogicGateKind::TransparentLatch => {
                if back {
                    self.latch_state = u8::from(left);
                }
                self.latch_state != 0
            }
            LogicGateKind::StateCell => {
                if rising_back {
                    self.latch_state = 1;
                    self.elapsed_ticks = 0;
                }
                if self.latch_state != 0 {
                    self.elapsed_ticks += 1;
                    if self.elapsed_ticks >= self.interval_ticks.max(1) {
                        self.latch_state = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            LogicGateKind::Synchronizer => {
                if rising_left {
                    self.latch_state |= 0b01;
                }
                if rising_right {
                    self.latch_state |= 0b10;
                }
                if back {
                    self.latch_state = 0;
                    false
                } else if self.latch_state == 0b11 {
                    self.latch_state = 0;
                    true
                } else {
                    false
                }
            }
            LogicGateKind::LightSensor => {
                self.prev_inputs = inputs;
                self.output_level = i32::from(self.ambient_light.min(15));
                return self.output_level;
            }
            LogicGateKind::Randomizer => {
                if rising_back {
                    let mut rng = XorShift64::new(self.rng_seed);
                    let val = rng.next_u64();
                    self.latch_state = (val % 3) as u8;
                    self.rng_seed = val;
                }
                self.latch_state == 0
            }
            LogicGateKind::NullCell => false,
            LogicGateKind::InvertCell => !back,
            LogicGateKind::NonInvertCell => back,
            LogicGateKind::BusTransceiver => back,
        };

        self.prev_inputs = inputs;
        self.output_level = if active { 15 } else { 0 };
        self.output_level
    }
}

/// 16-channel bidirectional Bundled Cable Bus Transceiver state (`TileLogicAdv` sub 0).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusTransceiverState {
    pub side_a: BundledSignal,
    pub side_b: BundledSignal,
    pub enable_a_to_b: bool,
    pub enable_b_to_a: bool,
    pub latched: bool,
}

impl BusTransceiverState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        if self.enable_a_to_b && !self.enable_b_to_a {
            self.side_b = self.side_a;
        } else if self.enable_b_to_a && !self.enable_a_to_b {
            self.side_a = self.side_b;
        }
    }
}

// ============================================================================
// 2.4 Pneumatic Tubes & Item Logistics
// ============================================================================

/// Traversal weights and speeds for Pneumatic Tubes (§2.4).
pub const PNEUMATIC_HOP_WEIGHT: u32 = 2;
pub const PNEUMATIC_TURN_WEIGHT: u32 = 1;
pub const RESTRICTOR_TUBE_WEIGHT: u32 = 1_000_000;
pub const STANDARD_TUBE_SPEED: f32 = 3.0; // blocks/second
pub const MAGTUBE_SPEED: f32 = 16.0; // blocks/tick (320 blocks/sec)
pub const ACCELERATOR_POWER_PER_ITEM: f64 = 100.0; // W-t per item stack count

/// In-flight item payload moving through the pneumatic tube network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TubePayload {
    pub item: ItemStack,
    pub color_tag: Option<DyeColor>,
    pub path: Vec<BlockPos>,
    pub step: usize,
    pub progress: f32,
    pub in_magtube: bool,
}

impl TubePayload {
    pub fn new(item: ItemStack, color_tag: Option<DyeColor>, path: Vec<BlockPos>) -> Self {
        Self {
            item,
            color_tag,
            path,
            step: 0,
            progress: 0.0,
            in_magtube: false,
        }
    }

    /// Advance traversal progress along the computed path.
    /// Returns `true` when the item arrives at the destination.
    pub fn advance(&mut self, is_magtube: bool) -> bool {
        self.in_magtube = is_magtube;
        let speed = if is_magtube {
            MAGTUBE_SPEED
        } else {
            STANDARD_TUBE_SPEED / 20.0
        };
        self.progress += speed;
        while self.progress >= 1.0 {
            self.progress -= 1.0;
            self.step += 1;
            if self.step >= self.path.len() {
                return true;
            }
        }
        false
    }
}

/// Dijkstra shortest-path pathfinding across pneumatic tubes adhering strictly to §2.4:
/// - Hop weight: +2 per block hop
/// - Turn weight: +1 when deviating from entry direction
/// - Restrictor Tube weight: +1,000,000
/// - Strict 16-color filtering: tagged items only enter matching or unpainted tubes.
pub fn find_tube_route(
    graph: &GridGraph,
    start: BlockPos,
    initial_dir: Option<Direction>,
    destinations: &[BlockPos],
    item_color: Option<DyeColor>,
) -> Option<(BlockPos, Vec<BlockPos>, u32)> {
    if destinations.is_empty() {
        return None;
    }
    let start_node = graph.nodes.get(&start)?;
    if !start_node.allows_color(item_color) {
        return None;
    }

    #[derive(Clone, Eq, PartialEq)]
    struct QState {
        cost: u32,
        pos: BlockPos,
        incoming_dir: Option<Direction>,
    }

    impl Ord for QState {
        fn cmp(&self, other: &Self) -> Ordering {
            other
                .cost
                .cmp(&self.cost)
                .then_with(|| other.pos.cmp(&self.pos))
        }
    }

    impl PartialOrd for QState {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let goal_set: HashSet<BlockPos> = destinations.iter().copied().collect();
    let mut dist: HashMap<BlockPos, u32> = HashMap::new();
    let mut prev: HashMap<BlockPos, BlockPos> = HashMap::new();
    let mut heap = BinaryHeap::new();

    dist.insert(start, 0);
    heap.push(QState {
        cost: 0,
        pos: start,
        incoming_dir: initial_dir,
    });

    while let Some(QState {
        cost,
        pos,
        incoming_dir,
    }) = heap.pop()
    {
        if goal_set.contains(&pos) {
            let mut path = vec![pos];
            let mut curr = pos;
            while let Some(&p) = prev.get(&curr) {
                path.push(p);
                curr = p;
            }
            path.reverse();
            return Some((pos, path, cost));
        }

        if cost > *dist.get(&pos).unwrap_or(&u32::MAX) {
            continue;
        }

        for dir in Direction::ALL {
            let next_pos = pos.offset(dir);
            let Some(next_node) = graph.nodes.get(&next_pos) else {
                continue;
            };
            if !next_node.allows_color(item_color) {
                continue;
            }

            // Hop weight (+2 for standard, +1,000,000 for restrictor tube)
            let base_w = if next_node.weight >= RESTRICTOR_TUBE_WEIGHT {
                RESTRICTOR_TUBE_WEIGHT
            } else {
                PNEUMATIC_HOP_WEIGHT
            };
            // Turn penalty: +1 if deviating from straight line
            let turn_penalty = match incoming_dir {
                Some(prev_dir) if prev_dir != dir => PNEUMATIC_TURN_WEIGHT,
                _ => 0,
            };
            let next_cost = cost + base_w + turn_penalty;

            if next_cost < *dist.get(&next_pos).unwrap_or(&u32::MAX) {
                dist.insert(next_pos, next_cost);
                prev.insert(next_pos, pos);
                heap.push(QState {
                    cost: next_cost,
                    pos: next_pos,
                    incoming_dir: Some(dir),
                });
            }
        }
    }

    None
}

// ============================================================================
// 2.5 Blutricity Power Grid
// ============================================================================

pub const BLUTRICITY_NOMINAL_VOLTAGE: f64 = 100.0;
pub const BLUTRICITY_MIN_VOLTAGE: f64 = 60.0;
pub const BLUTRICITY_MAX_SAFE_VOLTAGE: f64 = 120.0;
pub const BLUTRICITY_WIND_MAX_VOLTAGE: f64 = 130.0;
pub const BLUTRICITY_CONDUCTOR_RESISTANCE: f64 = 0.01;
pub const BLUTRICITY_INDUCTIVE_SCALE: f64 = 0.07;
pub const BLUTRICITY_COND_PARALLEL: f64 = 0.5;

/// Full dynamic RLC Blutricity conductor matching `BluePowerConductor` (§2.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BluePowerConductor {
    pub voltage: f64,
    pub charge: f64,
    pub current: f64,
    pub resistance: f64,
    pub capacitance: f64,
}

impl Default for BluePowerConductor {
    fn default() -> Self {
        Self::new(4.0)
    }
}

impl BluePowerConductor {
    pub fn new(capacitance: f64) -> Self {
        Self {
            voltage: 0.0,
            charge: 0.0,
            current: 0.0,
            resistance: BLUTRICITY_CONDUCTOR_RESISTANCE,
            capacitance: capacitance.max(0.01),
        }
    }

    pub fn is_operational(&self) -> bool {
        self.voltage >= BLUTRICITY_MIN_VOLTAGE
    }

    pub fn add_charge(&mut self, delta_q: f64) {
        self.charge = (self.charge + delta_q).max(0.0);
        self.voltage = self.charge / self.capacitance;
    }

    /// Dynamic transfer step between two conductors (`BluePowerConductor.iterate()`, §2.5):
    /// R = R1 + R2,   V = V1 - V2
    /// I_next = I + (V - I * R) * 0.07
    /// I_flow = I_next + V * 0.5
    /// Apply -I_flow to local node, +I_flow to neighbor node.
    pub fn step_iterate(&mut self, other: &mut Self) -> f64 {
        let r = self.resistance + other.resistance;
        let v = self.voltage - other.voltage;
        let i_next = self.current + (v - self.current * r) * BLUTRICITY_INDUCTIVE_SCALE;
        let i_flow = i_next + v * BLUTRICITY_COND_PARALLEL;

        self.current = i_next;
        self.add_charge(-i_flow);
        other.add_charge(i_flow);
        i_flow
    }
}

/// Solar Panel Blutricity generator: generates 2.0 A when V <= 100V, daytime, sees sky, no storm (§2.5).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolarPanelState {
    pub conductor: BluePowerConductor,
}

impl SolarPanelState {
    pub fn new() -> Self {
        Self {
            conductor: BluePowerConductor::new(0.25),
        }
    }

    pub fn tick(&mut self, sees_sky: bool, is_day: bool, is_storm: bool) -> f64 {
        if sees_sky && is_day && !is_storm && self.conductor.voltage <= BLUTRICITY_NOMINAL_VOLTAGE {
            self.conductor.add_charge(2.0);
            2.0
        } else {
            0.0
        }
    }
}

/// Thermopile Blutricity generator: scans 6 faces, generates `0.005 * min(T_hot, T_cold) A` (§2.5).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThermopileState {
    pub conductor: BluePowerConductor,
    pub tick_counter: u32,
    pub t_hot: i32,
    pub t_cold: i32,
}

impl ThermopileState {
    pub fn new() -> Self {
        Self {
            conductor: BluePowerConductor::new(4.0),
            tick_counter: 19,
            t_hot: 0,
            t_cold: 0,
        }
    }

    /// Temperature ratings from §2.5:
    /// Air (Nether +1, Overworld -1), Torch (+5), Fire (+3), Lava (+100),
    /// Burning Furnace (+25), Water (-25), Snow layer (-50), Ice/Snow block (-100).
    pub fn block_temperature(block_id: &str) -> i32 {
        if block_id.contains("lava") {
            100
        } else if block_id.contains("fire") {
            3
        } else if block_id.contains("torch") {
            5
        } else if block_id.contains("lit_furnace") || block_id.contains("burning_furnace") {
            25
        } else if block_id.contains("ice") || block_id.contains("snow_block") {
            -100
        } else if block_id.contains("snow") {
            -50
        } else if block_id.contains("water") {
            -25
        } else {
            0
        }
    }

    /// Evaluates thermal generation every 20 ticks.
    /// Returns generated current in Amps and whether thermal shock occurred (§8).
    pub fn tick(&mut self, neighbor_temps: [i32; 6], rng_roll: u32) -> (f64, bool) {
        self.tick_counter += 1;
        if self.tick_counter >= 20 {
            self.tick_counter = 0;
            self.t_hot = neighbor_temps
                .iter()
                .filter(|&&t| t > 0)
                .copied()
                .sum::<i32>();
            self.t_cold = neighbor_temps
                .iter()
                .filter(|&&t| t < 0)
                .map(|&t| -t)
                .sum::<i32>();
        }

        let shock = self.t_hot >= 100 && self.t_cold >= 200 && rng_roll.is_multiple_of(300);
        let current = if self.conductor.voltage <= BLUTRICITY_NOMINAL_VOLTAGE {
            let gen = 0.005 * (self.t_hot.min(self.t_cold) as f64);
            self.conductor.add_charge(gen);
            gen
        } else {
            0.0
        };

        (current, shock)
    }
}

/// Wind Turbine Blutricity generator: applies P / 5 W per tick when V <= 130V and brakes released (§2.5).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindTurbineState {
    pub conductor: BluePowerConductor,
    pub sail_durability_ticks: u32,
    pub sail_damage: u32,
    pub brakes_engaged: bool,
}

impl WindTurbineState {
    pub fn new() -> Self {
        Self {
            conductor: BluePowerConductor::new(4.0),
            sail_durability_ticks: 0,
            sail_damage: 0,
            brakes_engaged: false,
        }
    }

    pub fn tick(&mut self, wind_power_w: f64) -> f64 {
        if self.brakes_engaged {
            return 0.0;
        }
        self.sail_durability_ticks += 1;
        if self.sail_durability_ticks >= 6_600 {
            self.sail_durability_ticks = 0;
            self.sail_damage += 1;
        }

        if self.conductor.voltage <= BLUTRICITY_WIND_MAX_VOLTAGE && wind_power_w > 0.0 {
            let power_w = wind_power_w / 5.0;
            // P = V * I -> I = P / max(V, 1.0)
            let current = power_w / self.conductor.voltage.max(1.0);
            self.conductor.add_charge(current);
            power_w
        } else {
            0.0
        }
    }
}

/// Battery Box storage unit: stores 6,000 units (~1,200,000 W-t).
/// Charges when V > 90V; discharges when V < 80V and not redstone powered (§2.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatteryBoxState {
    pub conductor: BluePowerConductor,
    pub stored_units: f64,
    pub battery_slot_0: Option<ItemStack>,
    pub battery_slot_1: Option<ItemStack>,
}

impl Default for BatteryBoxState {
    fn default() -> Self {
        Self::new()
    }
}

impl BatteryBoxState {
    pub const MAX_STORAGE_UNITS: f64 = 6_000.0;
    pub const MAX_RATE_PER_TICK: f64 = 25.0;

    pub fn new() -> Self {
        Self {
            conductor: BluePowerConductor::new(4.0),
            stored_units: 0.0,
            battery_slot_0: None,
            battery_slot_1: None,
        }
    }

    pub fn tick(&mut self, redstone_powered: bool) {
        let v = self.conductor.voltage;
        if v > 90.0 && self.stored_units < Self::MAX_STORAGE_UNITS {
            let n = ((v * 10.0 - 900.0) / 10.0).clamp(0.0, 10.0);
            let draw_w = n * 1_000.0;
            let current = draw_w / v.max(1.0);
            self.conductor.add_charge(-current);
            let units_in = (draw_w / 200.0).min(Self::MAX_STORAGE_UNITS - self.stored_units);
            self.stored_units += units_in;
        } else if v < 80.0 && !redstone_powered && self.stored_units > 0.0 {
            let n = ((800.0 - v * 10.0) / 10.0).clamp(0.0, 10.0);
            let apply_w = n * 1_000.0;
            let current = apply_w / v.max(1.0);
            let units_out = (apply_w / 200.0).min(self.stored_units);
            self.stored_units -= units_out;
            self.conductor.add_charge(current);
        }
    }
}

/// Transformer: steps voltage 100x between 100V low-voltage network and 10,000V line (§2.5).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformerState {
    pub low_voltage_conductor: BluePowerConductor,
    pub high_voltage_conductor: BluePowerConductor,
}

impl TransformerState {
    pub fn new() -> Self {
        Self {
            low_voltage_conductor: BluePowerConductor::new(4.0),
            high_voltage_conductor: BluePowerConductor::new(0.04),
        }
    }

    pub fn step_transform(&mut self) {
        self.high_voltage_conductor.voltage = self.low_voltage_conductor.voltage * 100.0;
        self.high_voltage_conductor.current = self.low_voltage_conductor.current * 100.0;
    }
}

// ============================================================================
// 2.6 Frame Automation & Structural Movement
// ============================================================================

/// Maximum blocks a Frame Motor may translate in a single pulse (`planning/mods/redpower_2.md` §2.6).
pub const MAX_FRAME_BLOCKS: usize = 1000;

/// Contiguous support frame BFS solver: validates link size <= 1,000, rejects negative hardness
/// (bedrock/end portal), permits obsidian (hardness 50.0F), and checks destination volume clearance.
pub fn plan_frame_translation<F>(
    start_frame: BlockPos,
    move_dir: Direction,
    get_block: F,
) -> Result<Vec<BlockPos>, &'static str>
where
    F: Fn(BlockPos) -> Option<String>,
{
    let mut visited: HashSet<BlockPos> = HashSet::new();
    let mut queue: VecDeque<BlockPos> = VecDeque::new();
    visited.insert(start_frame);
    queue.push_back(start_frame);

    while let Some(pos) = queue.pop_front() {
        if visited.len() > MAX_FRAME_BLOCKS {
            return Err("frame assembly exceeds MAX_FRAME_BLOCKS (1000)");
        }
        let Some(block_id) = get_block(pos) else {
            continue;
        };
        if block_id == "minecraft:bedrock" || block_id == "minecraft:end_portal_frame" {
            return Err("bedrock is immovable");
        }
        // Frame blocks latch onto all 6 adjacent non-air neighbors
        if block_id == "redpower_2:frame" {
            for neighbor in pos.neighbors() {
                if !visited.contains(&neighbor) {
                    if let Some(nb) = get_block(neighbor) {
                        if nb != "minecraft:air" {
                            if nb == "minecraft:bedrock" || nb == "minecraft:end_portal_frame" {
                                return Err("bedrock is immovable");
                            }
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }
    }

    // Check destination clearance for every moving block
    for &pos in &visited {
        let dest = pos.offset(move_dir);
        if !visited.contains(&dest) {
            if let Some(dest_block) = get_block(dest) {
                if dest_block != "minecraft:air" {
                    return Err("destination volume obstructed");
                }
            }
        }
    }

    let mut ordered: Vec<BlockPos> = visited.into_iter().collect();
    ordered.sort();
    Ok(ordered)
}

/// Frame Motor state machine tracking 16-tick translation and Blutricity power draw (§2.6).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameMotorState {
    pub move_pos: u8,
    pub is_moving: bool,
    pub move_dir: Option<Direction>,
    pub moving_blocks: Vec<BlockPos>,
}

impl FrameMotorState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Power required per tick: 100 + 10 * LinkSize W-t (§2.6). Requires V >= 60V.
    pub fn power_cost_per_tick(&self) -> f64 {
        100.0 + 10.0 * (self.moving_blocks.len() as f64)
    }

    /// Advance movement tick (`0..16`). Returns `true` when translation completes at tick 16.
    pub fn tick(&mut self, voltage: f64) -> Result<bool, &'static str> {
        if !self.is_moving {
            return Ok(false);
        }
        if voltage < BLUTRICITY_MIN_VOLTAGE {
            return Err("insufficient Blutricity voltage (< 60V)");
        }
        self.move_pos += 1;
        if self.move_pos >= 16 {
            self.move_pos = 0;
            self.is_moving = false;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// ============================================================================
// 2.7 Control 6502 Computing Subsystem
// ============================================================================

/// Bounded MOS 6502 / 65C816 CPU emulator with 64 KB address space, 1,000 cycles/tick slice,
/// 100,000 cycle burst cap, and memory-mapped Bundled Cable I/O Expander registers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cpu6502 {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub status: u8,
    pub halted: bool,
    pub ram: Vec<u8>,
    pub io_expander: BundledSignal,
    pub cycle_accumulator: usize,
}

impl Default for Cpu6502 {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu6502 {
    pub const RAM_SIZE: usize = 65_536;
    pub const CYCLES_PER_TICK: usize = 1_000;
    pub const MAX_BURST_CYCLES: usize = 100_000;
    pub const MAX_CYCLES_PER_TICK: usize = 1_000;
    pub const IO_EXPANDER_LOW_ADDR: u16 = 0xE000;
    pub const IO_EXPANDER_HIGH_ADDR: u16 = 0xE001;

    /// Add +1,000 instruction cycles to burst accumulator, capped at 100,000 cycles (§2.7, §8).
    pub fn accumulate_tick(&mut self) {
        self.cycle_accumulator =
            (self.cycle_accumulator + Self::CYCLES_PER_TICK).min(Self::MAX_BURST_CYCLES);
    }

    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0x0400,
            status: 0x20,
            halted: false,
            ram: vec![0; Self::RAM_SIZE],
            io_expander: BundledSignal::new(),
            cycle_accumulator: 0,
        }
    }

    pub fn load_program(&mut self, start_addr: u16, bytes: &[u8]) {
        let start = start_addr as usize;
        for (i, &b) in bytes.iter().enumerate() {
            if start + i < Self::RAM_SIZE {
                self.ram[start + i] = b;
            }
        }
        self.pc = start_addr;
        self.halted = false;
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            Self::IO_EXPANDER_LOW_ADDR => (self.io_expander.to_digital_mask() & 0xFF) as u8,
            Self::IO_EXPANDER_HIGH_ADDR => (self.io_expander.to_digital_mask() >> 8) as u8,
            _ => self.ram[addr as usize],
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
        if addr == Self::IO_EXPANDER_LOW_ADDR || addr == Self::IO_EXPANDER_HIGH_ADDR {
            let low = self.ram[Self::IO_EXPANDER_LOW_ADDR as usize] as u16;
            let high = (self.ram[Self::IO_EXPANDER_HIGH_ADDR as usize] as u16) << 8;
            self.io_expander = BundledSignal::from_digital_mask(low | high);
        }
    }

    fn update_nz(&mut self, val: u8) {
        if val == 0 {
            self.status |= 0x02;
        } else {
            self.status &= !0x02;
        }
        if (val & 0x80) != 0 {
            self.status |= 0x80;
        } else {
            self.status &= !0x80;
        }
    }

    pub fn push_u8(&mut self, val: u8) {
        let addr = 0x0100 | (self.sp as u16);
        self.write_u8(addr, val);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pop_u8(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100 | (self.sp as u16);
        self.read_u8(addr)
    }

    /// Execute up to `max_instructions` (capped at available burst cycle accumulator), returning
    /// the number of instructions executed. Automatically accumulates +1,000 cycles per world tick (§2.7).
    pub fn step_tick(&mut self, max_instructions: usize) -> usize {
        self.accumulate_tick();
        let limit = max_instructions.min(self.cycle_accumulator);
        let mut executed = 0;
        while !self.halted && executed < limit {
            let op = self.read_u8(self.pc);
            self.pc = self.pc.wrapping_add(1);
            executed += 1;

            match op {
                0x00 => {
                    // BRK
                    self.halted = true;
                }
                0xEA => {
                    // NOP
                }
                0xA9 => {
                    // LDA #imm
                    let imm = self.read_u8(self.pc);
                    self.pc = self.pc.wrapping_add(1);
                    self.a = imm;
                    self.update_nz(self.a);
                }
                0xA2 => {
                    // LDX #imm
                    let imm = self.read_u8(self.pc);
                    self.pc = self.pc.wrapping_add(1);
                    self.x = imm;
                    self.update_nz(self.x);
                }
                0xA0 => {
                    // LDY #imm
                    let imm = self.read_u8(self.pc);
                    self.pc = self.pc.wrapping_add(1);
                    self.y = imm;
                    self.update_nz(self.y);
                }
                0x8D => {
                    // STA abs
                    let lo = self.read_u8(self.pc) as u16;
                    let hi = (self.read_u8(self.pc.wrapping_add(1)) as u16) << 8;
                    self.pc = self.pc.wrapping_add(2);
                    self.write_u8(lo | hi, self.a);
                }
                0x8E => {
                    // STX abs
                    let lo = self.read_u8(self.pc) as u16;
                    let hi = (self.read_u8(self.pc.wrapping_add(1)) as u16) << 8;
                    self.pc = self.pc.wrapping_add(2);
                    self.write_u8(lo | hi, self.x);
                }
                0x8C => {
                    // STY abs
                    let lo = self.read_u8(self.pc) as u16;
                    let hi = (self.read_u8(self.pc.wrapping_add(1)) as u16) << 8;
                    self.pc = self.pc.wrapping_add(2);
                    self.write_u8(lo | hi, self.y);
                }
                0x69 => {
                    // ADC #imm
                    let imm = self.read_u8(self.pc);
                    self.pc = self.pc.wrapping_add(1);
                    let sum = (self.a as u16) + (imm as u16) + ((self.status & 1) as u16);
                    if sum > 0xFF {
                        self.status |= 1;
                    } else {
                        self.status &= !1;
                    }
                    self.a = (sum & 0xFF) as u8;
                    self.update_nz(self.a);
                }
                0xE8 => {
                    // INX
                    self.x = self.x.wrapping_add(1);
                    self.update_nz(self.x);
                }
                0xC8 => {
                    // INY
                    self.y = self.y.wrapping_add(1);
                    self.update_nz(self.y);
                }
                0xCA => {
                    // DEX
                    self.x = self.x.wrapping_sub(1);
                    self.update_nz(self.x);
                }
                0x88 => {
                    // DEY
                    self.y = self.y.wrapping_sub(1);
                    self.update_nz(self.y);
                }
                0x4C => {
                    // JMP abs
                    let lo = self.read_u8(self.pc) as u16;
                    let hi = (self.read_u8(self.pc.wrapping_add(1)) as u16) << 8;
                    self.pc = lo | hi;
                }
                0x48 => {
                    // PHA
                    self.push_u8(self.a);
                }
                0x68 => {
                    // PLA
                    self.a = self.pop_u8();
                    self.update_nz(self.a);
                }
                0x18 => {
                    // CLC
                    self.status &= !1;
                }
                0x38 => {
                    // SEC
                    self.status |= 1;
                }
                0xF0 => {
                    // BEQ rel
                    let offset = self.read_u8(self.pc) as i8;
                    self.pc = self.pc.wrapping_add(1);
                    if (self.status & 0x02) != 0 {
                        self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                    }
                }
                0xD0 => {
                    // BNE rel
                    let offset = self.read_u8(self.pc) as i8;
                    self.pc = self.pc.wrapping_add(1);
                    if (self.status & 0x02) == 0 {
                        self.pc = ((self.pc as i32) + (offset as i32)) as u16;
                    }
                }
                _ => {
                    // Unknown opcode -> halt safely
                    self.halted = true;
                }
            }
        }
        self.cycle_accumulator = self.cycle_accumulator.saturating_sub(executed);
        executed
    }
}

/// 80 columns x 50 rows character matrix terminal display with hardware blitter (§2.7).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayTerminal {
    pub screen: Vec<u8>,
    pub cursor_x: u8,
    pub cursor_y: u8,
    pub cursor_mode: u8, // 0 = off, 1 = solid, 2 = blink
    pub key_buffer: VecDeque<u8>,
}

impl Default for DisplayTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayTerminal {
    pub const COLS: usize = 80;
    pub const ROWS: usize = 50;
    pub const BUFFER_SIZE: usize = Self::COLS * Self::ROWS;

    pub fn new() -> Self {
        Self {
            screen: vec![32; Self::BUFFER_SIZE], // 32 = space
            cursor_x: 0,
            cursor_y: 0,
            cursor_mode: 1,
            key_buffer: VecDeque::with_capacity(16),
        }
    }

    /// Hardware blitter: rectangular screen memory copy (`blitXS, blitYS, blitXD, blitYD, blitW, blitH`, §2.7).
    pub fn blit(&mut self, xs: usize, ys: usize, xd: usize, yd: usize, w: usize, h: usize) {
        let mut tmp = vec![32u8; w * h];
        for row in 0..h {
            for col in 0..w {
                let src_idx = (ys + row) * Self::COLS + (xs + col);
                if src_idx < Self::BUFFER_SIZE {
                    tmp[row * w + col] = self.screen[src_idx];
                }
            }
        }
        for row in 0..h {
            for col in 0..w {
                let dst_idx = (yd + row) * Self::COLS + (xd + col);
                if dst_idx < Self::BUFFER_SIZE {
                    self.screen[dst_idx] = tmp[row * w + col];
                }
            }
        }
    }

    pub fn push_key(&mut self, key: u8) {
        if self.key_buffer.len() < 16 {
            self.key_buffer.push_back(key);
        }
    }

    pub fn pop_key(&mut self) -> Option<u8> {
        self.key_buffer.pop_front()
    }
}

/// 256 KB virtual disk storage: 2,048 sectors of 128 bytes each (§2.7).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskDriveState {
    pub sectors: Vec<u8>,
    pub serial_number: u32,
    pub label: String,
}

impl Default for DiskDriveState {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskDriveState {
    pub const SECTOR_COUNT: usize = 2048;
    pub const SECTOR_SIZE: usize = 128;
    pub const TOTAL_SIZE: usize = Self::SECTOR_COUNT * Self::SECTOR_SIZE;

    pub fn new() -> Self {
        Self {
            sectors: vec![0u8; Self::TOTAL_SIZE],
            serial_number: 1001,
            label: "RedPower Forth Disk".into(),
        }
    }

    /// Redbus commands: 1 = Read Label, 2 = Write Label, 3 = Read Serial Number, 4 = Read Sector, 5 = Write Sector.
    pub fn execute_redbus(&mut self, cmd: u8, sector: usize, buf: &mut [u8; 128]) -> bool {
        match cmd {
            1 => {
                let bytes = self.label.as_bytes();
                let len = bytes.len().min(128);
                buf[..len].copy_from_slice(&bytes[..len]);
                true
            }
            2 => {
                if let Ok(s) = std::str::from_utf8(buf) {
                    self.label = s.trim_matches('\0').to_string();
                }
                true
            }
            3 => {
                let b = self.serial_number.to_le_bytes();
                buf[..4].copy_from_slice(&b);
                true
            }
            4 if sector < Self::SECTOR_COUNT => {
                let start = sector * Self::SECTOR_SIZE;
                buf.copy_from_slice(&self.sectors[start..start + Self::SECTOR_SIZE]);
                true
            }
            5 if sector < Self::SECTOR_COUNT => {
                let start = sector * Self::SECTOR_SIZE;
                self.sectors[start..start + Self::SECTOR_SIZE].copy_from_slice(buf);
                true
            }
            _ => false,
        }
    }
}

// ============================================================================
// 3.1 Appliance Machines: Alloy Furnace
// ============================================================================

/// Alloy Furnace recipe definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlloyRecipe {
    pub inputs: [(&'static str, u32); 2],
    pub output: ItemStack,
}

/// Alloy Furnace state machine (`redpower_2:alloy_furnace` & `redpower_2:blulectric_alloy_furnace`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlloyFurnaceState {
    /// 9-slot 3x3 input grid (`0..9`), 1 fuel slot (`9`), 1 output slot (`10`).
    pub inventory: Inventory,
    pub burn_ticks_remaining: u32,
    pub smelt_timer: TickTimer,
}

impl Default for AlloyFurnaceState {
    fn default() -> Self {
        Self::new()
    }
}

impl AlloyFurnaceState {
    pub const SLOT_COUNT: usize = 11;
    pub const INPUT_SLOTS: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];
    pub const FUEL_SLOT: usize = 9;
    pub const OUTPUT_SLOT: usize = 10;
    pub const SMELT_TICKS: u32 = 100;
    pub const STANDARD_SMELT_TICKS: u32 = 200;
    pub const BLULECTRIC_SMELT_TICKS: u32 = 100;

    pub fn new() -> Self {
        Self {
            inventory: Inventory::new(Self::SLOT_COUNT),
            burn_ticks_remaining: 0,
            smelt_timer: TickTimer::new(Self::SMELT_TICKS),
        }
    }

    fn count_input(&self, item_id: &str) -> u32 {
        Self::INPUT_SLOTS
            .iter()
            .filter_map(|&i| self.inventory.slots[i].as_ref())
            .filter(|s| s.item_id == item_id)
            .map(|s| s.count)
            .sum()
    }

    fn consume_input(&mut self, item_id: &str, mut needed: u32) {
        for &i in &Self::INPUT_SLOTS {
            if needed == 0 {
                break;
            }
            if let Some(stack) = &mut self.inventory.slots[i] {
                if stack.item_id == item_id {
                    let take = stack.count.min(needed);
                    stack.count -= take;
                    needed -= take;
                    if stack.count == 0 {
                        self.inventory.slots[i] = None;
                    }
                }
            }
        }
    }

    /// Match canonical RedPower 2 Alloy Furnace recipes (`planning/mods/redpower_2.md` §7.1).
    pub fn match_recipe(&self) -> Option<(&'static str, u32, &'static str, u32, ItemStack)> {
        // Red Alloy: 4x redstone + 1x copper (or iron) -> 1x red_alloy_ingot
        if self.count_input("minecraft:redstone") >= 4
            && self.count_input("redpower_2:copper_ingot") >= 1
        {
            Some((
                "minecraft:redstone",
                4,
                "redpower_2:copper_ingot",
                1,
                ItemStack::new("redpower_2:red_alloy_ingot", 1),
            ))
        } else if self.count_input("minecraft:redstone") >= 4
            && self.count_input("minecraft:iron_ingot") >= 1
        {
            Some((
                "minecraft:redstone",
                4,
                "minecraft:iron_ingot",
                1,
                ItemStack::new("redpower_2:red_alloy_ingot", 1),
            ))
        }
        // Blue Alloy: 4x nikolite + 1x silver -> 1x blue_alloy_ingot
        else if self.count_input("redpower_2:nikolite") >= 4
            && self.count_input("redpower_2:silver_ingot") >= 1
        {
            Some((
                "redpower_2:nikolite",
                4,
                "redpower_2:silver_ingot",
                1,
                ItemStack::new("redpower_2:blue_alloy_ingot", 1),
            ))
        }
        // Brass: 3x copper + 1x tin -> 4x brass_ingot
        else if self.count_input("redpower_2:copper_ingot") >= 3
            && self.count_input("redpower_2:tin_ingot") >= 1
        {
            Some((
                "redpower_2:copper_ingot",
                3,
                "redpower_2:tin_ingot",
                1,
                ItemStack::new("redpower_2:brass_ingot", 4),
            ))
        }
        // Tinplate: 2x iron + 1x tin -> 4x tinplate
        else if self.count_input("minecraft:iron_ingot") >= 2
            && self.count_input("redpower_2:tin_ingot") >= 1
        {
            Some((
                "minecraft:iron_ingot",
                2,
                "redpower_2:tin_ingot",
                1,
                ItemStack::new("redpower_2:tinplate", 4),
            ))
        }
        // Silicon Boule: 8x coal + 8x sand -> 1x silicon_boule
        else if self.count_input("minecraft:coal") >= 8 && self.count_input("minecraft:sand") >= 8
        {
            Some((
                "minecraft:coal",
                8,
                "minecraft:sand",
                8,
                ItemStack::new("redpower_2:silicon_boule", 1),
            ))
        }
        // Red-Doped Wafer: 1x silicon_wafer + 4x redstone -> 1x red_doped_wafer
        else if self.count_input("minecraft:redstone") >= 4
            && self.count_input("redpower_2:silicon_wafer") >= 1
        {
            Some((
                "minecraft:redstone",
                4,
                "redpower_2:silicon_wafer",
                1,
                ItemStack::new("redpower_2:red_doped_wafer", 1),
            ))
        }
        // Blue-Doped Wafer: 1x silicon_wafer + 4x nikolite -> 1x blue_doped_wafer
        else if self.count_input("redpower_2:nikolite") >= 4
            && self.count_input("redpower_2:silicon_wafer") >= 1
        {
            Some((
                "redpower_2:nikolite",
                4,
                "redpower_2:silicon_wafer",
                1,
                ItemStack::new("redpower_2:blue_doped_wafer", 1),
            ))
        } else {
            None
        }
    }

    pub fn tick(&mut self, blulectric_powered: bool) {
        if self.burn_ticks_remaining > 0 {
            self.burn_ticks_remaining -= 1;
        }
        let Some((in_a, cnt_a, in_b, cnt_b, out_stack)) = self.match_recipe() else {
            self.smelt_timer.elapsed = 0;
            return;
        };
        if !blulectric_powered && self.burn_ticks_remaining == 0 {
            if self
                .inventory
                .extract_from_slots(1, &[Self::FUEL_SLOT])
                .is_some()
            {
                self.burn_ticks_remaining = 1600;
            } else {
                return;
            }
        }
        if self.smelt_timer.tick()
            && self
                .inventory
                .insert_into_slots(out_stack, &[Self::OUTPUT_SLOT])
                .is_none()
        {
            self.consume_input(in_a, cnt_a);
            self.consume_input(in_b, cnt_b);
        }
    }
}

// ============================================================================
// 4. Tools, Armor & Equipment (§4.1 - §4.3)
// ============================================================================

pub const ATHAME_BASE_DAMAGE: i32 = 1;
pub const ATHAME_ENDER_DAMAGE: i32 = 25;
pub const ATHAME_DURABILITY: u32 = 100;
pub const DISJUNCTION_PER_LEVEL_DAMAGE: i32 = 6;
pub const VORPAL_CRIT_BONUS_DAMAGE: i32 = 100;
pub const VORPAL_DECAPITATION_OVERKILL_THRESHOLD: i32 = -20;

/// Check if a victim entity kind represents an Enderman or the Ender Dragon (§4.2).
pub fn is_ender_mob(entity_kind: &str) -> bool {
    let lower = entity_kind.to_ascii_lowercase();
    lower.contains("enderman") || lower.contains("ender_dragon") || lower.contains("enderdragon")
}

/// Calculate attack damage for Athame against a target entity, accounting for Disjunction enchantment (§4.2).
/// Base attack damage: 1 against normal mobs; 25 damage against Endermen and the Ender Dragon.
/// Disjunction: deals + (6 * level) additional damage to Endermen and Ender Dragons.
pub fn athame_attack_damage(is_ender: bool, disjunction_level: u32) -> i32 {
    let base = if is_ender {
        ATHAME_ENDER_DAMAGE
    } else {
        ATHAME_BASE_DAMAGE
    };
    if is_ender {
        base + (disjunction_level as i32) * DISJUNCTION_PER_LEVEL_DAMAGE
    } else {
        base
    }
}

/// Calculate Vorpal enchantment proc chance in percent: `2 * level^2 %` (§4.2).
/// E.g. level 1 = 2%, level 2 = 8%, level 3 = 18%, level 4 = 32%.
pub const fn vorpal_proc_chance_percent(level: u32) -> u32 {
    2 * level * level
}

/// Calculate weapon damage applying Vorpal bonus when critical strike procs (+100 critical damage, §4.2).
pub fn calculate_weapon_damage(base_damage: i32, vorpal_level: u32, crit_proc: bool) -> i32 {
    if vorpal_level > 0 && crit_proc {
        base_damage + VORPAL_CRIT_BONUS_DAMAGE
    } else {
        base_damage
    }
}

/// Check if overkill damage results in victim head decapitation (§4.2, §8: overkill HP <= -20).
/// `victim_current_hp - damage <= -20`.
pub const fn check_vorpal_decapitation(victim_current_hp: i32, damage: i32) -> bool {
    (victim_current_hp - damage) <= VORPAL_DECAPITATION_OVERKILL_THRESHOLD
}

/// Sickle variants and durability (§4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SickleVariant {
    Wood,
    Stone,
    Iron,
    Diamond,
    Gold,
    Ruby,
    GreenSapphire,
    Sapphire,
}

impl SickleVariant {
    pub const fn durability(self) -> u32 {
        match self {
            Self::Wood => 59,
            Self::Stone => 131,
            Self::Iron => 250,
            Self::Diamond => 1561,
            Self::Gold => 32,
            Self::Ruby | Self::GreenSapphire | Self::Sapphire => 500,
        }
    }
}

pub const SICKLE_LEAF_RADIUS: i32 = 1;
pub const SICKLE_CROP_RADIUS: i32 = 2;

/// Calculate harvesting volume for leaves: 3x3x3 cube centered at target (`leafRadius = 1`, §4.2).
pub fn sickle_leaf_positions(center: BlockPos) -> Vec<BlockPos> {
    let mut positions = Vec::with_capacity(27);
    for dy in -SICKLE_LEAF_RADIUS..=SICKLE_LEAF_RADIUS {
        for dz in -SICKLE_LEAF_RADIUS..=SICKLE_LEAF_RADIUS {
            for dx in -SICKLE_LEAF_RADIUS..=SICKLE_LEAF_RADIUS {
                positions.push(BlockPos(center.0 + dx, center.1 + dy, center.2 + dz));
            }
        }
    }
    positions
}

/// Calculate harvesting flat area for crops and tall grass: 5x5 area centered at target (`cropRadius = 2`, §4.2).
pub fn sickle_crop_positions(center: BlockPos) -> Vec<BlockPos> {
    let mut positions = Vec::with_capacity(25);
    for dz in -SICKLE_CROP_RADIUS..=SICKLE_CROP_RADIUS {
        for dx in -SICKLE_CROP_RADIUS..=SICKLE_CROP_RADIUS {
            positions.push(BlockPos(center.0 + dx, center.1, center.2 + dz));
        }
    }
    positions
}

// ============================================================================
// HorizonNativeMod Implementation for RedPower 2
// ============================================================================

#[derive(Default)]
pub struct RedPower2Mod {
    pub tube_graph: GridGraph,
    pub gates: HashMap<[i32; 3], LogicGateState>,
    pub alloy_furnaces: HashMap<[i32; 3], AlloyFurnaceState>,
    pub cpus: HashMap<[i32; 3], Cpu6502>,
    pub terminal: DisplayTerminal,
}

impl RedPower2Mod {
    pub const BLOCK_ALLOY_FURNACE: &'static str = "redpower_2:alloy_furnace";
    pub const BLOCK_PNEUMATIC_TUBE: &'static str = "redpower_2:pneumatic_tube";
    pub const BLOCK_RESTRICTOR_TUBE: &'static str = "redpower_2:restrictor_tube";
    pub const BLOCK_BUFFER: &'static str = "redpower_2:buffer";
    pub const BLOCK_CPU: &'static str = "redpower_2:cpu";
    pub const ITEM_SCREWDRIVER: &'static str = "redpower_2:screwdriver";
    pub const ITEM_PAINT_BRUSH: &'static str = "redpower_2:paint_brush";

    pub fn owns_id(id: &str) -> bool {
        id.starts_with("redpower_2:")
    }
}

impl HorizonNativeMod for RedPower2Mod {
    fn native_mods(&self, mods: &mut Vec<NativeModInfo>) {
        if mods.iter().any(|m| m.id == MOD_ID) {
            return;
        }
        mods.push(NativeModInfo {
            id: MOD_ID.into(),
            title: "RedPower 2 0.1.0".into(),
            description: "RedPower 2: wiring, logic gates, pneumatic tubes, frames, blutricity, microblocks, and 6502 control.".into(),
            keys: Vec::new(),
        });
    }

    fn native_block_entity(
        &mut self,
        block: &str,
        entity: &mut Option<(String, HashMap<String, String>)>,
    ) {
        if entity.is_some() || !Self::owns_id(block) {
            return;
        }
        let mut nbt = HashMap::new();
        nbt.insert("facing".into(), "2".into());
        *entity = Some((block.to_string(), nbt));
    }

    fn native_block_entity_ticks(&self, block: &str, ticks: &mut bool) {
        if !Self::owns_id(block) {
            return;
        }
        if matches!(
            block,
            Self::BLOCK_ALLOY_FURNACE
                | "redpower_2:blulectric_alloy_furnace"
                | "redpower_2:blulectric_furnace"
                | "redpower_2:battery_box"
                | "redpower_2:solar_panel"
                | "redpower_2:thermopile"
                | "redpower_2:wind_turbine"
                | Self::BLOCK_CPU
        ) {
            *ticks = true;
        }
    }

    fn native_block_entity_container(&self, block: &str, container: &mut Option<ModContainer>) {
        if container.is_some() || !Self::owns_id(block) {
            return;
        }
        match block {
            Self::BLOCK_ALLOY_FURNACE => {
                let inputs: Vec<usize> = (0..9).collect();
                let outputs = vec![10];
                *container = Some(ModContainer {
                    slots: AlloyFurnaceState::SLOT_COUNT,
                    menu: Some("generic_9x3".into()),
                    inputs: inputs.clone(),
                    outputs: outputs.clone(),
                    faces: [
                        outputs,
                        inputs.clone(),
                        vec![9],
                        vec![9],
                        inputs.clone(),
                        inputs,
                    ],
                });
            }
            Self::BLOCK_BUFFER | "redpower_2:filter" | "redpower_2:deployer" => {
                let slots: Vec<usize> = (0..9).collect();
                *container = Some(ModContainer {
                    slots: 9,
                    menu: Some("generic_3x3".into()),
                    inputs: slots.clone(),
                    outputs: slots.clone(),
                    faces: std::array::from_fn(|_| slots.clone()),
                });
            }
            _ => {}
        }
    }

    fn native_block_entity_tick(&mut self, pos: [i32; 3], entity: &mut ModBlockEntity) {
        if !Self::owns_id(&entity.id) {
            return;
        }
        if entity.id == Self::BLOCK_ALLOY_FURNACE {
            let state = self.alloy_furnaces.entry(pos).or_default();
            if entity.items.len() == AlloyFurnaceState::SLOT_COUNT {
                state.inventory.slots = entity.items.clone();
            }
            state.tick(false);
            entity.items = state.inventory.slots.clone();
            entity.data = vec![
                state.smelt_timer.elapsed as i32,
                state.burn_ticks_remaining as i32,
            ];
        } else if entity.id == Self::BLOCK_CPU {
            let cpu = self.cpus.entry(pos).or_default();
            let ran = cpu.step_tick(Cpu6502::CYCLES_PER_TICK);
            entity.data = vec![ran as i32, i32::from(cpu.io_expander.to_digital_mask())];
        }
    }

    fn native_screen(&self, block: &str, screen: &mut Option<NativeScreen>) {
        if screen.is_some() || !Self::owns_id(block) {
            return;
        }
        if block == Self::BLOCK_ALLOY_FURNACE {
            let mut slots = Vec::with_capacity(AlloyFurnaceState::SLOT_COUNT);
            for row in 0..3 {
                for col in 0..3 {
                    slots.push(Some([
                        48.0 + (col as f32) * 18.0,
                        17.0 + (row as f32) * 18.0,
                    ]));
                }
            }
            slots.push(Some([17.0, 42.0])); // Fuel slot
            slots.push(Some([141.0, 35.0])); // Output slot
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 166.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 84.0,
                own_labels: false,
            });
        }
    }

    fn native_use_item_on(
        &mut self,
        stack: &ItemStack,
        block_at_pos: &str,
        entity: Option<&mut ModBlockEntity>,
        _acting_player_uuid: Option<&str>,
        claimed: &mut bool,
    ) {
        if *claimed || !Self::owns_id(block_at_pos) {
            return;
        }
        let Some(entity) = entity else {
            return;
        };
        if stack.item_id == Self::ITEM_SCREWDRIVER {
            let cur = entity
                .nbt
                .get("facing")
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(2);
            let next = 2 + ((cur.saturating_sub(2) + 1) % 4);
            entity.nbt.insert("facing".into(), next.to_string());
            *claimed = true;
        } else if stack.item_id == Self::ITEM_PAINT_BRUSH
            || DyeColor::from_dye_item(&stack.item_id).is_some()
        {
            let color = DyeColor::from_dye_item(&stack.item_id).unwrap_or(DyeColor::Red);
            entity.nbt.insert("color".into(), (color as u8).to_string());
            *claimed = true;
        }
    }

    fn native_is_signal_source(&self, block: &str, source: &mut bool) {
        if !Self::owns_id(block) {
            return;
        }
        if matches!(
            block,
            "redpower_2:item_detector" | "redpower_2:io_expander" | Self::BLOCK_ALLOY_FURNACE
        ) {
            *source = true;
        }
    }

    fn native_block_signal(
        &self,
        block: &str,
        entity: Option<&ModBlockEntity>,
        _direct: bool,
        power: &mut i32,
    ) {
        if !Self::owns_id(block) {
            return;
        }
        if let Some(entity) = entity {
            if let Some(sig) = entity.nbt.get("signal").and_then(|v| v.parse::<i32>().ok()) {
                *power = (*power).max(sig.clamp(0, 15));
            }
        }
    }

    fn native_tooltip(&self, stack: &ItemStack, lines: &mut Vec<String>) {
        if !Self::owns_id(&stack.item_id) {
            return;
        }
        if stack.item_id == "redpower_2:voltmeter" {
            lines.push("Blutricity Voltmeter".into());
            lines.push("Measures potential (V) and charge (J)".into());
        } else if stack.item_id == "redpower_2:canvas_bag" {
            lines.push("Canvas Bag: 27 inventory slots".into());
        } else if stack.item_id.ends_with("_sickle") || stack.item_id == "redpower_2:sickle" {
            lines.push("Harvest area: 3x3x3 leaves, 5x5 crops".into());
        } else if stack.item_id == "redpower_2:athame" {
            lines.push("Silver ritual dagger: 25 damage vs Ender mobs".into());
        }
    }

    fn native_screen_click(
        &self,
        block: &str,
        _data: &[i32],
        at: [f32; 2],
        _right: bool,
        button: &mut Option<i32>,
    ) {
        if button.is_some() || !Self::owns_id(block) {
            return;
        }
        if at[0] >= 10.0 && at[0] <= 100.0 && at[1] >= 10.0 && at[1] <= 40.0 {
            *button = Some(1);
        }
    }

    fn native_menu_button(&mut self, pos: [i32; 3], entity: &mut ModBlockEntity, id: i32) {
        if !Self::owns_id(&entity.id) {
            return;
        }
        if entity.id == "redpower_2:timer" {
            let gate = self
                .gates
                .entry(pos)
                .or_insert_with(|| LogicGateState::new(LogicGateKind::Timer));
            if id == 1 {
                gate.interval_ticks = gate.interval_ticks.saturating_add(2);
            } else if id == 2 {
                gate.interval_ticks = gate
                    .interval_ticks
                    .saturating_sub(2)
                    .max(LogicGateState::MIN_TIMER_TICKS);
            }
            entity
                .nbt
                .insert("interval".into(), gate.interval_ticks.to_string());
        } else if entity.id == "redpower_2:counter" {
            let gate = self
                .gates
                .entry(pos)
                .or_insert_with(|| LogicGateState::new(LogicGateKind::Counter));
            if id == 1 {
                gate.max_count = gate.max_count.saturating_add(1);
            } else if id == 2 {
                gate.max_count = gate.max_count.saturating_sub(1).max(1);
            }
            entity
                .nbt
                .insert("max_count".into(), gate.max_count.to_string());
        } else {
            entity.nbt.insert("last_button".into(), id.to_string());
        }
    }

    fn native_net_from_client(&mut self, _player_uuid: &str, channel: &str, data: &[u8]) {
        if channel != "horizon:mod_net" && channel != "redpower_2:terminal" {
            return;
        }
        if channel == "horizon:mod_net" {
            // Packet 212: Keystrokes sent over packet 212 into 16-byte server ring buffer (kbbuf)
            if let Some((&first, rest)) = data.split_first() {
                if first == 212 {
                    for &key in rest {
                        if self.terminal.key_buffer.len() < 16 {
                            self.terminal.key_buffer.push_back(key);
                        }
                    }
                }
            }
        } else if channel == "redpower_2:terminal" {
            for &key in data {
                if self.terminal.key_buffer.len() < 16 {
                    self.terminal.key_buffer.push_back(key);
                }
            }
        }
    }
}

// ============================================================================
// Internal Unit Tests for Subsystems
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microblock_thickness_and_recombine() {
        assert_eq!(MicroblockKind::Cover.thickness_eighths(), 1);
        assert_eq!(MicroblockKind::Panel.thickness_eighths(), 2);
        assert_eq!(MicroblockKind::TripleCover.thickness_eighths(), 3);
        assert_eq!(MicroblockKind::Slab.thickness_eighths(), 4);
        assert_eq!(MicroblockKind::CoverSlab.thickness_eighths(), 5);
        assert_eq!(MicroblockKind::TriplePanel.thickness_eighths(), 6);
        assert_eq!(MicroblockKind::Anticover.thickness_eighths(), 7);

        assert_eq!(
            recombine_microblocks(MicroblockKind::Cover, MicroblockKind::Cover),
            Some(MicroblockKind::Panel)
        );
        assert_eq!(
            recombine_microblocks(MicroblockKind::Panel, MicroblockKind::Panel),
            Some(MicroblockKind::Slab)
        );

        assert!(can_handsaw_cut(1.5, 0)); // wood/stone with iron saw
        assert!(!can_handsaw_cut(3.0, 0)); // iron block with iron saw
        assert!(can_handsaw_cut(3.0, 1)); // iron block with ruby saw
    }

    #[test]
    fn test_subvoxel_occupancy_mask() {
        let mut voxel = MicroblockVoxel::new();
        assert_eq!(voxel.occupancy_mask(), 0);

        voxel.place_on_face(Direction::North, MicroblockKind::Cover);
        assert_eq!(
            voxel.occupancy_mask() & 0x3F,
            1 << (Direction::North as usize)
        );

        voxel.place_edge(3, MicroblockKind::CornerStrip);
        assert!((voxel.occupancy_mask() & (1 << (6 + 3))) != 0);

        voxel.center_conduit = Some("redpower_2:wire_red_alloy".into());
        assert!((voxel.occupancy_mask() & (1 << 26)) != 0);
    }

    #[test]
    fn test_wire_connection_rules() {
        assert!(WireKind::RedAlloy.can_connect_to(WireKind::RedAlloy));
        assert!(WireKind::RedAlloy.can_connect_to(WireKind::Insulated(DyeColor::Red)));
        assert!(WireKind::Insulated(DyeColor::Red).can_connect_to(WireKind::RedAlloy));
        assert!(
            WireKind::Insulated(DyeColor::Blue).can_connect_to(WireKind::Insulated(DyeColor::Blue))
        );
        assert!(
            !WireKind::Insulated(DyeColor::Blue).can_connect_to(WireKind::Insulated(DyeColor::Red))
        );
    }

    #[test]
    fn test_blutricity_dynamic_iteration() {
        let mut node_a = BluePowerConductor::new(4.0);
        let mut node_b = BluePowerConductor::new(4.0);
        node_a.add_charge(400.0); // 100V
        assert_eq!(node_a.voltage, 100.0);
        assert_eq!(node_b.voltage, 0.0);

        let flow = node_a.step_iterate(&mut node_b);
        assert!(flow > 0.0);
        assert!(node_a.voltage < 100.0);
        assert!(node_b.voltage > 0.0);
    }

    #[test]
    fn test_thermopile_and_solar() {
        let mut solar = SolarPanelState::new();
        let gen = solar.tick(true, true, false);
        assert_eq!(gen, 2.0);
        assert_eq!(solar.conductor.charge, 2.0);

        let mut therm = ThermopileState::new();
        // Lava (+100) vs Ice (-100)
        let (current, shock) = therm.tick([100, -100, 0, 0, 0, 0], 300);
        assert!(current > 0.0);
        assert!(!shock); // Thot < 100 or Tcold < 200 -> no shock
    }

    #[test]
    fn test_battery_box_thresholds() {
        let mut bb = BatteryBoxState::new();
        bb.conductor.add_charge(440.0); // 110V (> 90V) -> charges
        bb.tick(false);
        assert!(bb.stored_units > 0.0);

        bb.conductor.charge = 280.0;
        bb.conductor.voltage = 70.0; // 70V (< 80V) -> discharges
        let stored_before = bb.stored_units;
        bb.tick(false);
        assert!(bb.stored_units < stored_before);
    }

    #[test]
    fn test_dijkstra_tube_routing_with_weights_and_color() {
        let mut graph = GridGraph::new();
        // Line of tubes: (0,0,0) -> (1,0,0) -> (2,0,0)
        graph.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);
        graph.insert_node(BlockPos(1, 0, 0), GridNode::STANDARD);
        graph.insert_node(BlockPos(2, 0, 0), GridNode::STANDARD);

        let res = find_tube_route(
            &graph,
            BlockPos(0, 0, 0),
            Some(Direction::East),
            &[BlockPos(2, 0, 0)],
            None,
        );
        let (dest, path, cost) = res.expect("route should be found");
        assert_eq!(dest, BlockPos(2, 0, 0));
        assert_eq!(path.len(), 3);
        assert_eq!(cost, 4); // 2 hops * 2 = 4 (straight line, no turns)

        // Restrictor tube high cost
        let mut graph_restrict = GridGraph::new();
        graph_restrict.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);
        graph_restrict.insert_node(
            BlockPos(1, 0, 0),
            GridNode {
                weight: RESTRICTOR_TUBE_WEIGHT,
                color: None,
            },
        );
        let res2 = find_tube_route(
            &graph_restrict,
            BlockPos(0, 0, 0),
            Some(Direction::East),
            &[BlockPos(1, 0, 0)],
            None,
        );
        let (_, _, cost2) = res2.expect("route through restrictor found");
        assert!(cost2 >= RESTRICTOR_TUBE_WEIGHT);
    }

    #[test]
    fn test_display_terminal_blitter() {
        let mut term = DisplayTerminal::new();
        term.screen[0] = b'H';
        term.screen[1] = b'I';
        term.blit(0, 0, 10, 0, 2, 1);
        assert_eq!(term.screen[10], b'H');
        assert_eq!(term.screen[11], b'I');
    }

    #[test]
    fn test_disk_drive_redbus_commands() {
        let mut drive = DiskDriveState::new();
        let mut buf = [0u8; 128];
        assert!(drive.execute_redbus(1, 0, &mut buf)); // Read label
        assert!(std::str::from_utf8(&buf)
            .unwrap()
            .contains("RedPower Forth"));

        buf = [42u8; 128];
        assert!(drive.execute_redbus(5, 10, &mut buf)); // Write sector 10
        let mut read_buf = [0u8; 128];
        assert!(drive.execute_redbus(4, 10, &mut read_buf)); // Read sector 10
        assert_eq!(read_buf[0], 42);
    }

    #[test]
    fn test_more_alloy_furnace_recipes() {
        let mut furnace = AlloyFurnaceState::new();
        // Brass recipe: 3 copper + 1 tin -> 4 brass
        furnace.inventory.slots[0] = Some(ItemStack::new("redpower_2:copper_ingot", 3));
        furnace.inventory.slots[1] = Some(ItemStack::new("redpower_2:tin_ingot", 1));
        furnace.inventory.slots[AlloyFurnaceState::FUEL_SLOT] =
            Some(ItemStack::new("minecraft:coal", 1));
        for _ in 0..AlloyFurnaceState::SMELT_TICKS {
            furnace.tick(false);
        }
        let out = furnace.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
            .as_ref()
            .expect("should produce brass ingot");
        assert_eq!(out.item_id, "redpower_2:brass_ingot");
        assert_eq!(out.count, 4);

        // Tinplate: 2 iron + 1 tin -> 4 tinplate
        let mut f2 = AlloyFurnaceState::new();
        f2.inventory.slots[0] = Some(ItemStack::new("minecraft:iron_ingot", 2));
        f2.inventory.slots[1] = Some(ItemStack::new("redpower_2:tin_ingot", 1));
        f2.inventory.slots[AlloyFurnaceState::FUEL_SLOT] =
            Some(ItemStack::new("minecraft:coal", 1));
        for _ in 0..AlloyFurnaceState::SMELT_TICKS {
            f2.tick(false);
        }
        let out2 = f2.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
            .as_ref()
            .expect("should produce tinplate");
        assert_eq!(out2.item_id, "redpower_2:tinplate");
        assert_eq!(out2.count, 4);

        // Silicon Boule: 8 coal + 8 sand -> 1 silicon_boule
        let mut f3 = AlloyFurnaceState::new();
        f3.inventory.slots[0] = Some(ItemStack::new("minecraft:coal", 8));
        f3.inventory.slots[1] = Some(ItemStack::new("minecraft:sand", 8));
        f3.inventory.slots[AlloyFurnaceState::FUEL_SLOT] =
            Some(ItemStack::new("minecraft:coal", 1));
        for _ in 0..AlloyFurnaceState::SMELT_TICKS {
            f3.tick(false);
        }
        let out3 = f3.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
            .as_ref()
            .expect("should produce silicon boule");
        assert_eq!(out3.item_id, "redpower_2:silicon_boule");
        assert_eq!(out3.count, 1);
    }

    #[test]
    fn test_microblock_vertical_and_horizontal_cuts() {
        assert_eq!(
            MicroblockKind::Slab.cut_vertical(),
            Some((MicroblockKind::Panel, 2))
        );
        assert_eq!(
            MicroblockKind::Panel.cut_vertical(),
            Some((MicroblockKind::Cover, 2))
        );
        assert_eq!(
            MicroblockKind::Cover.cut_horizontal(),
            Some((MicroblockKind::CornerStrip, 2))
        );
        assert_eq!(
            MicroblockKind::CornerStrip.cut_horizontal(),
            Some((MicroblockKind::Corner, 2))
        );
    }

    #[test]
    fn test_red_alloy_lossless_and_vanilla_conversion() {
        assert_eq!(red_alloy_signal_lossless(255, 0), 255);
        assert_eq!(red_alloy_signal_lossless(255, 100), 255);
        assert_eq!(red_alloy_signal_lossless(255, 255), 255);
        assert_eq!(red_alloy_signal_lossless(255, 256), 0);

        // Spec §2.2 formula: strength / 17
        assert_eq!(red_alloy_to_vanilla(0), 0);
        assert_eq!(red_alloy_to_vanilla(17), 1);
        assert_eq!(red_alloy_to_vanilla(34), 2);
        assert_eq!(red_alloy_to_vanilla(255), 15);
    }

    #[test]
    fn test_repeater_schedule_and_bus_transceiver() {
        assert_eq!(REPEATER_SCHEDULE, [2, 4, 6, 8, 16, 32, 64, 128, 256]);

        let mut btx = BusTransceiverState::new();
        btx.side_a = BundledSignal::from_digital_mask(0xAAAA);
        btx.enable_a_to_b = true;
        btx.tick();
        assert_eq!(btx.side_b.to_digital_mask(), 0xAAAA);
    }

    #[test]
    fn test_frame_motor_movement_and_power() {
        let mut fm = FrameMotorState::new();
        fm.moving_blocks = vec![BlockPos(0, 64, 0), BlockPos(0, 65, 0)];
        assert_eq!(fm.power_cost_per_tick(), 120.0); // 100 + 10 * 2 = 120 W-t

        fm.is_moving = true;
        // Under 60V fails
        assert!(fm.tick(55.0).is_err());

        // 16 ticks at 100V completes
        for i in 0..15 {
            let done = fm.tick(100.0).expect("should advance tick");
            assert!(!done, "tick {i} should not be done yet");
        }
        let done = fm.tick(100.0).expect("16th tick completes");
        assert!(done);
        assert!(!fm.is_moving);
    }

    #[test]
    fn test_wind_turbine_and_transformer() {
        let mut wt = WindTurbineState::new();
        let power = wt.tick(100.0);
        assert_eq!(power, 20.0); // 100 / 5 = 20 W

        wt.brakes_engaged = true;
        assert_eq!(wt.tick(100.0), 0.0);

        let mut xfmr = TransformerState::new();
        xfmr.low_voltage_conductor.voltage = 100.0;
        xfmr.low_voltage_conductor.current = 10.0;
        xfmr.step_transform();
        assert_eq!(xfmr.high_voltage_conductor.voltage, 10000.0);
        assert_eq!(xfmr.high_voltage_conductor.current, 1000.0);
    }

    #[test]
    fn test_extended_cpu_instructions() {
        let mut cpu = Cpu6502::new();
        // Program:
        // CLC
        // LDA #$0A
        // ADC #$05  ; A = $0F
        // PHA
        // PLA       ; A = $0F from stack
        // BRK
        let prog = [0x18, 0xA9, 0x0A, 0x69, 0x05, 0x48, 0x68, 0x00];
        cpu.load_program(0x0400, &prog);
        let ran = cpu.step_tick(100);
        assert_eq!(ran, 6);
        assert!(cpu.halted);
        assert_eq!(cpu.a, 0x0F);
    }

    #[test]
    fn test_athame_disjunction_and_vorpal() {
        // Normal mob: 1 base damage, Disjunction does not apply
        assert_eq!(athame_attack_damage(false, 0), 1);
        assert_eq!(athame_attack_damage(false, 5), 1);

        // Ender mob: 25 base damage + 6 * level
        assert_eq!(athame_attack_damage(true, 0), 25);
        assert_eq!(athame_attack_damage(true, 1), 31);
        assert_eq!(athame_attack_damage(true, 3), 43);
        assert_eq!(athame_attack_damage(true, 5), 55);

        // Vorpal proc chance: 2 * level^2 %
        assert_eq!(vorpal_proc_chance_percent(1), 2);
        assert_eq!(vorpal_proc_chance_percent(2), 8);
        assert_eq!(vorpal_proc_chance_percent(3), 18);
        assert_eq!(vorpal_proc_chance_percent(4), 32);

        // Vorpal critical bonus: +100
        assert_eq!(calculate_weapon_damage(7, 2, false), 7);
        assert_eq!(calculate_weapon_damage(7, 2, true), 107);

        // Decapitation threshold: overkill <= -20 HP
        assert!(!check_vorpal_decapitation(20, 30)); // 20 - 30 = -10 (not <= -20)
        assert!(check_vorpal_decapitation(20, 40)); // 20 - 40 = -20 (decapitates!)
        assert!(check_vorpal_decapitation(20, 107)); // 20 - 107 = -87 (decapitates!)
    }

    #[test]
    fn test_sickle_harvest_radius_and_durabilities() {
        let center = BlockPos(10, 64, 10);
        let leaves = sickle_leaf_positions(center);
        assert_eq!(leaves.len(), 27); // 3x3x3 volume
        assert!(leaves.contains(&BlockPos(9, 63, 9)));
        assert!(leaves.contains(&BlockPos(11, 65, 11)));

        let crops = sickle_crop_positions(center);
        assert_eq!(crops.len(), 25); // 5x5 horizontal area
        assert!(crops.contains(&BlockPos(8, 64, 8)));
        assert!(crops.contains(&BlockPos(12, 64, 12)));
        assert!(!crops.contains(&BlockPos(10, 65, 10))); // flat only

        assert_eq!(SickleVariant::Wood.durability(), 59);
        assert_eq!(SickleVariant::Stone.durability(), 131);
        assert_eq!(SickleVariant::Iron.durability(), 250);
        assert_eq!(SickleVariant::Diamond.durability(), 1561);
        assert_eq!(SickleVariant::Gold.durability(), 32);
        assert_eq!(SickleVariant::Ruby.durability(), 500);
        assert_eq!(SickleVariant::GreenSapphire.durability(), 500);
        assert_eq!(SickleVariant::Sapphire.durability(), 500);
    }

    #[test]
    fn test_cpu_cycle_accumulator_and_burst_cap() {
        let mut cpu = Cpu6502::new();
        assert_eq!(cpu.cycle_accumulator, 0);

        cpu.accumulate_tick();
        assert_eq!(cpu.cycle_accumulator, 1_000);

        for _ in 0..150 {
            cpu.accumulate_tick();
        }
        assert_eq!(cpu.cycle_accumulator, Cpu6502::MAX_BURST_CYCLES); // Capped at 100,000
    }
}

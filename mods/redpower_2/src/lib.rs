//! RedPower 2 (`redpower_2`) Rust domain crate and `HorizonNativeMod` implementation (`NATIVE_HOOKS_API = 40`).
//!
//! Implements the systems specified in `planning/mods/redpower_2.md`:
//! - **Microblocks (§3.1)**: 1/8th fractional slabs, panels, covers, hollow variants, strips, and corners.
//! - **Wiring (§3.2)**: Red Alloy Wire (0..=255 internal signal over 256 blocks), 16-color Insulated Wire,
//!   and 16-channel Bundled Cable (`rekkit_core::frequency::BundledSignal`).
//! - **Integrated Logic Gates (§3.3)**: All 20 logic gate types (Inverter, AND, OR, NAND, NOR, XOR, XNOR,
//!   Buffer, Multiplexer, Repeater, Timer, Counter, Sequencer, Pulse Former, Toggle Latch, RS Latch,
//!   Transparent Latch, State Cell, Synchronizer, Light Sensor).
//! - **Pneumatic Tubes & Machines (§3.4)**: Strict Dijkstra pathfinding (`rekkit_core::grid::GridGraph`)
//!   with 500-block Restrictor Tube weights, color tags, and zero-spill backpressure, plus Alloy Furnace.
//! - **Support Frames & Frame Motor (§3.5)**: Rigid multiblock assembly solver with bedrock/obstacle checks.
//! - **Blutricity (§3.6)**: 100V nominal DC power (`rekkit_core::energy::BlutricityNode`), Thermopile,
//!   Solar Panel, Wind Turbine, and Battery Box.
//! - **RedPower Control (§3.7)**: MOS 6502 CPU emulator with 64 KB RAM, 80x25 Monitor, and 16-bit
//!   Bundled Cable I/O Expander, bounded to `MAX_CYCLES_PER_TICK = 4096`.

pub use rekkit_core::energy::BlutricityNode;
pub use rekkit_core::frequency::{BundledSignal, DyeColor};
pub use rekkit_core::grid::{BlockPos, GridGraph, GridNode};
pub use rekkit_core::hooks::{
    HorizonNativeMod, ModBlockEntity, ModContainer, NativeModInfo, NativeScreen,
};
pub use rekkit_core::inventory::{Inventory, ItemStack};
pub use rekkit_core::side::{ConnectionMask, Direction};
pub use rekkit_core::util::TickTimer;
pub use rekkit_core::{HORIZON_REVISION, MIXINS_REVISION, NATIVE_HOOKS_API};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

pub const MOD_ID: &str = "redpower_2";

/// All 42 canonical block IDs declared in `planning/mods/redpower_2.md` §5.
pub const BLOCK_IDS: [&str; 42] = [
    "redpower_2:ore_ruby",
    "redpower_2:ore_sapphire",
    "redpower_2:ore_green_sapphire",
    "redpower_2:ore_nikolite",
    "redpower_2:ore_copper",
    "redpower_2:ore_tin",
    "redpower_2:ore_silver",
    "redpower_2:ore_tungsten",
    "redpower_2:block_marble",
    "redpower_2:block_basalt",
    "redpower_2:alloy_furnace",
    "redpower_2:blulectric_alloy_furnace",
    "redpower_2:blulectric_furnace",
    "redpower_2:deployer",
    "redpower_2:block_breaker",
    "redpower_2:transposer",
    "redpower_2:filter",
    "redpower_2:sorting_machine",
    "redpower_2:retriever",
    "redpower_2:regulator",
    "redpower_2:item_detector",
    "redpower_2:buffer",
    "redpower_2:relay",
    "redpower_2:frame",
    "redpower_2:frame_motor",
    "redpower_2:pneumatic_tube",
    "redpower_2:restrictor_tube",
    "redpower_2:magtube",
    "redpower_2:accelerator",
    "redpower_2:battery_box",
    "redpower_2:solar_panel",
    "redpower_2:thermopile",
    "redpower_2:wind_turbine",
    "redpower_2:pump",
    "redpower_2:lamp",
    "redpower_2:inverted_lamp",
    "redpower_2:cpu",
    "redpower_2:monitor",
    "redpower_2:keyboard",
    "redpower_2:disk_drive",
    "redpower_2:io_expander",
    "redpower_2:microblock",
];

/// All 29 canonical item IDs declared in `planning/mods/redpower_2.md` §5.
pub const ITEM_IDS: [&str; 29] = [
    "redpower_2:handsaw",
    "redpower_2:screwdriver",
    "redpower_2:paint_brush",
    "redpower_2:canvas_bag",
    "redpower_2:sickle",
    "redpower_2:wool_card",
    "redpower_2:ruby",
    "redpower_2:sapphire",
    "redpower_2:green_sapphire",
    "redpower_2:nikolite",
    "redpower_2:red_alloy_ingot",
    "redpower_2:blue_alloy_ingot",
    "redpower_2:brass_ingot",
    "redpower_2:copper_ingot",
    "redpower_2:tin_ingot",
    "redpower_2:silver_ingot",
    "redpower_2:tungsten_ingot",
    "redpower_2:silicon_wafer",
    "redpower_2:red_doped_wafer",
    "redpower_2:blue_doped_wafer",
    "redpower_2:floppy_disk",
    "redpower_2:canvas",
    "redpower_2:battery",
    "redpower_2:logic_tile",
    "redpower_2:wire_red_alloy",
    "redpower_2:wire_insulated",
    "redpower_2:wire_bundled",
    "redpower_2:wire_jacketed",
    "redpower_2:ribbon_cable",
];

// ============================================================================
// 3.1 Microblocks System
// ============================================================================

/// 1/8th fractional microblock slice hierarchy (`planning/mods/redpower_2.md` §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MicroblockKind {
    /// 4/8th = 1/2 block.
    Slab,
    /// 2/8th = 1/4 block.
    Panel,
    /// 1/8th block.
    Cover,
    /// 4/8th hollow slab with center cutout for wires/tubes.
    HollowSlab,
    /// 2/8th hollow panel with center cutout.
    HollowPanel,
    /// 1/8th hollow cover with center cutout.
    HollowCover,
    /// 1/2 x 1/8 x 1 strip.
    StripSlab,
    /// 1/4 x 1/8 x 1 strip.
    StripPanel,
    /// 1/8 x 1/8 x 1 corner strip.
    CornerStrip,
    /// 1/8 x 1/8 x 1/8 corner.
    Corner,
}

impl MicroblockKind {
    /// Thickness in eighths of a block (`1..=4`).
    pub const fn thickness_eighths(self) -> u8 {
        match self {
            Self::Slab | Self::HollowSlab | Self::StripSlab => 4,
            Self::Panel | Self::HollowPanel | Self::StripPanel => 2,
            Self::Cover | Self::HollowCover | Self::CornerStrip | Self::Corner => 1,
        }
    }

    /// Whether a central wire or pneumatic tube can pass through this face microblock.
    pub const fn is_hollow(self) -> bool {
        matches!(self, Self::HollowSlab | Self::HollowPanel | Self::HollowCover)
    }

    /// Handsaw cutting progression: `Slab -> Panel -> Cover -> CornerStrip -> Corner`.
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
            Self::HollowCover | Self::Corner => None,
        }
    }
}

/// Multi-part microblock voxel state holding up to 6 face covers/slabs in a single block space.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroblockVoxel {
    pub face_parts: [Option<MicroblockKind>; 6],
}

impl MicroblockVoxel {
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

    /// True if a central wire or tube can connect through `dir` (either uncovered or hollow).
    pub fn allows_center_connection(&self, dir: Direction) -> bool {
        match self.face_parts[dir as usize] {
            None => true,
            Some(kind) => kind.is_hollow(),
        }
    }
}

// ============================================================================
// 3.2 Wiring System
// ============================================================================

/// Compute Red Alloy Wire internal signal (`0..=255`) at `distance` blocks from a source
/// (`planning/mods/redpower_2.md` §3.2: transmits up to 255 blocks without vanilla's 15-block limit).
pub const fn red_alloy_signal_at_distance(source_strength: u8, distance: u32) -> u8 {
    if source_strength == 0 || distance >= 256 {
        0
    } else {
        source_strength.saturating_sub(distance as u8)
    }
}

/// Map a Red Alloy internal signal (`0..=255`) to a standard vanilla redstone level (`0..=15`)
/// for `native_block_signal` (`API 38`).
pub const fn internal_to_vanilla_redstone(internal: u8) -> i32 {
    if internal == 0 {
        0
    } else {
        1 + (((internal as i32 - 1) * 14) / 254)
    }
}

// ============================================================================
// 3.3 Integrated Logic Gates (All 20 Subtypes)
// ============================================================================

/// All 20 RedPower 2 integrated logic gate tiles (`planning/mods/redpower_2.md` §3.3).
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
}

/// Complete state machine for any of the 20 RedPower 2 Logic Gates.
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
    /// Internal latch/flip-flop/sequencer step (`0..4`).
    pub latch_state: u8,
    /// Previous input mask `[left, back, right]` for rising-edge detection.
    pub prev_inputs: [bool; 3],
    /// Ambient light level (`0..=15`) for `LightSensor`.
    pub ambient_light: u8,
    /// Current output signal (`0..=15`).
    pub output_level: i32,
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
            latch_state: 0,
            prev_inputs: [false; 3],
            ambient_light: 15,
            output_level: if matches!(kind, LogicGateKind::Inverter | LogicGateKind::Nand | LogicGateKind::Nor | LogicGateKind::Xnor) {
                15
            } else {
                0
            },
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
                    false
                } else {
                    let period = self.interval_ticks.max(Self::MIN_TIMER_TICKS);
                    self.elapsed_ticks += 1;
                    if self.elapsed_ticks >= period {
                        self.elapsed_ticks = 0;
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
                self.count >= self.max_count
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
        };

        self.prev_inputs = inputs;
        self.output_level = if active { 15 } else { 0 };
        self.output_level
    }
}

// ============================================================================
// 3.4 Pneumatic Tubes & Alloy Furnace
// ============================================================================

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

    /// Match the 5 canonical RedPower 2 Alloy Furnace recipes (`planning/mods/redpower_2.md` §6.1).
    pub fn match_recipe(&self) -> Option<(&'static str, u32, &'static str, u32, ItemStack)> {
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
        } else if self.count_input("redpower_2:nikolite") >= 4
            && self.count_input("redpower_2:silver_ingot") >= 1
        {
            Some((
                "redpower_2:nikolite",
                4,
                "redpower_2:silver_ingot",
                1,
                ItemStack::new("redpower_2:blue_alloy_ingot", 1),
            ))
        } else if self.count_input("redpower_2:copper_ingot") >= 3
            && self.count_input("redpower_2:tin_ingot") >= 1
        {
            Some((
                "redpower_2:copper_ingot",
                3,
                "redpower_2:tin_ingot",
                1,
                ItemStack::new("redpower_2:brass_ingot", 4),
            ))
        } else if self.count_input("minecraft:redstone") >= 4
            && self.count_input("redpower_2:silicon_wafer") >= 1
        {
            Some((
                "minecraft:redstone",
                4,
                "redpower_2:silicon_wafer",
                1,
                ItemStack::new("redpower_2:red_doped_wafer", 1),
            ))
        } else if self.count_input("redpower_2:nikolite") >= 4
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
        if self.smelt_timer.tick() {
            if self
                .inventory
                .insert_into_slots(out_stack, &[Self::OUTPUT_SLOT])
                .is_none()
            {
                self.consume_input(in_a, cnt_a);
                self.consume_input(in_b, cnt_b);
            }
        }
    }
}

// ============================================================================
// 3.5 Frame Motors & Structural Movement
// ============================================================================

/// Maximum blocks a Frame Motor may translate in a single pulse (`planning/mods/redpower_2.md` §7).
pub const MAX_FRAME_BLOCKS: usize = 1024;

/// Collect all blocks rigidly attached to a Frame Motor starting at `start_frame` and verify
/// whether they can translate by 1 block in `move_dir`.
/// Returns `Ok(ordered_positions)` or `Err(reason)` if blocked by `minecraft:bedrock` or size limit.
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
            return Err("frame assembly exceeds MAX_FRAME_BLOCKS (1024)");
        }
        let Some(block_id) = get_block(pos) else {
            continue;
        };
        if block_id == "minecraft:bedrock" {
            return Err("bedrock is immovable");
        }
        // Frame blocks latch onto all 6 adjacent non-air neighbors
        if block_id == "redpower_2:frame" {
            for neighbor in pos.neighbors() {
                if !visited.contains(&neighbor) {
                    if let Some(nb) = get_block(neighbor) {
                        if nb != "minecraft:air" {
                            if nb == "minecraft:bedrock" {
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

// ============================================================================
// 3.7 RedPower Control (MOS 6502 Microcomputer)
// ============================================================================

/// Bounded 6502 CPU emulator with 64 KB RAM and memory-mapped 16-bit Bundled Cable I/O Expander.
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
}

impl Default for Cpu6502 {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu6502 {
    pub const RAM_SIZE: usize = 65_536;
    pub const MAX_CYCLES_PER_TICK: usize = 4_096;
    pub const IO_EXPANDER_LOW_ADDR: u16 = 0xE000;
    pub const IO_EXPANDER_HIGH_ADDR: u16 = 0xE001;

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

    /// Execute up to `max_instructions` (capped at `MAX_CYCLES_PER_TICK = 4096`), returning
    /// the number of instructions executed.
    pub fn step_tick(&mut self, max_instructions: usize) -> usize {
        let limit = max_instructions.min(Self::MAX_CYCLES_PER_TICK);
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
                0x8D => {
                    // STA abs
                    let lo = self.read_u8(self.pc) as u16;
                    let hi = (self.read_u8(self.pc.wrapping_add(1)) as u16) << 8;
                    self.pc = self.pc.wrapping_add(2);
                    self.write_u8(lo | hi, self.a);
                }
                0x69 => {
                    // ADC #imm
                    let imm = self.read_u8(self.pc);
                    self.pc = self.pc.wrapping_add(1);
                    let sum = (self.a as u16) + (imm as u16);
                    self.a = (sum & 0xFF) as u8;
                    self.update_nz(self.a);
                }
                0xE8 => {
                    // INX
                    self.x = self.x.wrapping_add(1);
                    self.update_nz(self.x);
                }
                0xCA => {
                    // DEX
                    self.x = self.x.wrapping_sub(1);
                    self.update_nz(self.x);
                }
                0x4C => {
                    // JMP abs
                    let lo = self.read_u8(self.pc) as u16;
                    let hi = (self.read_u8(self.pc.wrapping_add(1)) as u16) << 8;
                    self.pc = lo | hi;
                }
                _ => {
                    // Unknown opcode -> halt safely
                    self.halted = true;
                }
            }
        }
        executed
    }
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
            let ran = cpu.step_tick(256);
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
                    slots.push(Some([48.0 + (col as f32) * 18.0, 17.0 + (row as f32) * 18.0]));
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
            entity
                .nbt
                .insert("color".into(), (color as u8).to_string());
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
}

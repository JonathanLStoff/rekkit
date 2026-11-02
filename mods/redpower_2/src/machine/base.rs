//! Foundational machine block infrastructure for RedPower 2 (`redpower_2`).
//!
//! Mirrors:
//! - `BlockMachine` (meta 0..15) and `BlockMachine2` (meta 0..1)
//! - `BlockMachinePanel` (meta 0..4)
//! - `TileMachine` and `TileMachinePanel` base tile entities
//! - 6-direction rotation handling (`IRotatable`)
//! - Active, powered, delay, and charged state flags (`ps` byte)
//! - Packet synchronization (subId 7 for TileMachine, subId 8 for TileMachinePanel)
//! - `IFrameSupport` callbacks and packet serialization
//! - Sided Redstone and Pneumatic Tube connectivity interfaces

use crate::TubePayload;
use rekkit_core::grid::BlockPos;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Machine packet sub-identifiers used in `Packet211TileDesc`.
pub const SUBID_TILE_MACHINE: u8 = 7;
pub const SUBID_TILE_MACHINE_PANEL: u8 = 8;

// ============================================================================
// 1. Machine Metadata & Kinds
// ============================================================================

/// Metadata variants for `BlockMachine` (`rpmachine`, meta 0..15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MachineKind {
    Deployer = 0,
    BlockBreaker = 1,
    Transposer = 2,
    Filter = 3,
    ItemDetector = 4,
    SortingMachine = 5,
    BatteryBox = 6,
    FrameMotor = 7,
    Retriever = 8,
    WindTurbine = 9,
    Regulator = 10,
    Thermopile = 11,
    Igniter = 12,
    Assembler = 13,
    Ejector = 14,
    Relay = 15,
}

impl MachineKind {
    pub const ALL: [Self; 16] = [
        Self::Deployer,
        Self::BlockBreaker,
        Self::Transposer,
        Self::Filter,
        Self::ItemDetector,
        Self::SortingMachine,
        Self::BatteryBox,
        Self::FrameMotor,
        Self::Retriever,
        Self::WindTurbine,
        Self::Regulator,
        Self::Thermopile,
        Self::Igniter,
        Self::Assembler,
        Self::Ejector,
        Self::Relay,
    ];

    pub const fn from_meta(meta: u8) -> Option<Self> {
        match meta & 0x0F {
            0 => Some(Self::Deployer),
            1 => Some(Self::BlockBreaker),
            2 => Some(Self::Transposer),
            3 => Some(Self::Filter),
            4 => Some(Self::ItemDetector),
            5 => Some(Self::SortingMachine),
            6 => Some(Self::BatteryBox),
            7 => Some(Self::FrameMotor),
            8 => Some(Self::Retriever),
            9 => Some(Self::WindTurbine),
            10 => Some(Self::Regulator),
            11 => Some(Self::Thermopile),
            12 => Some(Self::Igniter),
            13 => Some(Self::Assembler),
            14 => Some(Self::Ejector),
            15 => Some(Self::Relay),
            _ => None,
        }
    }

    pub const fn to_meta(self) -> u8 {
        self as u8
    }

    pub const fn block_id(self) -> &'static str {
        match self {
            Self::Deployer => "redpower_2:deployer",
            Self::BlockBreaker => "redpower_2:block_breaker",
            Self::Transposer => "redpower_2:transposer",
            Self::Filter => "redpower_2:filter",
            Self::ItemDetector => "redpower_2:item_detector",
            Self::SortingMachine => "redpower_2:sorting_machine",
            Self::BatteryBox => "redpower_2:battery_box",
            Self::FrameMotor => "redpower_2:frame_motor",
            Self::Retriever => "redpower_2:retriever",
            Self::WindTurbine => "redpower_2:wind_turbine",
            Self::Regulator => "redpower_2:regulator",
            Self::Thermopile => "redpower_2:thermopile",
            Self::Igniter => "redpower_2:igniter",
            Self::Assembler => "redpower_2:assembler",
            Self::Ejector => "redpower_2:ejector",
            Self::Relay => "redpower_2:relay",
        }
    }

    pub fn from_block_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.block_id() == id)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Deployer => "Deployer",
            Self::BlockBreaker => "Block Breaker",
            Self::Transposer => "Transposer",
            Self::Filter => "Filter",
            Self::ItemDetector => "Item Detector",
            Self::SortingMachine => "Sorting Machine",
            Self::BatteryBox => "Battery Box",
            Self::FrameMotor => "Frame Motor",
            Self::Retriever => "Retriever",
            Self::WindTurbine => "Wind Turbine",
            Self::Regulator => "Regulator",
            Self::Thermopile => "Thermopile",
            Self::Igniter => "Igniter",
            Self::Assembler => "Assembler",
            Self::Ejector => "Ejector",
            Self::Relay => "Relay",
        }
    }

    pub const fn item_name(self) -> &'static str {
        match self {
            Self::Deployer => "rpdeploy",
            Self::BlockBreaker => "rpbreaker",
            Self::Transposer => "rptranspose",
            Self::Filter => "rpfilter",
            Self::ItemDetector => "rpitemdet",
            Self::SortingMachine => "rpsorter",
            Self::BatteryBox => "rpbatbox",
            Self::FrameMotor => "rpmotor",
            Self::Retriever => "rpretriever",
            Self::WindTurbine => "rpkgen",
            Self::Regulator => "rpregulate",
            Self::Thermopile => "rpthermo",
            Self::Igniter => "rpignite",
            Self::Assembler => "rpassemble",
            Self::Ejector => "rpeject",
            Self::Relay => "rprelay",
        }
    }

    pub const fn has_gui(self) -> bool {
        matches!(
            self,
            Self::Deployer
                | Self::Filter
                | Self::ItemDetector
                | Self::SortingMachine
                | Self::BatteryBox
                | Self::Retriever
                | Self::Regulator
                | Self::Assembler
                | Self::Ejector
                | Self::Relay
        )
    }

    pub const fn default_slot_count(self) -> usize {
        match self {
            Self::Deployer => 9,
            Self::Filter => 9,
            Self::ItemDetector => 9,
            Self::SortingMachine => 40,
            Self::BatteryBox => 2,
            Self::Retriever => 9,
            Self::Regulator => 27,
            Self::Assembler => 34,
            Self::Ejector => 9,
            Self::Relay => 9,
            _ => 0,
        }
    }

    /// Calculate default texture indices `[bottom, top, north, south, west, east]`
    /// before rotation, precisely replicating Java `RenderMachine.renderWorldBlock`.
    pub const fn unrotated_texture_indices(
        self,
        active: bool,
        powered: bool,
        delay: bool,
        charged: bool,
    ) -> [u32; 6] {
        let act = if active { 1 } else { 0 };
        match self {
            Self::Deployer => [48, 53 + act, 56, 56, 55, 55],
            Self::ItemDetector => {
                let t1 = 96 + act;
                let t2 = t1 + 2 + if powered { 2 } else { 0 };
                [102, 103, t1, t1, t2, t2]
            }
            Self::SortingMachine => {
                let act_idx = if charged {
                    if active {
                        2
                    } else {
                        1
                    }
                } else {
                    0
                };
                let t1 = 116 + if charged { 1 } else { 0 } + if active { 2 } else { 0 };
                [113 + act_idx, 112, t1, t1, t1, t1]
            }
            Self::Retriever => {
                let tex = 120 + if charged { 1 } else { 0 } + if delay || active { 2 } else { 0 };
                [124, 125, tex, tex, tex, tex]
            }
            Self::Regulator => {
                let t1 = 104 + act;
                let t2 = t1 + 2 + if powered { 2 } else { 0 };
                [102, 103, t1, t1, t2, t2]
            }
            Self::Igniter => [48, 164 + act, 167, 167, 166, 166],
            Self::Assembler => [172 + act, 168 + act, 171, 171, 170, 170],
            Self::Ejector => [58, 89, 91 + act, 91 + act, 90, 90],
            Self::Relay => [58, 89, 93 + act, 93 + act, 90, 90],
            Self::Filter => {
                let tex = 59 + act + 2; // md == 3 adds +2 -> 61 + act
                [58, 57, tex, tex, tex, tex]
            }
            _ => {
                let tex = 59 + act;
                [58, 57, tex, tex, tex, tex]
            }
        }
    }
}

/// Metadata variants for `BlockMachine2` (`rpmachine2`, meta 0..1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Machine2Kind {
    Sortron = 0,
    Manager = 1,
}

impl Machine2Kind {
    pub const ALL: [Self; 2] = [Self::Sortron, Self::Manager];

    pub const fn from_meta(meta: u8) -> Option<Self> {
        match meta & 0x01 {
            0 => Some(Self::Sortron),
            1 => Some(Self::Manager),
            _ => None,
        }
    }

    pub const fn to_meta(self) -> u8 {
        self as u8
    }

    pub const fn block_id(self) -> &'static str {
        match self {
            Self::Sortron => "redpower_2:sortron",
            Self::Manager => "redpower_2:manager",
        }
    }

    pub fn from_block_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.block_id() == id)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Sortron => "Sortron",
            Self::Manager => "Manager",
        }
    }

    pub const fn item_name(self) -> &'static str {
        match self {
            Self::Sortron => "rpsortron",
            Self::Manager => "rpmanager",
        }
    }
}

/// Metadata variants for `BlockMachinePanel` (`machinePanel`, meta 0..4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum MachinePanelKind {
    SolarPanel = 0,
    Pump = 1,
    Accelerator = 2,
    Grate = 3,
    Transformer = 4,
}

impl MachinePanelKind {
    pub const ALL: [Self; 5] = [
        Self::SolarPanel,
        Self::Pump,
        Self::Accelerator,
        Self::Grate,
        Self::Transformer,
    ];

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

    pub const fn to_meta(self) -> u8 {
        self as u8
    }

    pub const fn block_id(self) -> &'static str {
        match self {
            Self::SolarPanel => "redpower_2:solar_panel",
            Self::Pump => "redpower_2:pump",
            Self::Accelerator => "redpower_2:accelerator",
            Self::Grate => "redpower_2:grate",
            Self::Transformer => "redpower_2:transformer",
        }
    }

    pub fn from_block_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.block_id() == id)
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::SolarPanel => "Solar Panel",
            Self::Pump => "Pump",
            Self::Accelerator => "Accelerator",
            Self::Grate => "Grate",
            Self::Transformer => "Transformer",
        }
    }

    pub const fn item_name(self) -> &'static str {
        match self {
            Self::SolarPanel => "rpsolar",
            Self::Pump => "rppump",
            Self::Accelerator => "rpaccel",
            Self::Grate => "rpgrate",
            Self::Transformer => "rptransformer",
        }
    }

    /// `ItemMachinePanel:55`: Solar Panel requires a solid top surface on the block below.
    pub const fn requires_solid_base(self) -> bool {
        matches!(self, Self::SolarPanel)
    }
}

// ============================================================================
// 2. Machine State Flags (`ps` byte)
// ============================================================================

/// Machine status flags packed into the single `ps` byte in RedPower 2.
///
/// Bit 0 (`0x01`): `active`
/// Bit 1 (`0x02`): `powered`
/// Bit 2 (`0x04`): `delay`
/// Bit 3 (`0x08`): `charged`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MachineFlags {
    pub active: bool,
    pub powered: bool,
    pub delay: bool,
    pub charged: bool,
}

impl MachineFlags {
    pub const NONE: Self = Self {
        active: false,
        powered: false,
        delay: false,
        charged: false,
    };

    pub const fn from_bits(bits: u8) -> Self {
        Self {
            active: (bits & 1) != 0,
            powered: (bits & 2) != 0,
            delay: (bits & 4) != 0,
            charged: (bits & 8) != 0,
        }
    }

    pub const fn to_bits(self) -> u8 {
        (if self.active { 1 } else { 0 })
            | (if self.powered { 2 } else { 0 })
            | (if self.delay { 4 } else { 0 })
            | (if self.charged { 8 } else { 0 })
    }
}

// ============================================================================
// 3. Shared Traits (IRotatable, IFrameSupport, Connectivity)
// ============================================================================

/// Multi-directional block rotation interface mirroring `com.eloraam.redpower.core.IRotatable`.
pub trait Rotatable {
    /// Maximum rotation value for this part (0 = cannot rotate).
    fn part_max_rotation(&self, part: u32, secondary: bool) -> u32;

    /// Current rotation value for this part.
    fn part_rotation(&self, part: u32, secondary: bool) -> u32;

    /// Update part rotation.
    fn set_part_rotation(&mut self, part: u32, secondary: bool, rot: u32);

    /// Cycle part rotation when right-clicked with Screwdriver (`ItemScrewdriver:61-66`).
    /// Returns `true` if rotation was cycled.
    fn rotate_with_screwdriver(&mut self, part: u32, secondary: bool) -> bool {
        let max_rot = self.part_max_rotation(part, secondary);
        if max_rot == 0 {
            return false;
        }
        let cur = self.part_rotation(part, secondary);
        let next = if cur + 1 > max_rot { 0 } else { cur + 1 };
        self.set_part_rotation(part, secondary, next);
        true
    }
}

/// RedPower 2 support frame structural movement interface mirroring `IFrameSupport`.
pub trait FrameSupport {
    /// Return the serialized frame packet payload (subId + rotation + flags).
    fn frame_packet(&self) -> Vec<u8>;

    /// Restore machine state from serialized frame payload.
    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String>;

    fn on_frame_refresh(&mut self) {}
    fn on_frame_pickup(&mut self) {}
    fn on_frame_drop(&mut self) {}
}

/// Sided redstone output & connection interface for machines.
pub trait RedstoneConnectable {
    fn is_powering_to(&self, _side: Direction) -> bool {
        false
    }
    fn can_connect_redstone(&self, _side: Direction) -> bool {
        true
    }
}

/// Sided pneumatic tube item entry & connection interface for machines.
pub trait TubeConnectable {
    fn can_connect_tube(&self, _side: Direction) -> bool {
        false
    }
    fn handle_tube_item(&mut self, _payload: TubePayload) -> bool {
        false
    }
}

// ============================================================================
// 4. Texture Orientation Mapping (RenderContext.orientTextures)
// ============================================================================

/// Orient 6 face textures given the block's facing direction, exactly replicating
/// `RenderContext.orientTextures(int down)`.
///
/// Base texture array order: `[Down, Up, North, South, West, East]`.
pub fn orient_textures(rotation: Direction, base_tex: [u32; 6]) -> [u32; 6] {
    let mut tex = base_tex;
    match rotation {
        Direction::Down => {
            // Case 0: identity
        }
        Direction::Up => {
            // Case 1: swap(0, 1), swap(4, 5)
            tex.swap(0, 1);
            tex.swap(4, 5);
        }
        Direction::North => {
            // Case 2: swap(0, 2), swap(1, 3), swap(0, 4), swap(1, 5)
            tex.swap(0, 2);
            tex.swap(1, 3);
            tex.swap(0, 4);
            tex.swap(1, 5);
        }
        Direction::South => {
            // Case 3: swap(0, 3), swap(1, 2), swap(0, 4), swap(1, 5)
            tex.swap(0, 3);
            tex.swap(1, 2);
            tex.swap(0, 4);
            tex.swap(1, 5);
        }
        Direction::West => {
            // Case 4: swap(0, 4), swap(1, 5), swap(2, 3)
            tex.swap(0, 4);
            tex.swap(1, 5);
            tex.swap(2, 3);
        }
        Direction::East => {
            // Case 5: swap(0, 5), swap(1, 4), swap(0, 1)
            tex.swap(0, 5);
            tex.swap(1, 4);
            tex.swap(0, 1);
        }
    }
    tex
}

// ============================================================================
// 5. TileMachine Base Implementation
// ============================================================================

/// Foundational 6-direction machine tile entity abstraction mirroring `TileMachine`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileMachine {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub metadata: u8,
}

impl TileMachine {
    pub fn new(metadata: u8) -> Self {
        Self {
            rotation: Direction::Down,
            flags: MachineFlags::NONE,
            metadata,
        }
    }

    pub fn with_rotation(mut self, rotation: Direction) -> Self {
        self.rotation = rotation;
        self
    }

    /// Compute facing direction from player placement coordinates, exactly replicating
    /// `TileMachine.getFacing(md ent)`.
    pub fn calculate_facing(
        player_pos: [f64; 3],
        player_yaw: f32,
        player_eye_offset: f64,
        block_pos: [i32; 3],
    ) -> Direction {
        let yawrx = ((player_yaw * 4.0 / 360.0 + 0.5).floor() as i32) & 3;
        let dx = (player_pos[0] - block_pos[0] as f64).abs();
        let dz = (player_pos[2] - block_pos[2] as f64).abs();
        if dx < 2.0 && dz < 2.0 {
            let p = player_pos[1] + 1.82 - player_eye_offset - block_pos[1] as f64;
            if p > 2.0 {
                return Direction::Down; // 0
            }
            if p < 0.0 {
                return Direction::Up; // 1
            }
        }
        match yawrx {
            0 => Direction::South, // 3
            1 => Direction::West,  // 4
            2 => Direction::North, // 2
            _ => Direction::East,  // 5
        }
    }

    /// Replicate `TileMachine.onBlockPlaced(ist, side, ent)`.
    pub fn on_block_placed(
        &mut self,
        player_pos: [f64; 3],
        player_yaw: f32,
        player_eye_offset: f64,
        block_pos: [i32; 3],
    ) {
        self.rotation = Self::calculate_facing(player_pos, player_yaw, player_eye_offset, block_pos);
    }

    /// Encode state into 2-byte packet payload: `[rotation, ps]`.
    pub fn write_to_packet(&self) -> [u8; 2] {
        [self.rotation as u8, self.flags.to_bits()]
    }

    /// Decode state from 2-byte packet payload: `[rotation, ps]`.
    pub fn read_from_packet(&mut self, pkt: [u8; 2]) {
        self.rotation = match pkt[0] {
            0 => Direction::Down,
            1 => Direction::Up,
            2 => Direction::North,
            3 => Direction::South,
            4 => Direction::West,
            5 => Direction::East,
            _ => Direction::Down,
        };
        self.flags = MachineFlags::from_bits(pkt[1]);
    }

    /// Write NBT tags `rot` and `ps`.
    pub fn write_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("rot".into(), (self.rotation as u8).to_string());
        nbt.insert("ps".into(), self.flags.to_bits().to_string());
    }

    /// Read NBT tags `rot` and `ps`.
    pub fn read_nbt(&mut self, nbt: &HashMap<String, String>) {
        if let Some(r) = nbt.get("rot").and_then(|v| v.parse::<u8>().ok()) {
            self.rotation = match r {
                0 => Direction::Down,
                1 => Direction::Up,
                2 => Direction::North,
                3 => Direction::South,
                4 => Direction::West,
                5 => Direction::East,
                _ => Direction::Down,
            };
        }
        if let Some(ps) = nbt.get("ps").and_then(|v| v.parse::<u8>().ok()) {
            self.flags = MachineFlags::from_bits(ps);
        }
    }

    /// Orient textures according to current rotation.
    pub fn orient_textures(&self, base_tex: [u32; 6]) -> [u32; 6] {
        orient_textures(self.rotation, base_tex)
    }
}

impl Rotatable for TileMachine {
    fn part_max_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if secondary {
            0
        } else {
            5
        }
    }

    fn part_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if secondary {
            0
        } else {
            self.rotation as u32
        }
    }

    fn set_part_rotation(&mut self, _part: u32, secondary: bool, rot: u32) {
        if !secondary {
            self.rotation = match rot {
                0 => Direction::Down,
                1 => Direction::Up,
                2 => Direction::North,
                3 => Direction::South,
                4 => Direction::West,
                5 => Direction::East,
                _ => Direction::Down,
            };
        }
    }
}

impl FrameSupport for TileMachine {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE, pkt[0], pkt[1]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 3 || data[0] != SUBID_TILE_MACHINE {
            return Err("invalid TileMachine frame packet header".into());
        }
        self.read_from_packet([data[1], data[2]]);
        Ok(())
    }
}

impl RedstoneConnectable for TileMachine {}
impl TubeConnectable for TileMachine {}

// ============================================================================
// 6. TileMachinePanel Base Implementation
// ============================================================================

/// Foundational 4-direction panel tile entity abstraction mirroring `TileMachinePanel`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileMachinePanel {
    pub rotation: u8, // 0..=3 (0: South, 1: West, 2: North, 3: East)
    pub flags: MachineFlags,
    pub metadata: u8,
    pub light_value: u8,
}

impl TileMachinePanel {
    pub fn new(metadata: u8) -> Self {
        Self {
            rotation: 0,
            flags: MachineFlags::NONE,
            metadata,
            light_value: 0,
        }
    }

    pub fn with_rotation(mut self, rotation: u8) -> Self {
        self.rotation = rotation & 3;
        self
    }

    /// Compute 4-way horizontal facing from yaw, mirroring `TileMachinePanel.onBlockPlaced`:
    /// `(floor(yaw * 4.0 / 360.0 + 0.5) & 3)`.
    pub fn calculate_panel_facing(player_yaw: f32) -> u8 {
        ((player_yaw * 4.0 / 360.0 + 0.5).floor() as i32 & 3) as u8
    }

    /// Replicate `TileMachinePanel.onBlockPlaced(ist, side, ent)`.
    pub fn on_block_placed(&mut self, player_yaw: f32) {
        self.rotation = Self::calculate_panel_facing(player_yaw);
    }

    /// Encode state into 2-byte packet payload: `[rotation, ps]`.
    pub fn write_to_packet(&self) -> [u8; 2] {
        [self.rotation & 3, self.flags.to_bits()]
    }

    /// Decode state from 2-byte packet payload: `[rotation, ps]`.
    pub fn read_from_packet(&mut self, pkt: [u8; 2]) {
        self.rotation = pkt[0] & 3;
        self.flags = MachineFlags::from_bits(pkt[1]);
    }

    /// Write NBT tags `rot` and `ps`.
    pub fn write_nbt(&self, nbt: &mut HashMap<String, String>) {
        nbt.insert("rot".into(), self.rotation.to_string());
        nbt.insert("ps".into(), self.flags.to_bits().to_string());
    }

    /// Read NBT tags `rot` and `ps`.
    pub fn read_nbt(&mut self, nbt: &HashMap<String, String>) {
        if let Some(r) = nbt.get("rot").and_then(|v| v.parse::<u8>().ok()) {
            self.rotation = r & 3;
        }
        if let Some(ps) = nbt.get("ps").and_then(|v| v.parse::<u8>().ok()) {
            self.flags = MachineFlags::from_bits(ps);
        }
    }

    pub fn light_value(&self) -> u8 {
        self.light_value
    }
}

impl Rotatable for TileMachinePanel {
    fn part_max_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if secondary {
            0
        } else {
            3
        }
    }

    fn part_rotation(&self, _part: u32, secondary: bool) -> u32 {
        if secondary {
            0
        } else {
            (self.rotation & 3) as u32
        }
    }

    fn set_part_rotation(&mut self, _part: u32, secondary: bool, rot: u32) {
        if !secondary {
            self.rotation = (rot & 3) as u8;
        }
    }
}

impl FrameSupport for TileMachinePanel {
    fn frame_packet(&self) -> Vec<u8> {
        let pkt = self.write_to_packet();
        vec![SUBID_TILE_MACHINE_PANEL, pkt[0], pkt[1]]
    }

    fn handle_frame_packet(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < 3 || data[0] != SUBID_TILE_MACHINE_PANEL {
            return Err("invalid TileMachinePanel frame packet header".into());
        }
        self.read_from_packet([data[1], data[2]]);
        Ok(())
    }
}

impl RedstoneConnectable for TileMachinePanel {}
impl TubeConnectable for TileMachinePanel {}

// ============================================================================
// 7. Machine Placement Validation
// ============================================================================

/// Check if a `BlockMachine` can be placed at `pos`.
pub fn can_place_machine(pos: BlockPos, _kind: MachineKind) -> bool {
    pos.1 >= 0 && pos.1 < 256
}

/// Check if a `BlockMachinePanel` can be placed at `pos`, enforcing the solid-support
/// constraint on `SolarPanel` (`ItemMachinePanel:55`).
pub fn can_place_machine_panel(
    pos: BlockPos,
    kind: MachinePanelKind,
    is_solid_below: bool,
) -> bool {
    if pos.1 < 0 || pos.1 >= 256 {
        return false;
    }
    if kind.requires_solid_base() && !is_solid_below {
        return false;
    }
    true
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_flags_bit_packing() {
        for bits in 0..=15u8 {
            let flags = MachineFlags::from_bits(bits);
            assert_eq!(flags.to_bits(), bits);
            assert_eq!(flags.active, (bits & 1) != 0);
            assert_eq!(flags.powered, (bits & 2) != 0);
            assert_eq!(flags.delay, (bits & 4) != 0);
            assert_eq!(flags.charged, (bits & 8) != 0);
        }
    }

    #[test]
    fn test_machine_rotation_calculation_six_directions() {
        let block_pos = [10, 64, 10];

        // 1. Player directly above looking down: p > 2.0 -> Down (0)
        let player_above = [10.5, 68.0, 10.5];
        let eye_offset = 1.62;
        let facing = TileMachine::calculate_facing(player_above, 0.0, eye_offset, block_pos);
        assert_eq!(facing, Direction::Down);

        // 2. Player directly below looking up: p < 0.0 -> Up (1)
        let player_below = [10.5, 63.0, 10.5];
        let facing = TileMachine::calculate_facing(player_below, 0.0, eye_offset, block_pos);
        assert_eq!(facing, Direction::Up);

        // 3. Player level at yaw 0 (South): yawrx = 0 -> South (3)
        let player_level = [10.5, 65.0, 10.5]; // p = 65 + 1.82 - 1.62 - 64 = 1.2 (in [0, 2])
        assert_eq!(
            TileMachine::calculate_facing(player_level, 0.0, eye_offset, block_pos),
            Direction::South
        );

        // 4. Player level at yaw 90 (West): yawrx = 1 -> West (4)
        assert_eq!(
            TileMachine::calculate_facing(player_level, 90.0, eye_offset, block_pos),
            Direction::West
        );

        // 5. Player level at yaw 180 (North): yawrx = 2 -> North (2)
        assert_eq!(
            TileMachine::calculate_facing(player_level, 180.0, eye_offset, block_pos),
            Direction::North
        );

        // 6. Player level at yaw 270 (East): yawrx = 3 -> East (5)
        assert_eq!(
            TileMachine::calculate_facing(player_level, 270.0, eye_offset, block_pos),
            Direction::East
        );

        // 7. Player far away (dx >= 2.0): ignores height, uses yaw
        let player_far = [20.0, 75.0, 10.5];
        assert_eq!(
            TileMachine::calculate_facing(player_far, 0.0, eye_offset, block_pos),
            Direction::South
        );
        assert_eq!(
            TileMachine::calculate_facing(player_far, 180.0, eye_offset, block_pos),
            Direction::North
        );
    }

    #[test]
    fn test_panel_rotation_calculation() {
        assert_eq!(TileMachinePanel::calculate_panel_facing(0.0), 0);
        assert_eq!(TileMachinePanel::calculate_panel_facing(90.0), 1);
        assert_eq!(TileMachinePanel::calculate_panel_facing(180.0), 2);
        assert_eq!(TileMachinePanel::calculate_panel_facing(270.0), 3);
        assert_eq!(TileMachinePanel::calculate_panel_facing(360.0), 0);
    }

    #[test]
    fn test_screwdriver_rotation_cycling() {
        let mut machine = TileMachine::new(0);
        assert_eq!(machine.rotation, Direction::Down);

        // Cycle 6 directions: 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 0
        assert!(machine.rotate_with_screwdriver(0, false));
        assert_eq!(machine.rotation, Direction::Up);
        assert!(machine.rotate_with_screwdriver(0, false));
        assert_eq!(machine.rotation, Direction::North);
        assert!(machine.rotate_with_screwdriver(0, false));
        assert_eq!(machine.rotation, Direction::South);
        assert!(machine.rotate_with_screwdriver(0, false));
        assert_eq!(machine.rotation, Direction::West);
        assert!(machine.rotate_with_screwdriver(0, false));
        assert_eq!(machine.rotation, Direction::East);
        assert!(machine.rotate_with_screwdriver(0, false));
        assert_eq!(machine.rotation, Direction::Down);

        // Sneaking screwdriver rotation is not allowed for machines (sec = true -> max 0)
        assert!(!machine.rotate_with_screwdriver(0, true));
        assert_eq!(machine.rotation, Direction::Down);

        // Panel screwdriver rotation: 0 -> 1 -> 2 -> 3 -> 0
        let mut panel = TileMachinePanel::new(0);
        assert_eq!(panel.rotation, 0);
        assert!(panel.rotate_with_screwdriver(0, false));
        assert_eq!(panel.rotation, 1);
        assert!(panel.rotate_with_screwdriver(0, false));
        assert_eq!(panel.rotation, 2);
        assert!(panel.rotate_with_screwdriver(0, false));
        assert_eq!(panel.rotation, 3);
        assert!(panel.rotate_with_screwdriver(0, false));
        assert_eq!(panel.rotation, 0);

        // Sneaking panel rotation not allowed
        assert!(!panel.rotate_with_screwdriver(0, true));
    }

    #[test]
    fn test_packet_and_nbt_serialization_round_trip() {
        let mut machine = TileMachine::new(3).with_rotation(Direction::East);
        machine.flags.active = true;
        machine.flags.charged = true;

        // Packet serialization
        let pkt = machine.write_to_packet();
        let mut restored = TileMachine::new(3);
        restored.read_from_packet(pkt);
        assert_eq!(restored.rotation, Direction::East);
        assert!(restored.flags.active);
        assert!(!restored.flags.powered);
        assert!(!restored.flags.delay);
        assert!(restored.flags.charged);

        // NBT serialization
        let mut nbt = HashMap::new();
        machine.write_nbt(&mut nbt);
        let mut nbt_restored = TileMachine::new(3);
        nbt_restored.read_nbt(&nbt);
        assert_eq!(nbt_restored.rotation, Direction::East);
        assert_eq!(nbt_restored.flags, machine.flags);

        // Frame packet
        let frame_pkt = machine.frame_packet();
        assert_eq!(frame_pkt[0], SUBID_TILE_MACHINE);
        let mut frame_restored = TileMachine::new(3);
        frame_restored.handle_frame_packet(&frame_pkt).unwrap();
        assert_eq!(frame_restored.rotation, Direction::East);
        assert_eq!(frame_restored.flags, machine.flags);

        // Panel packet, NBT, and Frame packet
        let mut panel = TileMachinePanel::new(0).with_rotation(2);
        panel.flags.powered = true;
        panel.flags.delay = true;

        let panel_pkt = panel.write_to_packet();
        let mut panel_restored = TileMachinePanel::new(0);
        panel_restored.read_from_packet(panel_pkt);
        assert_eq!(panel_restored.rotation, 2);
        assert_eq!(panel_restored.flags, panel.flags);

        let mut panel_nbt = HashMap::new();
        panel.write_nbt(&mut panel_nbt);
        let mut panel_nbt_restored = TileMachinePanel::new(0);
        panel_nbt_restored.read_nbt(&panel_nbt);
        assert_eq!(panel_nbt_restored.rotation, 2);
        assert_eq!(panel_nbt_restored.flags, panel.flags);

        let panel_frame_pkt = panel.frame_packet();
        assert_eq!(panel_frame_pkt[0], SUBID_TILE_MACHINE_PANEL);
        let mut panel_frame_restored = TileMachinePanel::new(0);
        panel_frame_restored
            .handle_frame_packet(&panel_frame_pkt)
            .unwrap();
        assert_eq!(panel_frame_restored.rotation, 2);
        assert_eq!(panel_frame_restored.flags, panel.flags);
    }

    #[test]
    fn test_texture_orientations_all_faces() {
        let base = [0, 1, 2, 3, 4, 5];

        // Down (0): identity
        assert_eq!(orient_textures(Direction::Down, base), [0, 1, 2, 3, 4, 5]);

        // Up (1): swap(0, 1), swap(4, 5) -> [1, 0, 2, 3, 5, 4]
        assert_eq!(orient_textures(Direction::Up, base), [1, 0, 2, 3, 5, 4]);

        // North (2): swap(0, 2), swap(1, 3), swap(0, 4), swap(1, 5)
        // start: [0, 1, 2, 3, 4, 5]
        // (0, 2) -> [2, 1, 0, 3, 4, 5]
        // (1, 3) -> [2, 3, 0, 1, 4, 5]
        // (0, 4) -> [4, 3, 0, 1, 2, 5]
        // (1, 5) -> [4, 5, 0, 1, 2, 3]
        assert_eq!(orient_textures(Direction::North, base), [4, 5, 0, 1, 2, 3]);

        // South (3): swap(0, 3), swap(1, 2), swap(0, 4), swap(1, 5)
        // start: [0, 1, 2, 3, 4, 5]
        // (0, 3) -> [3, 1, 2, 0, 4, 5]
        // (1, 2) -> [3, 2, 1, 0, 4, 5]
        // (0, 4) -> [4, 2, 1, 0, 3, 5]
        // (1, 5) -> [4, 5, 1, 0, 3, 2]
        assert_eq!(orient_textures(Direction::South, base), [4, 5, 1, 0, 3, 2]);

        // West (4): swap(0, 4), swap(1, 5), swap(2, 3)
        // start: [0, 1, 2, 3, 4, 5]
        // (0, 4) -> [4, 1, 2, 3, 0, 5]
        // (1, 5) -> [4, 5, 2, 3, 0, 1]
        // (2, 3) -> [4, 5, 3, 2, 0, 1]
        assert_eq!(orient_textures(Direction::West, base), [4, 5, 3, 2, 0, 1]);

        // East (5): swap(0, 5), swap(1, 4), swap(0, 1)
        // start: [0, 1, 2, 3, 4, 5]
        // (0, 5) -> [5, 1, 2, 3, 4, 0]
        // (1, 4) -> [5, 4, 2, 3, 1, 0]
        // (0, 1) -> [4, 5, 2, 3, 1, 0]
        assert_eq!(orient_textures(Direction::East, base), [4, 5, 2, 3, 1, 0]);
    }

    #[test]
    fn test_placement_validation_and_kinds() {
        let pos = BlockPos(5, 64, 5);

        // Machine placement
        assert!(can_place_machine(pos, MachineKind::Deployer));
        assert!(!can_place_machine(BlockPos(5, -1, 5), MachineKind::Deployer));

        // Panel placement: solar panel requires solid base
        assert!(!can_place_machine_panel(pos, MachinePanelKind::SolarPanel, false));
        assert!(can_place_machine_panel(pos, MachinePanelKind::SolarPanel, true));

        // Other panels do not require solid base
        assert!(can_place_machine_panel(pos, MachinePanelKind::Pump, false));
        assert!(can_place_machine_panel(pos, MachinePanelKind::Accelerator, false));

        // Kinds round trip
        for meta in 0..16u8 {
            let kind = MachineKind::from_meta(meta).unwrap();
            assert_eq!(kind.to_meta(), meta);
            assert_eq!(MachineKind::from_block_id(kind.block_id()), Some(kind));
        }

        for meta in 0..5u8 {
            let panel = MachinePanelKind::from_meta(meta).unwrap();
            assert_eq!(panel.to_meta(), meta);
            assert_eq!(
                MachinePanelKind::from_block_id(panel.block_id()),
                Some(panel)
            );
        }
    }
}

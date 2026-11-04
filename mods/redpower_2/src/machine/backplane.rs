//! Backplane bus and 8 KB RAM expansion module: port of `TileBackplane`, `TileRAM` and the
//! `TileCPU.refreshBackplane` / `readOnlyMem` / `writeOnlyMem` address decoding.
//!
//! Pure logic. A CPU owns its 8 KB at `0x0000..0x2000`; each backplane stacked in the CPU's
//! facing direction (up to 7) maps one further 8 KB window, so up to 64 KB in total.

use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Canonical block IDs.
pub const BLOCK_ID_BACKPLANE: &str = "redpower_2:backplane";
pub const BLOCK_ID_RAM_MODULE: &str = "redpower_2:ram_module";

/// Size of one RAM window (`new byte[8192]`).
pub const RAM_BANK_SIZE: usize = 8192;
/// Onboard CPU memory size (`8 KB`).
pub const ONBOARD_RAM_SIZE: usize = 8192;
/// Backplanes a CPU can chain (`TileBackplane[7]`).
pub const MAX_BACKPLANES: usize = 7;
/// Maximum total addressable RAM (8 KB onboard + 7 * 8 KB backplanes = 64 KB).
pub const MAX_TOTAL_RAM: usize = 65536;
/// Value read from an address with no RAM behind it (floating/open bus, `0xFF = 255`).
pub const OPEN_BUS: u8 = 255;

/// What a backplane tile carries: a bare backplane has no memory, a RAM module has 8 KB.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Backplane {
    /// `TileBackplane`: meta 0, reads open bus (`255`), ignores writes.
    Bare { rotation: u8 },
    /// `TileRAM`: meta 1, 8 KB of volatile memory.
    Ram { rotation: u8, memory: Vec<u8> },
}

impl Backplane {
    /// Create a bare backplane (`TileBackplane`, meta 0).
    pub fn bare(rotation: u8) -> Self {
        Self::Bare { rotation: rotation & 3 }
    }

    /// Create an 8K RAM module (`TileRAM`, meta 1).
    pub fn ram(rotation: u8) -> Self {
        Self::Ram {
            rotation: rotation & 3,
            memory: vec![0; RAM_BANK_SIZE],
        }
    }

    /// Create a RAM module with pre-existing memory.
    pub fn ram_with_memory(rotation: u8, memory: Vec<u8>) -> Self {
        let memory = if memory.len() == RAM_BANK_SIZE {
            memory
        } else {
            let mut v = memory;
            v.resize(RAM_BANK_SIZE, 0);
            v
        };
        Self::Ram {
            rotation: rotation & 3,
            memory,
        }
    }

    /// Horizontal rotation index `0..=3`.
    pub fn rotation(&self) -> u8 {
        match self {
            Self::Bare { rotation } | Self::Ram { rotation, .. } => *rotation,
        }
    }

    /// Set rotation index `0..=3`.
    pub fn set_rotation(&mut self, rot: u8) {
        let rot = rot & 3;
        match self {
            Self::Bare { rotation } | Self::Ram { rotation, .. } => *rotation = rot,
        }
    }

    /// True if this is a bare backplane with no RAM module attached.
    pub fn is_bare(&self) -> bool {
        matches!(self, Self::Bare { .. })
    }

    /// True if this backplane carries an 8K RAM module.
    pub fn is_ram(&self) -> bool {
        matches!(self, Self::Ram { .. })
    }

    /// Read-only access to RAM bytes, if present.
    pub fn memory(&self) -> Option<&[u8]> {
        match self {
            Self::Bare { .. } => None,
            Self::Ram { memory, .. } => Some(memory),
        }
    }

    /// Mutable access to RAM bytes, if present.
    pub fn memory_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            Self::Bare { .. } => None,
            Self::Ram { memory, .. } => Some(memory),
        }
    }

    /// Extended metadata / subtype index: 0 = Bare, 1 = RAM module.
    pub fn metadata(&self) -> u8 {
        match self {
            Self::Bare { .. } => 0,
            Self::Ram { .. } => 1,
        }
    }

    /// Canonical block ID for this backplane configuration.
    pub fn block_id(&self) -> &'static str {
        match self {
            Self::Bare { .. } => BLOCK_ID_BACKPLANE,
            Self::Ram { .. } => BLOCK_ID_RAM_MODULE,
        }
    }

    /// Upgrade a bare backplane into a RAM module (item use, preserving rotation).
    pub fn upgrade_to_ram(&mut self) {
        if let Self::Bare { rotation } = *self {
            *self = Self::Ram {
                rotation,
                memory: vec![0; RAM_BANK_SIZE],
            };
        }
    }

    /// Downgrade a RAM module to a bare backplane (e.g. harvesting part 1, `TileRAM.onHarvestPart`).
    pub fn downgrade_to_bare(&mut self) {
        if let Self::Ram { rotation, .. } = *self {
            *self = Self::Bare { rotation };
        }
    }

    /// Harvest a part (`TileRAM.onHarvestPart`):
    /// Part 1 is the RAM module card; harvesting it drops the RAM item and leaves a bare backplane.
    /// Part 0 is the base backplane; harvesting it breaks the whole block.
    pub fn harvest_part(&mut self, part: usize) -> Option<&'static str> {
        match part {
            1 if self.is_ram() => {
                self.downgrade_to_bare();
                Some(BLOCK_ID_RAM_MODULE)
            }
            0 => {
                let dropped = self.block_id();
                Some(dropped)
            }
            _ => None,
        }
    }

    /// `readBackplane(addr)`; `addr` is the offset inside the 8 KB window.
    /// Returns 255 for bare backplane (as in Java `TileBackplane.readBackplane`),
    /// or `memory[addr & 8191]` for RAM module.
    pub fn read(&self, addr: usize) -> u8 {
        match self {
            Self::Bare { .. } => OPEN_BUS,
            Self::Ram { memory, .. } => memory[addr & (RAM_BANK_SIZE - 1)],
        }
    }

    /// `writeBackplane(addr, val)`.
    /// Bare backplane ignores writes; RAM module writes to memory.
    pub fn write(&mut self, addr: usize, val: u8) {
        if let Self::Ram { memory, .. } = self {
            memory[addr & (RAM_BANK_SIZE - 1)] = val;
        }
    }

    /// Persistence: `rot` byte plus the `ram` byte array (RAM only).
    pub fn to_nbt(&self) -> (u8, Option<&[u8]>) {
        match self {
            Self::Bare { rotation } => (*rotation, None),
            Self::Ram { rotation, memory } => (*rotation, Some(memory)),
        }
    }

    /// Reload a RAM module; a `ram` array of the wrong length is replaced by zeroed memory.
    pub fn ram_from_nbt(rotation: u8, ram: &[u8]) -> Self {
        let memory = if ram.len() == RAM_BANK_SIZE {
            ram.to_vec()
        } else {
            vec![0; RAM_BANK_SIZE]
        };
        Self::Ram {
            rotation: rotation & 3,
            memory,
        }
    }

    /// Encode into Horizon block entity NBT string map.
    pub fn to_nbt_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("rot".into(), self.rotation().to_string());
        if let Self::Ram { memory, .. } = self {
            let hex: String = memory.iter().map(|b| format!("{b:02x}")).collect();
            map.insert("ram".into(), hex);
        }
        map
    }

    /// Decode from Horizon block entity NBT string map.
    pub fn from_nbt_map(is_ram: bool, map: &HashMap<String, String>) -> Self {
        let rot: u8 = map
            .get("rot")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
            & 3;

        if is_ram {
            let memory = if let Some(hex) = map.get("ram") {
                if hex.len() == RAM_BANK_SIZE * 2 {
                    (0..RAM_BANK_SIZE)
                        .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap_or(0))
                        .collect()
                } else {
                    vec![0; RAM_BANK_SIZE]
                }
            } else {
                vec![0; RAM_BANK_SIZE]
            };
            Self::Ram {
                rotation: rot,
                memory,
            }
        } else {
            Self::Bare { rotation: rot }
        }
    }
}

// ============================================================================
// Orientation & Chaining Geometry
// ============================================================================

/// RedPower horizontal rotation (0..4) to Minecraft / RedPower side (2..=5).
/// `0 -> East (+X)`, `1 -> South (+Z)`, `2 -> West (-X)`, `3 -> North (-Z)`.
/// Corresponds to `CoreLib.rotToSide(r)`.
pub const fn rot_to_direction(rot: u8) -> Direction {
    match rot & 3 {
        0 => Direction::East,
        1 => Direction::South,
        2 => Direction::West,
        _ => Direction::North,
    }
}

/// The direction stepping from CPU along the backplane chain (away from CPU).
/// Corresponds to `CoreLib.rotToSide(cpu.Rotation)`.
pub const fn bus_forward(rot: u8) -> Direction {
    rot_to_direction(rot)
}

/// The direction pointing back towards the CPU (upstream).
/// Corresponds to `CoreLib.rotToSide(rot) ^ 1`.
pub const fn bus_backward(rot: u8) -> Direction {
    rot_to_direction(rot).opposite()
}

/// Step coordinate by direction offset.
pub fn step_coord(pos: [i32; 3], dir: Direction) -> [i32; 3] {
    let [dx, dy, dz] = dir.offset();
    [pos[0] + dx, pos[1] + dy, pos[2] + dz]
}

/// Validate whether two backplane rotations can chain together.
/// In Java `ItemBackplane.java` & `TileBackplane.java`, all chained backplanes
/// must share the same horizontal rotation along the axis.
pub fn can_chain(rot_a: u8, rot_b: u8) -> bool {
    (rot_a & 3) == (rot_b & 3)
}

/// Check placement and neighbor support rules (`TileBackplane.onBlockNeighborChange`):
/// Must have solid support block underneath, and must have either another backplane
/// or a CPU in direction `bus_backward(rot)`.
pub fn check_support(has_solid_floor: bool, upstream_is_backplane_or_cpu: bool) -> bool {
    has_solid_floor && upstream_is_backplane_or_cpu
}

// ============================================================================
// Memory Map & Bus Decoding
// ============================================================================

/// The CPU's view of memory: onboard 8 KB plus the chain of backplanes found behind it
/// (`refreshBackplane` stops at the first gap, so later slots are `None`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryMap {
    pub onboard: Vec<u8>,
    pub backplanes: [Option<Backplane>; MAX_BACKPLANES],
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self {
            onboard: vec![0; ONBOARD_RAM_SIZE],
            backplanes: Default::default(),
        }
    }
}

impl MemoryMap {
    /// Create a new empty memory map with 8 KB onboard RAM.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a chain of backplanes in order; stops at the first `None`, like `refreshBackplane`.
    pub fn refresh(&mut self, chain: Vec<Option<Backplane>>) {
        let mut ok = true;
        let mut it = chain.into_iter();
        for slot in self.backplanes.iter_mut() {
            let next = it.next().flatten();
            if ok && next.is_some() {
                *slot = next;
            } else {
                ok = false;
                *slot = None;
            }
        }
    }

    /// Refresh backplane chain from world block lookup starting from CPU position.
    /// Steps up to 7 blocks along `bus_forward(cpu_rot)`.
    /// The chain stops at the first missing block, gap, or rotation mismatch.
    pub fn refresh_from_lookup<F>(&mut self, cpu_pos: [i32; 3], cpu_rot: u8, mut lookup: F)
    where
        F: FnMut([i32; 3]) -> Option<Backplane>,
    {
        let forward = bus_forward(cpu_rot);
        let mut curr = cpu_pos;
        let mut chain_ok = true;

        for slot in self.backplanes.iter_mut() {
            if !chain_ok {
                *slot = None;
                continue;
            }
            curr = step_coord(curr, forward);
            match lookup(curr) {
                Some(bp) if can_chain(bp.rotation(), cpu_rot) => {
                    *slot = Some(bp);
                }
                _ => {
                    chain_ok = false;
                    *slot = None;
                }
            }
        }
    }

    /// `readOnlyMem`: 16-bit address, 255 where no RAM exists.
    /// Addresses `0x0000..=0x1FFF`: CPU onboard RAM.
    /// Addresses `0x2000..=0xFFFF`: backplane slots `0..6`, 8 KB each.
    pub fn read(&self, addr: usize) -> u8 {
        let addr = addr & 0xFFFF;
        if addr < ONBOARD_RAM_SIZE {
            return self.onboard[addr];
        }
        let bank = (addr >> 13) - 1;
        match &self.backplanes[bank] {
            Some(b) => b.read(addr & (RAM_BANK_SIZE - 1)),
            None => OPEN_BUS,
        }
    }

    /// `writeOnlyMem`: writes to absent backplanes or bare backplanes are dropped.
    /// Addresses `0x0000..=0x1FFF`: CPU onboard RAM.
    /// Addresses `0x2000..=0xFFFF`: backplane slots `0..6`.
    pub fn write(&mut self, addr: usize, val: u8) {
        let addr = addr & 0xFFFF;
        if addr < ONBOARD_RAM_SIZE {
            self.onboard[addr] = val;
        } else {
            let bank = (addr >> 13) - 1;
            if let Some(b) = &mut self.backplanes[bank] {
                b.write(addr & (RAM_BANK_SIZE - 1), val);
            }
        }
    }

    /// Total addressable RAM in bytes (8 KB onboard + 8 KB per RAM module in contiguous chain).
    pub fn ram_size(&self) -> usize {
        ONBOARD_RAM_SIZE
            + self
                .backplanes
                .iter()
                .flatten()
                .filter(|b| b.is_ram())
                .count()
                * RAM_BANK_SIZE
    }

    /// Number of active RAM modules connected in the contiguous backplane chain.
    pub fn installed_ram_modules(&self) -> usize {
        self.backplanes
            .iter()
            .flatten()
            .filter(|b| b.is_ram())
            .count()
    }

    /// Starting base address for backplane bank `0..7` (`0x2000`, `0x4000`, ..., `0xE000`).
    pub const fn bank_base_address(bank: usize) -> usize {
        (bank + 1) * RAM_BANK_SIZE
    }

    /// Check if backplane bank `bank` (0..7) has an active RAM module.
    pub fn is_bank_ram(&self, bank: usize) -> bool {
        if bank < MAX_BACKPLANES {
            self.backplanes[bank].as_ref().map_or(false, Backplane::is_ram)
        } else {
            false
        }
    }

    /// Map a 16-bit address to backplane bank index `0..6` and offset `0..8191`,
    /// or `None` if the address falls in onboard RAM (`0x0000..=0x1FFF`).
    pub const fn address_to_bank(addr: usize) -> Option<(usize, usize)> {
        let addr = addr & 0xFFFF;
        if addr < ONBOARD_RAM_SIZE {
            None
        } else {
            Some(((addr >> 13) - 1, addr & (RAM_BANK_SIZE - 1)))
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_bus_without_backplane() {
        let mut m = MemoryMap::default();
        assert_eq!(m.read(0x2000), OPEN_BUS);
        m.write(0x2000, 7);
        assert_eq!(m.read(0x2000), OPEN_BUS);
    }

    #[test]
    fn bare_backplane_reads_open_bus() {
        // TileBackplane::readBackplane returns 255 (OPEN_BUS) in original Java
        let mut m = MemoryMap::default();
        m.refresh(vec![Some(Backplane::bare(0))]);
        assert_eq!(m.read(0x2000), OPEN_BUS);
        m.write(0x2000, 42);
        // Write to bare backplane is dropped
        assert_eq!(m.read(0x2000), OPEN_BUS);
        assert_eq!(m.ram_size(), ONBOARD_RAM_SIZE);
    }

    #[test]
    fn ram_module_extends_memory() {
        let mut m = MemoryMap::default();
        m.refresh(vec![Some(Backplane::ram(0)), Some(Backplane::ram(0))]);
        m.write(0x2005, 0xAB);
        m.write(0x4000, 0xCD);
        assert_eq!(m.read(0x2005), 0xAB);
        assert_eq!(m.read(0x4000), 0xCD);
        assert_eq!(m.read(0x6000), OPEN_BUS);
        assert_eq!(m.ram_size(), 24 * 1024);
        assert_eq!(m.installed_ram_modules(), 2);
    }

    #[test]
    fn chain_stops_at_gap() {
        let mut m = MemoryMap::default();
        m.refresh(vec![
            Some(Backplane::bare(0)),
            None,
            Some(Backplane::ram(0)),
        ]);
        assert_eq!(m.read(0x2000), OPEN_BUS);
        assert_eq!(m.read(0x6000), OPEN_BUS);
        assert!(m.backplanes[2].is_none());
    }

    #[test]
    fn full_chain_reaches_64k() {
        let mut m = MemoryMap::default();
        m.refresh((0..7).map(|_| Some(Backplane::ram(0))).collect());
        m.write(0xFFFF, 9);
        assert_eq!(m.read(0xFFFF), 9);
        assert_eq!(m.ram_size(), MAX_TOTAL_RAM);
        assert_eq!(m.installed_ram_modules(), 7);
    }

    #[test]
    fn nbt_roundtrip_and_bad_length() {
        let mut b = Backplane::ram(3);
        b.write(10, 42);
        let (rot, ram) = b.to_nbt();
        assert_eq!(Backplane::ram_from_nbt(rot, ram.unwrap()), b);
        assert_eq!(Backplane::ram_from_nbt(1, &[1, 2]).read(0), 0);
    }

    #[test]
    fn nbt_map_roundtrip() {
        let mut b = Backplane::ram(2);
        b.write(0, 0x55);
        b.write(8191, 0xAA);
        let map = b.to_nbt_map();
        let restored = Backplane::from_nbt_map(true, &map);
        assert_eq!(restored.rotation(), 2);
        assert_eq!(restored.read(0), 0x55);
        assert_eq!(restored.read(8191), 0xAA);

        let bare = Backplane::bare(1);
        let bare_map = bare.to_nbt_map();
        let bare_restored = Backplane::from_nbt_map(false, &bare_map);
        assert_eq!(bare_restored.rotation(), 1);
        assert!(bare_restored.is_bare());
    }

    #[test]
    fn harvest_part_and_upgrade() {
        let mut bp = Backplane::bare(1);
        assert_eq!(bp.block_id(), BLOCK_ID_BACKPLANE);
        assert_eq!(bp.metadata(), 0);

        bp.upgrade_to_ram();
        assert_eq!(bp.block_id(), BLOCK_ID_RAM_MODULE);
        assert_eq!(bp.metadata(), 1);
        assert!(bp.is_ram());

        bp.write(100, 0x77);
        assert_eq!(bp.read(100), 0x77);

        // Harvesting part 1 (RAM card) drops RAM module and downgrades backplane to bare
        let dropped = bp.harvest_part(1);
        assert_eq!(dropped, Some(BLOCK_ID_RAM_MODULE));
        assert!(bp.is_bare());
        assert_eq!(bp.rotation(), 1);
        assert_eq!(bp.read(100), OPEN_BUS);

        // Harvesting part 0 breaks backplane
        let dropped_base = bp.harvest_part(0);
        assert_eq!(dropped_base, Some(BLOCK_ID_BACKPLANE));
    }

    #[test]
    fn geometry_and_directions() {
        assert_eq!(rot_to_direction(0), Direction::East);
        assert_eq!(rot_to_direction(1), Direction::South);
        assert_eq!(rot_to_direction(2), Direction::West);
        assert_eq!(rot_to_direction(3), Direction::North);

        // Forward is away from CPU; backward is towards CPU
        assert_eq!(bus_forward(0), Direction::East);
        assert_eq!(bus_backward(0), Direction::West);

        assert_eq!(bus_forward(1), Direction::South);
        assert_eq!(bus_backward(1), Direction::North);

        // Stepping coordinates
        let origin = [10, 64, 20];
        let next_east = step_coord(origin, bus_forward(0));
        assert_eq!(next_east, [11, 64, 20]);
        let prev_west = step_coord(next_east, bus_backward(0));
        assert_eq!(prev_west, origin);

        assert!(can_chain(0, 0));
        assert!(!can_chain(0, 1));
        assert!(check_support(true, true));
        assert!(!check_support(false, true));
        assert!(!check_support(true, false));
    }

    #[test]
    fn refresh_from_lookup_simulation() {
        let cpu_pos = [0, 60, 0];
        let cpu_rot = 0; // East (+X)

        let mut world_blocks: HashMap<[i32; 3], Backplane> = HashMap::new();
        // 3 consecutive backplanes behind CPU along +X
        world_blocks.insert([1, 60, 0], Backplane::ram(0));
        world_blocks.insert([2, 60, 0], Backplane::bare(0));
        world_blocks.insert([3, 60, 0], Backplane::ram(0));
        // Gap at [4, 60, 0]
        world_blocks.insert([5, 60, 0], Backplane::ram(0));

        let mut m = MemoryMap::new();
        m.refresh_from_lookup(cpu_pos, cpu_rot, |p| world_blocks.get(&p).cloned());

        assert!(m.backplanes[0].as_ref().unwrap().is_ram());
        assert!(m.backplanes[1].as_ref().unwrap().is_bare());
        assert!(m.backplanes[2].as_ref().unwrap().is_ram());
        // Stopped at gap
        assert!(m.backplanes[3].is_none());
        assert!(m.backplanes[4].is_none());
        assert_eq!(m.ram_size(), 24 * 1024); // 8 KB onboard + 2 * 8 KB RAM modules
    }

    #[test]
    fn address_to_bank_mapping() {
        assert_eq!(MemoryMap::address_to_bank(0x0000), None);
        assert_eq!(MemoryMap::address_to_bank(0x1FFF), None);
        assert_eq!(MemoryMap::address_to_bank(0x2000), Some((0, 0)));
        assert_eq!(MemoryMap::address_to_bank(0x3FFF), Some((0, 8191)));
        assert_eq!(MemoryMap::address_to_bank(0x4000), Some((1, 0)));
        assert_eq!(MemoryMap::address_to_bank(0xFFFF), Some((6, 8191)));

        assert_eq!(MemoryMap::bank_base_address(0), 0x2000);
        assert_eq!(MemoryMap::bank_base_address(1), 0x4000);
        assert_eq!(MemoryMap::bank_base_address(6), 0xE000);
    }
}

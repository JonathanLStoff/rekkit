//! Backplane bus and 8 KB RAM expansion module: port of `TileBackplane`, `TileRAM` and the
//! `TileCPU.refreshBackplane` / `readOnlyMem` / `writeOnlyMem` address decoding.
//!
//! Pure logic. A CPU owns its 8 KB at `0x0000..0x2000`; each backplane stacked in the CPU's
//! facing direction (up to 7) maps one further 8 KB window, so 64 KB in total.

use serde::{Deserialize, Serialize};

/// Size of one RAM window (`new byte[8192]`).
pub const RAM_BANK_SIZE: usize = 8192;
/// Backplanes a CPU can chain (`TileBackplane[7]`).
pub const MAX_BACKPLANES: usize = 7;
/// Value read from an address with no backplane behind it.
pub const OPEN_BUS: u8 = 255;

/// What a backplane tile carries: a bare backplane has no memory, a RAM module has 8 KB.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Backplane {
    /// `TileBackplane`: reads 0, ignores writes.
    Bare { rotation: u8 },
    /// `TileRAM`: 8 KB of memory.
    Ram { rotation: u8, memory: Vec<u8> },
}

impl Backplane {
    pub fn bare(rotation: u8) -> Self {
        Self::Bare { rotation }
    }

    pub fn ram(rotation: u8) -> Self {
        Self::Ram {
            rotation,
            memory: vec![0; RAM_BANK_SIZE],
        }
    }

    pub fn rotation(&self) -> u8 {
        match self {
            Self::Bare { rotation } | Self::Ram { rotation, .. } => *rotation,
        }
    }

    /// `readBackplane(addr)`; `addr` is the offset inside the 8 KB window.
    pub fn read(&self, addr: usize) -> u8 {
        match self {
            Self::Bare { .. } => 0,
            Self::Ram { memory, .. } => memory[addr & (RAM_BANK_SIZE - 1)],
        }
    }

    /// `writeBackplane(addr, val)`.
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
        Self::Ram { rotation, memory }
    }
}

/// The CPU's view of memory: onboard 8 KB plus the chain of backplanes found in front of it
/// (`refreshBackplane` stops at the first gap, so later slots are `None`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryMap {
    pub onboard: Vec<u8>,
    pub backplanes: [Option<Backplane>; MAX_BACKPLANES],
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self {
            onboard: vec![0; RAM_BANK_SIZE],
            backplanes: Default::default(),
        }
    }
}

impl MemoryMap {
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

    /// `readOnlyMem`: 16-bit address, 255 where no backplane exists.
    pub fn read(&self, addr: usize) -> u8 {
        let addr = addr & 0xFFFF;
        if addr < RAM_BANK_SIZE {
            return self.onboard[addr];
        }
        match &self.backplanes[(addr >> 13) - 1] {
            Some(b) => b.read(addr & (RAM_BANK_SIZE - 1)),
            None => OPEN_BUS,
        }
    }

    /// `writeOnlyMem`: writes to absent backplanes are dropped.
    pub fn write(&mut self, addr: usize, val: u8) {
        let addr = addr & 0xFFFF;
        if addr < RAM_BANK_SIZE {
            self.onboard[addr] = val;
        } else if let Some(b) = &mut self.backplanes[(addr >> 13) - 1] {
            b.write(addr & (RAM_BANK_SIZE - 1), val);
        }
    }

    /// Total addressable RAM in bytes (8 KB per RAM module in the contiguous chain).
    pub fn ram_size(&self) -> usize {
        RAM_BANK_SIZE
            + self
                .backplanes
                .iter()
                .flatten()
                .filter(|b| matches!(b, Backplane::Ram { .. }))
                .count()
                * RAM_BANK_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_bus_without_backplane() {
        let mut m = MemoryMap::default();
        assert_eq!(m.read(0x2000), 255);
        m.write(0x2000, 7);
        assert_eq!(m.read(0x2000), 255);
    }

    #[test]
    fn ram_module_extends_memory() {
        let mut m = MemoryMap::default();
        m.refresh(vec![Some(Backplane::ram(0)), Some(Backplane::ram(0))]);
        m.write(0x2005, 0xAB);
        m.write(0x4000, 0xCD);
        assert_eq!(m.read(0x2005), 0xAB);
        assert_eq!(m.read(0x4000), 0xCD);
        assert_eq!(m.read(0x6000), 255);
        assert_eq!(m.ram_size(), 24 * 1024);
    }

    #[test]
    fn chain_stops_at_gap_and_bare_reads_zero() {
        let mut m = MemoryMap::default();
        m.refresh(vec![
            Some(Backplane::bare(0)),
            None,
            Some(Backplane::ram(0)),
        ]);
        assert_eq!(m.read(0x2000), 0);
        assert_eq!(m.read(0x6000), 255);
        assert!(m.backplanes[2].is_none());
    }

    #[test]
    fn full_chain_reaches_64k() {
        let mut m = MemoryMap::default();
        m.refresh((0..7).map(|_| Some(Backplane::ram(0))).collect());
        m.write(0xFFFF, 9);
        assert_eq!(m.read(0xFFFF), 9);
        assert_eq!(m.ram_size(), 64 * 1024);
    }

    #[test]
    fn nbt_roundtrip_and_bad_length() {
        let mut b = Backplane::ram(3);
        b.write(10, 42);
        let (rot, ram) = b.to_nbt();
        assert_eq!(Backplane::ram_from_nbt(rot, ram.unwrap()), b);
        assert_eq!(Backplane::ram_from_nbt(1, &[1, 2]).read(0), 0);
    }
}

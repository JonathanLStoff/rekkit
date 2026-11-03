//! Disk drive Redbus device: port of `TileDiskDrive` register file and command set.
//!
//! Storage is behind [`DiskStore`] so the engine layer decides where sectors live.

use serde::{Deserialize, Serialize};

pub const SECTOR_SIZE: usize = 128;
/// Largest accepted sector number (`sector > 2048` is rejected, as in the original).
pub const MAX_SECTOR: u32 = 2048;
/// Ticks the drive stays Active after a command.
pub const ACCESS_TIME: u32 = 5;

/// Which disk is inserted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskKind {
    /// Damage 0: writable, label and serial stored in NBT, sectors in a save file.
    Blank,
    /// Damage 1/2: read-only built-in image (`redforth.img`, `redforthxp.img`).
    System(u8),
}

/// Backing storage for the inserted disk.
pub trait DiskStore {
    /// Label tag of a blank disk.
    fn label(&self) -> Option<String>;
    fn set_label(&mut self, label: &str);
    /// Serial number; created on demand, `None` if the file cannot be created.
    fn serial(&mut self) -> Option<String>;
    /// Read one sector (128 bytes); `None` on short read / failure.
    fn read_sector(&mut self, sector: u32) -> Option<[u8; SECTOR_SIZE]>;
    fn write_sector(&mut self, sector: u32, data: &[u8; SECTOR_SIZE]) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskDrive {
    pub rbaddr: i32,
    pub sector: u32,
    pub cmdreg: i32,
    pub databuf: Vec<u8>,
    pub access_time: u32,
    pub active: bool,
}

impl Default for DiskDrive {
    fn default() -> Self {
        Self {
            rbaddr: 2,
            sector: 0,
            cmdreg: 0,
            databuf: vec![0; SECTOR_SIZE],
            access_time: 0,
            active: false,
        }
    }
}

impl DiskDrive {
    pub fn rb_read(&self, reg: usize) -> u8 {
        if reg < SECTOR_SIZE {
            return self.databuf[reg];
        }
        match reg {
            128 => (self.sector & 0xFF) as u8,
            129 => (self.sector >> 8) as u8,
            130 => (self.cmdreg & 0xFF) as u8,
            _ => 0,
        }
    }

    pub fn rb_write(&mut self, reg: usize, dat: u8) {
        if reg < SECTOR_SIZE {
            self.databuf[reg] = dat;
            return;
        }
        match reg {
            128 => self.sector = (self.sector & 0xFF00) | dat as u32,
            129 => self.sector = (self.sector & 0xFF) | ((dat as u32) << 8),
            130 => self.cmdreg = dat as i32,
            _ => {}
        }
    }

    fn fill_str(&mut self, s: &str) {
        self.databuf.fill(0);
        let b = s.as_bytes();
        let n = b.len().min(SECTOR_SIZE);
        self.databuf[..n].copy_from_slice(&b[..n]);
    }

    /// Per-tick update (`g()`): run a pending command, then count down activity.
    /// `disk` is `None` when the drive is empty (or holds a non-disk item).
    /// Returns true when `active` changed (block must be updated).
    pub fn tick(&mut self, disk: Option<(&DiskKind, &mut dyn DiskStore)>) -> bool {
        let mut changed = false;
        if self.cmdreg != 0 && self.cmdreg != -1 {
            match disk {
                None => self.cmdreg = -1,
                Some((kind, store)) => {
                    self.run_cmd(kind, store);
                    self.access_time = ACCESS_TIME;
                    if !self.active {
                        self.active = true;
                        changed = true;
                    }
                }
            }
        }
        if self.access_time > 0 {
            self.access_time -= 1;
            if self.access_time == 0 {
                self.active = false;
                changed = true;
            }
        }
        changed
    }

    fn run_cmd(&mut self, kind: &DiskKind, store: &mut dyn DiskStore) {
        let system = matches!(kind, DiskKind::System(_));
        match self.cmdreg {
            1 => {
                // Read label. A blank disk without label tag leaves the buffer cleared.
                self.databuf.fill(0);
                if system {
                    self.fill_str("System Disk");
                } else if let Some(l) = store.label() {
                    self.fill_str(&l);
                }
                self.cmdreg = 0;
            }
            2 => {
                if system {
                    self.cmdreg = -1;
                } else {
                    let len = self
                        .databuf
                        .iter()
                        .take(64)
                        .take_while(|&&b| b != 0)
                        .count();
                    let label: String = self.databuf[..len]
                        .iter()
                        .map(|&b| (b & 0x7F) as char)
                        .collect();
                    store.set_label(&label);
                    self.cmdreg = 0;
                }
            }
            3 => {
                if let DiskKind::System(n) = kind {
                    self.fill_str(&format!("{:016}", n));
                } else {
                    let s = store.serial();
                    self.databuf.fill(0);
                    if let Some(s) = s {
                        self.fill_str(&s);
                    }
                }
                self.cmdreg = 0;
            }
            4 => {
                if self.sector > MAX_SECTOR {
                    self.cmdreg = -1;
                } else if let Some(d) = store.read_sector(self.sector) {
                    self.databuf.copy_from_slice(&d);
                    self.cmdreg = 0;
                } else {
                    self.cmdreg = -1;
                }
            }
            5 => {
                if system || self.sector > MAX_SECTOR {
                    self.cmdreg = -1;
                } else {
                    let mut d = [0u8; SECTOR_SIZE];
                    d.copy_from_slice(&self.databuf);
                    self.cmdreg = if store.write_sector(self.sector, &d) {
                        0
                    } else {
                        -1
                    };
                }
            }
            _ => self.cmdreg = -1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Mem {
        label: Option<String>,
        sectors: std::collections::HashMap<u32, [u8; SECTOR_SIZE]>,
    }
    impl DiskStore for Mem {
        fn label(&self) -> Option<String> {
            self.label.clone()
        }
        fn set_label(&mut self, l: &str) {
            self.label = Some(l.into());
        }
        fn serial(&mut self) -> Option<String> {
            Some("ABC123".into())
        }
        fn read_sector(&mut self, s: u32) -> Option<[u8; SECTOR_SIZE]> {
            self.sectors.get(&s).copied()
        }
        fn write_sector(&mut self, s: u32, d: &[u8; SECTOR_SIZE]) -> bool {
            self.sectors.insert(s, *d);
            true
        }
    }

    #[test]
    fn write_then_read_sector() {
        let mut d = DiskDrive::default();
        let mut m = Mem::default();
        let k = DiskKind::Blank;
        d.rb_write(0, 42);
        d.rb_write(128, 7);
        d.rb_write(130, 5);
        assert!(d.tick(Some((&k, &mut m))));
        assert_eq!(d.rb_read(130), 0);
        d.rb_write(0, 0);
        d.rb_write(130, 4);
        d.tick(Some((&k, &mut m)));
        assert_eq!(d.rb_read(0), 42);
        assert_eq!(d.rb_read(128), 7);
    }

    #[test]
    fn system_disk_label_and_readonly() {
        let mut d = DiskDrive::default();
        let mut m = Mem::default();
        let k = DiskKind::System(1);
        d.rb_write(130, 1);
        d.tick(Some((&k, &mut m)));
        assert_eq!(&d.databuf[..11], b"System Disk");
        d.rb_write(130, 5);
        d.tick(Some((&k, &mut m)));
        assert_eq!(d.rb_read(130), 255);
    }

    #[test]
    fn no_disk_errors_and_activity_expires() {
        let mut d = DiskDrive::default();
        d.rb_write(130, 4);
        d.tick(None);
        assert_eq!(d.cmdreg, -1);
        let mut m = Mem::default();
        let k = DiskKind::Blank;
        d.cmdreg = 3;
        d.tick(Some((&k, &mut m)));
        assert!(d.active);
        for _ in 0..5 {
            d.tick(Some((&k, &mut m)));
        }
        assert!(!d.active);
    }
}

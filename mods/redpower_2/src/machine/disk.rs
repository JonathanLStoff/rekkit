//! Disk drive Redbus device: port of `TileDiskDrive` and `ItemDisk`.
//!
//! Features:
//! - Peripheral block: `TileDiskDrive` (BlockPeripheral meta 2)
//! - Redbus device address: default 2 (`rbaddr: 2`)
//! - Capacity: 256 KB virtual storage formatted into 2048 sectors of 128 bytes each
//! - Memory-mapped registers:
//!   - Reg 0..127: Data buffer (128 bytes)
//!   - Reg 128: Sector number low byte (`sector & 0xFF`)
//!   - Reg 129: Sector number high byte (`sector >> 8`)
//!   - Reg 130: Command register (write cmd / read status)
//!   - Reg 131: Status flags register (bit 0 = disk present, bit 1 = motor running)
//! - Commands:
//!   - 1 = Read Label
//!   - 2 = Write Label
//!   - 3 = Read Serial Number
//!   - 4 = Read Sector
//!   - 5 = Write Sector
//! - Floppy Disk Items:
//!   - `redpower_2:blank_disk` (ItemDisk meta 0): Writable 256 KB magnetic disk
//!   - `redpower_2:floppy_disk` (ItemDisk meta 0): Alias for `blank_disk`
//!   - `redpower_2:forth_boot_disk` (ItemDisk meta 1): RedPower Forth Boot Disk (`redforth.img`, read-only)
//!   - `redpower_2:forth_extra_disk` (ItemDisk meta 2): RedPower Forth Extra Disk (`redforthxp.img`, read-only)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Size of one sector in bytes.
pub const SECTOR_SIZE: usize = 128;
/// Total number of sectors on a floppy disk (256 KB / 128 bytes = 2048 sectors).
pub const TOTAL_SECTORS: usize = 2048;
/// Largest accepted sector number (`sector > 2048` is rejected, matching the original).
pub const MAX_SECTOR: u32 = 2048;
/// Total disk capacity in bytes (256 KB).
pub const TOTAL_CAPACITY_BYTES: usize = TOTAL_SECTORS * SECTOR_SIZE;
/// World ticks the drive motor stays active after executing a command.
pub const ACCESS_TIME: u32 = 5;
/// Default Redbus peripheral address for Disk Drive.
pub const DEFAULT_RBADDR: i32 = 2;

/// Built-in RedPower Forth boot disk image (`redforth.img`, 5888 bytes).
pub static REDFORTH_IMG: &[u8] = include_bytes!("redforth.img");
/// Built-in RedPower Forth extra disk image (`redforthxp.img`, 6912 bytes).
pub static REDFORTH_XP_IMG: &[u8] = include_bytes!("redforthxp.img");

/// Helper to read sector from an embedded disk image.
pub fn read_system_image_sector(img: &[u8], sector: u32) -> [u8; SECTOR_SIZE] {
    let mut buf = [0u8; SECTOR_SIZE];
    let start = (sector as usize) * SECTOR_SIZE;
    if start < img.len() {
        let end = (start + SECTOR_SIZE).min(img.len());
        let count = end - start;
        buf[..count].copy_from_slice(&img[start..end]);
    }
    buf
}

/// Generate a 16-character hexadecimal serial number (matching `DiskLib.generateSerialNumber`).
pub fn generate_serial_number() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0x1234_5678_9ABC_DEF0);
    let mut rng = (seed ^ 0x5DEECE66D) as u64;
    let mut out = String::with_capacity(16);
    for _ in 0..16 {
        rng = rng.wrapping_mul(0x5DEECE66D).wrapping_add(0xB);
        let digit = ((rng >> 16) & 0xF) as u8;
        out.push(char::from_digit(digit as u32, 16).unwrap_or('0'));
    }
    out
}

/// Type of floppy disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloppyDiskType {
    /// Meta 0: Blank Floppy Disk (`redpower_2:blank_disk` / `redpower_2:floppy_disk`).
    Blank,
    /// Meta 1: RedPower Forth Boot Disk (`redpower_2:forth_boot_disk`).
    ForthBoot,
    /// Meta 2: RedPower Forth Extra Disk (`redpower_2:forth_extra_disk`).
    ForthExtra,
}

/// Which disk is inserted (used by low-level `run_cmd`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskKind {
    /// Damage 0: writable, label and serial stored in NBT, sectors in save storage.
    Blank,
    /// Damage 1/2: read-only built-in system image (1 = `redforth.img`, 2 = `redforthxp.img`).
    System(u8),
}

/// Backing storage interface for a disk.
pub trait DiskStore {
    /// Label tag of the disk.
    fn label(&self) -> Option<String>;
    /// Set label tag of the disk.
    fn set_label(&mut self, label: &str);
    /// Serial number; created on demand, `None` if creation fails.
    fn serial(&mut self) -> Option<String>;
    /// Read one 128-byte sector; returns `None` on invalid sector / read error.
    fn read_sector(&mut self, sector: u32) -> Option<[u8; SECTOR_SIZE]>;
    /// Write one 128-byte sector; returns false if disk is read-only or sector invalid.
    fn write_sector(&mut self, sector: u32, data: &[u8; SECTOR_SIZE]) -> bool;
}

/// In-memory representation of a Floppy Disk item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloppyDisk {
    pub disk_type: FloppyDiskType,
    pub label: Option<String>,
    pub serial: Option<String>,
    pub sectors: HashMap<u32, Vec<u8>>,
}

impl FloppyDisk {
    /// Create a new blank floppy disk (256 KB writable).
    pub fn new_blank() -> Self {
        Self {
            disk_type: FloppyDiskType::Blank,
            label: None,
            serial: None,
            sectors: HashMap::new(),
        }
    }

    /// Create a RedPower Forth boot disk (read-only system disk).
    pub fn new_forth_boot() -> Self {
        Self {
            disk_type: FloppyDiskType::ForthBoot,
            label: Some("System Disk".into()),
            serial: Some("0000000000000001".into()),
            sectors: HashMap::new(),
        }
    }

    /// Create a RedPower Forth extra disk (read-only system disk).
    pub fn new_forth_extra() -> Self {
        Self {
            disk_type: FloppyDiskType::ForthExtra,
            label: Some("System Disk".into()),
            serial: Some("0000000000000002".into()),
            sectors: HashMap::new(),
        }
    }

    /// Construct a disk from its type.
    pub fn from_type(disk_type: FloppyDiskType) -> Self {
        match disk_type {
            FloppyDiskType::Blank => Self::new_blank(),
            FloppyDiskType::ForthBoot => Self::new_forth_boot(),
            FloppyDiskType::ForthExtra => Self::new_forth_extra(),
        }
    }

    /// Construct a disk from its mod item identifier.
    pub fn from_item_id(item_id: &str) -> Option<Self> {
        match item_id {
            "redpower_2:blank_disk" | "redpower_2:floppy_disk" => Some(Self::new_blank()),
            "redpower_2:forth_boot_disk" => Some(Self::new_forth_boot()),
            "redpower_2:forth_extra_disk" => Some(Self::new_forth_extra()),
            _ => None,
        }
    }

    /// Mod item identifier for this disk.
    pub fn item_id(&self) -> &'static str {
        match self.disk_type {
            FloppyDiskType::Blank => "redpower_2:blank_disk",
            FloppyDiskType::ForthBoot => "redpower_2:forth_boot_disk",
            FloppyDiskType::ForthExtra => "redpower_2:forth_extra_disk",
        }
    }

    /// Item metadata damage value.
    pub fn meta(&self) -> u16 {
        match self.disk_type {
            FloppyDiskType::Blank => 0,
            FloppyDiskType::ForthBoot => 1,
            FloppyDiskType::ForthExtra => 2,
        }
    }

    /// Item rarity (0 = Common, 1 = Rare for system disks).
    pub fn rarity(&self) -> u8 {
        match self.disk_type {
            FloppyDiskType::Blank => 0,
            FloppyDiskType::ForthBoot | FloppyDiskType::ForthExtra => 1,
        }
    }

    /// Unlocalized item name (`ItemDisk.d`).
    pub fn unlocalized_name(&self) -> &'static str {
        match self.disk_type {
            FloppyDiskType::Blank => "item.disk",
            FloppyDiskType::ForthBoot => "item.disk.forth",
            FloppyDiskType::ForthExtra => "item.disk.forthxp",
        }
    }

    /// Display name (`ItemDisk.l`): returns label if set, else standard name.
    pub fn display_name(&self) -> String {
        if let Some(ref l) = self.label {
            if !l.is_empty() {
                return l.clone();
            }
        }
        match self.disk_type {
            FloppyDiskType::Blank => "Blank Floppy Disk".into(),
            FloppyDiskType::ForthBoot => "RedPower Forth Boot Disk".into(),
            FloppyDiskType::ForthExtra => "RedPower Forth Extra Disk".into(),
        }
    }

    /// Low-level disk kind for command execution.
    pub fn disk_kind(&self) -> DiskKind {
        match self.disk_type {
            FloppyDiskType::Blank => DiskKind::Blank,
            FloppyDiskType::ForthBoot => DiskKind::System(1),
            FloppyDiskType::ForthExtra => DiskKind::System(2),
        }
    }
}

impl DiskStore for FloppyDisk {
    fn label(&self) -> Option<String> {
        match self.disk_type {
            FloppyDiskType::Blank => self.label.clone(),
            FloppyDiskType::ForthBoot | FloppyDiskType::ForthExtra => Some("System Disk".into()),
        }
    }

    fn set_label(&mut self, label: &str) {
        if self.disk_type == FloppyDiskType::Blank {
            self.label = Some(label.to_string());
        }
    }

    fn serial(&mut self) -> Option<String> {
        match self.disk_type {
            FloppyDiskType::Blank => {
                if self.serial.is_none() {
                    self.serial = Some(generate_serial_number());
                }
                self.serial.clone()
            }
            FloppyDiskType::ForthBoot => Some("0000000000000001".into()),
            FloppyDiskType::ForthExtra => Some("0000000000000002".into()),
        }
    }

    fn read_sector(&mut self, sector: u32) -> Option<[u8; SECTOR_SIZE]> {
        if sector > MAX_SECTOR {
            return None;
        }
        match self.disk_type {
            FloppyDiskType::Blank => {
                let bytes = self.sectors.get(&sector).and_then(|v| {
                    let mut arr = [0u8; SECTOR_SIZE];
                    if v.len() == SECTOR_SIZE {
                        arr.copy_from_slice(v);
                        Some(arr)
                    } else {
                        None
                    }
                }).unwrap_or([0u8; SECTOR_SIZE]);
                Some(bytes)
            }
            FloppyDiskType::ForthBoot => Some(read_system_image_sector(REDFORTH_IMG, sector)),
            FloppyDiskType::ForthExtra => Some(read_system_image_sector(REDFORTH_XP_IMG, sector)),
        }
    }

    fn write_sector(&mut self, sector: u32, data: &[u8; SECTOR_SIZE]) -> bool {
        if sector > MAX_SECTOR {
            return false;
        }
        match self.disk_type {
            FloppyDiskType::Blank => {
                self.sectors.insert(sector, data.to_vec());
                true
            }
            FloppyDiskType::ForthBoot | FloppyDiskType::ForthExtra => false,
        }
    }
}

/// Disk Drive state machine and Redbus peripheral controller (`TileDiskDrive`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskDrive {
    /// Peripheral bus address (default 2).
    pub rbaddr: i32,
    /// Facing rotation (0..3).
    pub rotation: u8,
    /// Whether a disk is currently inserted.
    pub has_disk: bool,
    /// Whether the drive motor is currently active / running.
    pub active: bool,
    /// Countdown ticks for drive activity (`accessTime`).
    pub access_time: u32,
    /// Selected sector number (0..2048).
    pub sector: u32,
    /// Command register (`cmdreg`).
    pub cmdreg: i32,
    /// 128-byte data sector buffer.
    pub databuf: Vec<u8>,
    /// Currently inserted floppy disk item (if any).
    pub inserted_disk: Option<FloppyDisk>,
}

impl Default for DiskDrive {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskDrive {
    /// Create a new Disk Drive in default state.
    pub fn new() -> Self {
        Self {
            rbaddr: DEFAULT_RBADDR,
            rotation: 0,
            has_disk: false,
            active: false,
            access_time: 0,
            sector: 0,
            cmdreg: 0,
            databuf: vec![0; SECTOR_SIZE],
            inserted_disk: None,
        }
    }

    /// Insert a floppy disk into the drive (`ItemDisk.onItemUseFirst` / `TileDiskDrive.setDisk`).
    /// Returns true if accepted, false if drive already contains a disk.
    pub fn insert_disk(&mut self, disk: FloppyDisk) -> bool {
        if self.has_disk || self.inserted_disk.is_some() {
            return false;
        }
        self.inserted_disk = Some(disk);
        self.has_disk = true;
        true
    }

    /// Eject the inserted disk (`TileDiskDrive.ejectDisk`).
    /// Ejection is blocked while the drive motor is running (`Active`).
    /// Returns the removed disk, or `None` if empty or busy.
    pub fn eject_disk(&mut self) -> Option<FloppyDisk> {
        if !self.has_disk || self.active {
            return None;
        }
        let disk = self.inserted_disk.take();
        self.has_disk = false;
        disk
    }

    /// Read Redbus register (`TileDiskDrive.rbRead`).
    ///
    /// - 0..=127: Data buffer bytes
    /// - 128: Sector low byte (`sector & 0xFF`)
    /// - 129: Sector high byte (`(sector >> 8) & 0xFF`)
    /// - 130: Command register (`cmdreg & 0xFF`)
    /// - 131: Status register (bit 0 = disk present, bit 1 = motor running)
    pub fn rb_read(&self, reg: usize) -> u8 {
        if reg < SECTOR_SIZE {
            return self.databuf[reg];
        }
        match reg {
            128 => (self.sector & 0xFF) as u8,
            129 => (self.sector >> 8) as u8,
            130 => (self.cmdreg & 0xFF) as u8,
            131 => self.status_flags(),
            _ => 0,
        }
    }

    /// Write Redbus register (`TileDiskDrive.rbWrite`).
    pub fn rb_write(&mut self, reg: usize, dat: u8) {
        if reg < SECTOR_SIZE {
            self.databuf[reg] = dat;
            return;
        }
        match reg {
            128 => self.sector = (self.sector & 0xFF00) | (dat as u32),
            129 => self.sector = (self.sector & 0x00FF) | ((dat as u32) << 8),
            130 => self.cmdreg = dat as i32,
            _ => {}
        }
    }

    /// Status flags register:
    /// - Bit 0 (0x01): Disk present (`hasDisk`)
    /// - Bit 1 (0x02): Motor running (`Active`)
    pub fn status_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.has_disk || self.inserted_disk.is_some() {
            flags |= 0x01;
        }
        if self.active {
            flags |= 0x02;
        }
        flags
    }

    fn fill_str(&mut self, s: &str) {
        self.databuf.fill(0);
        let b = s.as_bytes();
        let n = b.len().min(SECTOR_SIZE);
        self.databuf[..n].copy_from_slice(&b[..n]);
    }

    /// Tick the drive using its own inserted disk.
    /// Returns true if drive active state changed.
    pub fn tick_self(&mut self) -> bool {
        let mut changed = false;
        if self.cmdreg != 0 && self.cmdreg != -1 {
            if let Some(mut disk) = self.inserted_disk.take() {
                let kind = disk.disk_kind();
                self.run_cmd(&kind, &mut disk);
                self.inserted_disk = Some(disk);
                self.has_disk = true;
                self.access_time = ACCESS_TIME;
                if !self.active {
                    self.active = true;
                    changed = true;
                }
            } else {
                self.cmdreg = -1;
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

    /// Low-level per-tick update (`g()`): run a pending command with external disk storage,
    /// then count down motor activity.
    /// Returns true when `active` changed (block must be visually updated).
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
                // Read label: blank disk without label leaves databuf zeroed.
                self.databuf.fill(0);
                if system {
                    self.fill_str("System Disk");
                } else if let Some(l) = store.label() {
                    self.fill_str(&l);
                }
                self.cmdreg = 0;
            }
            2 => {
                // Write label: system disks are read-only.
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
                // Read serial number: 16 ASCII characters.
                self.databuf.fill(0);
                if let DiskKind::System(n) = kind {
                    self.fill_str(&format!("{:016}", n));
                } else if let Some(s) = store.serial() {
                    self.fill_str(&s);
                }
                self.cmdreg = 0;
            }
            4 => {
                // Read sector: 0..2048 sectors.
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
                // Write sector: 0..2048 sectors. System disks reject writes.
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

/// Disk item use handler (`ItemDisk.onItemUseFirst`): attempts to insert the disk into the drive.
pub fn on_item_use_first(disk: FloppyDisk, drive: &mut DiskDrive) -> bool {
    drive.insert_disk(disk)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_drive_default_placement_level1() {
        let drive = DiskDrive::new();
        assert_eq!(drive.rbaddr, 2);
        assert_eq!(drive.sector, 0);
        assert_eq!(drive.cmdreg, 0);
        assert_eq!(drive.databuf.len(), 128);
        assert!(!drive.has_disk);
        assert!(!drive.active);
        assert_eq!(drive.status_flags(), 0);
    }

    #[test]
    fn test_disk_insertion_and_ejection_level2() {
        let mut drive = DiskDrive::new();
        let disk = FloppyDisk::new_blank();

        // Insertion succeeds
        assert!(drive.insert_disk(disk));
        assert!(drive.has_disk);
        assert_eq!(drive.status_flags() & 1, 1);

        // Second insertion fails
        let disk2 = FloppyDisk::new_blank();
        assert!(!drive.insert_disk(disk2));

        // Eject succeeds when idle
        let ejected = drive.eject_disk();
        assert!(ejected.is_some());
        assert!(!drive.has_disk);
        assert_eq!(drive.status_flags() & 1, 0);

        // Re-insert and start command: eject blocked while active
        assert!(drive.insert_disk(FloppyDisk::new_blank()));
        drive.rb_write(130, 3); // Read serial
        drive.tick_self();
        assert!(drive.active);
        assert_eq!(drive.status_flags(), 3); // Disk present + active motor
        assert!(drive.eject_disk().is_none()); // Blocked!

        // Wait for motor to spin down
        for _ in 0..ACCESS_TIME {
            drive.tick_self();
        }
        assert!(!drive.active);
        assert!(drive.eject_disk().is_some()); // Now succeeds!
    }

    #[test]
    fn test_blank_disk_sector_read_write_level2() {
        let mut drive = DiskDrive::new();
        drive.insert_disk(FloppyDisk::new_blank());

        // Write test data into databuf
        drive.rb_write(0, 0xAB);
        drive.rb_write(1, 0xCD);
        drive.rb_write(127, 0xEF);

        // Select sector 42
        drive.rb_write(128, 42);
        drive.rb_write(129, 0);

        // Write sector command (5)
        drive.rb_write(130, 5);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0); // Command succeeded

        // Clear databuf
        for i in 0..128 {
            drive.rb_write(i, 0);
        }
        assert_eq!(drive.rb_read(0), 0);

        // Read sector command (4)
        drive.rb_write(130, 4);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0);

        // Verify data restored
        assert_eq!(drive.rb_read(0), 0xAB);
        assert_eq!(drive.rb_read(1), 0xCD);
        assert_eq!(drive.rb_read(127), 0xEF);
    }

    #[test]
    fn test_blank_disk_label_read_write() {
        let mut drive = DiskDrive::new();
        drive.insert_disk(FloppyDisk::new_blank());

        // Write label "MyDisk" into databuf
        let label = b"MyDisk";
        for (i, &b) in label.iter().enumerate() {
            drive.rb_write(i, b);
        }
        drive.rb_write(label.len(), 0); // null terminator

        // Execute command 2 (Write Label)
        drive.rb_write(130, 2);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0);

        // Clear databuf
        for i in 0..128 {
            drive.rb_write(i, 0);
        }

        // Execute command 1 (Read Label)
        drive.rb_write(130, 1);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0);
        assert_eq!(&drive.databuf[..6], b"MyDisk");
    }

    #[test]
    fn test_serial_number_generation() {
        let mut drive = DiskDrive::new();
        drive.insert_disk(FloppyDisk::new_blank());

        // Execute command 3 (Read Serial)
        drive.rb_write(130, 3);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0);

        let ser1 = std::str::from_utf8(&drive.databuf[..16]).unwrap().to_string();
        assert_eq!(ser1.len(), 16);

        // Repeated read returns identical serial
        drive.rb_write(130, 3);
        drive.tick_self();
        let ser2 = std::str::from_utf8(&drive.databuf[..16]).unwrap().to_string();
        assert_eq!(ser1, ser2);
    }

    #[test]
    fn test_system_boot_disk_readonly() {
        let mut drive = DiskDrive::new();
        drive.insert_disk(FloppyDisk::new_forth_boot());

        // Command 1: Read label returns "System Disk"
        drive.rb_write(130, 1);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0);
        assert_eq!(&drive.databuf[..11], b"System Disk");

        // Command 2: Write label fails
        drive.rb_write(130, 2);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 255); // -1 as u8

        // Command 3: Serial returns "0000000000000001"
        drive.rb_write(130, 3);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0);
        assert_eq!(&drive.databuf[..16], b"0000000000000001");

        // Command 4: Read sector 0 returns 6502 code
        drive.rb_write(128, 0);
        drive.rb_write(129, 0);
        drive.rb_write(130, 4);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0);
        assert_eq!(drive.databuf[0], 0x18); // CLC instruction in Forth bootstrap

        // Command 5: Write sector fails on system disk
        drive.rb_write(130, 5);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 255); // -1 as u8
    }

    #[test]
    fn test_system_extra_disk_readonly() {
        let mut drive = DiskDrive::new();
        drive.insert_disk(FloppyDisk::new_forth_extra());

        // Command 3: Serial returns "0000000000000002"
        drive.rb_write(130, 3);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0);
        assert_eq!(&drive.databuf[..16], b"0000000000000002");

        // Command 4: Read sector 0
        drive.rb_write(128, 0);
        drive.rb_write(129, 0);
        drive.rb_write(130, 4);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 0);
        assert_eq!(drive.databuf[0], 0x18);
    }

    #[test]
    fn test_invalid_sector_rejection() {
        let mut drive = DiskDrive::new();
        drive.insert_disk(FloppyDisk::new_blank());

        // Sector 2049 (> 2048) rejected
        drive.rb_write(128, 1);
        drive.rb_write(129, 8); // 2049
        drive.rb_write(130, 4);
        drive.tick_self();
        assert_eq!(drive.rb_read(130), 255); // -1
    }

    #[test]
    fn test_status_flags_and_activity_timeout() {
        let mut drive = DiskDrive::new();
        assert_eq!(drive.status_flags(), 0);

        drive.insert_disk(FloppyDisk::new_blank());
        assert_eq!(drive.status_flags(), 1); // disk present

        drive.rb_write(130, 1);
        drive.tick_self();
        assert_eq!(drive.status_flags(), 3); // disk present + active motor

        // Count down 5 ticks of access time
        for _ in 0..ACCESS_TIME {
            drive.tick_self();
        }
        assert_eq!(drive.status_flags(), 1); // motor stopped, disk still present
    }

    #[test]
    fn test_persistence_roundtrip_level5() {
        let mut drive = DiskDrive::new();
        let mut disk = FloppyDisk::new_blank();
        disk.set_label("SaveDisk");
        disk.write_sector(5, &[0x77; SECTOR_SIZE]);
        drive.insert_disk(disk);

        drive.rb_write(128, 5);
        drive.rb_write(129, 0);

        // Serialize drive (simulating chunk save)
        let serialized = serde_json::to_string(&drive).expect("serialization failed");

        // Deserialize drive (simulating chunk reload)
        let mut loaded: DiskDrive = serde_json::from_str(&serialized).expect("deserialization failed");

        assert!(loaded.has_disk);
        assert_eq!(loaded.rbaddr, 2);
        assert_eq!(loaded.sector, 5);

        // Verify sector data preserved across reload
        loaded.rb_write(130, 4); // Read sector
        loaded.tick_self();
        assert_eq!(loaded.rb_read(130), 0);
        assert_eq!(loaded.databuf[0], 0x77);
        assert_eq!(loaded.databuf[127], 0x77);

        // Verify label preserved across reload
        loaded.rb_write(130, 1); // Read label
        loaded.tick_self();
        assert_eq!(&loaded.databuf[..8], b"SaveDisk");
    }
}

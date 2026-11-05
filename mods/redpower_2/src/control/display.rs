//! Display / Monitor terminal Redbus peripheral (BlockPeripheral meta 0, §2.7).
//!
//! Port of `TileDisplay` (character buffer, hardware blitter, keyboard ring buffer,
//! Redbus MMIO interface at address 1), `ContainerDisplay` (RLE screen synchronization)
//! and `GuiDisplay` (key code translation and 350x230 native screen layout).

use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const BLOCK_ID_DISPLAY: &str = "redpower_2:display";
pub const BLOCK_ID_MONITOR: &str = "redpower_2:monitor";

/// 80 columns x 50 rows character matrix terminal.
pub const COLS: usize = 80;
pub const ROWS: usize = 50;
pub const SCREEN_SIZE: usize = COLS * ROWS; // 4,000 bytes
pub const KB_SIZE: usize = 16;
pub const DEFAULT_RB_ADDR: i32 = 1;
pub const REDBUS_CONNECT_CLASS: i32 = 66;
pub const CONNECTABLE_MASK_ALL: u32 = 16777215; // 0x00FF_FFFF: all 6 faces

/// Native screen dimensions for terminal GUI.
pub const GUI_WIDTH: f32 = 350.0;
pub const GUI_HEIGHT: f32 = 230.0;

/// Hardware blitter modes (`TileDisplay.runblitter`).
pub const BLIT_MODE_IDLE: u8 = 0;
/// Mode 1: fill rectangle `[blit_xd, blit_yd, blit_w, blit_h]` with character `blit_xs`.
pub const BLIT_MODE_FILL: u8 = 1;
/// Mode 2: invert character bits (`^ 128`, XOR 0x80) in rectangle.
pub const BLIT_MODE_INVERT: u8 = 2;
/// Mode 3: copy rectangle from `(blit_xs, blit_ys)` to `(blit_xd, blit_yd)` of dimensions `(blit_w, blit_h)`.
pub const BLIT_MODE_COPY: u8 = 3;

/// Cursor display modes (`cursMode`).
pub const CURS_MODE_OFF: u8 = 0;
pub const CURS_MODE_SOLID: u8 = 1;
pub const CURS_MODE_BLINK: u8 = 2;

/// Redbus MMIO register definitions.
pub const REG_MEM_ROW: usize = 0;
pub const REG_CURS_X: usize = 1;
pub const REG_CURS_Y: usize = 2;
pub const REG_CURS_MODE: usize = 3;
pub const REG_KB_START: usize = 4;
pub const REG_KB_POS: usize = 5;
pub const REG_KB_DATA: usize = 6;
pub const REG_BLIT_MODE: usize = 7;
pub const REG_BLIT_XS: usize = 8;
pub const REG_BLIT_YS: usize = 9;
pub const REG_BLIT_XD: usize = 10;
pub const REG_BLIT_YD: usize = 11;
pub const REG_BLIT_W: usize = 12;
pub const REG_BLIT_H: usize = 13;
pub const REG_ROW_MEMORY_START: usize = 16;
pub const REG_ROW_MEMORY_END: usize = 96;

/// Display / Monitor peripheral state (`TileDisplay`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Display {
    /// 4,000-byte character matrix (80x50), initialized to ASCII space (32).
    pub screen: Vec<u8>,
    /// Horizontal rotation index `0..=3` (East=0, South=1, West=2, North=3).
    pub rotation: i32,
    /// Active 80-byte memory window row (`0..=49`).
    pub mem_row: u8,
    /// Cursor X position (`0..=79`).
    pub curs_x: u8,
    /// Cursor Y position (`0..=49`).
    pub curs_y: u8,
    /// Cursor mode: 0 = off, 1 = solid block, 2 = blink.
    pub curs_mode: u8,
    /// Keyboard ring buffer read head index (`0..=15`).
    pub kb_start: u8,
    /// Keyboard ring buffer write tail index (`0..=15`).
    pub kb_pos: u8,
    /// 16-key keyboard input ring buffer.
    pub kb_buf: [u8; KB_SIZE],
    /// Blitter source X (or fill character byte).
    pub blit_xs: u8,
    /// Blitter source Y.
    pub blit_ys: u8,
    /// Blitter destination X.
    pub blit_xd: u8,
    /// Blitter destination Y.
    pub blit_yd: u8,
    /// Blitter rectangle width.
    pub blit_w: u8,
    /// Blitter rectangle height.
    pub blit_h: u8,
    /// Blitter operational mode: 0=idle, 1=fill, 2=invert, 3=block copy.
    pub blit_mode: u8,
    /// Redbus peripheral device address (default 1).
    pub rbaddr: i32,
    /// Dirty flag indicating video memory or registers changed.
    pub dirty: bool,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            screen: vec![32; SCREEN_SIZE], // ASCII space (32), exactly as TileDisplay() constructor
            rotation: 0,
            mem_row: 0,
            curs_x: 0,
            curs_y: 0,
            curs_mode: CURS_MODE_BLINK,
            kb_start: 0,
            kb_pos: 0,
            kb_buf: [0; KB_SIZE],
            blit_xs: 0,
            blit_ys: 0,
            blit_xd: 0,
            blit_yd: 0,
            blit_w: 0,
            blit_h: 0,
            blit_mode: BLIT_MODE_IDLE,
            rbaddr: DEFAULT_RB_ADDR,
            dirty: false,
        }
    }
}

impl Display {
    /// Create a new `Display` initialized to default state with empty space screen.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate rotation index `0..=3` from entity placer's yaw:
    /// `floor(yaw * 4 / 360 + 0.5) + 1 & 3` (`TileDisplay.onBlockPlaced`).
    pub fn rotation_from_yaw(yaw: f32) -> i32 {
        ((yaw * 4.0 / 360.0 + 0.5).floor() as i32 + 1) & 3
    }

    /// Map horizontal rotation index to Minecraft `Direction`.
    pub fn rot_to_direction(rot: i32) -> Direction {
        match rot & 3 {
            0 => Direction::East,
            1 => Direction::South,
            2 => Direction::West,
            _ => Direction::North,
        }
    }

    /// Get Redbus device address (`IRedbusConnectable.rbGetAddr`).
    pub const fn rb_get_addr(&self) -> i32 {
        self.rbaddr
    }

    /// Set Redbus device address (`IRedbusConnectable.rbSetAddr`).
    pub fn rb_set_addr(&mut self, addr: i32) {
        self.rbaddr = addr;
        self.dirty = true;
    }

    /// Redbus connection mask: connects in all 6 directions (`TileDisplay.getConnectableMask`).
    pub const fn connectable_mask() -> u32 {
        CONNECTABLE_MASK_ALL
    }

    /// Redbus connect class: class 66 for ribbon cables (`TileDisplay.getConnectClass`).
    pub const fn connect_class(_side: i32) -> i32 {
        REDBUS_CONNECT_CLASS
    }

    /// Corner power mode (`TileDisplay.getCornerPowerMode`).
    pub const fn corner_power_mode() -> i32 {
        0
    }

    /// Redbus MMIO register read (`TileDisplay.rbRead`).
    ///
    /// Registers 16..96 map to the 80 characters of `mem_row`.
    /// Registers 0..13 map control, cursor, keyboard and blitter registers.
    pub fn rb_read(&self, reg: usize) -> u8 {
        if (REG_ROW_MEMORY_START..REG_ROW_MEMORY_END).contains(&reg) {
            let offset = self.mem_row as usize * COLS + (reg - REG_ROW_MEMORY_START);
            return self.screen.get(offset).copied().unwrap_or(0);
        }
        match reg {
            REG_MEM_ROW => self.mem_row,
            REG_CURS_X => self.curs_x,
            REG_CURS_Y => self.curs_y,
            REG_CURS_MODE => self.curs_mode,
            REG_KB_START => self.kb_start,
            REG_KB_POS => self.kb_pos,
            REG_KB_DATA => self.kb_buf[self.kb_start as usize & 15],
            REG_BLIT_MODE => self.blit_mode,
            REG_BLIT_XS => self.blit_xs,
            REG_BLIT_YS => self.blit_ys,
            REG_BLIT_XD => self.blit_xd,
            REG_BLIT_YD => self.blit_yd,
            REG_BLIT_W => self.blit_w,
            REG_BLIT_H => self.blit_h,
            _ => 0,
        }
    }

    /// Redbus MMIO register write (`TileDisplay.rbWrite`).
    pub fn rb_write(&mut self, reg: usize, dat: u8) {
        self.dirty = true;
        if (REG_ROW_MEMORY_START..REG_ROW_MEMORY_END).contains(&reg) {
            let offset = self.mem_row as usize * COLS + (reg - REG_ROW_MEMORY_START);
            if offset < self.screen.len() {
                self.screen[offset] = dat;
            }
            return;
        }
        match reg {
            REG_MEM_ROW => self.mem_row = dat.min((ROWS - 1) as u8),
            REG_CURS_X => self.curs_x = dat,
            REG_CURS_Y => self.curs_y = dat,
            REG_CURS_MODE => self.curs_mode = dat,
            REG_KB_START => self.kb_start = dat & 15,
            REG_KB_POS => self.kb_pos = dat & 15,
            REG_KB_DATA => self.kb_buf[self.kb_start as usize & 15] = dat,
            REG_BLIT_MODE => self.blit_mode = dat,
            REG_BLIT_XS => self.blit_xs = dat,
            REG_BLIT_YS => self.blit_ys = dat,
            REG_BLIT_XD => self.blit_xd = dat,
            REG_BLIT_YD => self.blit_yd = dat,
            REG_BLIT_W => self.blit_w = dat,
            REG_BLIT_H => self.blit_h = dat,
            _ => {}
        }
    }

    /// Push a key byte into the 16-key keyboard buffer (`TileDisplay.pushKey`).
    /// Drops the key if the ring buffer is full. Returns true if queued.
    pub fn push_key(&mut self, b: u8) -> bool {
        let np = (self.kb_pos + 1) & 15;
        if np != self.kb_start {
            self.kb_buf[self.kb_pos as usize] = b;
            self.kb_pos = np;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Pop a key byte from the keyboard buffer.
    pub fn pop_key(&mut self) -> Option<u8> {
        if self.kb_start == self.kb_pos {
            None
        } else {
            let b = self.kb_buf[self.kb_start as usize];
            self.kb_start = (self.kb_start + 1) & 15;
            self.dirty = true;
            Some(b)
        }
    }

    /// Peek at the current key under `kb_start` without advancing the pointer.
    pub fn peek_key(&self) -> Option<u8> {
        if self.kb_start == self.kb_pos {
            None
        } else {
            Some(self.kb_buf[self.kb_start as usize])
        }
    }

    /// Number of keys queued in the keyboard ring buffer.
    pub fn key_count(&self) -> usize {
        ((self.kb_pos + 16 - self.kb_start) & 15) as usize
    }

    /// Clear the entire 80x50 character screen to a given character byte.
    pub fn clear_screen(&mut self, fill: u8) {
        self.screen.fill(fill);
        self.dirty = true;
    }

    /// Scroll display up by `lines` rows, filling revealed bottom lines with spaces.
    pub fn scroll_up(&mut self, lines: usize) {
        if lines == 0 {
            return;
        }
        if lines >= ROWS {
            self.clear_screen(32);
            return;
        }
        let copy_rows = ROWS - lines;
        self.blit_xs = 0;
        self.blit_ys = lines as u8;
        self.blit_xd = 0;
        self.blit_yd = 0;
        self.blit_w = COLS as u8;
        self.blit_h = copy_rows as u8;
        self.blit_mode = BLIT_MODE_COPY;
        self.run_blitter();

        // Clear bottom rows to spaces
        self.blit_xs = 32;
        self.blit_xd = 0;
        self.blit_yd = copy_rows as u8;
        self.blit_w = COLS as u8;
        self.blit_h = lines as u8;
        self.blit_mode = BLIT_MODE_FILL;
        self.run_blitter();
    }

    /// World tick update (`TileDisplay.g()`): runs the hardware blitter if scheduled.
    /// Returns true if the screen was modified.
    pub fn tick(&mut self) -> bool {
        self.run_blitter()
    }

    /// Execute scheduled blitter operation (`TileDisplay.runblitter`).
    pub fn run_blitter(&mut self) -> bool {
        if self.blit_mode == BLIT_MODE_IDLE {
            return false;
        }
        self.dirty = true;

        let w = (self.blit_w as i32).min(COLS as i32 - self.blit_xd as i32);
        let h = (self.blit_h as i32).min(ROWS as i32 - self.blit_yd as i32);
        if w < 0 || h < 0 {
            self.blit_mode = BLIT_MODE_IDLE;
            return true;
        }

        let doffs = self.blit_yd as usize * COLS + self.blit_xd as usize;
        match self.blit_mode {
            BLIT_MODE_FILL => {
                let (w, h) = (w as usize, h as usize);
                for j in 0..h {
                    for i in 0..w {
                        self.screen[doffs + COLS * j + i] = self.blit_xs;
                    }
                }
                self.blit_mode = BLIT_MODE_IDLE;
            }
            BLIT_MODE_INVERT => {
                let (w, h) = (w as usize, h as usize);
                for j in 0..h {
                    for i in 0..w {
                        self.screen[doffs + COLS * j + i] ^= 0x80;
                    }
                }
                self.blit_mode = BLIT_MODE_IDLE;
            }
            m => {
                let w = w.min(COLS as i32 - self.blit_xs as i32);
                let h = h.min(ROWS as i32 - self.blit_ys as i32);
                if w < 0 || h < 0 {
                    self.blit_mode = BLIT_MODE_IDLE;
                } else if m == BLIT_MODE_COPY {
                    let (w, h) = (w as usize, h as usize);
                    let soffs = self.blit_ys as usize * COLS + self.blit_xs as usize;
                    // Forward row-major copy without overlap buffering, matching original Java behavior
                    for j in 0..h {
                        for i in 0..w {
                            self.screen[doffs + COLS * j + i] = self.screen[soffs + COLS * j + i];
                        }
                    }
                    self.blit_mode = BLIT_MODE_IDLE;
                } else {
                    self.blit_mode = BLIT_MODE_IDLE;
                }
            }
        }
        true
    }

    /// Encode into Horizon block entity NBT string map.
    pub fn to_nbt_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("rot".into(), self.rotation.to_string());
        map.insert("row".into(), self.mem_row.to_string());
        map.insert("cx".into(), self.curs_x.to_string());
        map.insert("cy".into(), self.curs_y.to_string());
        map.insert("cm".into(), self.curs_mode.to_string());
        map.insert("kbs".into(), self.kb_start.to_string());
        map.insert("kbp".into(), self.kb_pos.to_string());
        map.insert("blxs".into(), self.blit_xs.to_string());
        map.insert("blys".into(), self.blit_ys.to_string());
        map.insert("blxd".into(), self.blit_xd.to_string());
        map.insert("blyd".into(), self.blit_yd.to_string());
        map.insert("blw".into(), self.blit_w.to_string());
        map.insert("blh".into(), self.blit_h.to_string());
        map.insert("blmd".into(), self.blit_mode.to_string());
        map.insert("rbaddr".into(), self.rbaddr.to_string());

        // Hex encode 4000-byte video buffer
        let fb_hex: String = self.screen.iter().map(|b| format!("{b:02x}")).collect();
        map.insert("fb".into(), fb_hex);

        // Hex encode 16-byte keyboard buffer
        let kbb_hex: String = self.kb_buf.iter().map(|b| format!("{b:02x}")).collect();
        map.insert("kbb".into(), kbb_hex);

        map
    }

    /// Decode from Horizon block entity NBT string map.
    pub fn from_nbt_map(map: &HashMap<String, String>) -> Self {
        let mut d = Self::default();

        let get_u8 = |k: &str, def: u8| -> u8 {
            map.get(k).and_then(|s| s.parse::<u8>().ok()).unwrap_or(def)
        };
        let get_i32 = |k: &str, def: i32| -> i32 {
            map.get(k)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(def)
        };

        d.rotation = get_i32("rot", 0) & 3;
        d.mem_row = get_u8("row", 0).min((ROWS - 1) as u8);
        d.curs_x = get_u8("cx", 0);
        d.curs_y = get_u8("cy", 0);
        d.curs_mode = get_u8("cm", CURS_MODE_BLINK);
        d.kb_start = get_u8("kbs", 0) & 15;
        d.kb_pos = get_u8("kbp", 0) & 15;
        d.blit_xs = get_u8("blxs", 0);
        d.blit_ys = get_u8("blys", 0);
        d.blit_xd = get_u8("blxd", 0);
        d.blit_yd = get_u8("blyd", 0);
        d.blit_w = get_u8("blw", 0);
        d.blit_h = get_u8("blh", 0);
        d.blit_mode = get_u8("blmd", 0);
        d.rbaddr = get_i32("rbaddr", DEFAULT_RB_ADDR);

        if let Some(hex) = map.get("fb") {
            if hex.len() == SCREEN_SIZE * 2 {
                for i in 0..SCREEN_SIZE {
                    if let Ok(val) = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16) {
                        d.screen[i] = val;
                    }
                }
            }
        }

        if let Some(hex) = map.get("kbb") {
            if hex.len() == KB_SIZE * 2 {
                for i in 0..KB_SIZE {
                    if let Ok(val) = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16) {
                        d.kb_buf[i] = val;
                    }
                }
            }
        }

        d
    }

    /// Compute RLE delta update between current screen and `prev_screen` (`ContainerDisplay.getDisplayRLE`).
    /// Copies updated bytes into `prev_screen`.
    pub fn get_display_rle(&self, prev_screen: &mut [u8]) -> Option<Vec<u8>> {
        let mut rle = RleCompressor::new();
        let len = SCREEN_SIZE.min(prev_screen.len());
        for (i, cur) in prev_screen.iter_mut().enumerate().take(len) {
            let diff = self.screen[i] != *cur;
            rle.add_byte(self.screen[i], diff);
            *cur = self.screen[i];
        }
        rle.flush();
        rle.get_bytes()
    }
}

/// Key code translation matching `GuiDisplay.a(char, int)`.
///
/// Modifiers: shift -> 64, ctrl -> 32.
/// LWJGL special keys: 199=Home(132), 200=Up(128), 203=Left(130), 205=Right(131),
/// 207=End(133), 208=Down(129), 210=Insert(134).
pub fn encode_key(ch: char, lwjgl: i32, shift: bool, ctrl: bool) -> Option<u8> {
    let ch = if ch == '\n' { '\r' } else { ch };
    let m = (if shift { 64 } else { 0 }) | (if ctrl { 32 } else { 0 });
    Some(match lwjgl {
        199 => 132 | m,
        200 => 128 | m,
        203 => 130 | m,
        205 => 131 | m,
        207 => 133 | m,
        208 => 129 | m,
        210 => 134 | m,
        _ => {
            let c = ch as u32;
            if c > 0 && c <= 127 {
                c as u8
            } else {
                return None;
            }
        }
    })
}

/// Streaming RLE compressor (`ContainerDisplay.RLECompressor`).
#[derive(Debug)]
pub struct RleCompressor {
    output: Vec<u8>,
    datbuf: [u8; 256],
    srledat: u8,
    rleoffs: usize,
    srleoffs: usize,
    datpos: usize,
    changed: bool,
}

impl Default for RleCompressor {
    fn default() -> Self {
        Self {
            output: Vec::new(),
            datbuf: [0u8; 256],
            srledat: 0,
            rleoffs: 0,
            srleoffs: 0,
            datpos: 0,
            changed: false,
        }
    }
}

impl RleCompressor {
    pub fn new() -> Self {
        Self::default()
    }

    fn write_rle(&mut self) {
        self.output.push(self.rleoffs as u8);
        self.datpos = 0;
        self.rleoffs = 0;
        self.srleoffs = 0;
    }

    fn write_srle(&mut self) {
        self.output.push(0xFF);
        self.output.push(self.srleoffs as u8);
        self.output.push(self.srledat);
        self.datpos = 0;
        self.rleoffs = 0;
        self.srleoffs = 0;
    }

    fn write_dat(&mut self, bytes: usize) {
        if bytes != 0 {
            self.output.push((128 | bytes) as u8);
            self.output.extend_from_slice(&self.datbuf[..bytes]);
            self.datpos = self.datpos.saturating_sub(bytes);
        }
    }

    pub fn add_byte(&mut self, b: u8, diff: bool) {
        if diff {
            self.changed = true;
            if self.rleoffs > 5 && self.rleoffs >= self.srleoffs {
                let to_write = self.datpos - self.rleoffs;
                self.write_dat(to_write);
                self.write_rle();
            }
            self.rleoffs = 0;
        } else {
            self.rleoffs += 1;
            if self.rleoffs >= 127 {
                self.datpos += 1;
                let to_write = self.datpos - self.rleoffs;
                self.write_dat(to_write);
                self.write_rle();
                return;
            }
        }

        if self.srleoffs == 0 {
            self.srledat = b;
            self.srleoffs = 1;
        } else if b == self.srledat {
            self.srleoffs += 1;
            if self.srleoffs >= 127 {
                self.datpos += 1;
                let to_write = self.datpos - self.srleoffs;
                self.write_dat(to_write);
                self.write_srle();
                return;
            }
        } else {
            if self.srleoffs > 5 && self.srleoffs >= self.rleoffs {
                let to_write = self.datpos - self.srleoffs;
                self.write_dat(to_write);
                self.write_srle();
            }
            self.srledat = b;
            self.srleoffs = 1;
        }

        self.datbuf[self.datpos] = b;
        self.datpos += 1;
        let rem = self.srleoffs.max(self.rleoffs);
        if rem <= 5 && self.datpos >= 126 {
            let to_write = self.datpos;
            self.write_dat(to_write);
            self.srleoffs = 0;
            self.rleoffs = 0;
        } else if self.datpos >= rem && self.datpos - rem >= 126 {
            let to_write = self.datpos - rem;
            self.write_dat(to_write);
        }
    }

    pub fn flush(&mut self) {
        self.datpos = self.datpos.saturating_sub(self.rleoffs);
        self.srleoffs = self.srleoffs.saturating_sub(self.rleoffs);
        if self.datpos != 0 {
            if self.srleoffs > 5 {
                let to_write = self.datpos - self.srleoffs;
                self.write_dat(to_write);
                self.write_srle();
            } else {
                let to_write = self.datpos;
                self.write_dat(to_write);
            }
        }
    }

    pub fn get_bytes(&self) -> Option<Vec<u8>> {
        if !self.changed {
            None
        } else {
            Some(self.output.clone())
        }
    }
}

/// Decompress a `ContainerDisplay` RLE screen update into `out` (`ContainerDisplay.decompress`).
pub fn rle_decompress(compress: &[u8], out: &mut [u8]) {
    let (mut opos, mut i) = (0usize, 0usize);
    while i < compress.len() {
        if opos >= out.len() {
            return;
        }
        let cmd = compress[i] as usize;
        i += 1;
        if cmd & 128 == 0 {
            opos += cmd & 127;
        } else if cmd == 255 {
            if i + 2 > compress.len() {
                return;
            }
            let ln = (compress[i] as usize).min(out.len() - opos);
            for j in 0..ln {
                out[opos + j] = compress[i + 1];
            }
            opos += ln;
            i += 2;
        } else {
            let ln = (cmd & 127).min(out.len() - opos).min(compress.len() - i);
            out[opos..opos + ln].copy_from_slice(&compress[i..i + ln]);
            opos += ln;
            i += ln;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_defaults_and_level1_placement() {
        let d = Display::new();
        assert_eq!(d.rbaddr, 1);
        assert_eq!(d.curs_mode, CURS_MODE_BLINK);
        assert_eq!(d.screen.len(), SCREEN_SIZE);
        assert!(d.screen.iter().all(|&b| b == 32)); // Initialized with ASCII space
        assert_eq!(Display::connectable_mask(), 16777215);
        assert_eq!(Display::connect_class(0), 66);
        assert_eq!(Display::corner_power_mode(), 0);

        // Yaw to horizontal rotation
        assert_eq!(Display::rotation_from_yaw(0.0), 1); // South
        assert_eq!(Display::rotation_from_yaw(90.0), 2); // West
        assert_eq!(Display::rotation_from_yaw(180.0), 3); // North
        assert_eq!(Display::rotation_from_yaw(270.0), 0); // East

        assert_eq!(Display::rot_to_direction(0), Direction::East);
        assert_eq!(Display::rot_to_direction(1), Direction::South);
        assert_eq!(Display::rot_to_direction(2), Direction::West);
        assert_eq!(Display::rot_to_direction(3), Direction::North);
    }

    #[test]
    fn test_display_character_buffer_write_level3() {
        let mut d = Display::new();
        // Row selection and clamp
        d.rb_write(REG_MEM_ROW, 200);
        assert_eq!(d.rb_read(REG_MEM_ROW), 49);

        d.rb_write(REG_MEM_ROW, 5);
        assert_eq!(d.rb_read(REG_MEM_ROW), 5);

        // Write row character buffer via registers 16..96
        d.rb_write(16, b'H');
        d.rb_write(17, b'E');
        d.rb_write(18, b'L');
        d.rb_write(19, b'L');
        d.rb_write(20, b'O');
        d.rb_write(95, b'!');

        assert_eq!(d.screen[5 * COLS], b'H');
        assert_eq!(d.screen[5 * COLS + 1], b'E');
        assert_eq!(d.screen[5 * COLS + 2], b'L');
        assert_eq!(d.screen[5 * COLS + 3], b'L');
        assert_eq!(d.screen[5 * COLS + 4], b'O');
        assert_eq!(d.screen[5 * COLS + 79], b'!');

        assert_eq!(d.rb_read(16), b'H');
        assert_eq!(d.rb_read(17), b'E');
        assert_eq!(d.rb_read(20), b'O');
        assert_eq!(d.rb_read(95), b'!');

        // Cursor coordinates and modes
        d.rb_write(REG_CURS_X, 42);
        d.rb_write(REG_CURS_Y, 24);
        d.rb_write(REG_CURS_MODE, CURS_MODE_SOLID);
        assert_eq!(d.rb_read(REG_CURS_X), 42);
        assert_eq!(d.rb_read(REG_CURS_Y), 24);
        assert_eq!(d.rb_read(REG_CURS_MODE), 1);
    }

    #[test]
    fn test_display_blitter_scrolling_and_area_fill_level3() {
        let mut d = Display::new();

        // 1. Area fill (mode 1)
        d.rb_write(REG_BLIT_XD, 10);
        d.rb_write(REG_BLIT_YD, 5);
        d.rb_write(REG_BLIT_W, 20);
        d.rb_write(REG_BLIT_H, 4);
        d.rb_write(REG_BLIT_XS, b'#');
        d.rb_write(REG_BLIT_MODE, BLIT_MODE_FILL);

        assert!(d.tick());
        assert_eq!(d.rb_read(REG_BLIT_MODE), BLIT_MODE_IDLE);

        for y in 5..9 {
            for x in 10..30 {
                assert_eq!(d.screen[y * COLS + x], b'#');
            }
        }
        assert_eq!(d.screen[4 * COLS + 10], 32);
        assert_eq!(d.screen[5 * COLS + 9], 32);

        // 2. Invert mode (mode 2)
        d.rb_write(REG_BLIT_XD, 10);
        d.rb_write(REG_BLIT_YD, 5);
        d.rb_write(REG_BLIT_W, 5);
        d.rb_write(REG_BLIT_H, 1);
        d.rb_write(REG_BLIT_MODE, BLIT_MODE_INVERT);

        d.tick();
        assert_eq!(d.screen[5 * COLS + 10], b'#' ^ 0x80);
        assert_eq!(d.screen[5 * COLS + 14], b'#' ^ 0x80);
        assert_eq!(d.screen[5 * COLS + 15], b'#');

        // 3. Block copy / Terminal scrolling (mode 3)
        // Put text on row 1
        d.rb_write(REG_MEM_ROW, 1);
        for col in 0..10 {
            d.rb_write(REG_ROW_MEMORY_START + col, b'0' + col as u8);
        }

        // Scroll up by copying row 1 to row 0
        d.rb_write(REG_BLIT_XS, 0);
        d.rb_write(REG_BLIT_YS, 1);
        d.rb_write(REG_BLIT_XD, 0);
        d.rb_write(REG_BLIT_YD, 0);
        d.rb_write(REG_BLIT_W, COLS as u8);
        d.rb_write(REG_BLIT_H, 1);
        d.rb_write(REG_BLIT_MODE, BLIT_MODE_COPY);

        d.tick();
        assert_eq!(d.screen[0], b'0');
        assert_eq!(d.screen[9], b'9');

        // Full terminal scroll helper
        d.clear_screen(b'.');
        d.screen[COLS] = b'T';
        d.screen[COLS + 1] = b'E';
        d.screen[COLS + 2] = b'S';
        d.screen[COLS + 3] = b'T';

        d.scroll_up(1);
        assert_eq!(d.screen[0], b'T');
        assert_eq!(d.screen[1], b'E');
        assert_eq!(d.screen[2], b'S');
        assert_eq!(d.screen[3], b'T');
        assert_eq!(d.screen[COLS], b'.');
        // Bottom row was cleared to space (32)
        assert_eq!(d.screen[(ROWS - 1) * COLS], 32);
    }

    #[test]
    fn test_display_keyboard_ring_buffer() {
        let mut d = Display::new();
        assert_eq!(d.key_count(), 0);
        assert_eq!(d.pop_key(), None);

        // Push 15 keys (ring capacity is 15 keys with 1 sentinel)
        for i in 1..=15u8 {
            assert!(d.push_key(i));
        }
        assert_eq!(d.key_count(), 15);
        // 16th key is dropped when buffer is full
        assert!(!d.push_key(99));
        assert_eq!(d.key_count(), 15);

        // Read through Redbus MMIO reg 6
        assert_eq!(d.rb_read(REG_KB_DATA), 1);
        assert_eq!(d.pop_key(), Some(1));
        assert_eq!(d.rb_read(REG_KB_DATA), 2);

        // Push wrapped
        assert!(d.push_key(100));
        assert_eq!(d.key_count(), 15);
    }

    #[test]
    fn test_display_persistence_level5() {
        let mut d = Display::new();
        d.rotation = 2;
        d.rbaddr = 7;
        d.mem_row = 12;
        d.curs_x = 33;
        d.curs_y = 14;
        d.curs_mode = CURS_MODE_SOLID;
        d.screen[12 * COLS + 33] = b'X';
        d.push_key(b'K');

        let map = d.to_nbt_map();
        let restored = Display::from_nbt_map(&map);

        assert_eq!(restored.rotation, 2);
        assert_eq!(restored.rbaddr, 7);
        assert_eq!(restored.mem_row, 12);
        assert_eq!(restored.curs_x, 33);
        assert_eq!(restored.curs_y, 14);
        assert_eq!(restored.curs_mode, 1);
        assert_eq!(restored.screen[12 * COLS + 33], b'X');
        assert_eq!(restored.peek_key(), Some(b'K'));
    }

    #[test]
    fn test_rle_compression_and_decompression() {
        let mut d = Display::new();
        let mut shadow = vec![32u8; SCREEN_SIZE];

        // No changes -> get_display_rle returns None
        assert_eq!(d.get_display_rle(&mut shadow), None);

        // Make changes
        d.screen[0] = b'R';
        d.screen[1] = b'E';
        d.screen[2] = b'D';
        d.screen[3] = b'B';
        d.screen[4] = b'U';
        d.screen[5] = b'S';

        let rle = d.get_display_rle(&mut shadow).expect("RLE packet");
        assert!(!rle.is_empty());

        let mut client_screen = vec![32u8; SCREEN_SIZE];
        rle_decompress(&rle, &mut client_screen);

        assert_eq!(&client_screen[0..6], b"REDBUS");
        assert_eq!(client_screen[6], 32);
    }

    #[test]
    fn test_key_encoding_modifiers() {
        assert_eq!(encode_key('\0', 200, false, false), Some(128)); // Up
        assert_eq!(encode_key('\0', 208, true, false), Some(129 | 64)); // Shift+Down
        assert_eq!(encode_key('\0', 203, false, true), Some(130 | 32)); // Ctrl+Left
        assert_eq!(encode_key('\0', 205, true, true), Some(131 | 96)); // Ctrl+Shift+Right
        assert_eq!(encode_key('\n', 28, false, false), Some(13)); // Enter -> \r (13)
        assert_eq!(encode_key('A', 30, false, false), Some(b'A'));
        assert_eq!(encode_key('z', 44, false, false), Some(b'z'));
        assert_eq!(encode_key('ÿ', 0, false, false), None);
    }
}

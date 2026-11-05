//! Display terminal Redbus device: port of `TileDisplay` (register file, blitter, keyboard
//! ring buffer), `ContainerDisplay` (RLE screen sync) and `GuiDisplay` (key encoding).

use serde::{Deserialize, Serialize};

pub const COLS: usize = 80;
pub const ROWS: usize = 50;
pub const SCREEN_SIZE: usize = COLS * ROWS;
pub const KB_SIZE: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    pub screen: Vec<u8>,
    pub rotation: i32,
    pub mem_row: u8,
    pub curs_x: u8,
    pub curs_y: u8,
    /// 0 = off, 1 = solid, 2 = blink.
    pub curs_mode: u8,
    pub kb_start: u8,
    pub kb_pos: u8,
    pub kb_buf: [u8; KB_SIZE],
    pub blit_xs: u8,
    pub blit_ys: u8,
    pub blit_xd: u8,
    pub blit_yd: u8,
    pub blit_w: u8,
    pub blit_h: u8,
    pub blit_mode: u8,
    pub rbaddr: i32,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            screen: vec![0; SCREEN_SIZE],
            rotation: 0,
            mem_row: 0,
            curs_x: 0,
            curs_y: 0,
            curs_mode: 2,
            kb_start: 0,
            kb_pos: 0,
            kb_buf: [0; KB_SIZE],
            blit_xs: 0,
            blit_ys: 0,
            blit_xd: 0,
            blit_yd: 0,
            blit_w: 0,
            blit_h: 0,
            blit_mode: 0,
            rbaddr: 1,
        }
    }
}

impl Display {
    /// Rotation from the placer's yaw: `floor(yaw * 4 / 360 + 0.5) + 1 & 3`.
    pub fn rotation_from_yaw(yaw: f32) -> i32 {
        ((yaw * 4.0 / 360.0 + 0.5).floor() as i32 + 1) & 3
    }

    pub fn rb_read(&self, reg: usize) -> u8 {
        if (16..96).contains(&reg) {
            return self.screen[self.mem_row as usize * COLS + reg - 16];
        }
        match reg {
            0 => self.mem_row,
            1 => self.curs_x,
            2 => self.curs_y,
            3 => self.curs_mode,
            4 => self.kb_start,
            5 => self.kb_pos,
            6 => self.kb_buf[self.kb_start as usize & 15],
            7 => self.blit_mode,
            8 => self.blit_xs,
            9 => self.blit_ys,
            10 => self.blit_xd,
            11 => self.blit_yd,
            12 => self.blit_w,
            13 => self.blit_h,
            _ => 0,
        }
    }

    pub fn rb_write(&mut self, reg: usize, dat: u8) {
        if (16..96).contains(&reg) {
            self.screen[self.mem_row as usize * COLS + reg - 16] = dat;
            return;
        }
        match reg {
            0 => self.mem_row = dat.min((ROWS - 1) as u8),
            1 => self.curs_x = dat,
            2 => self.curs_y = dat,
            3 => self.curs_mode = dat,
            4 => self.kb_start = dat & 15,
            5 => self.kb_pos = dat & 15,
            6 => self.kb_buf[self.kb_start as usize] = dat,
            7 => self.blit_mode = dat,
            8 => self.blit_xs = dat,
            9 => self.blit_ys = dat,
            10 => self.blit_xd = dat,
            11 => self.blit_yd = dat,
            12 => self.blit_w = dat,
            13 => self.blit_h = dat,
            _ => {}
        }
    }

    /// Server side of a GUI key packet (`TileDisplay.pushKey`): dropped when the ring is full.
    pub fn push_key(&mut self, b: u8) {
        let np = (self.kb_pos + 1) & 15;
        if np != self.kb_start {
            self.kb_buf[self.kb_pos as usize] = b;
            self.kb_pos = np;
        }
    }

    /// Per-tick update (`g()`): run the blitter. Returns true when the screen may have changed.
    pub fn tick(&mut self) -> bool {
        self.run_blitter()
    }

    fn run_blitter(&mut self) -> bool {
        if self.blit_mode == 0 {
            return false;
        }
        let w = (self.blit_w as i32).min(COLS as i32 - self.blit_xd as i32);
        let h = (self.blit_h as i32).min(ROWS as i32 - self.blit_yd as i32);
        if w < 0 || h < 0 {
            self.blit_mode = 0;
            return true;
        }
        let (w, h) = (w as usize, h as usize);
        let doffs = self.blit_yd as usize * COLS + self.blit_xd as usize;
        match self.blit_mode {
            1 => {
                for j in 0..h {
                    for i in 0..w {
                        self.screen[doffs + COLS * j + i] = self.blit_xs;
                    }
                }
                self.blit_mode = 0;
            }
            2 => {
                for j in 0..h {
                    for i in 0..w {
                        self.screen[doffs + COLS * j + i] ^= 0x80;
                    }
                }
                self.blit_mode = 0;
            }
            m => {
                let w = (w as i32).min(COLS as i32 - self.blit_xs as i32);
                let h = (h as i32).min(ROWS as i32 - self.blit_ys as i32);
                if w < 0 || h < 0 {
                    self.blit_mode = 0;
                } else if m == 3 {
                    let (w, h) = (w as usize, h as usize);
                    let soffs = self.blit_ys as usize * COLS + self.blit_xs as usize;
                    // Forward row-major copy, as the original (overlap is not buffered).
                    for j in 0..h {
                        for i in 0..w {
                            self.screen[doffs + COLS * j + i] = self.screen[soffs + COLS * j + i];
                        }
                    }
                    self.blit_mode = 0;
                }
            }
        }
        true
    }
}

/// Key code the GUI sends for a key press (`GuiDisplay.a(char, int)`). `lwjgl` is the LWJGL
/// key code, `ch` the typed character, `ctrl`/`shift` the modifier state (shift -> 64, ctrl -> 32
/// in the original's `p()`/`o()` order). Returns `None` when nothing is sent.
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

/// Decompress a `ContainerDisplay` RLE screen update into `out`.
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
    fn registers_roundtrip_and_row_clamp() {
        let mut d = Display::default();
        d.rb_write(0, 200);
        assert_eq!(d.rb_read(0), 49);
        d.rb_write(0, 2);
        d.rb_write(16, b'A');
        d.rb_write(95, b'Z');
        assert_eq!(d.screen[2 * 80], b'A');
        assert_eq!(d.rb_read(95), b'Z');
        assert_eq!(d.rb_read(3), 2);
    }

    #[test]
    fn keyboard_ring_holds_fifteen() {
        let mut d = Display::default();
        for k in 0..20u8 {
            d.push_key(k + 1);
        }
        assert_eq!(d.kb_pos, 15);
        assert_eq!(d.rb_read(6), 1);
        d.rb_write(4, 1);
        assert_eq!(d.rb_read(6), 2);
        d.push_key(99);
        assert_eq!(d.kb_pos, 0);
    }

    #[test]
    fn blitter_fill_invert_copy() {
        let mut d = Display {
            blit_xd: 70,
            blit_yd: 48,
            blit_w: 20,
            blit_h: 10,
            blit_xs: b'x',
            blit_mode: 1,
            ..Default::default()
        };
        assert!(d.tick());
        assert_eq!(d.blit_mode, 0);
        assert_eq!(d.screen[48 * 80 + 79], b'x');
        assert_eq!(d.screen[49 * 80 + 70], b'x');
        assert_eq!(d.screen.iter().filter(|&&b| b == b'x').count(), 20);
        d.blit_mode = 2;
        d.tick();
        assert_eq!(d.screen[48 * 80 + 70], b'x' ^ 0x80);
        d.blit_xs = 70;
        d.blit_ys = 48;
        d.blit_xd = 0;
        d.blit_yd = 0;
        d.blit_w = 3;
        d.blit_h = 2;
        d.blit_mode = 3;
        d.tick();
        assert_eq!(d.screen[81], b'x' ^ 0x80);
        assert!(!d.tick());
    }

    #[test]
    fn key_encoding() {
        assert_eq!(encode_key('\0', 200, false, false), Some(128));
        assert_eq!(encode_key('\0', 203, true, true), Some(130 | 96));
        assert_eq!(encode_key('\n', 28, false, false), Some(13));
        assert_eq!(encode_key('é', 18, false, false), None);
    }

    #[test]
    fn rle() {
        let mut out = vec![0u8; 10];
        rle_decompress(&[2, 0x82, 7, 8, 255, 3, 9, 1], &mut out);
        assert_eq!(out, vec![0, 0, 7, 8, 9, 9, 9, 0, 0, 0]);
        assert_eq!(Display::rotation_from_yaw(0.0), 1);
    }
}

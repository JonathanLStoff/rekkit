//! ContainerDisplay and GuiDisplay for the Display / Terminal peripheral
//! (`com.eloraam.redpower.control.ContainerDisplay`, `com.eloraam.redpower.control.GuiDisplay`,
//! `TileDisplay`, `TileKeyboard`).
//!
//! Provides the terminal user interface:
//! - 80 columns x 50 rows character matrix terminal (monochrome glyph rendering).
//! - Double-sized rendering rects (`drawDoubledRect`).
//! - Hardware cursor modes (0=off, 1=solid block, 2=blink).
//! - RLE screen synchronization (`ContainerDisplay.getDisplayRLE` / `decompress`).
//! - Keyboard input capture and forwarding into the terminal ring buffer via Packet 212.
//! - Keyboard block (`redpower_2:keyboard`) contract and interaction integration.

use crate::control::display::{
    encode_key, rle_decompress, Display, BLIT_MODE_IDLE, COLS, CURS_MODE_BLINK, CURS_MODE_OFF,
    CURS_MODE_SOLID, ROWS, SCREEN_SIZE,
};
use crate::core::foundation::Packet212GuiEvent;
use serde::{Deserialize, Serialize};

/// GUI dimensions for GuiDisplay (`com.eloraam.redpower.control.GuiDisplay`).
pub const GUI_WIDTH: f32 = 350.0;
pub const GUI_HEIGHT: f32 = 230.0;

/// Screen offset within GuiDisplay window.
pub const SCREEN_X_OFFSET: f32 = 15.0;
pub const SCREEN_Y_OFFSET: f32 = 15.0;

/// Character glyph cell dimensions on screen (doubled pixels).
pub const GLYPH_WIDTH: f32 = 4.0;
pub const GLYPH_HEIGHT: f32 = 4.0;

/// Hardware cursor blink period in world ticks (Java RedPower 2 cursor blinks every ~10 ticks).
pub const CURSOR_BLINK_INTERVAL_TICKS: u64 = 10;

/// ContainerDisplay manages synchronized state between server `Display` block entity
/// and client terminal GUI (`GuiDisplay`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerDisplay {
    /// Window ID assigned by container manager.
    pub window_id: u8,
    /// Player entity ID viewing the display container.
    pub player_id: u64,
    /// Cached copy of terminal character screen buffer on client/container.
    pub client_screen: Vec<u8>,
    /// Cursor X position (0..80).
    pub curs_x: u8,
    /// Cursor Y position (0..50).
    pub curs_y: u8,
    /// Cursor mode: 0=off, 1=solid, 2=blink.
    pub curs_mode: u8,
    /// Blitter parameters mirror.
    pub blit_mode: u8,
}

impl Default for ContainerDisplay {
    fn default() -> Self {
        Self {
            window_id: 0,
            player_id: 0,
            client_screen: vec![32u8; SCREEN_SIZE],
            curs_x: 0,
            curs_y: 0,
            curs_mode: CURS_MODE_BLINK,
            blit_mode: BLIT_MODE_IDLE,
        }
    }
}

impl ContainerDisplay {
    /// Create a new ContainerDisplay for a player opening a display/terminal.
    pub fn new(window_id: u8, player_id: u64) -> Self {
        Self {
            window_id,
            player_id,
            ..Default::default()
        }
    }

    /// Check if container can be interacted with / remains valid for player.
    pub fn can_interact_with(&self, player_id: u64) -> bool {
        self.player_id == player_id
    }

    /// Synchronize display server state to container, generating RLE packet delta if changed.
    pub fn update_from_tile(&mut self, tile: &mut Display) -> Option<Vec<u8>> {
        self.curs_x = tile.curs_x;
        self.curs_y = tile.curs_y;
        self.curs_mode = tile.curs_mode;
        self.blit_mode = tile.blit_mode;

        // Use tile's get_display_rle against client_screen
        tile.get_display_rle(&mut self.client_screen)
    }

    /// Apply an incoming RLE packet payload on client side to update `client_screen`.
    pub fn apply_rle_update(&mut self, rle_data: &[u8]) {
        rle_decompress(rle_data, &mut self.client_screen);
    }
}

/// Double-sized rendering rect representing doubled pixels in RedPower GUI renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoubledRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub u: f32,
    pub v: f32,
}

/// GuiDisplay simulates the client-side terminal display renderer and input handler.
#[derive(Debug, Clone, PartialEq)]
pub struct GuiDisplay {
    /// Window width.
    pub width: f32,
    /// Window height.
    pub height: f32,
    /// Terminal character buffer (80x50 ASCII chars).
    pub screen: Vec<u8>,
    /// Cursor X position (0..80).
    pub curs_x: u8,
    /// Cursor Y position (0..50).
    pub curs_y: u8,
    /// Cursor mode: 0=off, 1=solid, 2=blink.
    pub curs_mode: u8,
    /// Animation tick counter for cursor blink.
    pub tick_count: u64,
}

impl Default for GuiDisplay {
    fn default() -> Self {
        Self {
            width: GUI_WIDTH,
            height: GUI_HEIGHT,
            screen: vec![32u8; SCREEN_SIZE],
            curs_x: 0,
            curs_y: 0,
            curs_mode: CURS_MODE_BLINK,
            tick_count: 0,
        }
    }
}

impl GuiDisplay {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance GUI animation tick counter (used for blinking cursor).
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// Determines if the hardware cursor should be visible at current tick.
    pub fn is_cursor_visible(&self) -> bool {
        match self.curs_mode {
            CURS_MODE_OFF => false,
            CURS_MODE_SOLID => true,
            CURS_MODE_BLINK => (self.tick_count / CURSOR_BLINK_INTERVAL_TICKS) % 2 == 0,
            _ => false,
        }
    }

    /// Get character at column x and row y.
    pub fn char_at(&self, x: usize, y: usize) -> Option<u8> {
        if x < COLS && y < ROWS {
            Some(self.screen[y * COLS + x])
        } else {
            None
        }
    }

    /// Set character at column x and row y.
    pub fn set_char(&mut self, x: usize, y: usize, ch: u8) {
        if x < COLS && y < ROWS {
            self.screen[y * COLS + x] = ch;
        }
    }

    /// Calculate screen rect coordinates for character at (x, y) with doubled rendering rects.
    /// `drawDoubledRect` in GuiDisplay doubles pixel dimensions for monochrome 80x50 font rendering.
    pub fn calculate_char_rect(&self, x: usize, y: usize) -> DoubledRect {
        let rx = SCREEN_X_OFFSET + (x as f32) * GLYPH_WIDTH;
        let ry = SCREEN_Y_OFFSET + (y as f32) * GLYPH_HEIGHT;
        let ch = self.char_at(x, y).unwrap_or(32);

        // Texture coordinates in font texture (16x16 grid of glyphs)
        let glyph_u = ((ch % 16) as f32) * 8.0;
        let glyph_v = ((ch / 16) as f32) * 8.0;

        DoubledRect {
            x: rx,
            y: ry,
            width: GLYPH_WIDTH,
            height: GLYPH_HEIGHT,
            u: glyph_u,
            v: glyph_v,
        }
    }

    /// Calculate cursor rendering rect if cursor is visible.
    pub fn calculate_cursor_rect(&self) -> Option<DoubledRect> {
        if !self.is_cursor_visible() {
            return None;
        }
        if (self.curs_x as usize) < COLS && (self.curs_y as usize) < ROWS {
            let rx = SCREEN_X_OFFSET + (self.curs_x as f32) * GLYPH_WIDTH;
            let ry = SCREEN_Y_OFFSET + (self.curs_y as f32) * GLYPH_HEIGHT;
            Some(DoubledRect {
                x: rx,
                y: ry,
                width: GLYPH_WIDTH,
                height: GLYPH_HEIGHT,
                u: 0.0,
                v: 0.0,
            })
        } else {
            None
        }
    }

    /// Handle key typed in GUI (`GuiDisplay.a(char, int)`) and translate it into a Packet212GuiEvent
    /// to send to the server terminal (`GuiDisplay.sendKey`).
    pub fn key_typed(
        &self,
        window_id: u8,
        ch: char,
        lwjgl_key: i32,
        shift: bool,
        ctrl: bool,
    ) -> Option<Packet212GuiEvent> {
        let code = encode_key(ch, lwjgl_key, shift, ctrl)?;
        Some(Packet212GuiEvent {
            window_id,
            event_id: code as u32,
            body: Vec::new(),
        })
    }

    /// Send a key directly to a Display tile's ring buffer (`sendKey`).
    pub fn send_key_to_display(
        display: &mut Display,
        ch: char,
        lwjgl_key: i32,
        shift: bool,
        ctrl: bool,
    ) -> bool {
        if let Some(code) = encode_key(ch, lwjgl_key, shift, ctrl) {
            display.push_key(code)
        } else {
            false
        }
    }

    /// Sync screen from container client_screen.
    pub fn update_from_container(&mut self, container: &ContainerDisplay) {
        self.screen.copy_from_slice(&container.client_screen);
        self.curs_x = container.curs_x;
        self.curs_y = container.curs_y;
        self.curs_mode = container.curs_mode;
    }
}

/// Keyboard peripheral block helper (`redpower_2:keyboard`, BlockMultipart meta 2).
///
/// L1/L4 input peripheral forwarding player keystrokes into CPU terminal buffer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardBlock {
    /// Facing direction index 0..4 (horizontal: East, South, West, North).
    pub rotation: i32,
    /// Active link address / target terminal device ID (default 1).
    pub target_display_id: i32,
}

impl Default for KeyboardBlock {
    fn default() -> Self {
        Self {
            rotation: 0,
            target_display_id: 1,
        }
    }
}

impl KeyboardBlock {
    pub const BLOCK_ID: &'static str = "redpower_2:keyboard";

    pub fn new(rotation: i32) -> Self {
        Self {
            rotation: rotation & 3,
            target_display_id: 1,
        }
    }

    /// Forward a keystroke from player interaction into the target terminal buffer.
    pub fn forward_keystroke(
        &self,
        display: &mut Display,
        ch: char,
        key_code: i32,
        shift: bool,
        ctrl: bool,
    ) -> bool {
        GuiDisplay::send_key_to_display(display, ch, key_code, shift, ctrl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_display_initialization_and_interaction() {
        let container = ContainerDisplay::new(1, 42);
        assert_eq!(container.window_id, 1);
        assert_eq!(container.player_id, 42);
        assert_eq!(container.client_screen.len(), SCREEN_SIZE);
        assert!(container.can_interact_with(42));
        assert!(!container.can_interact_with(99));
    }

    #[test]
    fn test_container_display_sync_with_tile_and_rle() {
        let mut tile = Display::new();
        let mut container = ContainerDisplay::new(1, 42);

        // Initially both are blank (32)
        let delta = container.update_from_tile(&mut tile);
        assert!(delta.is_none());

        // Tile screen modified: write "HELLO" at row 0
        tile.screen[0] = b'H';
        tile.screen[1] = b'E';
        tile.screen[2] = b'L';
        tile.screen[3] = b'L';
        tile.screen[4] = b'O';

        let delta = container.update_from_tile(&mut tile);
        assert!(delta.is_some());
        let rle_bytes = delta.unwrap();

        // Apply RLE to a fresh container or verify container.client_screen was updated
        assert_eq!(&container.client_screen[0..5], b"HELLO");

        let mut client_container = ContainerDisplay::new(1, 42);
        client_container.apply_rle_update(&rle_bytes);
        assert_eq!(&client_container.client_screen[0..5], b"HELLO");
    }

    #[test]
    fn test_gui_display_cursor_blinking_and_visibility() {
        let mut gui = GuiDisplay::new();
        assert_eq!(gui.curs_mode, CURS_MODE_BLINK);

        // Tick 0: visible
        assert!(gui.is_cursor_visible());

        // Tick 10: blinking toggles off
        gui.tick_count = 10;
        assert!(!gui.is_cursor_visible());

        // Tick 20: blinking toggles on
        gui.tick_count = 20;
        assert!(gui.is_cursor_visible());

        // Mode solid: always visible
        gui.curs_mode = CURS_MODE_SOLID;
        assert!(gui.is_cursor_visible());
        gui.tick_count = 10;
        assert!(gui.is_cursor_visible());

        // Mode off: never visible
        gui.curs_mode = CURS_MODE_OFF;
        assert!(!gui.is_cursor_visible());
    }

    #[test]
    fn test_gui_display_rect_calculations() {
        let mut gui = GuiDisplay::new();
        gui.set_char(0, 0, b'A');
        assert_eq!(gui.char_at(0, 0), Some(b'A'));

        let rect = gui.calculate_char_rect(0, 0);
        assert_eq!(rect.x, SCREEN_X_OFFSET);
        assert_eq!(rect.y, SCREEN_Y_OFFSET);
        assert_eq!(rect.width, GLYPH_WIDTH);
        assert_eq!(rect.height, GLYPH_HEIGHT);

        gui.curs_x = 5;
        gui.curs_y = 2;
        gui.curs_mode = CURS_MODE_SOLID;
        let c_rect = gui
            .calculate_cursor_rect()
            .expect("cursor should be visible");
        assert_eq!(c_rect.x, SCREEN_X_OFFSET + 5.0 * GLYPH_WIDTH);
        assert_eq!(c_rect.y, SCREEN_Y_OFFSET + 2.0 * GLYPH_HEIGHT);
    }

    #[test]
    fn test_gui_display_key_typed_and_forwarding() {
        let gui = GuiDisplay::new();
        let mut tile = Display::new();

        // Standard character 'A'
        let event = gui.key_typed(3, 'A', 30, false, false).unwrap();
        assert_eq!(event.window_id, 3);
        assert_eq!(event.event_id, b'A' as u32);

        // Special arrow key Up (LWJGL 200 -> 128)
        let event_up = gui.key_typed(3, '\0', 200, false, false).unwrap();
        assert_eq!(event_up.event_id, 128);

        // Special key with Shift modifier
        let event_shift_up = gui.key_typed(3, '\0', 200, true, false).unwrap();
        assert_eq!(event_shift_up.event_id, 128 | 64);

        // Send key to display tile ring buffer
        let pushed = GuiDisplay::send_key_to_display(&mut tile, 'Z', 44, false, false);
        assert!(pushed);
        assert_eq!(tile.pop_key(), Some(b'Z'));
    }

    #[test]
    fn test_keyboard_block_forwarding() {
        let keyboard = KeyboardBlock::new(2);
        assert_eq!(keyboard.rotation, 2);
        assert_eq!(KeyboardBlock::BLOCK_ID, "redpower_2:keyboard");

        let mut tile = Display::new();
        let ok = keyboard.forward_keystroke(&mut tile, 'k', 37, false, false);
        assert!(ok);
        assert_eq!(tile.pop_key(), Some(b'k'));
    }
}

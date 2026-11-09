//! Lamp (`BlockLamp`): standard and inverted redstone lamps in 16 colors.
//!
//! Mirrors `eloraam.lighting.BlockLamp` and `eloraam.lighting.RenderLamp`:
//! - 16 dye colors matching `CoreLib.rawColorNames`.
//! - Standard lamp: unlit by default (`redpower_2:lamp`), toggles to lit (`redpower_2:lamp_lit`) when powered.
//! - Inverted lamp: lit by default (`redpower_2:lamp_inverted_lit`), toggles to unlit (`redpower_2:lamp_inverted`) when powered.
//! - Light emission: Level 15 when lit, 0 when unlit.
//! - Metadata: color index `0..=15`, preserved across state swaps.
//! - Rendering Pass 0: 1x1x1 cube with sprite index 16+md (unlit) or 32+md (lit).
//! - Rendering Pass 1: additive bloom halo box expanded by 0.05 blocks `[-0.05, -0.05, -0.05, 1.05, 1.05, 1.05]`
//!   tinted by `LIGHT_COLORS[md]` with 0.5 alpha.

use serde::{Deserialize, Serialize};

/// Canonical block IDs for lamp states.
pub const LAMP_BLOCK_ID: &str = "redpower_2:lamp";
pub const LAMP_LIT_BLOCK_ID: &str = "redpower_2:lamp_lit";
pub const LAMP_INVERTED_BLOCK_ID: &str = "redpower_2:lamp_inverted";
pub const LAMP_INVERTED_LIT_BLOCK_ID: &str = "redpower_2:lamp_inverted_lit";
pub const CAGE_LAMP_BLOCK_ID: &str = "redpower_2:cage_lamp";
pub const CAGE_LAMP_INVERTED_BLOCK_ID: &str = "redpower_2:cage_lamp_inverted";

/// Number of colors supported by lamps.
pub const LAMP_COLORS: usize = 16;

/// Lamp block hardness (`0.3F`).
pub const LAMP_HARDNESS: f32 = 0.3;

/// Lamp block explosion resistance (`1.5F`).
pub const LAMP_RESISTANCE: f32 = 1.5;

/// Light level emitted when lit.
pub const LIT_LIGHT_LEVEL: u8 = 15;

/// Light level emitted when unlit.
pub const UNLIT_LIGHT_LEVEL: u8 = 0;

/// 16 RGB color constants matching `RenderLamp.java:10-12`.
pub const LIGHT_COLORS: [u32; 16] = [
    16777215, // White (0xFFFFFF)
    12608256, // Orange (0xC06300)
    11868853, // Magenta (0xB51AB5)
    7308529,  // Light Blue (0x6F84F1)
    12566272, // Yellow (0xBFCF00)
    7074048,  // Lime (0x6BF100)
    15812213, // Pink (0xF14675)
    5460819,  // Gray (0x535353)
    9671571,  // Light Gray (0x939393)
    34695,    // Cyan (0x008787)
    6160576,  // Purple (0x5E00C0)
    1250240,  // Blue (0x1313C0)
    5187328,  // Brown (0x4F2700)
    558848,   // Green (0x088700)
    10620678, // Red (0xA20F06)
    2039583,  // Black (0x1F1F1F)
];

/// 16 color names in order matching `CoreLib.rawColorNames`.
pub const COLOR_NAMES: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

/// Lamp state kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LampKind {
    Standard,
    StandardLit,
    Inverted,
    InvertedLit,
}

impl LampKind {
    pub fn is_lit(self) -> bool {
        matches!(self, Self::StandardLit | Self::InvertedLit)
    }

    pub fn is_inverted(self) -> bool {
        matches!(self, Self::Inverted | Self::InvertedLit)
    }

    pub fn block_id(self) -> &'static str {
        match self {
            Self::Standard => LAMP_BLOCK_ID,
            Self::StandardLit => LAMP_LIT_BLOCK_ID,
            Self::Inverted => LAMP_INVERTED_BLOCK_ID,
            Self::InvertedLit => LAMP_INVERTED_LIT_BLOCK_ID,
        }
    }

    pub fn from_block_id(id: &str) -> Option<Self> {
        match id {
            LAMP_BLOCK_ID => Some(Self::Standard),
            LAMP_LIT_BLOCK_ID => Some(Self::StandardLit),
            LAMP_INVERTED_BLOCK_ID => Some(Self::Inverted),
            LAMP_INVERTED_LIT_BLOCK_ID => Some(Self::InvertedLit),
            _ => None,
        }
    }
}

/// Determine whether a given block ID is any lamp variant.
pub fn is_lamp_block(id: &str) -> bool {
    LampKind::from_block_id(id).is_some()
}

/// Determine whether the given lamp block is in a lit state.
pub fn is_lit(id: &str) -> bool {
    LampKind::from_block_id(id).is_some_and(|k| k.is_lit())
}

/// Determine whether the given lamp block is an inverted variant.
pub fn is_inverted(id: &str) -> bool {
    LampKind::from_block_id(id).is_some_and(|k| k.is_inverted())
}

/// Return light emission for block ID (15 if lit, 0 otherwise).
pub fn light_emission(id: &str) -> u8 {
    if is_lit(id) {
        LIT_LIGHT_LEVEL
    } else {
        UNLIT_LIGHT_LEVEL
    }
}

/// Check power state for a lamp block and compute any required block state toggle.
///
/// Mirrors `BlockLamp.checkPowerState`:
/// - `!this.lit && isPowered` -> swap to `blockLampOn` with preserved metadata.
/// - `this.lit && !isPowered` -> swap to `blockLampOff` with preserved metadata.
/// - Inverted lamps:
///   - Inverted unlit (`lamp_inverted`) is powered; unpowered -> swaps to `lamp_inverted_lit`.
///   - Inverted lit (`lamp_inverted_lit`) is unpowered; powered -> swaps to `lamp_inverted`.
pub fn check_power_state(
    current_id: &str,
    color: u8,
    is_powered: bool,
) -> Option<(&'static str, u8)> {
    let kind = LampKind::from_block_id(current_id)?;
    let color_meta = color & 0x0F;

    match kind {
        LampKind::Standard => {
            if is_powered {
                Some((LAMP_LIT_BLOCK_ID, color_meta))
            } else {
                None
            }
        }
        LampKind::StandardLit => {
            if !is_powered {
                Some((LAMP_BLOCK_ID, color_meta))
            } else {
                None
            }
        }
        LampKind::Inverted => {
            if !is_powered {
                Some((LAMP_INVERTED_LIT_BLOCK_ID, color_meta))
            } else {
                None
            }
        }
        LampKind::InvertedLit => {
            if is_powered {
                Some((LAMP_INVERTED_BLOCK_ID, color_meta))
            } else {
                None
            }
        }
    }
}

// ============================================================================
// Multipass Render Descriptors (Pass 0 Base & Pass 1 Bloom)
// ============================================================================

/// Rendering information for Pass 0 (solid block rendering).
#[derive(Debug, Clone, PartialEq)]
pub struct Pass0RenderInfo {
    pub bounds: [f64; 6],
    pub sprite_index: u32,
    pub emissive: bool,
    pub local_lights: [f32; 6],
}

/// Rendering information for Pass 1 (bloom halo rendering).
#[derive(Debug, Clone, PartialEq)]
pub struct Pass1BloomInfo {
    pub bounds: [f64; 6],
    pub rgb_tint: [f32; 3],
    pub alpha: f32,
    pub blend_src: u32,
    pub blend_dst: u32,
}

/// Build Pass 0 render information matching `RenderLamp.java:27-54`.
pub fn pass0_render(block_id: &str, color: u8) -> Option<Pass0RenderInfo> {
    let kind = LampKind::from_block_id(block_id)?;
    let md = (color & 0x0F) as u32;

    if kind.is_lit() {
        Some(Pass0RenderInfo {
            bounds: [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            sprite_index: 32 + md,
            emissive: true,
            local_lights: [1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        })
    } else {
        Some(Pass0RenderInfo {
            bounds: [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            sprite_index: 16 + md,
            emissive: false,
            local_lights: [0.5, 1.0, 0.8, 0.8, 0.6, 0.6],
        })
    }
}

/// Build Pass 1 bloom halo information matching `RenderLamp.java:55-79`.
pub fn pass1_bloom(block_id: &str, color: u8) -> Option<Pass1BloomInfo> {
    let kind = LampKind::from_block_id(block_id)?;
    if !kind.is_lit() {
        return None;
    }

    let md = (color & 0x0F) as usize;
    let tc = LIGHT_COLORS[md];
    let r = ((tc >> 16) & 0xFF) as f32 / 255.0;
    let g = ((tc >> 8) & 0xFF) as f32 / 255.0;
    let b = (tc & 0xFF) as f32 / 255.0;

    Some(Pass1BloomInfo {
        bounds: [-0.05, -0.05, -0.05, 1.05, 1.05, 1.05],
        rgb_tint: [r, g, b],
        alpha: 0.5,
        blend_src: 770, // GL_SRC_ALPHA
        blend_dst: 1,   // GL_ONE (additive)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lamp_constants_and_colors() {
        assert_eq!(LAMP_COLORS, 16);
        assert_eq!(LIGHT_COLORS.len(), 16);
        assert_eq!(COLOR_NAMES.len(), 16);
        assert_eq!(LIGHT_COLORS[0], 0xFFFFFF); // White
        assert_eq!(LIGHT_COLORS[15], 2039583); // Black
        assert_eq!(LAMP_HARDNESS, 0.3);
        assert_eq!(LAMP_RESISTANCE, 1.5);
    }

    #[test]
    fn test_lamp_emission_levels() {
        assert_eq!(light_emission(LAMP_BLOCK_ID), 0);
        assert_eq!(light_emission(LAMP_LIT_BLOCK_ID), 15);
        assert_eq!(light_emission(LAMP_INVERTED_BLOCK_ID), 0);
        assert_eq!(light_emission(LAMP_INVERTED_LIT_BLOCK_ID), 15);
    }

    #[test]
    fn test_lamp_power_toggle_standard() {
        for c in 0..16 {
            // Unpowered standard lamp stays unlit
            assert_eq!(check_power_state(LAMP_BLOCK_ID, c, false), None);

            // Powered standard lamp toggles to lit with metadata preserved
            assert_eq!(
                check_power_state(LAMP_BLOCK_ID, c, true),
                Some((LAMP_LIT_BLOCK_ID, c))
            );

            // Powered lit lamp stays lit
            assert_eq!(check_power_state(LAMP_LIT_BLOCK_ID, c, true), None);

            // Unpowered lit lamp toggles off with metadata preserved
            assert_eq!(
                check_power_state(LAMP_LIT_BLOCK_ID, c, false),
                Some((LAMP_BLOCK_ID, c))
            );
        }
    }

    #[test]
    fn test_lamp_power_toggle_inverted() {
        for c in 0..16 {
            // Unpowered inverted lit lamp stays lit
            assert_eq!(
                check_power_state(LAMP_INVERTED_LIT_BLOCK_ID, c, false),
                None
            );

            // Powered inverted lit lamp toggles to unlit with metadata preserved
            assert_eq!(
                check_power_state(LAMP_INVERTED_LIT_BLOCK_ID, c, true),
                Some((LAMP_INVERTED_BLOCK_ID, c))
            );

            // Powered unlit lamp stays unlit
            assert_eq!(check_power_state(LAMP_INVERTED_BLOCK_ID, c, true), None);

            // Unpowered inverted lamp toggles to lit with metadata preserved
            assert_eq!(
                check_power_state(LAMP_INVERTED_BLOCK_ID, c, false),
                Some((LAMP_INVERTED_LIT_BLOCK_ID, c))
            );
        }
    }

    #[test]
    fn test_pass0_and_pass1_rendering() {
        for c in 0..16 {
            // Unlit standard lamp: pass 0 uses sprite 16+md, pass 1 bloom is disabled
            let p0_off = pass0_render(LAMP_BLOCK_ID, c).unwrap();
            assert_eq!(p0_off.sprite_index, 16 + c as u32);
            assert!(!p0_off.emissive);
            assert_eq!(pass1_bloom(LAMP_BLOCK_ID, c), None);

            // Lit standard lamp: pass 0 uses sprite 32+md, pass 1 bloom enabled with color tint
            let p0_on = pass0_render(LAMP_LIT_BLOCK_ID, c).unwrap();
            assert_eq!(p0_on.sprite_index, 32 + c as u32);
            assert!(p0_on.emissive);

            let p1 = pass1_bloom(LAMP_LIT_BLOCK_ID, c).unwrap();
            assert_eq!(p1.bounds, [-0.05, -0.05, -0.05, 1.05, 1.05, 1.05]);
            assert_eq!(p1.alpha, 0.5);
            assert_eq!(p1.blend_src, 770);
            assert_eq!(p1.blend_dst, 1);
        }
    }
}

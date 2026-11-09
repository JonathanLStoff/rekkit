//! BuildCraft compatibility layer for RedPower 2.
//!
//! Mirrors `com.eloraam.redpower.compat`:
//! - `TileBlueEngine`: Blulectric Engine converting Blutricity to BuildCraft MJ.
//! - Sided power delivery at `rotation ^ 1`.

pub mod engine;

pub use engine::*;

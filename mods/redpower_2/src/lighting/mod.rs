//! RedPower 2 lighting subsystem.
//!
//! Mirrors `eloraam.lighting`:
//! - `BlockLamp`: standard and inverted lamps in 16 colors.
//! - `RenderLamp`: multipass block rendering with Pass 0 diffuse/emissive and Pass 1 halo bloom.

pub mod lamp;

pub use lamp::*;

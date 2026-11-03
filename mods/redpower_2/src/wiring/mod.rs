//! RedPower 2 wiring: ribbon cable 8-bit bus and the red alloy / bundled wire signal network.

#[allow(clippy::needless_range_loop, clippy::collapsible_match)]
pub mod blutricity;
pub mod hook;
pub mod network;
pub mod ribbon;
pub mod rules;

pub use network::{Tile, WireKind, WireNode, WireWorld};
pub use ribbon::{RibbonCable, RIBBON_ITEM_ID, RIBBON_THICKNESS_SIXTEENTHS};

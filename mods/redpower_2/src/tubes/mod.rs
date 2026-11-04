//! RedPower 2 pneumatic tube subtypes.

pub mod redstone_tube;
pub mod restrictor_tube;

pub use redstone_tube::{RedstoneTube, REDSTONE_TUBE_EXTENDED_ID, REDSTONE_TUBE_WIRING_CLASS};
pub use restrictor_tube::{RestrictorTube, RESTRICTOR_TUBE_EXTENDED_ID, RESTRICTOR_TUBE_WEIGHT};

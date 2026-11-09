//! RedPower 2 Blutricity generation and power physics.
//!
//! Submodules:
//! - [`solar`]: `TileSolarPanel` daylight generation (2.0A, V <= 100V).
//! - [`thermopile`]: `TileThermopile` thermal gradient generation ($0.005\text{ A} \times \min(T_h, T_c)$).
//! - [`transformer`]: `TileTransformer` 100x voltage step up/down between 100V and 10,000V.
//! - [`turbine`]: `TileWindTurbine` multiblock generation, raytrace clearance, and brakes.

pub mod solar;
pub mod thermopile;
pub mod transformer;
pub mod turbine;

pub use solar::{SolarEnvironment, TileSolarPanel};
pub use thermopile::TileThermopile;
pub use transformer::TileTransformer;
pub use turbine::TileWindTurbine;

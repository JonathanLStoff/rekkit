//! Igniter (`redpower_2:igniter`): port of `TileIgniter` (machine meta 12).
//!
//! Pure logic. The engine layer applies the returned [`FireAction`] to the block in front.

use super::base::MachineFlags;
use rekkit_core::side::Direction;
use serde::{Deserialize, Serialize};

/// Ticks between a state change and the settle tick.
pub const IGNITER_DELAY: u32 = 5;

/// What to do to the block in front (`fireAction`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireAction {
    /// Place fire if the front block is air.
    PlaceFireIfAir,
    /// Remove the front block if it is fire.
    ExtinguishIfFire,
}

/// Result of a neighbour change or tick.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IgniterOutcome {
    pub action: Option<FireAction>,
    /// Block state changed and must be synced to clients.
    pub update_block: bool,
}

/// `TileIgniter`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileIgniter {
    pub rotation: Direction,
    pub flags: MachineFlags,
    pub scheduled_tick: Option<u32>,
}

impl TileIgniter {
    pub fn new(rotation: Direction) -> Self {
        Self {
            rotation,
            flags: MachineFlags::NONE,
            scheduled_tick: None,
        }
    }

    /// The side acted on (`Rotation ^ 1`).
    pub fn front(&self) -> Direction {
        self.rotation.opposite()
    }

    fn action(&self) -> FireAction {
        if self.flags.active {
            FireAction::PlaceFireIfAir
        } else {
            FireAction::ExtinguishIfFire
        }
    }

    /// `onBlockNeighborChange`; `powered` is the redstone state on any face.
    pub fn on_neighbor_change(&mut self, powered: bool) -> IgniterOutcome {
        let none = IgniterOutcome::default();
        if powered {
            if self.flags.powered {
                return none;
            }
            self.flags.powered = true;
            if self.flags.delay || self.flags.active {
                return none;
            }
            self.flags.active = true;
        } else {
            if !self.flags.powered {
                return none;
            }
            self.flags.powered = false;
            if self.flags.delay {
                return none;
            }
            self.flags.active = false;
        }
        self.flags.delay = true;
        self.scheduled_tick = Some(IGNITER_DELAY);
        IgniterOutcome {
            action: Some(self.action()),
            update_block: true,
        }
    }

    /// `isOnFire`: only a rotation-0 igniter, while active, burns its faces.
    pub fn is_on_fire(&self, _face: Direction) -> bool {
        self.rotation == Direction::Down && self.flags.active
    }

    /// `onTileTick`.
    pub fn on_tile_tick(&mut self) -> IgniterOutcome {
        self.scheduled_tick = None;
        self.flags.delay = false;
        if self.flags.active != self.flags.powered {
            self.flags.active = self.flags.powered;
            return IgniterOutcome {
                action: Some(self.action()),
                update_block: true,
            };
        }
        IgniterOutcome::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_lights_then_unpower_during_delay_settles_on_tick() {
        let mut i = TileIgniter::new(Direction::North);
        let o = i.on_neighbor_change(true);
        assert_eq!(o.action, Some(FireAction::PlaceFireIfAir));
        assert!(i.flags.active && i.flags.delay);
        // unpowered inside the delay: no action yet
        assert_eq!(i.on_neighbor_change(false).action, None);
        assert!(i.flags.active);
        let o = i.on_tile_tick();
        assert_eq!(o.action, Some(FireAction::ExtinguishIfFire));
        assert!(!i.flags.active && !i.flags.delay);
    }

    #[test]
    fn repeated_power_is_ignored() {
        let mut i = TileIgniter::new(Direction::North);
        i.on_neighbor_change(true);
        assert_eq!(i.on_neighbor_change(true), IgniterOutcome::default());
    }

    #[test]
    fn unpower_after_delay_extinguishes() {
        let mut i = TileIgniter::new(Direction::North);
        i.on_neighbor_change(true);
        i.on_tile_tick();
        let o = i.on_neighbor_change(false);
        assert_eq!(o.action, Some(FireAction::ExtinguishIfFire));
        assert_eq!(i.scheduled_tick, Some(IGNITER_DELAY));
    }
}

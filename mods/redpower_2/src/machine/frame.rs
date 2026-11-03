//! Frame solver: port of `FrameLib.FrameSolver`.
//!
//! Pure logic over a [`FrameWorld`] view; the engine layer supplies the world.

use crate::core::spatial::WorldCoord;
use std::collections::{HashSet, VecDeque};

/// Default `settings.machine.frame.linksize`.
pub const DEFAULT_FRAME_LINK_SIZE: usize = 1000;

/// What `IFrameLink` says about a tile.
#[derive(Debug, Clone, Default)]
pub struct FrameLink {
    pub moving: bool,
    /// Bit `i` set: the frame links out through side `i`.
    pub connect_out: u8,
    /// Bit `i` set: the frame accepts a link through side `i`.
    pub connect_in: u8,
    /// Motor that owns this frame (`getFrameLinkset`), if any.
    pub linkset: Option<WorldCoord>,
}

/// Read-only world view the solver needs.
pub trait FrameWorld {
    /// World height (`getHeight`).
    fn height(&self) -> i32;
    /// Block id at `wc`; 0 is air.
    fn block_id(&self, wc: WorldCoord) -> u32;
    /// The chunk holding `wc` is loaded.
    fn is_loaded(&self, wc: WorldCoord) -> bool;
    /// `getBlockHardness`.
    fn hardness(&self, wc: WorldCoord) -> f32;
    /// Block at `wc` can be replaced by a moved block (`canPlaceBlockAt` with replace).
    fn can_replace(&self, wc: WorldCoord) -> bool;
    /// `IFrameLink` tile at `wc`.
    fn frame_link(&self, wc: WorldCoord) -> Option<FrameLink>;
}

pub struct FrameSolver<'a, W: FrameWorld> {
    world: &'a W,
    scanmap: HashSet<WorldCoord>,
    scanpos: VecDeque<WorldCoord>,
    framemap: HashSet<WorldCoord>,
    frameset: Vec<WorldCoord>,
    clearset: Vec<WorldCoord>,
    movedir: i32,
    motorpos: WorldCoord,
    pub valid: bool,
}

impl<'a, W: FrameWorld> FrameSolver<'a, W> {
    /// `wc` is the first frame, `motorpos` the motor; `movedir < 0` means a scan for dropping.
    pub fn new(world: &'a W, wc: WorldCoord, motorpos: WorldCoord, movedir: i32) -> Self {
        let mut scanmap = HashSet::new();
        scanmap.insert(motorpos);
        scanmap.insert(wc);
        let mut scanpos = VecDeque::new();
        scanpos.push_back(wc);
        Self {
            world,
            scanmap,
            scanpos,
            framemap: HashSet::new(),
            frameset: Vec::new(),
            clearset: Vec::new(),
            movedir,
            motorpos,
            valid: true,
        }
    }

    /// One BFS step; returns whether a block was added to the frame set.
    fn step(&mut self) -> bool {
        let Some(wc) = self.scanpos.pop_front() else {
            return false;
        };
        if wc.y < 0 || wc.y >= self.world.height() - 1 {
            return false;
        }
        let bid = self.world.block_id(wc);
        if self.movedir >= 0 && !self.world.is_loaded(wc) {
            self.valid = false;
            return false;
        }
        if bid == 0 {
            return false;
        }
        if self.movedir >= 0 && self.world.hardness(wc) < 0.0 {
            self.valid = false;
            return false;
        }
        self.framemap.insert(wc);
        self.frameset.push(wc);
        let Some(link) = self.world.frame_link(wc) else {
            return true;
        };
        if link.moving && self.movedir >= 0 {
            self.valid = false;
            return true;
        }
        for i in 0..6i32 {
            if link.connect_out & (1 << i) == 0 {
                continue;
            }
            let sp = wc.coord_step(i);
            if self.scanmap.contains(&sp) {
                continue;
            }
            if let Some(other) = self.world.frame_link(sp) {
                if other.connect_in & (1 << (i ^ 1)) == 0 {
                    continue;
                }
                if self.movedir < 0 && other.linkset != Some(self.motorpos) {
                    continue;
                }
            }
            self.scanmap.insert(sp);
            self.scanpos.push_back(sp);
        }
        true
    }

    pub fn solve(&mut self) -> bool {
        while self.valid && !self.scanpos.is_empty() {
            self.step();
        }
        self.valid
    }

    /// Like [`solve`](Self::solve) but gives up (false) after `limit` blocks.
    pub fn solve_limit(&mut self, mut limit: i64) -> bool {
        while self.valid && !self.scanpos.is_empty() {
            if self.step() {
                limit -= 1;
            }
            if limit == 0 {
                return false;
            }
        }
        self.valid
    }

    /// Add the destination cells of every frame block; blocks in the way must be replaceable.
    pub fn add_moved(&mut self) -> bool {
        let snapshot = self.frameset.clone();
        for wc in snapshot {
            let sp = wc.coord_step(self.movedir);
            let bid = self.world.block_id(sp);
            if !self.world.is_loaded(sp) {
                self.valid = false;
                return false;
            }
            if !self.framemap.contains(&sp) {
                if bid != 0 {
                    if !self.world.can_replace(sp) {
                        self.valid = false;
                        return false;
                    }
                    self.clearset.push(sp);
                }
                self.framemap.insert(sp);
                self.frameset.push(sp);
            }
        }
        self.valid
    }

    pub fn sort(&mut self, dir: i32) {
        self.frameset
            .sort_by(|a, b| WorldCoord::compare_dir(dir, a, b));
    }

    pub fn frame_set(&self) -> &[WorldCoord] {
        &self.frameset
    }

    pub fn clear_set(&self) -> &[WorldCoord] {
        &self.clearset
    }
}

/// Ticks a frame move takes (`MovePos` 0..16).
pub const MOVE_TICKS: i32 = 16;
/// Voltage at which the motor counts as charged.
pub const MOTOR_MIN_VOLTS: f64 = 60.0;

/// Direction (0..5) the motor pushes the frame, from its rotation (`TileMotor.getDriveSide`).
pub fn drive_side(rotation: i32) -> i32 {
    let n: i32 = match rotation >> 2 {
        0 => 13604,
        1 => 13349,
        2 => 20800,
        3 => 16720,
        4 => 8496,
        _ => 12576,
    };
    (n >> ((rotation & 3) << 2)) & 7
}

/// What a motor tick asks the engine layer to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorEvent {
    None,
    /// `MovePos` reached 16: run `dropFrame(true)` (re-place the moved blocks one step on).
    Drop,
}

/// Frame motor state machine (`TileMotor`); the engine layer does the world edits.
#[derive(Debug, Clone, PartialEq)]
pub struct MotorState {
    pub rotation: i32,
    pub move_dir: i32,
    /// -1 idle, 0..=16 moving.
    pub move_pos: i32,
    pub powered: bool,
    pub active: bool,
    pub charged: bool,
    pub link_size: i32,
}

impl MotorState {
    pub fn new(rotation: i32) -> Self {
        Self {
            rotation,
            move_dir: 4,
            move_pos: -1,
            powered: false,
            active: false,
            charged: false,
            link_size: -1,
        }
    }

    /// Side of the motor the first frame sits on (`Rotation >> 2 ^ 1`).
    pub fn frame_side(&self) -> i32 {
        (self.rotation >> 2) ^ 1
    }

    /// Power drawn per tick while moving: `100 + 10 * LinkSize`.
    pub fn power_cost(&self) -> f64 {
        100.0 + 10.0 * f64::from(self.link_size)
    }

    /// Move progress for rendering (`getMoveScaled`).
    pub fn move_scaled(&self) -> f32 {
        (self.move_pos.max(0) as f32) / MOVE_TICKS as f32
    }

    /// Server tick (`TileMotor.g`) after the conductor has been iterated by the grid.
    pub fn tick(
        &mut self,
        cond: &mut crate::wiring::blutricity::BlueConductor,
        tick: u64,
    ) -> MotorEvent {
        let mut ev = MotorEvent::None;
        if (0..MOVE_TICKS).contains(&self.move_pos) {
            self.move_pos += 1;
        }
        if self.move_pos >= 0 {
            cond.draw_power(tick, self.power_cost());
        }
        if self.move_pos >= MOVE_TICKS {
            ev = MotorEvent::Drop;
            self.move_pos = -1;
            self.active = false;
        }
        if self.move_pos < 0 {
            if cond.voltage(tick) < MOTOR_MIN_VOLTS {
                if self.charged && cond.flow == 0 {
                    self.charged = false;
                }
            } else {
                self.charged = true;
            }
        }
        ev
    }

    /// Neighbour/redstone change (`onBlockNeighborChange`); returns true when a frame pick
    /// should be attempted.
    pub fn on_neighbor_change(&mut self, redstone: bool) -> bool {
        if redstone {
            if self.charged && !self.powered && self.move_pos < 0 {
                self.powered = true;
                return true;
            }
        } else if self.powered {
            self.powered = false;
        }
        false
    }

    /// Solve the frame for a pick; on success starts the move.
    pub fn pick<W: FrameWorld>(
        &mut self,
        world: &W,
        at: WorldCoord,
        limit: i64,
    ) -> Option<Vec<WorldCoord>> {
        self.move_dir = drive_side(self.rotation);
        let mut fs = FrameSolver::new(world, at.coord_step(self.frame_side()), at, self.move_dir);
        if fs.solve_limit(limit) && fs.add_moved() {
            self.link_size = fs.frame_set().len() as i32;
            self.move_pos = 0;
            self.active = true;
            return Some(fs.frame_set().to_vec());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct W {
        blocks: HashMap<WorldCoord, (u32, f32, Option<FrameLink>)>,
    }
    impl W {
        fn put(&mut self, x: i32, y: i32, z: i32, h: f32, link: Option<FrameLink>) {
            self.blocks.insert(WorldCoord::new(x, y, z), (1, h, link));
        }
    }
    impl FrameWorld for W {
        fn height(&self) -> i32 {
            256
        }
        fn block_id(&self, wc: WorldCoord) -> u32 {
            self.blocks.get(&wc).map_or(0, |b| b.0)
        }
        fn is_loaded(&self, _: WorldCoord) -> bool {
            true
        }
        fn hardness(&self, wc: WorldCoord) -> f32 {
            self.blocks.get(&wc).map_or(0.0, |b| b.1)
        }
        fn can_replace(&self, _: WorldCoord) -> bool {
            true
        }
        fn frame_link(&self, wc: WorldCoord) -> Option<FrameLink> {
            self.blocks.get(&wc).and_then(|b| b.2.clone())
        }
    }
    fn all() -> Option<FrameLink> {
        Some(FrameLink {
            connect_out: 63,
            connect_in: 63,
            ..Default::default()
        })
    }

    #[test]
    fn collects_connected_frames() {
        let mut w = W::default();
        for x in 1..=3 {
            w.put(x, 64, 0, 1.0, all());
        }
        let mut s = FrameSolver::new(&w, WorldCoord::new(1, 64, 0), WorldCoord::new(0, 64, 0), 5);
        assert!(s.solve());
        assert_eq!(s.frame_set().len(), 3);
    }

    #[test]
    fn bedrock_blocks_move() {
        let mut w = W::default();
        w.put(1, 64, 0, 1.0, all());
        w.put(2, 64, 0, -1.0, None);
        w.put(1, 65, 0, 1.0, None);
        let mut s = FrameSolver::new(&w, WorldCoord::new(1, 64, 0), WorldCoord::new(0, 64, 0), 5);
        assert!(!s.solve());
    }

    #[test]
    fn limit_and_add_moved() {
        let mut w = W::default();
        for x in 1..=5 {
            w.put(x, 64, 0, 1.0, all());
        }
        let mut s = FrameSolver::new(&w, WorldCoord::new(1, 64, 0), WorldCoord::new(0, 64, 0), 1);
        assert!(!s.solve_limit(3));
        let mut s = FrameSolver::new(&w, WorldCoord::new(1, 64, 0), WorldCoord::new(0, 64, 0), 1);
        assert!(s.solve());
        assert!(s.add_moved());
        assert_eq!(s.frame_set().len(), 10);
        assert!(s.clear_set().is_empty());
        s.sort(1);
        assert_eq!(s.frame_set()[0].y, 65);
    }

    #[test]
    fn drive_side_table() {
        assert_eq!(drive_side(0), 13604 & 7);
        assert_eq!(drive_side(1), (13604 >> 4) & 7);
        for r in 0..24 {
            assert!(drive_side(r) < 6, "rot {r}");
        }
    }

    #[test]
    fn motor_moves_16_ticks_and_draws_power() {
        use crate::wiring::blutricity::BlueConductor;
        let mut w = W::default();
        for x in 1..=2 {
            w.put(x, 64, 0, 1.0, all());
        }
        // rotation 0: frame side = 1 (up); put frames above instead
        let mut w2 = W::default();
        for y in 65..=66 {
            w2.put(0, y, 0, 1.0, all());
        }
        let mut m = MotorState::new(0);
        let at = WorldCoord::new(0, 64, 0);
        assert!(m.pick(&w, at, 1000).is_some_and(|v| v.is_empty()));
        m.move_pos = -1;
        let set = m.pick(&w2, at, 1000).expect("pick");
        assert_eq!(m.link_size, 4);
        assert_eq!(set.len(), 4);
        assert_eq!(m.power_cost(), 140.0);
        let mut c = BlueConductor::endpoint();
        c.apply_power(1, 1000.0);
        let mut drops = 0;
        for t in 0..16u64 {
            if m.tick(&mut c, 2 + t) == MotorEvent::Drop {
                drops += 1;
            }
        }
        assert_eq!(drops, 1);
        assert_eq!(m.move_pos, -1);
        assert!(!m.active);
    }

    #[test]
    fn motor_needs_charge_to_start() {
        let mut m = MotorState::new(0);
        assert!(!m.on_neighbor_change(true));
        m.charged = true;
        assert!(m.on_neighbor_change(true));
        assert!(!m.on_neighbor_change(true));
        m.on_neighbor_change(false);
        assert!(!m.powered);
    }
}

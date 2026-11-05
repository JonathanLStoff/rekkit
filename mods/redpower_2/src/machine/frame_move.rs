//! Support frames, moving-frame placeholders and the pick/drop world edits
//! (`TileFrameTube.rebuildSticky`, `TileFrameMoving`, `TileMotor.pickFrame/dropFrame`).
//!
//! Pure logic over [`MoveWorld`]; the engine layer implements the world.

use super::frame::{drive_side, FrameLink, FrameSolver, FrameWorld, MotorState};
use crate::core::spatial::WorldCoord;

/// `TileFrameTube.rebuildSticky`: a side is sticky when it has no cover, or the
/// cover's class (`cover >> 8`) is 1 or 4.
pub fn rebuild_sticky(cover_sides: u8, covers: &[i32; 6]) -> u8 {
    let mut ss = 0u8;
    for (i, cover) in covers.iter().enumerate() {
        let m = 1u8 << i;
        let n = cover >> 8;
        if cover_sides & m == 0 || n == 1 || n == 4 {
            ss |= m;
        }
    }
    ss
}

/// `TileFrameTube.canAddCover`: only cover classes 0, 1, 3, 4 on a free side 0..=5.
pub fn frame_can_add_cover(cover_sides: u8, side: i32, cover: i32) -> bool {
    if !(0..=5).contains(&side) {
        return false;
    }
    matches!(cover >> 8, 0 | 1 | 3 | 4) && cover_sides & (1 << side) == 0
}

/// `IFrameLink` of a support frame: links out through its sticky sides, accepts on all.
pub fn support_frame_link(sticky: u8, linkset: Option<WorldCoord>) -> FrameLink {
    FrameLink {
        moving: false,
        connect_out: sticky,
        connect_in: 63,
        linkset,
    }
}

/// A block in transit (`TileFrameMoving`): id, metadata and tile payload preserved.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MovingFrame {
    pub block_id: u32,
    pub meta: u8,
    pub motor: WorldCoord,
    /// Serialized tile entity (NBT), carried unchanged.
    pub tile: Option<Vec<u8>>,
    /// Carried as an opaque crate (tile not `IFrameSupport`, or FrameAlwaysCrate).
    pub crate_: bool,
    pub last_move_pos: u8,
}

impl MovingFrame {
    /// `setContents`.
    pub fn new(
        block_id: u32,
        meta: u8,
        motor: WorldCoord,
        tile: Option<Vec<u8>>,
        tile_supports_frame: bool,
        always_crate: bool,
    ) -> Self {
        let crate_ = tile.is_some() && (always_crate || !tile_supports_frame);
        Self {
            block_id,
            meta,
            motor,
            tile,
            crate_,
            last_move_pos: 0,
        }
    }

    /// Placeholder link: moving, joins nothing new, belongs to its motor.
    pub fn link(&self) -> FrameLink {
        FrameLink {
            moving: true,
            connect_out: 63,
            connect_in: 63,
            linkset: Some(self.motor),
        }
    }

    /// `setPartBounds`: unit cube offset by `scaled` along `move_dir`.
    pub fn bounds(move_dir: i32, scaled: f32) -> [f32; 6] {
        let mut b = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let (axis, sign) = match move_dir {
            0 => (1, -1.0),
            1 => (1, 1.0),
            2 => (2, -1.0),
            3 => (2, 1.0),
            4 => (0, -1.0),
            _ => (0, 1.0),
        };
        b[axis] += sign * scaled;
        b[axis + 3] += sign * scaled;
        b
    }
}

/// World mutation the pick/drop needs; the engine layer implements it.
pub trait MoveWorld: FrameWorld {
    /// Set a block to air without drops.
    fn clear_block(&mut self, wc: WorldCoord);
    /// Remove and return block id, metadata, tile payload, and whether the tile is `IFrameSupport`.
    fn take_block(&mut self, wc: WorldCoord) -> (u32, u8, Option<Vec<u8>>, bool);
    fn set_placeholder(&mut self, wc: WorldCoord, mf: MovingFrame);
    fn placeholder(&self, wc: WorldCoord) -> Option<MovingFrame>;
    /// Replace whatever is at `dest` (a placeholder or air) with the real block and tile.
    fn place_block(&mut self, dest: WorldCoord, id: u32, meta: u8, tile: Option<Vec<u8>>);
    /// `IFrameSupport.onFramePickup` / `onFrameDrop`.
    fn frame_pickup(&mut self, _wc: WorldCoord) {}
    fn frame_drop(&mut self, _wc: WorldCoord) {}
}

/// `TileMotor.pickFrame`: solve, clear what is in the way, swap every frame block for a
/// placeholder and start the move. Returns false (nothing changed) if it cannot move.
pub fn pick_frame<W: MoveWorld>(
    world: &mut W,
    motor: &mut MotorState,
    pos: WorldCoord,
    limit: i64,
    always_crate: bool,
) -> bool {
    motor.move_dir = drive_side(motor.rotation);
    let (set, clears) = {
        let mut fs = FrameSolver::new(
            &*world,
            pos.coord_step(motor.frame_side()),
            pos,
            motor.move_dir,
        );
        if !fs.solve_limit(limit) || !fs.add_moved() {
            return false;
        }
        (fs.frame_set().to_vec(), fs.clear_set().to_vec())
    };
    motor.link_size = set.len() as i32;
    motor.move_pos = 0;
    motor.active = true;
    for c in clears {
        world.clear_block(c);
    }
    for &sp in &set {
        let (id, meta, tile, supports) = world.take_block(sp);
        world.set_placeholder(
            sp,
            MovingFrame::new(id, meta, pos, tile, supports, always_crate),
        );
    }
    for &sp in &set {
        world.frame_pickup(sp);
    }
    true
}

/// `TileMotor.dropFrame`: re-instantiate every carried block, one step on if `fw`.
pub fn drop_frame<W: MoveWorld>(world: &mut W, motor: &mut MotorState, pos: WorldCoord, fw: bool) {
    let set = {
        let mut fs = FrameSolver::new(&*world, pos.coord_step(motor.frame_side()), pos, -1);
        if !fs.solve() {
            return;
        }
        fs.sort(motor.move_dir);
        fs.frame_set().to_vec()
    };
    motor.link_size = 0;
    for &sp in &set {
        let Some(mf) = world.placeholder(sp) else {
            continue;
        };
        let dest = if fw {
            sp.coord_step(motor.move_dir)
        } else {
            sp
        };
        if mf.block_id != 0 {
            world.place_block(dest, mf.block_id, mf.meta, mf.tile);
        }
        if fw {
            world.clear_block(sp);
        }
    }
    for &sp in &set {
        world.frame_drop(sp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct W {
        blocks: HashMap<WorldCoord, (u32, bool)>, // id, is frame
        holders: HashMap<WorldCoord, MovingFrame>,
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
        fn hardness(&self, _: WorldCoord) -> f32 {
            1.0
        }
        fn can_replace(&self, _: WorldCoord) -> bool {
            true
        }
        fn frame_link(&self, wc: WorldCoord) -> Option<FrameLink> {
            if let Some(m) = self.holders.get(&wc) {
                return Some(m.link());
            }
            match self.blocks.get(&wc) {
                Some((_, true)) => Some(support_frame_link(63, None)),
                _ => None,
            }
        }
    }
    impl MoveWorld for W {
        fn clear_block(&mut self, wc: WorldCoord) {
            self.blocks.remove(&wc);
            self.holders.remove(&wc);
        }
        fn take_block(&mut self, wc: WorldCoord) -> (u32, u8, Option<Vec<u8>>, bool) {
            let id = self.blocks.remove(&wc).map_or(0, |b| b.0);
            (id, 3, Some(vec![1, 2]), true)
        }
        fn set_placeholder(&mut self, wc: WorldCoord, mf: MovingFrame) {
            self.blocks.insert(wc, (99, false));
            self.holders.insert(wc, mf);
        }
        fn placeholder(&self, wc: WorldCoord) -> Option<MovingFrame> {
            self.holders.get(&wc).cloned()
        }
        fn place_block(&mut self, d: WorldCoord, id: u32, _m: u8, _t: Option<Vec<u8>>) {
            self.holders.remove(&d);
            self.blocks.insert(d, (id, true));
        }
    }

    #[test]
    fn sticky_and_covers() {
        assert_eq!(rebuild_sticky(0, &[0; 6]), 63);
        let mut c = [0; 6];
        c[2] = 2 << 8;
        c[3] = 1 << 8;
        assert_eq!(rebuild_sticky(0b1100, &c), 63 & !0b100);
        assert!(frame_can_add_cover(0, 1, 3 << 8));
        assert!(!frame_can_add_cover(0, 1, 2 << 8));
        assert!(!frame_can_add_cover(2, 1, 0));
        assert_eq!(MovingFrame::bounds(3, 0.5), [0.0, 0.0, 0.5, 1.0, 1.0, 1.5]);
    }

    #[test]
    fn placeholder_crate_rules() {
        let m = WorldCoord::new(0, 0, 0);
        assert!(!MovingFrame::new(1, 0, m, Some(vec![]), true, false).crate_);
        assert!(MovingFrame::new(1, 0, m, Some(vec![]), true, true).crate_);
        assert!(MovingFrame::new(1, 0, m, Some(vec![]), false, false).crate_);
        assert!(!MovingFrame::new(1, 0, m, None, false, false).crate_);
    }

    #[test]
    fn pick_then_drop_moves_blocks_one_step() {
        let mut w = W::default();
        for x in 1..=2 {
            w.blocks.insert(WorldCoord::new(x, 64, 0), (7, true));
        }
        // block in the way of the destination gets cleared
        let pos = WorldCoord::new(0, 64, 0);
        let mut m = MotorState::new(4 << 2); // frame on side 5 (+x)
        assert!(pick_frame(&mut w, &mut m, pos, 1000, false));
        assert!(m.active && m.move_pos == 0);
        let dir = m.move_dir;
        assert_eq!(w.holders.len() as i32, m.link_size);
        drop_frame(&mut w, &mut m, pos, true);
        assert!(w.holders.is_empty());
        let moved = WorldCoord::new(1, 64, 0).coord_step(dir);
        assert_eq!(w.block_id(moved), 7);
        assert_eq!(m.link_size, 0);
    }

    #[test]
    fn pick_fails_over_limit_and_changes_nothing() {
        let mut w = W::default();
        for x in 1..=5 {
            w.blocks.insert(WorldCoord::new(x, 64, 0), (7, true));
        }
        let mut m = MotorState::new(4 << 2);
        assert!(!pick_frame(
            &mut w,
            &mut m,
            WorldCoord::new(0, 64, 0),
            3,
            false
        ));
        assert!(w.holders.is_empty() && !m.active);
    }
}

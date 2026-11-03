//! Pneumatic tube router: port of `TubeLib` (`RouteFinder`, `OutRouteFinder`, `InRouteFinder`,
//! `WorldRoute`, `getConnections`, class compatibility). The FIFO buffer is `breaker::TubeBuffer`.
//!
//! Pure logic: the engine layer answers the questions in [`TubeWorld`]. Sides use the original
//! numbering (0 -Y, 1 +Y, 2 -Z, 3 +Z, 4 -X, 5 +X); `side ^ 1` is the opposite.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

pub use super::breaker::TubeBuffer;

pub type Pos = (i32, i32, i32);

/// `TubeLib.isCompatible`: tube connection classes (0 plain, 1..16 coloured, 17/18 redstone
/// bridges, 19..34 coloured redstone) that may join.
pub fn is_compatible(a: i32, b: i32) -> bool {
    if a == b {
        return true;
    }
    let one_way = |a: i32, b: i32| -> bool {
        (a == 0 && (b == 17 || (1..=16).contains(&b)))
            || (a == 17 && (b == 18 || (1..=16).contains(&b) || (19..=34).contains(&b)))
            || (a == 18 && (19..=34).contains(&b))
    };
    one_way(a, b) || one_way(b, a)
}

/// Position one step along `side`.
pub fn step(p: Pos, side: i32) -> Pos {
    match side {
        0 => (p.0, p.1 - 1, p.2),
        1 => (p.0, p.1 + 1, p.2),
        2 => (p.0, p.1, p.2 - 1),
        3 => (p.0, p.1, p.2 + 1),
        4 => (p.0 - 1, p.1, p.2),
        _ => (p.0 + 1, p.1, p.2),
    }
}

/// What a neighbour offers a tube on one of its sides (`isConSide`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Neighbour {
    None,
    /// An inventory with a non-empty side range, joins class 0 tubes only.
    Inventory,
    Tube {
        con_class: i32,
        sides: u8,
    },
}

/// `TubeLib.isConSide` for one neighbour.
pub fn is_con_side(n: Neighbour, col: i32, side: i32) -> bool {
    match n {
        Neighbour::None => false,
        Neighbour::Inventory => is_compatible(col, 0),
        Neighbour::Tube { con_class, sides } => {
            is_compatible(col, con_class) && sides & (1 << side) != 0
        }
    }
}

/// How the router sees one point on the way (`addPoint` outcomes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Point {
    /// Not part of the network.
    Skip,
    /// A tube that routes: queue it with its entry side `side ^ 1`.
    Route,
    /// A destination: solved at `weight + extra`, `start` and `side` as given.
    Solved { start: i32, side: i32, extra: i32 },
    /// A tube that cannot take the item but passes it on: queue with `side1` and `weight + extra`.
    Pass { side: i32, extra: i32 },
}

/// The world as the router queries it.
pub trait TubeWorld {
    /// `getConnections`: bitmask of sides of the tube at `pos` that join something.
    fn connections(&self, pos: Pos) -> u8;
    /// Classify `pos` reached through `side` (travel direction) from start side `start`.
    fn point(&self, pos: Pos, start: i32, side: i32) -> Point;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldRoute {
    pub pos: Pos,
    pub start: i32,
    pub side: i32,
    pub weight: i32,
    pub solved: bool,
    seq: u32,
}

impl Ord for WorldRoute {
    // BinaryHeap is a max-heap: lowest weight first, then insertion order.
    fn cmp(&self, o: &Self) -> Ordering {
        o.weight
            .cmp(&self.weight)
            .then_with(|| o.seq.cmp(&self.seq))
    }
}
impl PartialOrd for WorldRoute {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// `TubeLib.RouteFinder`.
pub struct RouteFinder<'a, W: TubeWorld> {
    world: &'a W,
    pub start_dir: i32,
    pub result: Option<WorldRoute>,
    scanmap: HashSet<Pos>,
    queue: BinaryHeap<WorldRoute>,
    seq: u32,
}

impl<'a, W: TubeWorld> RouteFinder<'a, W> {
    pub fn new(world: &'a W) -> Self {
        Self {
            world,
            start_dir: 0,
            result: None,
            scanmap: HashSet::new(),
            queue: BinaryHeap::new(),
            seq: 0,
        }
    }

    fn push(&mut self, pos: Pos, start: i32, side: i32, weight: i32, solved: bool) {
        self.seq += 1;
        self.queue.push(WorldRoute {
            pos,
            start,
            side,
            weight,
            solved,
            seq: self.seq,
        });
    }

    fn add_point(&mut self, pos: Pos, start: i32, side: i32, weight: i32) {
        match self.world.point(pos, start, side) {
            Point::Skip => {}
            Point::Route => {
                if self.scanmap.insert(pos) {
                    self.push(pos, start, side ^ 1, weight, false);
                }
            }
            Point::Solved {
                start: s,
                side: sd,
                extra,
            } => {
                self.push(pos, s, sd, weight + extra, true);
            }
            Point::Pass { side: sd, extra } => {
                if self.scanmap.insert(pos) {
                    self.push(pos, start, sd, weight + extra, false);
                }
            }
        }
    }

    /// `find`: the start side of the cheapest route from `from` over `sides`, or -1.
    pub fn find(&mut self, from: Pos, sides: u8) -> i32 {
        for n in 0..6 {
            if sides & (1 << n) != 0 {
                let w = if n == self.start_dir { 0 } else { 1 };
                self.add_point(step(from, n), n, n, w);
            }
        }
        while let Some(wr) = self.queue.pop() {
            if wr.solved {
                self.result = Some(wr);
                return wr.start;
            }
            let cons = self.world.connections(wr.pos);
            for n in 0..6 {
                if n != wr.side && cons & (1 << n) != 0 {
                    self.add_point(step(wr.pos, n), wr.start, n, wr.weight + 2);
                }
            }
        }
        -1
    }

    pub fn result_point(&self) -> Option<Pos> {
        self.result.map(|r| r.pos)
    }
}

/// `TubeLib.findRoute` / `OutRouteFinder` rules, supplied by the engine layer.
pub trait OutRules {
    /// `state != 3 && priority == 0 && canAddToInventory(item, pos, side1)`.
    fn inventory_accepts(&self, pos: Pos, side1: i32) -> bool;
    /// Tube at `pos`: `Some((can_enter_state, can_enter_0, can_route, weight))` or `None`.
    fn tube_at(&self, pos: Pos, side1: i32) -> Option<TubeAnswer>;
    fn connections(&self, pos: Pos) -> u8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TubeAnswer {
    pub can_enter_state: bool,
    pub can_enter_idle: bool,
    pub can_route: bool,
    /// `tubeWeight(side1, state)`; restrictor tubes report 1,000,000.
    pub weight: i32,
}

struct OutWorld<'a, R: OutRules>(&'a R);

impl<R: OutRules> TubeWorld for OutWorld<'_, R> {
    fn connections(&self, pos: Pos) -> u8 {
        self.0.connections(pos)
    }
    fn point(&self, pos: Pos, start: i32, side: i32) -> Point {
        let side1 = (side ^ 1) & 0xFF;
        if self.0.inventory_accepts(pos, side1) {
            return Point::Solved {
                start,
                side,
                extra: 0,
            };
        }
        match self.0.tube_at(pos, side1) {
            Some(t) if t.can_enter_state => Point::Solved {
                start,
                side: side1,
                extra: t.weight,
            },
            Some(t) if t.can_enter_idle && t.can_route => Point::Pass {
                side: side1,
                extra: t.weight,
            },
            _ => Point::Skip,
        }
    }
}

/// `TubeLib.findRoute(world, wc, item, sides, state, start)`: the start side or -1.
pub fn find_route<R: OutRules>(rules: &R, from: Pos, sides: u8, start_dir: i32) -> i32 {
    let w = OutWorld(rules);
    let mut rf = RouteFinder::new(&w);
    rf.start_dir = start_dir;
    rf.find(from, sides)
}

/// Like [`find_route`] but also returns the destination and its side.
pub fn find_route_full<R: OutRules>(
    rules: &R,
    from: Pos,
    sides: u8,
    start_dir: i32,
) -> Option<(i32, WorldRoute)> {
    let w = OutWorld(rules);
    let mut rf = RouteFinder::new(&w);
    rf.start_dir = start_dir;
    let s = rf.find(from, sides);
    rf.result.map(|r| (s, r))
}

/// `TubeLib.getConnections`: `own_sides`/`own_class` are the tube's, `neighbour(side)` the thing
/// across that side.
pub fn get_connections(own_sides: u8, own_class: i32, neighbour: impl Fn(i32) -> Neighbour) -> u8 {
    // Neighbour sees us from the opposite side.
    (0..6)
        .filter(|&s| own_sides & (1 << s) != 0 && is_con_side(neighbour(s), own_class, s ^ 1))
        .fold(0u8, |m, s| m | (1 << s))
}

/// `InRouteFinder`: an inventory holding a wanted stack is a solved point.
pub trait InRules {
    fn connections(&self, pos: Pos) -> u8;
    fn is_tube(&self, pos: Pos) -> bool;
    /// For an inventory at `pos` entered via `side2`: `Some(slot_match)` if it holds a match.
    fn inventory_match(&self, pos: Pos, side2: i32) -> Option<Option<i32>>;
}

struct InWorld<'a, R: InRules>(&'a R);

impl<R: InRules> TubeWorld for InWorld<'_, R> {
    fn connections(&self, pos: Pos) -> u8 {
        self.0.connections(pos)
    }
    fn point(&self, pos: Pos, start: i32, side: i32) -> Point {
        match self.0.inventory_match(pos, side) {
            None => {
                if self.0.is_tube(pos) {
                    Point::Route
                } else {
                    Point::Skip
                }
            }
            Some(Some(sm)) => Point::Solved {
                start: sm,
                side: (side ^ 1) & 63,
                extra: 0,
            },
            Some(None) => {
                if self.0.is_tube(pos) {
                    let _ = start;
                    Point::Route
                } else {
                    Point::Skip
                }
            }
        }
    }
}

/// Route toward a wanted inventory; returns the matching filter slot and the result route.
pub fn find_in_route<R: InRules>(rules: &R, from: Pos, sides: u8) -> Option<(i32, WorldRoute)> {
    let w = InWorld(rules);
    let mut rf = RouteFinder::new(&w);
    let s = rf.find(from, sides);
    if s < 0 {
        return None;
    }
    rf.result.map(|r| (s, r))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Net {
        tubes: HashMap<Pos, i32>, // pos -> extra weight
        chest: Pos,
    }
    impl OutRules for Net {
        fn inventory_accepts(&self, pos: Pos, _s: i32) -> bool {
            pos == self.chest
        }
        fn tube_at(&self, pos: Pos, _s: i32) -> Option<TubeAnswer> {
            self.tubes.get(&pos).map(|&w| TubeAnswer {
                can_enter_state: false,
                can_enter_idle: true,
                can_route: true,
                weight: w,
            })
        }
        fn connections(&self, pos: Pos) -> u8 {
            (0..6)
                .filter(|&s| {
                    let p = step(pos, s);
                    self.tubes.contains_key(&p) || p == self.chest
                })
                .fold(0, |m, s| m | (1 << s))
        }
    }

    #[test]
    fn compat_table() {
        assert!(is_compatible(0, 17) && is_compatible(17, 18) && is_compatible(3, 0));
        assert!(is_compatible(19, 18) && is_compatible(5, 17));
        assert!(!is_compatible(0, 18) && !is_compatible(1, 2) && !is_compatible(1, 19));
    }

    #[test]
    fn routes_straight_to_chest() {
        let mut tubes = HashMap::new();
        for x in 1..=3 {
            tubes.insert((x, 0, 0), 0);
        }
        let net = Net {
            tubes,
            chest: (4, 0, 0),
        };
        let (s, r) = find_route_full(&net, (0, 0, 0), 1 << 5, 5).unwrap();
        assert_eq!(s, 5);
        assert_eq!(r.pos, (4, 0, 0));
        // start weight 0, then hops through tubes 1,2,3 (+2 each from 1->2, 2->3, 3->chest).
        assert_eq!(r.weight, 6);
    }

    #[test]
    fn restrictor_is_last_resort() {
        // Two branches off (1,0,0): a restrictor on +X side, a longer plain loop up and around.
        let mut tubes = HashMap::new();
        tubes.insert((1, 0, 0), 0);
        tubes.insert((2, 0, 0), 1_000_000); // restrictor, adjacent to chest
        tubes.insert((1, 1, 0), 0);
        tubes.insert((2, 1, 0), 0);
        tubes.insert((3, 1, 0), 0);
        tubes.insert((3, 0, 0), 0);
        let net = Net {
            tubes,
            chest: (4, 0, 0),
        };
        let (_, r) = find_route_full(&net, (0, 0, 0), 1 << 5, 5).unwrap();
        assert!(r.weight < 1_000_000);
    }

    #[test]
    fn no_route_is_minus_one() {
        let net = Net {
            tubes: HashMap::new(),
            chest: (9, 9, 9),
        };
        assert_eq!(find_route(&net, (0, 0, 0), 0x3f, 0), -1);
    }

    #[test]
    fn connections_mask() {
        let m = get_connections(0b111111, 0, |s| {
            if s == 5 {
                Neighbour::Tube {
                    con_class: 0,
                    sides: 1 << 4,
                }
            } else if s == 1 {
                Neighbour::Inventory
            } else {
                Neighbour::None
            }
        });
        assert_eq!(m, (1 << 5) | (1 << 1));
    }

    #[test]
    fn buffer_is_fifo() {
        let mut b = TubeBuffer::default();
        assert!(b.is_empty());
        b.pop();
        assert!(!b.plugged);
    }
}

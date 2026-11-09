//! Core subsystem integration scenarios for RedPower 2.
//!
//! Validates:
//! 1. Microblock compound placement, sub-voxel occupancy bitmasks, and hollow center pass-through.
//! 2. Pneumatic tube Dijkstra shortest-path routing, turn penalties, restrictor tube overflow (+1,000,000 weight),
//!    and color tag routing.
//! 3. Red alloy wire corner boundary pathfinding, signal decay, color isolation, and bundled cable bus multiplexing.

use redpower_2::tubes::restrictor_tube::RESTRICTOR_TUBE_WEIGHT;
use redpower_2::wiring::network::{
    Tile, WireKind as NetWireKind, WireNode, WireWorld, CON_DIR_MASKS,
};
use redpower_2::{
    find_tube_route, internal_to_vanilla_redstone, recombine_microblocks,
    red_alloy_signal_at_distance, red_alloy_signal_lossless, red_alloy_to_vanilla, BlockPos,
    Direction, DyeColor, GridGraph, GridNode, MicroblockKind, MicroblockVoxel, WireKind,
    WorldCoord,
};

// ============================================================================
// 1. Microblock Compound Placement & Sub-Voxel Occupancy Scenarios
// ============================================================================

#[test]
fn test_microblock_compound_faces_edges_corners_occupancy() {
    let mut voxel = MicroblockVoxel::new();
    assert_eq!(voxel.occupancy_mask(), 0);

    // Place solid covers on 3 distinct faces (Down, North, West)
    assert!(voxel.place_on_face(Direction::Down, MicroblockKind::Cover));
    assert!(voxel.place_on_face(Direction::North, MicroblockKind::Cover));
    assert!(voxel.place_on_face(Direction::West, MicroblockKind::Cover));

    // Faces occupy bits 0..5
    let mask = voxel.occupancy_mask();
    assert_ne!(mask & (1 << (Direction::Down as usize)), 0);
    assert_ne!(mask & (1 << (Direction::North as usize)), 0);
    assert_ne!(mask & (1 << (Direction::West as usize)), 0);

    // Cannot place second cover on already occupied face
    assert!(!voxel.place_on_face(Direction::Down, MicroblockKind::Cover));
    assert!(!voxel.place_on_face(Direction::North, MicroblockKind::HollowCover));

    // Solid covers seal center connection through that face
    assert!(!voxel.allows_center_connection(Direction::Down));
    assert!(!voxel.allows_center_connection(Direction::North));
    assert!(!voxel.allows_center_connection(Direction::West));

    // Hollow cover on Up face allows center conduit connection
    assert!(voxel.place_on_face(Direction::Up, MicroblockKind::HollowCover));
    assert!(voxel.allows_center_connection(Direction::Up));

    // Edge strips occupy bits 6..17
    for edge in 0..12 {
        assert!(voxel.place_edge(edge, MicroblockKind::CornerStrip));
        assert_ne!(voxel.occupancy_mask() & (1 << (6 + edge)), 0);
    }
    // Duplicate edge strip rejected
    assert!(!voxel.place_edge(0, MicroblockKind::CornerStrip));

    // Corners occupy bits 18..25
    for corner in 0..8 {
        assert!(voxel.place_corner(corner, MicroblockKind::Corner));
        assert_ne!(voxel.occupancy_mask() & (1 << (18 + corner)), 0);
    }
    // Duplicate corner rejected
    assert!(!voxel.place_corner(3, MicroblockKind::Corner));

    // Center conduit occupies bit 26
    voxel.center_conduit = Some("redpower_2:wire_red_alloy".into());
    assert_ne!(voxel.occupancy_mask() & (1 << 26), 0);
}

#[test]
fn test_microblock_slicing_and_recombination() {
    // 2 Covers (1/8 + 1/8) = 1 Panel (1/4)
    assert_eq!(
        recombine_microblocks(MicroblockKind::Cover, MicroblockKind::Cover),
        Some(MicroblockKind::Panel)
    );

    // 2 Panels (1/4 + 1/4) = 1 Slab (1/2)
    assert_eq!(
        recombine_microblocks(MicroblockKind::Panel, MicroblockKind::Panel),
        Some(MicroblockKind::Slab)
    );

    // Incompatible kinds do not recombine
    assert_eq!(
        recombine_microblocks(MicroblockKind::Cover, MicroblockKind::Corner),
        None
    );
}

// ============================================================================
// 2. Pneumatic Tube Dijkstra Routing with Restrictor & Color Tag Scenarios
// ============================================================================

#[test]
fn test_pneumatic_tube_dijkstra_shortest_path_and_turn_penalty() {
    let mut graph = GridGraph::new();

    // Route 1 (Straight): (0,0,0) -> (1,0,0) -> (2,0,0) -> (3,0,0) [3 hops, 0 turns]
    // Cost: 3 hops * 2 = 6
    graph.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(1, 0, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(2, 0, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(3, 0, 0), GridNode::STANDARD);

    // Route 2 (Turn): (0,0,0) -> (0,0,1) -> (1,0,1) -> (2,0,1) [3 hops, 1 turn]
    // Cost: 3 hops * 2 + 1 = 7
    graph.insert_node(BlockPos(0, 0, 1), GridNode::STANDARD);
    graph.insert_node(BlockPos(1, 0, 1), GridNode::STANDARD);
    graph.insert_node(BlockPos(2, 0, 1), GridNode::STANDARD);

    let res = find_tube_route(
        &graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(3, 0, 0)],
        None,
    );
    let (dest, path, cost) = res.expect("straight route should be found");
    assert_eq!(dest, BlockPos(3, 0, 0));
    assert_eq!(path.len(), 4);
    assert_eq!(cost, 6);
}

#[test]
fn test_pneumatic_tube_restrictor_overflow_routing() {
    let mut graph = GridGraph::new();

    // Start at (0,0,0)
    graph.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);

    // Primary path to Dest A (2, 0, 0) via restrictor tube (+1,000,000 weight)
    graph.insert_node(
        BlockPos(1, 0, 0),
        GridNode {
            weight: RESTRICTOR_TUBE_WEIGHT as u32,
            color: None,
        },
    );
    graph.insert_node(BlockPos(2, 0, 0), GridNode::STANDARD); // Dest A

    // Alternate longer path to Dest B (0, 0, 4) via 4 standard tubes (no restrictor)
    graph.insert_node(BlockPos(0, 0, 1), GridNode::STANDARD);
    graph.insert_node(BlockPos(0, 0, 2), GridNode::STANDARD);
    graph.insert_node(BlockPos(0, 0, 3), GridNode::STANDARD);
    graph.insert_node(BlockPos(0, 0, 4), GridNode::STANDARD); // Dest B

    // Dijkstra should prefer Dest B because Dest A incurs +1,000,000 restrictor penalty
    let res = find_tube_route(
        &graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(2, 0, 0), BlockPos(0, 0, 4)],
        None,
    );
    let (dest, path, cost) = res.expect("route should prefer un-restricted path");
    assert_eq!(dest, BlockPos(0, 0, 4));
    assert_eq!(path.last().copied(), Some(BlockPos(0, 0, 4)));
    assert!(cost < RESTRICTOR_TUBE_WEIGHT as u32);

    // If Dest B is not available, Dijkstra routes through restrictor to Dest A
    let overflow_res = find_tube_route(
        &graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(2, 0, 0)],
        None,
    );
    let (dest_a, _, cost_a) = overflow_res.expect("overflow should route through restrictor");
    assert_eq!(dest_a, BlockPos(2, 0, 0));
    assert!(cost_a >= RESTRICTOR_TUBE_WEIGHT as u32);
}

#[test]
fn test_pneumatic_tube_color_tag_filtering() {
    let mut graph = GridGraph::new();
    graph.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);

    // Blue-tagged tube branch leading to (2, 0, 0)
    graph.insert_node(
        BlockPos(1, 0, 0),
        GridNode {
            weight: 0,
            color: Some(DyeColor::Blue),
        },
    );
    graph.insert_node(BlockPos(2, 0, 0), GridNode::STANDARD);

    // Red-tagged item cannot route through Blue tube
    let red_res = find_tube_route(
        &graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(2, 0, 0)],
        Some(DyeColor::Red),
    );
    assert!(red_res.is_none(), "Red item cannot enter Blue tube");

    // Blue-tagged item successfully routes through Blue tube
    let blue_res = find_tube_route(
        &graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(2, 0, 0)],
        Some(DyeColor::Blue),
    );
    assert!(
        blue_res.is_some(),
        "Blue item should route through Blue tube"
    );

    // Untagged item (None) can enter any colored tube
    let untagged_res = find_tube_route(
        &graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(2, 0, 0)],
        None,
    );
    assert!(
        untagged_res.is_some(),
        "Untagged item should route through colored tube"
    );
}

// ============================================================================
// 3. Red Alloy Wire Pathfinding & Corner Boundary Traversal Scenarios
// ============================================================================

#[test]
fn test_red_alloy_wire_distance_decay_and_lossless_propagation() {
    // Distance decay: signal decreases by 1 per block hop
    assert_eq!(red_alloy_signal_at_distance(255, 0), 255);
    assert_eq!(red_alloy_signal_at_distance(255, 10), 245);
    assert_eq!(red_alloy_signal_at_distance(255, 100), 155);
    assert_eq!(red_alloy_signal_at_distance(255, 254), 1);
    assert_eq!(red_alloy_signal_at_distance(255, 255), 0);
    assert_eq!(red_alloy_signal_at_distance(255, 500), 0);

    // Lossless propagation across up to 256 blocks
    assert_eq!(red_alloy_signal_lossless(255, 0), 255);
    assert_eq!(red_alloy_signal_lossless(255, 100), 255);
    assert_eq!(red_alloy_signal_lossless(255, 255), 255);
    assert_eq!(red_alloy_signal_lossless(255, 256), 0);

    // Vanilla redstone level mapping: strength / 17
    assert_eq!(internal_to_vanilla_redstone(0), 0);
    assert_eq!(internal_to_vanilla_redstone(16), 0);
    assert_eq!(internal_to_vanilla_redstone(17), 1);
    assert_eq!(internal_to_vanilla_redstone(255), 15);
    assert_eq!(red_alloy_to_vanilla(255), 15);
}

#[test]
fn test_wire_network_propagation_and_corner_traversal() {
    let mut w = WireWorld::new();
    // Build a 5-block straight line of Red Alloy wire along the X axis
    for x in 1..=5 {
        let mut m = 0;
        if x > 1 {
            m |= CON_DIR_MASKS[4]; // West
        }
        if x < 5 {
            m |= CON_DIR_MASKS[5]; // East
        }
        w.set(
            WorldCoord::new(x, 0, 0),
            Tile::Wire(WireNode::new(NetWireKind::Red, m, 0)),
        );
    }

    // Power node 1 with strength 255
    if let Some(Tile::Wire(n)) = w.tiles.get_mut(&WorldCoord::new(1, 0, 0)) {
        n.powering[0] = 255;
    }

    // Propagate signal through network
    w.update_current(WorldCoord::new(1, 0, 0));

    // Verify distance decay: -1 per block hop
    let strengths: Vec<i32> = (1..=5)
        .map(|x| w.strength(WorldCoord::new(x, 0, 0), 0))
        .collect();
    assert_eq!(strengths, vec![255, 254, 253, 252, 251]);

    // Clearing powering source resets the wire network back to 0
    if let Some(Tile::Wire(n)) = w.tiles.get_mut(&WorldCoord::new(1, 0, 0)) {
        n.powering[0] = 0;
    }
    w.update_current(WorldCoord::new(1, 0, 0));
    for x in 1..=5 {
        assert_eq!(w.strength(WorldCoord::new(x, 0, 0), 0), 0);
    }
}

#[test]
fn test_wire_color_isolation_and_bundled_multiplexing() {
    // WireKind isolation rules:
    // RedAlloy connects to RedAlloy and any Insulated
    assert!(WireKind::RedAlloy.can_connect_to(WireKind::RedAlloy));
    assert!(WireKind::RedAlloy.can_connect_to(WireKind::Insulated(DyeColor::Red)));
    assert!(WireKind::RedAlloy.can_connect_to(WireKind::Insulated(DyeColor::Blue)));
    assert!(WireKind::Insulated(DyeColor::Red).can_connect_to(WireKind::RedAlloy));

    // Insulated wires connect ONLY to identical color
    assert!(WireKind::Insulated(DyeColor::Red).can_connect_to(WireKind::Insulated(DyeColor::Red)));
    assert!(WireKind::Insulated(DyeColor::Blue).can_connect_to(WireKind::Insulated(DyeColor::Blue)));
    assert!(!WireKind::Insulated(DyeColor::Red).can_connect_to(WireKind::Insulated(DyeColor::Blue)));

    // Bundled cable 16-channel independent transmission
    let mut w = WireWorld::new();
    for x in 1..=3 {
        let mut m = 0;
        if x > 1 {
            m |= CON_DIR_MASKS[4];
        }
        if x < 3 {
            m |= CON_DIR_MASKS[5];
        }
        w.set(
            WorldCoord::new(x, 0, 0),
            Tile::Wire(WireNode::new(NetWireKind::Cable, m, 0)),
        );
    }

    // Drive channel 3 (Color 2) through source tile at (0,0,0)
    let mut pm = [0u32; 17];
    pm[3] = CON_DIR_MASKS[5];
    w.set(WorldCoord::new(0, 0, 0), Tile::Source { powering_mask: pm });
    if let Some(Tile::Wire(n)) = w.tiles.get_mut(&WorldCoord::new(1, 0, 0)) {
        n.connection_mask |= CON_DIR_MASKS[4];
        n.connectable_mask |= CON_DIR_MASKS[4];
    }

    w.update_current(WorldCoord::new(1, 0, 0));
    assert_eq!(w.strength(WorldCoord::new(3, 0, 0), 3), 252);
    assert_eq!(w.strength(WorldCoord::new(3, 0, 0), 4), 0);
    assert_eq!(w.bundled_mask(WorldCoord::new(3, 0, 0)), 1 << 2);
}

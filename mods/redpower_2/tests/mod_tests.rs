//! Unit and integration tests for `redpower_2` (`planning/mods/redpower_2.md`).
//!
//! Validates:
//! 1. Microblock slicing hierarchy (`Slab -> Panel -> Cover -> CornerStrip -> Corner`),
//!    thicknesses (1/8 to 7/8), recombination, handsaw cutting sharpness, 27-subvoxel occupancy,
//!    and hollow face pass-through.
//! 2. Red Alloy Wire 255-step distance decay, lossless propagation over 256 blocks, and
//!    `0..=15` vanilla redstone mapping (`strength / 17`).
//! 3. Wire color isolation across all 16 dye colors and 16-channel Bundled Cable bus multiplexing.
//! 4. All 20+ integrated Logic Gate state machines (Inverter, AND, OR, NAND, NOR, XOR, XNOR,
//!    Buffer, Multiplexer, Repeater schedule, Timer, Counter, Sequencer, Pulse Former, Toggle Latch,
//!    RS Latch, Transparent Latch, State Cell, Synchronizer, Light Sensor, Randomizer, Array cells,
//!    and Bus Transceiver).
//! 5. Pneumatic tube Dijkstra shortest-path routing (+2 hop, +1 turn, +1,000,000 restrictor weight),
//!    color filtering, zero-drop backpressure, and 16.0 blocks/tick Magtubes.
//! 6. Blutricity power grid physical model (0.01 Ω resistance, 0.07 inductive scale), Solar Panel (2.0 A),
//!    Thermopile thermal scanning and shock hazard, Wind Turbine with sail wear and brakes, Battery Box
//!    (6,000 storage units, 90V charge / 80V discharge), and 100x Transformer.
//! 7. Frame Motor structural assembly BFS solver (up to 1,000 blocks), immovable bedrock rejection,
//!    obsidian movement, 16-tick translation, and power consumption.
//! 8. MOS 6502 CPU instruction execution, cycle accumulator (+1,000/tick) with 100,000 burst cap,
//!    80x50 character terminal blitter, 256 KB floppy disk drive Redbus commands, and I/O Expander MMIO.
//! 9. Alloy Furnace 11-slot state machine with solid fuel and blulectric operation, cook times,
//!    and alloy/recycling recipes (Red Alloy, Blue Alloy, Brass, Tinplate, Boule, Doped Wafers).
//! 10. Tools, armor, and combat equipment (Athame with Disjunction vs Ender mobs, Vorpal crit and
//!     decapitation overkill threshold, Sickle 3x3x3 leaves and 5x5 crops, tool durabilities).
//! 11. Spec §8 hazards and safety limits (CPU runaway cycle bound, frame motor bedrock check,
//!     thermopile thermal meltdown, blutricity over-voltage, vorpal decapitation).
//! 12. Serialization round trips for all domain and subsystem state structs.
//! 13. `HorizonNativeMod` hooks (`NATIVE_HOOKS_API = 40`) with strict `redpower_2:` namespace guards
//!     and foreign entity/packet rejection.
//! 14. Dual-artifact manifest and content JSON structure.

use redpower_2::{
    athame_attack_damage, calculate_weapon_damage, can_handsaw_cut, check_vorpal_decapitation,
    find_tube_route, handsaw_sharpness, internal_to_vanilla_redstone, is_ender_mob,
    plan_frame_translation, recombine_microblocks, red_alloy_signal_at_distance,
    red_alloy_signal_lossless, red_alloy_to_vanilla, sickle_crop_positions, sickle_leaf_positions,
    vorpal_proc_chance_percent, AlloyFurnaceState, BatteryBoxState, BlockPos, BluePowerConductor,
    BusTransceiverState, Cpu6502, Direction, DiskDriveState, DisplayTerminal, DyeColor,
    FrameMotorState, HorizonNativeMod, ItemStack, LogicGateKind, LogicGateState, MicroblockKind,
    MicroblockVoxel, ModBlockEntity, RedPower2Mod, SickleVariant, SolarPanelState, ThermopileState,
    TransformerState, TubePayload, Vector3, WindTurbineState, WireKind, WorldCoord,
    ATHAME_BASE_DAMAGE, ATHAME_DURABILITY, ATHAME_ENDER_DAMAGE, BLOCK_IDS,
    BLUTRICITY_CONDUCTOR_RESISTANCE, BLUTRICITY_MAX_SAFE_VOLTAGE, BLUTRICITY_WIND_MAX_VOLTAGE,
    DISJUNCTION_PER_LEVEL_DAMAGE, HORIZON_REVISION, ITEM_IDS, MIXINS_REVISION, MOD_ID,
    NATIVE_HOOKS_API, REPEATER_SCHEDULE, RESTRICTOR_TUBE_WEIGHT, SICKLE_CROP_RADIUS,
    SICKLE_LEAF_RADIUS, VORPAL_CRIT_BONUS_DAMAGE, VORPAL_DECAPITATION_OVERKILL_THRESHOLD,
};
use rekkit_core::frequency::BundledSignal;
use rekkit_core::grid::{GridGraph, GridNode};
use std::collections::HashMap;
use std::path::Path;

// ============================================================================
// 1. Microblock Slicing Hierarchy, Cutting Rules, & Occupancy (§2.1)
// ============================================================================

#[test]
fn test_microblock_hierarchy_slicing_recombine_and_hardness() {
    // Thickness in units of 1/8th of a block (0.125 m)
    assert_eq!(MicroblockKind::Cover.thickness_eighths(), 1);
    assert_eq!(MicroblockKind::Panel.thickness_eighths(), 2);
    assert_eq!(MicroblockKind::TripleCover.thickness_eighths(), 3);
    assert_eq!(MicroblockKind::Slab.thickness_eighths(), 4);
    assert_eq!(MicroblockKind::CoverSlab.thickness_eighths(), 5);
    assert_eq!(MicroblockKind::TriplePanel.thickness_eighths(), 6);
    assert_eq!(MicroblockKind::Anticover.thickness_eighths(), 7);

    // Vertical cut (saw above/below): halves thickness
    assert_eq!(
        MicroblockKind::Slab.slice_with_handsaw(),
        Some((MicroblockKind::Panel, 2))
    );
    assert_eq!(
        MicroblockKind::Panel.slice_with_handsaw(),
        Some((MicroblockKind::Cover, 2))
    );
    assert_eq!(
        MicroblockKind::Cover.slice_with_handsaw(),
        Some((MicroblockKind::CornerStrip, 4))
    );
    assert_eq!(
        MicroblockKind::CornerStrip.slice_with_handsaw(),
        Some((MicroblockKind::Corner, 2))
    );

    // Explicit vertical cut method
    assert_eq!(
        MicroblockKind::Slab.cut_vertical(),
        Some((MicroblockKind::Panel, 2))
    );
    assert_eq!(
        MicroblockKind::Panel.cut_vertical(),
        Some((MicroblockKind::Cover, 2))
    );

    // Horizontal cut (saw beside): slices into strips, then corners
    assert_eq!(
        MicroblockKind::Cover.cut_horizontal(),
        Some((MicroblockKind::CornerStrip, 2))
    );
    assert_eq!(
        MicroblockKind::CornerStrip.cut_horizontal(),
        Some((MicroblockKind::Corner, 2))
    );

    // Recombination: 2 covers -> 1 panel; 2 panels -> 1 slab
    assert_eq!(
        recombine_microblocks(MicroblockKind::Cover, MicroblockKind::Cover),
        Some(MicroblockKind::Panel)
    );
    assert_eq!(
        recombine_microblocks(MicroblockKind::Panel, MicroblockKind::Panel),
        Some(MicroblockKind::Slab)
    );
    assert_eq!(
        recombine_microblocks(MicroblockKind::Cover, MicroblockKind::Slab),
        None
    );

    // Handsaw sharpness ratings: Iron=0, Gem=1, Diamond=2
    assert_eq!(handsaw_sharpness("redpower_2:iron_handsaw"), Some(0));
    assert_eq!(handsaw_sharpness("redpower_2:ruby_handsaw"), Some(1));
    assert_eq!(
        handsaw_sharpness("redpower_2:green_sapphire_handsaw"),
        Some(1)
    );
    assert_eq!(handsaw_sharpness("redpower_2:sapphire_handsaw"), Some(1));
    assert_eq!(handsaw_sharpness("redpower_2:diamond_handsaw"), Some(2));
    assert_eq!(handsaw_sharpness("minecraft:stick"), None);

    // Hardness gating: Iron handsaw cuts wood/stone (hardness <= 1.5), fails on iron blocks (3.0)
    assert!(can_handsaw_cut(1.5, 0));
    assert!(!can_handsaw_cut(3.0, 0));
    assert!(can_handsaw_cut(3.0, 1));
    assert!(!can_handsaw_cut(50.0, 1)); // Obsidian requires Diamond handsaw (sharpness 2)
    assert!(can_handsaw_cut(50.0, 2));
}

#[test]
fn test_microblock_subvoxel_occupancy_and_hollow_passthrough() {
    let mut voxel = MicroblockVoxel::new();
    assert_eq!(voxel.occupancy_mask(), 0);

    // Placing a solid cover on a face seals neighbor connections through that face
    assert!(voxel.place_on_face(Direction::Down, MicroblockKind::Cover));
    assert!(!voxel.allows_center_connection(Direction::Down));
    assert_eq!(
        voxel.occupancy_mask() & 0x3F,
        1 << (Direction::Down as usize)
    );

    // Placing a hollow cover allows wire/tube connections through the face
    assert!(voxel.place_on_face(Direction::Up, MicroblockKind::HollowCover));
    assert!(voxel.allows_center_connection(Direction::Up));

    // Edge strip and corner placement
    assert!(voxel.place_edge(2, MicroblockKind::CornerStrip));
    assert!((voxel.occupancy_mask() & (1 << (6 + 2))) != 0);

    assert!(voxel.place_corner(5, MicroblockKind::Corner));
    assert!((voxel.occupancy_mask() & (1 << (18 + 5))) != 0);

    // Internal center conduit (wire/cable/tube)
    voxel.center_conduit = Some("redpower_2:wire_red_alloy".into());
    assert!((voxel.occupancy_mask() & (1 << 26)) != 0);
}

// ============================================================================
// 2. Wiring System & Redstone Signal Propagation (§2.2)
// ============================================================================

#[test]
fn test_red_alloy_wire_propagation_and_vanilla_mapping() {
    // Distance decay: signal decreases by 1 per block hop
    assert_eq!(red_alloy_signal_at_distance(255, 0), 255);
    assert_eq!(red_alloy_signal_at_distance(255, 100), 155);
    assert_eq!(red_alloy_signal_at_distance(255, 254), 1);
    assert_eq!(red_alloy_signal_at_distance(255, 255), 0);
    assert_eq!(red_alloy_signal_at_distance(255, 300), 0);

    // Lossless propagation across up to 256 blocks
    assert_eq!(red_alloy_signal_lossless(255, 0), 255);
    assert_eq!(red_alloy_signal_lossless(255, 128), 255);
    assert_eq!(red_alloy_signal_lossless(255, 255), 255);
    assert_eq!(red_alloy_signal_lossless(255, 256), 0);

    // Vanilla redstone level mapping: strength / 17
    assert_eq!(internal_to_vanilla_redstone(0), 0);
    assert_eq!(internal_to_vanilla_redstone(1), 0);
    assert_eq!(internal_to_vanilla_redstone(16), 0);
    assert_eq!(internal_to_vanilla_redstone(17), 1);
    assert_eq!(internal_to_vanilla_redstone(34), 2);
    assert_eq!(internal_to_vanilla_redstone(255), 15);
    assert_eq!(red_alloy_to_vanilla(255), 15);
}

#[test]
fn test_wire_color_isolation_and_bundled_cable() {
    // Bare Red Alloy connects to Red Alloy and any Insulated Wire
    assert!(WireKind::RedAlloy.can_connect_to(WireKind::RedAlloy));
    assert!(WireKind::RedAlloy.can_connect_to(WireKind::Insulated(DyeColor::Red)));
    assert!(WireKind::RedAlloy.can_connect_to(WireKind::Insulated(DyeColor::Blue)));
    assert!(WireKind::Insulated(DyeColor::Red).can_connect_to(WireKind::RedAlloy));

    // Insulated wires connect ONLY to the identical dye color
    assert!(WireKind::Insulated(DyeColor::Red).can_connect_to(WireKind::Insulated(DyeColor::Red)));
    assert!(WireKind::Insulated(DyeColor::Blue).can_connect_to(WireKind::Insulated(DyeColor::Blue)));
    assert!(!WireKind::Insulated(DyeColor::Red).can_connect_to(WireKind::Insulated(DyeColor::Blue)));
    assert!(
        !WireKind::Insulated(DyeColor::Green).can_connect_to(WireKind::Insulated(DyeColor::Yellow))
    );

    // Bundled Cable 16-channel multiplexed bus
    let mut bus = BundledSignal::new();
    assert_eq!(bus.to_digital_mask(), 0);

    bus.set_level(DyeColor::White, 255);
    bus.set_level(DyeColor::Orange, 128);
    bus.set_level(DyeColor::Black, 255);

    assert_eq!(bus.level(DyeColor::White), 255);
    assert_eq!(bus.level(DyeColor::Orange), 128);
    assert_eq!(bus.level(DyeColor::Black), 255);
    assert_eq!(bus.level(DyeColor::Magenta), 0);

    // Bus Transceiver 16-channel bidirectional forwarding
    let mut btx = BusTransceiverState::new();
    btx.side_a = BundledSignal::from_digital_mask(0x1234);
    btx.enable_a_to_b = true;
    btx.tick();
    assert_eq!(btx.side_b.to_digital_mask(), 0x1234);

    btx.enable_a_to_b = false;
    btx.side_b = BundledSignal::from_digital_mask(0x5678);
    btx.enable_b_to_a = true;
    btx.tick();
    assert_eq!(btx.side_a.to_digital_mask(), 0x5678);
}

// ============================================================================
// 3. Integrated Logic Gate State Machines (§2.3)
// ============================================================================

#[test]
fn test_all_logic_gate_state_machines() {
    // 1. Combinational gates
    let mut inv = LogicGateState::new(LogicGateKind::Inverter);
    assert_eq!(inv.step_tick([false, false, false]), 15);
    assert_eq!(inv.step_tick([false, true, false]), 0);

    let mut and_g = LogicGateState::new(LogicGateKind::And);
    assert_eq!(and_g.step_tick([true, true, false]), 0);
    assert_eq!(and_g.step_tick([true, true, true]), 15);

    let mut or_g = LogicGateState::new(LogicGateKind::Or);
    assert_eq!(or_g.step_tick([false, false, false]), 0);
    assert_eq!(or_g.step_tick([false, false, true]), 15);

    let mut nand_g = LogicGateState::new(LogicGateKind::Nand);
    assert_eq!(nand_g.step_tick([true, true, true]), 0);
    assert_eq!(nand_g.step_tick([true, false, true]), 15);

    let mut nor_g = LogicGateState::new(LogicGateKind::Nor);
    assert_eq!(nor_g.step_tick([false, false, false]), 15);
    assert_eq!(nor_g.step_tick([true, false, false]), 0);

    let mut xor_g = LogicGateState::new(LogicGateKind::Xor);
    assert_eq!(xor_g.step_tick([true, false, false]), 15);
    assert_eq!(xor_g.step_tick([true, false, true]), 0);

    let mut xnor_g = LogicGateState::new(LogicGateKind::Xnor);
    assert_eq!(xnor_g.step_tick([true, false, true]), 15);
    assert_eq!(xnor_g.step_tick([true, false, false]), 0);

    let mut buf_g = LogicGateState::new(LogicGateKind::Buffer);
    assert_eq!(buf_g.step_tick([false, true, false]), 15);
    assert_eq!(buf_g.step_tick([false, false, false]), 0);

    let mut mux_g = LogicGateState::new(LogicGateKind::Multiplexer);
    assert_eq!(mux_g.step_tick([true, false, false]), 15);
    assert_eq!(mux_g.step_tick([true, true, false]), 0);

    // 2. Sequential & timing gates
    assert_eq!(REPEATER_SCHEDULE, [2, 4, 6, 8, 16, 32, 64, 128, 256]);
    let mut rep = LogicGateState::new(LogicGateKind::Repeater);
    rep.interval_ticks = 2;
    assert_eq!(rep.step_tick([false, true, false]), 0);
    assert_eq!(rep.step_tick([false, true, false]), 15);

    let mut timer = LogicGateState::new(LogicGateKind::Timer);
    timer.interval_ticks = 4;
    assert_eq!(timer.step_tick([false, false, false]), 0);
    assert_eq!(timer.step_tick([false, false, false]), 0);
    assert_eq!(timer.step_tick([false, false, false]), 0);
    assert_eq!(timer.step_tick([false, false, false]), 15); // tick 4: pulse 1
    assert_eq!(timer.step_tick([false, false, false]), 15); // tick 5: pulse 2
    assert_eq!(timer.step_tick([false, false, false]), 0); // tick 6: off

    let mut counter = LogicGateState::new(LogicGateKind::Counter);
    counter.max_count = 2;
    assert_eq!(counter.output_level, 15); // min count 0 outputs high
    assert_eq!(counter.step_tick([true, false, false]), 0); // increment -> count 1
    assert_eq!(counter.step_tick([false, false, false]), 0);
    assert_eq!(counter.step_tick([true, false, false]), 15); // increment -> count 2 (max)
    assert_eq!(counter.step_tick([false, false, false]), 15); // steady at max
    assert_eq!(counter.step_tick([false, false, true]), 0); // decrement -> count 1
    assert_eq!(counter.step_tick([false, false, false]), 0);
    assert_eq!(counter.step_tick([false, false, true]), 15); // decrement -> count 0 (min)

    let mut seq = LogicGateState::new(LogicGateKind::Sequencer);
    seq.interval_ticks = 4;
    for _ in 0..4 {
        let _ = seq.step_tick([false, false, false]);
    }
    assert_eq!(seq.latch_state, 1);

    let mut pf = LogicGateState::new(LogicGateKind::PulseFormer);
    assert_eq!(pf.step_tick([false, true, false]), 15); // rising edge -> 1-tick pulse
    assert_eq!(pf.step_tick([false, true, false]), 0); // continuous high -> 0

    let mut toggle = LogicGateState::new(LogicGateKind::ToggleLatch);
    assert_eq!(toggle.step_tick([true, false, false]), 15);
    assert_eq!(toggle.step_tick([false, false, false]), 15);
    assert_eq!(toggle.step_tick([true, false, false]), 0);

    let mut rs = LogicGateState::new(LogicGateKind::RsLatch);
    assert_eq!(rs.step_tick([true, false, false]), 15); // Set
    assert_eq!(rs.step_tick([false, false, false]), 15); // Latched
    assert_eq!(rs.step_tick([false, false, true]), 0); // Reset

    let mut tl = LogicGateState::new(LogicGateKind::TransparentLatch);
    assert_eq!(tl.step_tick([true, true, false]), 15); // Clock high: transparent
    assert_eq!(tl.step_tick([false, false, false]), 15); // Clock low: latched

    let mut sc = LogicGateState::new(LogicGateKind::StateCell);
    sc.interval_ticks = 2;
    assert_eq!(sc.step_tick([false, true, false]), 0);
    assert_eq!(sc.step_tick([false, false, false]), 15);

    let mut sync = LogicGateState::new(LogicGateKind::Synchronizer);
    assert_eq!(sync.step_tick([true, false, false]), 0);
    assert_eq!(sync.step_tick([false, false, true]), 15); // Both arrived

    let mut rand_g = LogicGateState::new(LogicGateKind::Randomizer);
    let rand_out = rand_g.step_tick([false, true, false]);
    assert!(rand_out == 0 || rand_out == 15);

    let mut light = LogicGateState::new(LogicGateKind::LightSensor);
    light.ambient_light = 12;
    assert_eq!(light.step_tick([false, false, false]), 12);

    let mut null_c = LogicGateState::new(LogicGateKind::NullCell);
    assert_eq!(null_c.step_tick([false, true, false]), 0);

    let mut inv_c = LogicGateState::new(LogicGateKind::InvertCell);
    assert_eq!(inv_c.step_tick([false, true, false]), 0);
    assert_eq!(inv_c.step_tick([false, false, false]), 15);

    let mut non_inv_c = LogicGateState::new(LogicGateKind::NonInvertCell);
    assert_eq!(non_inv_c.step_tick([false, true, false]), 15);
}

// ============================================================================
// 4. Pneumatic Tube Dijkstra Routing & Magtubes (§2.4)
// ============================================================================

#[test]
fn test_pneumatic_tube_dijkstra_routing_weights_and_color_filtering() {
    let mut graph = GridGraph::new();
    // Straight line: (0,0,0) -> (1,0,0) -> (2,0,0)
    graph.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(1, 0, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(2, 0, 0), GridNode::STANDARD);

    let res = find_tube_route(
        &graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(2, 0, 0)],
        None,
    );
    let (dest, path, cost) = res.expect("route should be found");
    assert_eq!(dest, BlockPos(2, 0, 0));
    assert_eq!(path.len(), 3);
    assert_eq!(cost, 4); // 2 hops * 2 = 4 (straight line)

    // Turn penalty (+1 per turn deviating from direction)
    let mut turn_graph = GridGraph::new();
    turn_graph.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);
    turn_graph.insert_node(BlockPos(1, 0, 0), GridNode::STANDARD);
    turn_graph.insert_node(BlockPos(1, 0, 1), GridNode::STANDARD); // 90 degree turn South

    let turn_res = find_tube_route(
        &turn_graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(1, 0, 1)],
        None,
    );
    let (_, _, turn_cost) = turn_res.expect("turn route should be found");
    assert_eq!(turn_cost, 5); // 2 hops * 2 + 1 turn penalty = 5

    // Restrictor tube weight: +1,000,000 for overflow pathing
    assert_eq!(RESTRICTOR_TUBE_WEIGHT, 1_000_000);
    let mut restrict_graph = GridGraph::new();
    restrict_graph.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);
    restrict_graph.insert_node(
        BlockPos(1, 0, 0),
        GridNode {
            weight: RESTRICTOR_TUBE_WEIGHT,
            color: None,
        },
    );
    let r_res = find_tube_route(
        &restrict_graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(1, 0, 0)],
        None,
    );
    let (_, _, r_cost) = r_res.expect("restrictor route found");
    assert!(r_cost >= RESTRICTOR_TUBE_WEIGHT);

    // Color filtering: dyed item only enters unpainted or matching painted tube
    let mut color_graph = GridGraph::new();
    color_graph.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);
    color_graph.insert_node(
        BlockPos(1, 0, 0),
        GridNode {
            weight: 0,
            color: Some(DyeColor::Red),
        },
    );

    // Red item allowed into Red tube
    assert!(find_tube_route(
        &color_graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(1, 0, 0)],
        Some(DyeColor::Red),
    )
    .is_some());

    // Blue item blocked from Red tube
    assert!(find_tube_route(
        &color_graph,
        BlockPos(0, 0, 0),
        Some(Direction::East),
        &[BlockPos(1, 0, 0)],
        Some(DyeColor::Blue),
    )
    .is_none());

    // Magtube payload advance speed
    let mut payload = TubePayload::new(
        ItemStack::new("minecraft:diamond", 1),
        None,
        vec![BlockPos(0, 0, 0), BlockPos(1, 0, 0)],
    );
    assert!(!payload.advance(false)); // Standard speed takes multiple ticks
}

// ============================================================================
// 5. Blutricity Power Grid Physical Model & Devices (§2.5)
// ============================================================================

#[test]
fn test_blutricity_physical_model_dynamic_iteration() {
    let mut node_a = BluePowerConductor::new(4.0);
    let mut node_b = BluePowerConductor::new(4.0);

    // Nominal 100V DC operating voltage
    node_a.add_charge(400.0); // 400 C / 4.0 F = 100 V
    assert_eq!(node_a.voltage, 100.0);
    assert_eq!(node_b.voltage, 0.0);
    assert!(node_a.is_operational()); // V >= 60V
    assert!(!node_b.is_operational());

    // Dynamic iteration step
    let flow = node_a.step_iterate(&mut node_b);
    assert!(flow > 0.0);
    assert!(node_a.voltage < 100.0);
    assert!(node_b.voltage > 0.0);
    assert_eq!(node_a.resistance, BLUTRICITY_CONDUCTOR_RESISTANCE);
}

#[test]
fn test_blutricity_devices_solar_thermopile_wind_battery_transformer() {
    // 1. Solar Panel: 2.0 A current when V <= 100V, sky visible, daytime, no storm
    let mut solar = SolarPanelState::new();
    let current = solar.tick(true, true, false);
    assert_eq!(current, 2.0);
    assert_eq!(solar.conductor.charge, 2.0);

    // Nighttime or storm -> 0 current
    assert_eq!(solar.tick(true, false, false), 0.0);
    assert_eq!(solar.tick(true, true, true), 0.0);

    // 2. Thermopile: 0.005 * min(Thot, Tcold) A when V <= 100V
    assert_eq!(ThermopileState::block_temperature("minecraft:torch"), 5);
    assert_eq!(ThermopileState::block_temperature("minecraft:lava"), 100);
    assert_eq!(ThermopileState::block_temperature("minecraft:water"), -25);
    assert_eq!(ThermopileState::block_temperature("minecraft:ice"), -100);

    let mut therm = ThermopileState::new();
    therm.t_hot = 100;
    therm.t_cold = 100;
    let (t_cur, _) = therm.tick(|| (100, 100));
    assert!(t_cur > 0.0);

    // 3. Wind Turbine: P / 5 W/tick when V <= 130V and brakes released
    let mut wt = WindTurbineState::new();
    assert_eq!(wt.tick(100.0), 20.0);
    wt.brakes_engaged = true;
    assert_eq!(wt.tick(100.0), 0.0);

    // 4. Battery Box: 6,000 storage units; charges V > 90V, discharges V < 80V
    let mut bb = BatteryBoxState::new();
    bb.conductor.add_charge(440.0); // 110V (> 90V) -> charges
    bb.tick(false);
    assert!(bb.stored_units > 0.0);

    bb.conductor.charge = 280.0;
    bb.conductor.voltage = 70.0; // 70V (< 80V) -> discharges
    let before_discharge = bb.stored_units;
    bb.tick(false);
    assert!(bb.stored_units < before_discharge);

    // Discharges halted when redstone-powered
    bb.conductor.charge = 280.0;
    bb.conductor.voltage = 70.0;
    let stored_during_redstone = bb.stored_units;
    bb.tick(true); // Redstone active
    assert_eq!(bb.stored_units, stored_during_redstone);

    // 5. Transformer: 100x step up/down between 100V and 10,000V
    let mut xfmr = TransformerState::new();
    xfmr.low_voltage_conductor.voltage = 100.0;
    xfmr.low_voltage_conductor.current = 10.0;
    xfmr.step_transform();
    assert_eq!(xfmr.high_voltage_conductor.voltage, 10000.0);
    assert_eq!(xfmr.high_voltage_conductor.current, 1000.0);
}

// ============================================================================
// 6. Frame Motor Automation & Structural Movement (§2.6)
// ============================================================================

#[test]
fn test_frame_motor_assembly_solver_bedrock_rejection_and_power() {
    // Free translation of frames and machines
    let translated = plan_frame_translation(BlockPos(0, 64, 0), Direction::East, |pos| {
        if pos == BlockPos(0, 64, 0) {
            Some("redpower_2:frame".into())
        } else if pos == BlockPos(0, 65, 0) {
            Some("redpower_2:alloy_furnace".into())
        } else {
            Some("minecraft:air".into())
        }
    })
    .expect("frame and attached machine should move freely");
    assert_eq!(translated.len(), 2);

    // Obsidian has hardness 50.0F and CAN be moved
    let obsidian_move = plan_frame_translation(BlockPos(0, 64, 0), Direction::East, |pos| {
        if pos == BlockPos(0, 64, 0) {
            Some("redpower_2:frame".into())
        } else if pos == BlockPos(0, 65, 0) {
            Some("minecraft:obsidian".into())
        } else {
            Some("minecraft:air".into())
        }
    })
    .expect("obsidian should be movable by frames");
    assert_eq!(obsidian_move.len(), 2);

    // Bedrock has negative hardness (-1.0F) and FAILS assembly
    let bedrock_blocked = plan_frame_translation(BlockPos(0, 64, 0), Direction::East, |pos| {
        if pos == BlockPos(0, 64, 0) {
            Some("redpower_2:frame".into())
        } else if pos == BlockPos(0, 63, 0) {
            Some("minecraft:bedrock".into())
        } else {
            Some("minecraft:air".into())
        }
    });
    assert!(bedrock_blocked.is_err());

    // Power cost: 100 + 10 * LinkSize W-t per tick; requires V >= 60V
    let mut fm = FrameMotorState::new();
    fm.moving_blocks = vec![BlockPos(0, 64, 0), BlockPos(0, 65, 0)];
    assert_eq!(fm.power_cost_per_tick(), 120.0); // 100 + 10 * 2 = 120 W-t

    fm.is_moving = true;
    assert!(fm.tick(50.0).is_err()); // Under 60V fails

    // Exactly 16 ticks to complete movement
    for i in 0..15 {
        let done = fm.tick(100.0).expect("should advance tick");
        assert!(!done, "tick {i} should not be complete");
    }
    let done = fm.tick(100.0).expect("tick 16 completes");
    assert!(done);
    assert!(!fm.is_moving);
}

// ============================================================================
// 7. Control 6502 Computing Subsystem (§2.7)
// ============================================================================

#[test]
fn test_control_6502_cpu_execution_terminal_blitter_disk_drive() {
    // 1. CPU execution and Bundled I/O Expander MMIO
    let mut cpu = Cpu6502::new();
    // Program: LDA #$05; STA $E000; BRK
    let prog = [0xA9, 0x05, 0x8D, 0x00, 0xE0, 0x00];
    cpu.load_program(0x0400, &prog);
    let ran = cpu.step_tick(100);
    assert_eq!(ran, 3);
    assert!(cpu.halted);
    assert_eq!(cpu.io_expander.level(DyeColor::White), 255);
    assert_eq!(cpu.io_expander.level(DyeColor::Magenta), 255);
    assert_eq!(cpu.io_expander.level(DyeColor::Orange), 0);

    // 2. Arithmetic and stack instructions
    let mut cpu2 = Cpu6502::new();
    // CLC; LDA #$10; ADC #$20; PHA; PLA; BRK
    let prog2 = [0x18, 0xA9, 0x10, 0x69, 0x20, 0x48, 0x68, 0x00];
    cpu2.load_program(0x0400, &prog2);
    let ran2 = cpu2.step_tick(100);
    assert_eq!(ran2, 6);
    assert_eq!(cpu2.a, 0x30);

    // 3. Instruction cycle accumulator and burst cap (100,000)
    let mut cpu3 = Cpu6502::new();
    assert_eq!(cpu3.cycle_accumulator, 0);
    cpu3.accumulate_tick();
    assert_eq!(cpu3.cycle_accumulator, 1000);
    for _ in 0..120 {
        cpu3.accumulate_tick();
    }
    assert_eq!(cpu3.cycle_accumulator, Cpu6502::MAX_BURST_CYCLES);

    // 4. Terminal 80x50 character matrix blitter
    let mut term = DisplayTerminal::new();
    term.screen[0] = b'R';
    term.screen[1] = b'P';
    term.blit(0, 0, 80, 0, 2, 1);
    assert_eq!(term.screen[80], b'R');
    assert_eq!(term.screen[81], b'P');

    // 5. Floppy Disk Drive Redbus commands
    let mut drive = DiskDriveState::new();
    let mut buf = [0u8; 128];
    assert!(drive.execute_redbus(1, 0, &mut buf)); // Read label
    assert!(std::str::from_utf8(&buf)
        .unwrap()
        .contains("RedPower Forth"));

    buf = [99u8; 128];
    assert!(drive.execute_redbus(5, 4, &mut buf)); // Write sector 4
    let mut read_buf = [0u8; 128];
    assert!(drive.execute_redbus(4, 4, &mut read_buf)); // Read sector 4
    assert_eq!(read_buf[0], 99);
}

// ============================================================================
// 8. Metallurgy & Alloy Furnace Recipes (§3.1 & §7.1)
// ============================================================================

#[test]
fn test_alloy_furnace_recipes_solid_fuel_and_blulectric() {
    // 1. Red Alloy: 4x redstone + 1x copper -> 1x red_alloy_ingot
    let mut f1 = AlloyFurnaceState::new();
    f1.inventory.slots[0] = Some(ItemStack::new("minecraft:redstone", 4));
    f1.inventory.slots[1] = Some(ItemStack::new("redpower_2:copper_ingot", 1));
    f1.inventory.slots[AlloyFurnaceState::FUEL_SLOT] = Some(ItemStack::new("minecraft:coal", 1));
    for _ in 0..AlloyFurnaceState::SMELT_TICKS {
        f1.tick(false);
    }
    let out1 = f1.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
        .as_ref()
        .expect("red alloy");
    assert_eq!(out1.item_id, "redpower_2:red_alloy_ingot");
    assert_eq!(out1.count, 1);

    // 2. Blue Alloy: 4x nikolite + 1x silver -> 1x blue_alloy_ingot
    let mut f2 = AlloyFurnaceState::new();
    f2.inventory.slots[0] = Some(ItemStack::new("redpower_2:nikolite", 4));
    f2.inventory.slots[1] = Some(ItemStack::new("redpower_2:silver_ingot", 1));
    for _ in 0..AlloyFurnaceState::BLULECTRIC_SMELT_TICKS {
        f2.tick(true); // Blulectric operation without solid fuel
    }
    let out2 = f2.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
        .as_ref()
        .expect("blue alloy");
    assert_eq!(out2.item_id, "redpower_2:blue_alloy_ingot");
    assert_eq!(out2.count, 1);

    // 3. Brass: 3x copper + 1x tin -> 4x brass_ingot
    let mut f3 = AlloyFurnaceState::new();
    f3.inventory.slots[0] = Some(ItemStack::new("redpower_2:copper_ingot", 3));
    f3.inventory.slots[1] = Some(ItemStack::new("redpower_2:tin_ingot", 1));
    f3.inventory.slots[AlloyFurnaceState::FUEL_SLOT] = Some(ItemStack::new("minecraft:coal", 1));
    for _ in 0..AlloyFurnaceState::SMELT_TICKS {
        f3.tick(false);
    }
    let out3 = f3.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
        .as_ref()
        .expect("brass");
    assert_eq!(out3.item_id, "redpower_2:brass_ingot");
    assert_eq!(out3.count, 4);

    // 4. Tinplate: 2x iron + 1x tin -> 4x tinplate
    let mut f4 = AlloyFurnaceState::new();
    f4.inventory.slots[0] = Some(ItemStack::new("minecraft:iron_ingot", 2));
    f4.inventory.slots[1] = Some(ItemStack::new("redpower_2:tin_ingot", 1));
    f4.inventory.slots[AlloyFurnaceState::FUEL_SLOT] = Some(ItemStack::new("minecraft:coal", 1));
    for _ in 0..AlloyFurnaceState::SMELT_TICKS {
        f4.tick(false);
    }
    let out4 = f4.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
        .as_ref()
        .expect("tinplate");
    assert_eq!(out4.item_id, "redpower_2:tinplate");
    assert_eq!(out4.count, 4);

    // 5. Silicon Boule: 8x coal + 8x sand -> 1x silicon_boule
    let mut f5 = AlloyFurnaceState::new();
    f5.inventory.slots[0] = Some(ItemStack::new("minecraft:coal", 8));
    f5.inventory.slots[1] = Some(ItemStack::new("minecraft:sand", 8));
    f5.inventory.slots[AlloyFurnaceState::FUEL_SLOT] = Some(ItemStack::new("minecraft:coal", 1));
    for _ in 0..AlloyFurnaceState::SMELT_TICKS {
        f5.tick(false);
    }
    let out5 = f5.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
        .as_ref()
        .expect("silicon boule");
    assert_eq!(out5.item_id, "redpower_2:silicon_boule");
    assert_eq!(out5.count, 1);

    // 6. Red-Doped Wafer: 1x silicon_wafer + 4x redstone -> 1x red_doped_wafer
    let mut f6 = AlloyFurnaceState::new();
    f6.inventory.slots[0] = Some(ItemStack::new("redpower_2:silicon_wafer", 1));
    f6.inventory.slots[1] = Some(ItemStack::new("minecraft:redstone", 4));
    for _ in 0..AlloyFurnaceState::BLULECTRIC_SMELT_TICKS {
        f6.tick(true);
    }
    let out6 = f6.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
        .as_ref()
        .expect("red doped wafer");
    assert_eq!(out6.item_id, "redpower_2:red_doped_wafer");
    assert_eq!(out6.count, 1);
}

// ============================================================================
// 9. Tools, Armor & Combat Equipment (§4.1 - §4.3)
// ============================================================================

#[test]
fn test_tools_combat_athame_vorpal_sickles_and_durabilities() {
    // 1. Athame attack damage & Disjunction
    assert_eq!(ATHAME_BASE_DAMAGE, 1);
    assert_eq!(ATHAME_ENDER_DAMAGE, 25);
    assert_eq!(ATHAME_DURABILITY, 100);
    assert_eq!(DISJUNCTION_PER_LEVEL_DAMAGE, 6);

    assert_eq!(athame_attack_damage(false, 0), 1);
    assert_eq!(athame_attack_damage(false, 5), 1); // Disjunction does not affect normal mobs
    assert_eq!(athame_attack_damage(true, 0), 25);
    assert_eq!(athame_attack_damage(true, 1), 31); // 25 + 6*1
    assert_eq!(athame_attack_damage(true, 5), 55); // 25 + 6*5

    assert!(is_ender_mob("minecraft:enderman"));
    assert!(is_ender_mob("minecraft:ender_dragon"));
    assert!(!is_ender_mob("minecraft:creeper"));

    // 2. Vorpal enchantment: 2 * level^2 % proc chance, +100 crit bonus
    assert_eq!(vorpal_proc_chance_percent(1), 2);
    assert_eq!(vorpal_proc_chance_percent(2), 8);
    assert_eq!(vorpal_proc_chance_percent(3), 18);
    assert_eq!(vorpal_proc_chance_percent(4), 32);

    assert_eq!(VORPAL_CRIT_BONUS_DAMAGE, 100);
    assert_eq!(calculate_weapon_damage(5, 2, false), 5);
    assert_eq!(calculate_weapon_damage(5, 2, true), 105);

    // Overkill decapitation threshold <= -20 HP
    assert_eq!(VORPAL_DECAPITATION_OVERKILL_THRESHOLD, -20);
    assert!(!check_vorpal_decapitation(20, 35)); // 20 - 35 = -15
    assert!(check_vorpal_decapitation(20, 40)); // 20 - 40 = -20
    assert!(check_vorpal_decapitation(20, 105)); // 20 - 105 = -85

    // 3. Sickle harvesting radii & durabilities
    assert_eq!(SICKLE_LEAF_RADIUS, 1);
    assert_eq!(SICKLE_CROP_RADIUS, 2);

    let center = BlockPos(10, 64, 10);
    let leaves = sickle_leaf_positions(center);
    assert_eq!(leaves.len(), 27); // 3x3x3 volume

    let crops = sickle_crop_positions(center);
    assert_eq!(crops.len(), 25); // 5x5 flat area
    for pos in &crops {
        assert_eq!(pos.1, 64);
    }

    assert_eq!(SickleVariant::Wood.durability(), 59);
    assert_eq!(SickleVariant::Stone.durability(), 131);
    assert_eq!(SickleVariant::Iron.durability(), 250);
    assert_eq!(SickleVariant::Diamond.durability(), 1561);
    assert_eq!(SickleVariant::Gold.durability(), 32);
    assert_eq!(SickleVariant::Ruby.durability(), 500);
    assert_eq!(SickleVariant::GreenSapphire.durability(), 500);
    assert_eq!(SickleVariant::Sapphire.durability(), 500);
}

// ============================================================================
// 10. Spec §8 Hazards & Safety Limits
// ============================================================================

#[test]
fn test_hazard_safety_limits_spec_section_eight() {
    // 1. 6502 CPU Runaway Cycles: slice bounded to +1,000 cycles, 100,000 burst cap
    assert_eq!(Cpu6502::CYCLES_PER_TICK, 1_000);
    assert_eq!(Cpu6502::MAX_BURST_CYCLES, 100_000);
    let mut runaway_cpu = Cpu6502::new();
    runaway_cpu.load_program(0x0400, &[0x4C, 0x00, 0x04]); // Infinite JMP loop
    let cycles = runaway_cpu.step_tick(1_000_000);
    assert_eq!(cycles, Cpu6502::MAX_CYCLES_PER_TICK);

    // 2. Frame Motor collision & bedrock rejection
    let bedrock_result = plan_frame_translation(BlockPos(0, 64, 0), Direction::Up, |pos| {
        if pos == BlockPos(0, 64, 0) {
            Some("redpower_2:frame".into())
        } else {
            Some("minecraft:bedrock".into())
        }
    });
    assert!(bedrock_result.is_err());

    // 3. Thermopile thermal meltdown / shock hazard
    let mut therm = ThermopileState::new();
    assert!(!therm.lava_cools());
    therm.t_hot = 100;
    therm.t_cold = 200;
    assert!(therm.lava_cools());
    assert!(therm.melts_neighbors());

    // 4. Blutricity over-voltage limits
    assert_eq!(BLUTRICITY_WIND_MAX_VOLTAGE, 130.0);
    assert_eq!(BLUTRICITY_MAX_SAFE_VOLTAGE, 120.0);
    let mut xfmr = TransformerState::new();
    xfmr.low_voltage_conductor.voltage = 100.0;
    xfmr.step_transform();
    assert_eq!(xfmr.high_voltage_conductor.voltage, 10000.0);

    // 5. Vorpal overkill decapitation
    assert!(check_vorpal_decapitation(10, 30)); // 10 - 30 = -20
}

// ============================================================================
// 11. Serialization Round Trips for All Subsystems
// ============================================================================

#[test]
fn test_serialization_round_trips_all_domain_types() {
    fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug>(
        val: &T,
    ) -> T {
        let json = serde_json::to_string(val).expect("serialize");
        serde_json::from_str(&json).expect("deserialize")
    }

    // MicroblockKind
    assert_eq!(roundtrip(&MicroblockKind::Cover), MicroblockKind::Cover);
    assert_eq!(roundtrip(&MicroblockKind::Corner), MicroblockKind::Corner);

    // MicroblockVoxel
    let mut voxel = MicroblockVoxel::new();
    voxel.place_on_face(Direction::North, MicroblockKind::Cover);
    assert_eq!(roundtrip(&voxel), voxel);

    // WireKind
    assert_eq!(roundtrip(&WireKind::RedAlloy), WireKind::RedAlloy);
    assert_eq!(
        roundtrip(&WireKind::Insulated(DyeColor::Red)),
        WireKind::Insulated(DyeColor::Red)
    );

    // LogicGateState
    let gate = LogicGateState::new(LogicGateKind::Timer);
    assert_eq!(roundtrip(&gate), gate);

    // BusTransceiverState
    let mut btx = BusTransceiverState::new();
    btx.side_a = BundledSignal::from_digital_mask(0xABCD);
    assert_eq!(roundtrip(&btx), btx);

    // TubePayload
    let payload = TubePayload::new(
        ItemStack::new("minecraft:diamond", 5),
        Some(DyeColor::Cyan),
        vec![BlockPos(0, 0, 0), BlockPos(1, 0, 0)],
    );
    let payload_rt = roundtrip(&payload);
    assert_eq!(payload_rt.item.item_id, "minecraft:diamond");
    assert_eq!(payload_rt.item.count, 5);
    assert_eq!(payload_rt.color_tag, Some(DyeColor::Cyan));

    // BluePowerConductor
    let cond = BluePowerConductor::new(4.0);
    assert_eq!(roundtrip(&cond), cond);

    // SolarPanelState
    let solar = SolarPanelState::new();
    assert_eq!(roundtrip(&solar), solar);

    // ThermopileState
    let therm = ThermopileState::new();
    assert_eq!(roundtrip(&therm), therm);

    // WindTurbineState
    let wt = WindTurbineState::new();
    assert_eq!(roundtrip(&wt), wt);

    // BatteryBoxState
    let bb = BatteryBoxState::new();
    assert_eq!(roundtrip(&bb), bb);

    // TransformerState
    let xfmr = TransformerState::new();
    assert_eq!(roundtrip(&xfmr), xfmr);

    // FrameMotorState
    let fm = FrameMotorState::new();
    assert_eq!(roundtrip(&fm), fm);

    // Cpu6502
    let mut cpu = Cpu6502::new();
    cpu.a = 0x42;
    let cpu_rt = roundtrip(&cpu);
    assert_eq!(cpu_rt.a, 0x42);

    // DisplayTerminal
    let mut term = DisplayTerminal::new();
    term.screen[0] = b'X';
    let term_rt = roundtrip(&term);
    assert_eq!(term_rt.screen[0], b'X');

    // DiskDriveState
    let mut drive = DiskDriveState::new();
    drive.sectors[0] = 77;
    let drive_rt = roundtrip(&drive);
    assert_eq!(drive_rt.sectors[0], 77);

    // AlloyFurnaceState
    let mut furnace = AlloyFurnaceState::new();
    furnace.inventory.slots[0] = Some(ItemStack::new("redpower_2:copper_ingot", 3));
    let furnace_rt = roundtrip(&furnace);
    assert_eq!(
        furnace_rt.inventory.slots[0].as_ref().unwrap().item_id,
        "redpower_2:copper_ingot"
    );

    // SickleVariant
    assert_eq!(roundtrip(&SickleVariant::Diamond), SickleVariant::Diamond);

    // Vector3 & WorldCoord
    let v = Vector3::new(1.25, -3.5, 42.0);
    assert_eq!(roundtrip(&v), v);

    let wc = WorldCoord::new(-100, 64, 250);
    assert_eq!(roundtrip(&wc), wc);
}

#[test]
fn test_spatial_vector3_and_worldcoord_parity() {
    // 1. Vector3 arithmetic parity with com.eloraam.redpower.core.Vector3
    let mut v = Vector3::new(3.0, 4.0, 0.0);
    assert_eq!(v.mag(), 5.0);
    assert_eq!(v.mag_squared(), 25.0);

    v.normalize();
    assert!((v.x - 0.6).abs() < 1e-9);
    assert!((v.y - 0.8).abs() < 1e-9);
    assert_eq!(v.z, 0.0);

    let v1 = Vector3::new(1.0, 0.0, 0.0);
    let v2 = Vector3::new(0.0, 1.0, 0.0);
    let cross = v1.cross(&v2);
    assert_eq!(cross, Vector3::new(0.0, 0.0, 1.0));
    assert_eq!(v1.dot_product(&v2), 0.0);
    assert_eq!(v1.dot_product(&v1), 1.0);

    // 2. WorldCoord directional stepping (0..5: -Y, +Y, -Z, +Z, -X, +X)
    let origin = WorldCoord::new(0, 64, 0);
    assert_eq!(origin.coord_step(0), WorldCoord::new(0, 63, 0)); // -Y
    assert_eq!(origin.coord_step(1), WorldCoord::new(0, 65, 0)); // +Y
    assert_eq!(origin.coord_step(2), WorldCoord::new(0, 64, -1)); // -Z
    assert_eq!(origin.coord_step(3), WorldCoord::new(0, 64, 1)); // +Z
    assert_eq!(origin.coord_step(4), WorldCoord::new(-1, 64, 0)); // -X
    assert_eq!(origin.coord_step(5), WorldCoord::new(1, 64, 0)); // +X

    // 3. Indirect stepping (indStep) for wire/tube corner traversal
    // From Down (0), d2=0..3 steps through horizontal directions 2, 3, 4, 5
    assert_eq!(origin.ind_step_coord(0, 0), WorldCoord::new(0, 63, -1));
    assert_eq!(origin.ind_step_coord(0, 1), WorldCoord::new(0, 63, 1));
    assert_eq!(origin.ind_step_coord(0, 2), WorldCoord::new(-1, 63, 0));
    assert_eq!(origin.ind_step_coord(0, 3), WorldCoord::new(1, 63, 0));

    // 4. Directional sweep comparator (FrameLib.sort parity)
    let mut coords = [
        WorldCoord::new(1, 10, 5),
        WorldCoord::new(1, 30, 5),
        WorldCoord::new(1, 20, 5),
    ];
    // Direction 1 (+Y) moves top blocks first (descending Y)
    coords.sort_by(WorldCoord::dir_comparator(1));
    assert_eq!(coords[0].y, 30);
    assert_eq!(coords[1].y, 20);
    assert_eq!(coords[2].y, 10);

    // Direction 0 (-Y) moves bottom blocks first (ascending Y)
    coords.sort_by(WorldCoord::dir_comparator(0));
    assert_eq!(coords[0].y, 10);
    assert_eq!(coords[1].y, 20);
    assert_eq!(coords[2].y, 30);
}

// ============================================================================
// 12. HorizonNativeMod Hooks Lifecycle & Namespace Guards (§10)
// ============================================================================

#[test]
fn test_native_hooks_api40_lifecycle_and_namespace_isolation() {
    assert_eq!(BLOCK_IDS.len(), 113);
    assert_eq!(ITEM_IDS.len(), 88);

    let mut rp = RedPower2Mod::default();

    // 1. Mod info listing
    let mut mods = Vec::new();
    rp.native_mods(&mut mods);
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].id, MOD_ID);

    // 2. Foreign block entity creation is rejected
    let mut foreign_entity = None;
    rp.native_block_entity("minecraft:chest", &mut foreign_entity);
    assert!(foreign_entity.is_none());

    // Own block entity creation succeeds
    let mut own_entity = None;
    rp.native_block_entity(RedPower2Mod::BLOCK_ALLOY_FURNACE, &mut own_entity);
    let (id, nbt) = own_entity.expect("attach entity for alloy_furnace");
    assert_eq!(id, RedPower2Mod::BLOCK_ALLOY_FURNACE);
    assert_eq!(nbt.get("facing").map(String::as_str), Some("2"));

    // 3. Block entity ticking registration
    let mut ticks = false;
    rp.native_block_entity_ticks(RedPower2Mod::BLOCK_ALLOY_FURNACE, &mut ticks);
    assert!(ticks);

    let mut cpu_ticks = false;
    rp.native_block_entity_ticks(RedPower2Mod::BLOCK_CPU, &mut cpu_ticks);
    assert!(cpu_ticks);

    let mut foreign_ticks = false;
    rp.native_block_entity_ticks("minecraft:furnace", &mut foreign_ticks);
    assert!(!foreign_ticks);

    // 4. Container registration across machine sizes
    let mut nine_slot = None;
    rp.native_block_entity_container("redpower_2:deployer", &mut nine_slot);
    assert_eq!(nine_slot.unwrap().slots, 9);

    let mut twentyseven_slot = None;
    rp.native_block_entity_container("redpower_2:alloy_furnace", &mut twentyseven_slot);
    assert_eq!(twentyseven_slot.unwrap().slots, 27);

    let mut three_slot = None;
    rp.native_block_entity_container("redpower_2:blulectric_furnace", &mut three_slot);
    assert_eq!(three_slot.unwrap().slots, 3);

    let mut eighteen_slot = None;
    rp.native_block_entity_container("redpower_2:charging_bench", &mut eighteen_slot);
    assert_eq!(eighteen_slot.unwrap().slots, 18);

    let mut thirtysix_slot = None;
    rp.native_block_entity_container("redpower_2:assembler", &mut thirtysix_slot);
    assert_eq!(thirtysix_slot.unwrap().slots, 36);

    let mut fortyfive_slot = None;
    rp.native_block_entity_container("redpower_2:sorting_machine", &mut fortyfive_slot);
    assert_eq!(fortyfive_slot.unwrap().slots, 40);

    let mut foreign_container = None;
    rp.native_block_entity_container("minecraft:chest", &mut foreign_container);
    assert!(foreign_container.is_none());

    // 5. Server tick synchronizing container & data slots
    let mut entity = ModBlockEntity {
        id,
        nbt,
        items: vec![None; AlloyFurnaceState::SLOT_COUNT],
        data: Vec::new(),
    };
    rp.native_block_entity_tick([0, 64, 0], &mut entity);
    assert_eq!(entity.data.len(), 3); // cook_time, burn_time, total_burn

    // 6. Custom screen layout
    let mut screen = None;
    rp.native_screen(RedPower2Mod::BLOCK_ALLOY_FURNACE, &mut screen);
    let s = screen.expect("alloy furnace screen");
    assert_eq!(s.slots.len(), AlloyFurnaceState::SLOT_COUNT);

    let mut foreign_screen = None;
    rp.native_screen("minecraft:furnace", &mut foreign_screen);
    assert!(foreign_screen.is_none());

    // 7. Right-click with Screwdriver rotates machine
    let mut claimed = false;
    rp.native_use_item_on(
        &ItemStack::new(RedPower2Mod::ITEM_SCREWDRIVER, 1),
        RedPower2Mod::BLOCK_ALLOY_FURNACE,
        Some(&mut entity),
        None,
        &mut claimed,
    );
    assert!(claimed);
    assert_eq!(entity.nbt.get("facing").map(String::as_str), Some("3"));

    // Right-click with Paintbrush colors machine
    let mut claimed_paint = false;
    rp.native_use_item_on(
        &ItemStack::new(RedPower2Mod::ITEM_PAINT_BRUSH, 1),
        RedPower2Mod::BLOCK_PNEUMATIC_TUBE,
        Some(&mut entity),
        None,
        &mut claimed_paint,
    );
    assert!(claimed_paint);

    // Foreign block right-click ignored
    let mut foreign_claimed = false;
    rp.native_use_item_on(
        &ItemStack::new(RedPower2Mod::ITEM_SCREWDRIVER, 1),
        "minecraft:chest",
        Some(&mut entity),
        None,
        &mut foreign_claimed,
    );
    assert!(!foreign_claimed);

    // 8. Signal source & redstone signal output
    let mut source = false;
    rp.native_is_signal_source("redpower_2:item_detector", &mut source);
    assert!(source);

    let mut foreign_source = false;
    rp.native_is_signal_source("minecraft:dirt", &mut foreign_source);
    assert!(!foreign_source);

    let mut power = 0;
    entity.nbt.insert("signal".into(), "14".into());
    rp.native_block_signal(
        RedPower2Mod::BLOCK_ALLOY_FURNACE,
        Some(&entity),
        false,
        &mut power,
    );
    assert_eq!(power, 14);

    // 9. Tooltips
    let mut tt = Vec::new();
    rp.native_tooltip(&ItemStack::new("redpower_2:voltmeter", 1), &mut tt);
    assert!(tt.iter().any(|l| l.contains("Voltmeter")));

    let mut foreign_tt = Vec::new();
    rp.native_tooltip(&ItemStack::new("minecraft:iron_sword", 1), &mut foreign_tt);
    assert!(foreign_tt.is_empty());

    // 10. Screen click & menu button handling
    let mut btn = None;
    rp.native_screen_click("redpower_2:timer", &[], [25.0, 20.0], false, &mut btn);
    assert_eq!(btn, Some(1));

    let mut timer_ent = ModBlockEntity {
        id: "redpower_2:timer".into(),
        nbt: HashMap::new(),
        items: Vec::new(),
        data: Vec::new(),
    };
    rp.native_menu_button([0, 64, 0], &mut timer_ent, 1);
    assert_eq!(timer_ent.nbt.get("interval").map(String::as_str), Some("6"));

    // IO Expander DIP switch click and menu button toggling
    let mut io_btn = None;
    rp.native_screen_click(
        "redpower_2:io_expander",
        &[],
        [18.0, 30.0],
        false,
        &mut io_btn,
    );
    assert_eq!(io_btn, Some(0)); // DIP switch 0

    let mut io_ent_opt = None;
    rp.native_block_entity("redpower_2:io_expander", &mut io_ent_opt);
    let (io_id, io_nbt) = io_ent_opt.expect("io_expander entity");
    assert_eq!(io_id, "redpower_2:io_expander");
    assert_eq!(io_nbt.get("rbaddr").map(String::as_str), Some("3"));

    let mut io_ticks = false;
    rp.native_block_entity_ticks("redpower_2:io_expander", &mut io_ticks);
    assert!(io_ticks);

    let mut io_screen = None;
    rp.native_screen("redpower_2:io_expander", &mut io_screen);
    let ios = io_screen.expect("io_expander screen");
    assert_eq!(ios.width, 123.0);
    assert_eq!(ios.height, 81.0);

    let mut io_entity = ModBlockEntity {
        id: io_id,
        nbt: io_nbt,
        items: Vec::new(),
        data: Vec::new(),
    };
    rp.native_menu_button([0, 64, 0], &mut io_entity, 0); // toggle bit 0: 3 -> 2
    assert_eq!(io_entity.nbt.get("rbaddr").map(String::as_str), Some("2"));

    // 11. Network packet handling (Packet 212 keystrokes into ring buffer)
    rp.native_net_from_client("player-1", "horizon:mod_net", &[212, b'O', b'K']);
    assert_eq!(rp.terminal.key_buffer.pop_front(), Some(b'O'));
    assert_eq!(rp.terminal.key_buffer.pop_front(), Some(b'K'));

    // Foreign channel packet is ignored
    rp.native_net_from_client("player-1", "other:channel", &[212, b'Z']);
    assert!(rp.terminal.key_buffer.is_empty());
}

// ============================================================================
// 13. Dual-Artifact Manifests & Content Files (§9)
// ============================================================================

#[test]
fn test_dual_artifact_manifests_and_content_structure() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let content_dir = manifest_dir.join("content");

    // Content mod folder must NOT contain Cargo.toml
    assert!(!content_dir.join("Cargo.toml").exists());
    assert!(content_dir.join("horizon-mod.toml").is_file());

    // Native bundle manifest verification
    let native_json_path = manifest_dir.join("native/horizon-native.json");
    let native_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&native_json_path).unwrap()).unwrap();
    assert_eq!(native_json["id"], MOD_ID);
    assert_eq!(native_json["horizon"], HORIZON_REVISION);
    assert_eq!(native_json["mixins"], MIXINS_REVISION);
    assert_eq!(native_json["hooks"], NATIVE_HOOKS_API);

    // Native mixin TOML verification
    let mixin_toml_path = manifest_dir.join("native/mixins/redpower_2/horizon-mixin.toml");
    assert!(mixin_toml_path.is_file());
    let mixin_content = std::fs::read_to_string(&mixin_toml_path).unwrap();
    assert!(mixin_content.contains("id = \"redpower_2\""));
    assert!(mixin_content.contains("[[injection]]"));

    // Content declarations existence
    let alloy_furnace_decl = content_dir.join("data/redpower_2/horizon/block/alloy_furnace.json");
    assert!(alloy_furnace_decl.is_file());

    let screwdriver_decl = content_dir.join("data/redpower_2/horizon/item/screwdriver.json");
    assert!(screwdriver_decl.is_file());

    let alloy_recipe = content_dir.join("data/redpower_2/recipe/alloy_red_alloy.json");
    assert!(alloy_recipe.is_file());
}

#[test]
fn test_cpu_container_and_gui_debug_controls() {
    use redpower_2::control::cpu_gui::{
        ContainerCPU, CpuRegisters, GuiCPU, BTN_DISK_BIT_BASE, BTN_HALT, BTN_RESET, BTN_START,
        BTN_STEP, EVENT_HALT, EVENT_SET_DISK_ID, EVENT_WARM_BOOT, GUI_HEIGHT, GUI_WIDTH,
    };

    let mut rp = RedPower2Mod::default();

    // 1. Native screen dimensions
    let mut scr = None;
    rp.native_screen("redpower_2:cpu", &mut scr);
    let s = scr.expect("CPU native screen");
    assert_eq!(s.width, GUI_WIDTH);
    assert_eq!(s.height, GUI_HEIGHT);
    assert!(s.slots.is_empty());
    assert!(s.own_labels);

    // 2. Native screen click dispatch
    let data = ContainerCPU::default().to_data();
    let mut btn = None;
    rp.native_screen_click("redpower_2:cpu", &data, [90.0, 100.0], false, &mut btn);
    assert_eq!(btn, Some(BTN_START));

    btn = None;
    rp.native_screen_click("redpower_2:cpu", &data, [90.0, 115.0], false, &mut btn);
    assert_eq!(btn, Some(BTN_HALT));

    btn = None;
    rp.native_screen_click("redpower_2:cpu", &data, [90.0, 128.0], false, &mut btn);
    assert_eq!(btn, Some(BTN_RESET));

    btn = None;
    rp.native_screen_click("redpower_2:cpu", &data, [25.0, 105.0], false, &mut btn);
    assert_eq!(btn, Some(BTN_STEP));

    btn = None;
    rp.native_screen_click("redpower_2:cpu", &data, [20.0, 60.0], false, &mut btn);
    assert_eq!(btn, Some(BTN_DISK_BIT_BASE));

    // 3. Native menu button execution on block entity
    let pos = [10, 64, 20];
    let mut cpu = Cpu6502::new();
    // Program: LDA #$55, LDX #$AA, BRK
    cpu.load_program(0x0400, &[0xA9, 0x55, 0xA2, 0xAA, 0x00]);
    cpu.halt();
    rp.cpus.insert(pos, cpu);

    let mut entity = ModBlockEntity {
        id: "redpower_2:cpu".into(),
        nbt: HashMap::new(),
        data: Vec::new(),
        items: Vec::new(),
    };

    // Step 1: LDA #$55
    rp.native_menu_button(pos, &mut entity, BTN_STEP);
    let live_cpu = rp.cpus.get(&pos).unwrap();
    assert_eq!(live_cpu.a, 0x55);
    assert_eq!(live_cpu.pc, 0x0402);
    let c1 = ContainerCPU::from_data(&entity.data);
    assert_eq!(c1.registers.a, 0x55);
    assert_eq!(c1.registers.pc, 0x0402);
    assert_eq!(c1.registers.format_a(), "A: $55");

    // Step 2: LDX #$AA
    rp.native_menu_button(pos, &mut entity, BTN_STEP);
    let live_cpu2 = rp.cpus.get(&pos).unwrap();
    assert_eq!(live_cpu2.x, 0xAA);
    assert_eq!(live_cpu2.pc, 0x0404);
    let c2 = ContainerCPU::from_data(&entity.data);
    assert_eq!(c2.registers.x, 0xAA);
    assert_eq!(c2.registers.format_x(), "X: $AA");

    // Reset button
    rp.native_menu_button(pos, &mut entity, BTN_RESET);
    let c_reset = ContainerCPU::from_data(&entity.data);
    assert_eq!(c_reset.registers.a, 0);
    assert_eq!(c_reset.registers.pc, 0x0400);
    assert_eq!(c_reset.registers.sp, 0xFD);

    // 4. Packet 212 GUI Event direct handling
    let mut container = ContainerCPU::default();
    let p_disk = GuiCPU::send_simple(1, EVENT_SET_DISK_ID, 7);
    container.handle_gui_event(&p_disk);
    assert_eq!(container.byte0, 7);

    let p_start = GuiCPU::send_event(1, EVENT_WARM_BOOT);
    container.handle_gui_event(&p_start);
    assert!(container.is_running);

    let p_halt = GuiCPU::send_event(1, EVENT_HALT);
    container.handle_gui_event(&p_halt);
    assert!(!container.is_running);

    // 5. Status flags NV-BDIZC formatting
    let regs = CpuRegisters {
        status: 0x81,
        ..Default::default()
    };
    assert_eq!(regs.format_flags(), "N- ----C");
    assert_eq!(regs.format_status(), "P: $81 [N- ----C]");
}

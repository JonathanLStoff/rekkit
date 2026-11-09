//! Comprehensive unit & integration tests for `rekkit_core` (`planning/mods/rekkit_core.md`).
//!
//! Covers:
//! 1. IC2 EU voltage tiers, over-voltage explosion threshold, and cable distance losses.
//! 2. BuildCraft / Thermal Expansion MJ buffers, thermal engine stages, and 5% conduit loss.
//! 3. RedPower 2 Blutricity ($V, I, Q$) Ohm's law charge transfer and operating voltage threshold.
//! 4. Applied Energistics (`AE1`) idle drain & power-off state, plus cross-mod `convert_energy` ratios.
//! 5. ItemStack merging, sided Inventory insertion/extraction, and comparator signal (`0..=15`).
//! 6. FluidTank millibucket fill/drain simulation and NBT persistence roundtrips.
//! 7. 6-face `Direction`, `ConnectionMask` bitmasks, and CoFH `SideConfig` -> `ModContainer.faces`.
//! 8. `UnionFind` connected components and RedPower 2 Dijkstra pneumatic tube routing (color tags + 500-weight restrictors).
//! 9. 16-color `DyeColor`, `BundledSignal` 16-bit masks, and 4,096-channel `FrequencyKey` roundtrips.
//! 10. Solid and hollow 3D `MultiblockShape` validation (Coke Oven 3x3x3 and Blast Furnace 3x3x4).
//! 11. `HorizonNativeMod` trait defaults, `TickTimer`, and deterministic `XorShift64` RNG.

use rekkit_core::energy::{
    convert_energy, AeEnergyBuffer, BlutricityNode, EnergyUnit, EngineHeatStage, EuCableKind,
    EuPacket, EuStorage, EuTier, MjBuffer, OverVoltageError,
};
use rekkit_core::fluid::{FluidStack, FluidTank};
use rekkit_core::frequency::{BundledSignal, ChannelOwner, DyeColor, FrequencyKey};
use rekkit_core::grid::{BlockPos, GridGraph, GridNode};
use rekkit_core::hooks::{HorizonNativeMod, NativeModInfo};
use rekkit_core::inventory::{Inventory, ItemStack};
use rekkit_core::multiblock::MultiblockShape;
use rekkit_core::side::{ConnectionMask, Direction, SideConfig, SideMode};
use rekkit_core::util::{TickTimer, XorShift64};
use rekkit_core::{HORIZON_REVISION, MIXINS_REVISION, NATIVE_HOOKS_API};
use std::collections::HashMap;

#[test]
fn test_ic2_eu_voltage_tiers_overvoltage_and_cable_loss() {
    assert_eq!(EuTier::LV.max_voltage(), 32);
    assert_eq!(EuTier::MV.max_voltage(), 128);
    assert_eq!(EuTier::HV.max_voltage(), 512);
    assert_eq!(EuTier::EV.max_voltage(), 2048);

    // Macerator (LV max 32 EU/p) receiving 32 EU packet succeeds
    let mut macerator = EuStorage::new(800.0, EuTier::LV);
    assert_eq!(macerator.receive_packet(EuPacket::new(32.0, 32)), Ok(32.0));
    assert_eq!(macerator.stored, 32.0);

    // Sending 128 EU MV packet into LV Macerator triggers OverVoltageError (explosion!)
    assert_eq!(
        macerator.receive_packet(EuPacket::new(128.0, 128)),
        Err(OverVoltageError {
            packet_voltage: 128,
            max_safe_voltage: 32,
        })
    );

    // Copper cable loses 0.20 EU/block (1 EU every 5 blocks)
    let pkt = EuPacket::new(32.0, 32);
    let delivered_copper = pkt.after_cable_loss(EuCableKind::Copper, 10).unwrap();
    assert!((delivered_copper - 30.0).abs() < 1e-6);

    // Glass Fibre cable loses 0.025 EU/block (1 EU every 40 blocks) and handles 512 EU/p
    let hv_pkt = EuPacket::new(512.0, 512);
    let delivered_glass = hv_pkt
        .after_cable_loss(EuCableKind::GlassFibre, 40)
        .unwrap();
    assert!((delivered_glass - 511.0).abs() < 1e-6);

    // Sending 512 EU packet into Copper cable melts it
    assert!(hv_pkt.after_cable_loss(EuCableKind::Copper, 1).is_err());
}

#[test]
fn test_mj_buffer_conduit_loss_and_engine_heat_stages() {
    assert_eq!(EngineHeatStage::from_ratio(0.10), EngineHeatStage::Blue);
    assert_eq!(EngineHeatStage::from_ratio(0.30), EngineHeatStage::Green);
    assert_eq!(EngineHeatStage::from_ratio(0.60), EngineHeatStage::Yellow);
    assert_eq!(EngineHeatStage::from_ratio(0.85), EngineHeatStage::Red);
    assert_eq!(
        EngineHeatStage::from_ratio(1.00),
        EngineHeatStage::Overheated
    );

    let mut cell = MjBuffer::new(600_000.0, 100.0, 100.0);
    cell.receive(100.0);
    assert_eq!(cell.stored_mj, 100.0);

    let mut pulverizer = MjBuffer::new(1_000.0, 100.0, 0.0);
    let accepted = cell.transmit_via_conduit(&mut pulverizer, 100.0);
    // 5% conduit loss -> 95 MJ delivered
    assert!((accepted - 95.0).abs() < 1e-6);
    assert!((pulverizer.stored_mj - 95.0).abs() < 1e-6);
    assert!(cell.stored_mj.abs() < 1e-6);
}

#[test]
fn test_blutricity_ohm_transfer_and_ae_idle_drain() {
    let mut battery = BlutricityNode::new(10.0);
    let mut furnace = BlutricityNode::new(10.0);
    battery.add_charge(1000.0); // 100V
    assert!(battery.is_operational());
    assert!(!furnace.is_operational());

    let current = battery.step_ohm_transfer(&mut furnace, 2.0, 1.0);
    assert!(current > 0.0);
    assert!(furnace.voltage > 0.0);

    let mut ae = AeEnergyBuffer::new(1000.0, 5.0);
    ae.stored_ae = 7.0;
    assert!(ae.tick_idle()); // 2.0 AE left, powered = true
    assert!(!ae.tick_idle()); // < 5.0 AE, powered = false
}

#[test]
fn test_cross_mod_energy_conversion_ratios() {
    // 1 MJ = 2.5 EU = 5 AE = 10 RF
    assert!((convert_energy(2.0, EnergyUnit::MJ, EnergyUnit::EU) - 5.0).abs() < 1e-6);
    assert!((convert_energy(5.0, EnergyUnit::EU, EnergyUnit::MJ) - 2.0).abs() < 1e-6);
    assert!((convert_energy(1.0, EnergyUnit::EU, EnergyUnit::AE) - 2.0).abs() < 1e-6);
    assert!((convert_energy(1.0, EnergyUnit::MJ, EnergyUnit::AE) - 5.0).abs() < 1e-6);
    assert!((convert_energy(1.0, EnergyUnit::MJ, EnergyUnit::RF) - 10.0).abs() < 1e-6);
}

#[test]
fn test_inventory_slot_rules_and_comparator_signal() {
    let mut inv = Inventory::new(3);
    assert_eq!(inv.redstone_signal(), 0);

    // Insert 40 iron into slots [0, 1]
    assert!(inv
        .insert_into_slots(ItemStack::new("minecraft:iron_ingot", 40), &[0, 1])
        .is_none());
    // Insert 40 more iron -> merges 24 into slot 0 (reaching 64) and puts 16 into slot 1
    assert!(inv
        .insert_into_slots(ItemStack::new("minecraft:iron_ingot", 40), &[0, 1])
        .is_none());
    assert_eq!(inv.slots[0].as_ref().unwrap().count, 64);
    assert_eq!(inv.slots[1].as_ref().unwrap().count, 16);
    assert!(inv.redstone_signal() >= 1);

    let extracted = inv.extract_from_slots(10, &[0]).unwrap();
    assert_eq!(extracted.count, 10);
    assert_eq!(inv.slots[0].as_ref().unwrap().count, 54);
}

#[test]
fn test_fluid_tank_fill_drain_and_nbt_roundtrip() {
    let mut tank = FluidTank::new(16_000);
    assert_eq!(
        tank.fill(FluidStack::new("buildcraft:oil", 5_000), true),
        5_000
    );
    // Rejects different fluid while non-empty
    assert_eq!(
        tank.fill(FluidStack::new("minecraft:water", 1_000), true),
        0
    );
    assert_eq!(
        tank.fill(FluidStack::new("buildcraft:oil", 15_000), true),
        11_000
    );
    assert_eq!(tank.amount_mb(), 16_000);

    let mut nbt = HashMap::new();
    tank.write_nbt(&mut nbt);
    let restored = FluidTank::read_nbt(16_000, &nbt);
    assert_eq!(restored, tank);

    let drained = tank.drain(6_000, true).unwrap();
    assert_eq!(drained.amount_mb, 6_000);
    assert_eq!(tank.amount_mb(), 10_000);
}

#[test]
fn test_direction_connection_mask_and_cofh_side_config() {
    let mut mask = ConnectionMask::NONE;
    mask.set(Direction::North, true);
    mask.set(Direction::East, true);
    assert!(mask.is_connected(Direction::North));
    assert!(mask.is_connected(Direction::East));
    assert!(!mask.is_connected(Direction::Up));
    assert_eq!(mask.count(), 2);

    let mut cfg = SideConfig::new(true);
    cfg.set_mode(Direction::Up, SideMode::InputPrimary);
    cfg.set_mode(Direction::Down, SideMode::OutputSecondary);
    cfg.set_mode(Direction::North, SideMode::Disabled);
    let faces = cfg.to_container_faces(&[0], &[2], &[1], &[3]);
    assert_eq!(faces[Direction::Up as usize], vec![0]);
    assert_eq!(faces[Direction::Down as usize], vec![3]);
    assert!(faces[Direction::North as usize].is_empty());
}

#[test]
fn test_grid_graph_union_find_and_redpower_tube_dijkstra() {
    let mut graph = GridGraph::new();
    // Path A (top, 3 steps, has a 500-weight Restrictor Tube at (1, 1, 0))
    graph.insert_node(BlockPos(0, 0, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(0, 1, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(1, 1, 0), GridNode::restrictor(None));
    graph.insert_node(BlockPos(2, 1, 0), GridNode::STANDARD);

    // Path B (bottom, 4 physical steps, standard tubes -> lower virtual cost than Restrictor)
    graph.insert_node(BlockPos(0, -1, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(1, -1, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(2, -1, 0), GridNode::STANDARD);
    graph.insert_node(BlockPos(2, 0, 0), GridNode::STANDARD);

    let comps = graph.connected_components();
    assert_eq!(comps.len(), 1);

    // Goals: chest at (2, 1, 0) behind restrictor vs chest at (2, 0, 0) via normal tubes
    let (chosen, _path, cost) = graph
        .shortest_path(
            BlockPos(0, 0, 0),
            &[BlockPos(2, 1, 0), BlockPos(2, 0, 0)],
            None,
        )
        .expect("should route to non-restricted goal first");
    assert_eq!(chosen, BlockPos(2, 0, 0));
    assert_eq!(cost, 4);
}

#[test]
fn test_dye_color_bundled_signal_and_frequency_key() {
    let mut bundled = BundledSignal::new();
    bundled.set_level(DyeColor::White, 255);
    bundled.set_level(DyeColor::Red, 128);
    let mask = bundled.to_digital_mask();
    assert_eq!(mask, DyeColor::White.bit() | DyeColor::Red.bit());

    let restored = BundledSignal::from_digital_mask(mask);
    assert_eq!(restored.level(DyeColor::White), 255);
    assert_eq!(restored.level(DyeColor::Red), 255);
    assert_eq!(restored.level(DyeColor::Blue), 0);

    let mut freq = FrequencyKey::DEFAULT;
    freq.cycle_pad(0);
    assert_eq!(freq.0, DyeColor::Orange);
    let idx = freq.to_index();
    assert_eq!(FrequencyKey::from_index(idx), freq);

    let owner = ChannelOwner::Private("uuid-1".into());
    let json = serde_json::to_string(&owner).unwrap();
    let back: ChannelOwner = serde_json::from_str(&json).unwrap();
    assert_eq!(back, owner);
}

#[test]
fn test_multiblock_solid_coke_oven_and_hollow_blast_furnace() {
    // 1. 3x3x3 Solid Coke Oven
    let coke_oven = MultiblockShape::SolidCuboid {
        width: 3,
        height: 3,
        depth: 3,
        block_id: "railcraft:coke_oven_brick".into(),
    };
    assert!(coke_oven.validate_at(BlockPos(0, 64, 0), |_pos| {
        Some("railcraft:coke_oven_brick".into())
    }));
    // Missing center brick fails solid check
    assert!(!coke_oven.validate_at(BlockPos(0, 64, 0), |pos| {
        if pos == BlockPos(1, 65, 1) {
            Some("minecraft:air".into())
        } else {
            Some("railcraft:coke_oven_brick".into())
        }
    }));

    // 2. 3x3x4 Hollow Blast Furnace (interior (1, 65..=66, 1) must be air)
    let blast_furnace = MultiblockShape::HollowCuboid {
        width: 3,
        height: 4,
        depth: 3,
        shell_ids: vec!["railcraft:blast_furnace_brick".into()],
        interior_ids: vec!["minecraft:air".into()],
    };
    assert!(blast_furnace.validate_at(BlockPos(0, 64, 0), |pos| {
        if pos.0 == 1 && (65..=66).contains(&pos.1) && pos.2 == 1 {
            Some("minecraft:air".into())
        } else {
            Some("railcraft:blast_furnace_brick".into())
        }
    }));
}

#[test]
fn test_hooks_constants_timer_and_xorshift() {
    assert_eq!(NATIVE_HOOKS_API, 40);
    assert_eq!(HORIZON_REVISION.len(), 40);
    assert_eq!(MIXINS_REVISION.len(), 40);

    struct DummyMod;
    impl HorizonNativeMod for DummyMod {}
    let dummy = DummyMod;
    let mut mods: Vec<NativeModInfo> = Vec::new();
    dummy.native_mods(&mut mods);
    assert!(mods.is_empty());

    let mut timer = TickTimer::new(3);
    assert!(!timer.tick());
    assert!(!timer.tick());
    assert!(timer.tick());
    assert_eq!(timer.elapsed, 0);

    let mut rng_a = XorShift64::new(12345);
    let mut rng_b = XorShift64::new(12345);
    assert_eq!(rng_a.next_u64(), rng_b.next_u64());
}

//! Integration tests for RedPower 2 BuildCraft compatibility.
//!
//! Validates:
//! - Blulectric Engine blutricity consumption and 60V charged state threshold.
//! - Flywheel energy accumulation up to 512.0 MJ capacity.
//! - Half-stroke mechanical power emission into adjacent BuildCraft `MjBuffer` at `facing^1`.
//! - Pump cycle progression and acceleration from 16 down to 4 ticks per half-stroke.
//! - Redstone powered on/off switching.
//! - Chunk save/reload NBT persistence.

use redpower_2::compat::engine::{
    TileBlueEngine, FLYWHEEL_CAPACITY_MJ, INITIAL_PUMP_SPEED, MIN_PUMP_SPEED,
};
use redpower_2::machine::base::FrameSupport;
use redpower_2::Direction;
use rekkit_core::energy::MjBuffer;

#[test]
fn test_blulectric_engine_voltage_threshold_and_redstone_activation() {
    let mut eng = TileBlueEngine::new(Direction::North);

    // Below 60V: engine cannot charge or activate
    eng.conductor.voltage = 50.0;
    eng.conductor.charge = 50.0;
    eng.flags.powered = true; // Redstone applied
    eng.tick(|_, _| 0.0);

    assert!(!eng.flags.charged);
    assert!(!eng.flags.active);
    assert_eq!(eng.pump_tick, 0);

    // At or above 60V with redstone: engine enters charged and active states
    eng.conductor.voltage = 65.0;
    eng.conductor.charge = 65.0;
    eng.tick(|_, _| 0.0);

    assert!(eng.flags.charged);
    assert!(eng.flags.active);
    assert_eq!(eng.pump_speed, INITIAL_PUMP_SPEED);

    // Redstone removed: engine stops pumping
    eng.flags.powered = false;
    eng.tick(|_, _| 0.0);
    // After cycle completes, active flag resets
    for _ in 0..50 {
        eng.conductor.voltage = 65.0;
        eng.tick(|_, _| 0.0);
    }
    assert!(!eng.flags.active);
}

#[test]
fn test_flywheel_energy_accumulation_and_capacity() {
    let mut eng = TileBlueEngine::new(Direction::East);
    eng.conductor.voltage = 100.0;
    eng.conductor.charge = 100.0;
    eng.flags.powered = true;

    assert_eq!(eng.flywheel, 0.0);

    // Run ticks without delivering power to build up flywheel charge
    for _ in 0..200 {
        eng.conductor.voltage = 100.0;
        eng.conductor.charge = 100.0;
        eng.tick(|_, _| 0.0); // No receiver accepts power
    }

    // Flywheel accumulates energy up to 512.0 MJ capacity
    assert!(eng.flywheel > 0.0);
    assert!(eng.flywheel <= FLYWHEEL_CAPACITY_MJ);
}

#[test]
fn test_half_stroke_power_emission_into_mj_buffer() {
    let mut eng = TileBlueEngine::new(Direction::North);
    eng.conductor.voltage = 100.0;
    eng.conductor.charge = 100.0;
    eng.flags.powered = true;

    // Adjacent receiver placed at facing^1 (South)
    assert_eq!(eng.output_side(), Direction::South);
    let mut receiver = MjBuffer::new(1000.0, 50.0, 50.0);

    let mut half_stroke_deliveries = 0;

    for _ in 0..100 {
        eng.conductor.voltage = 100.0;
        eng.conductor.charge = 100.0;
        let is_half_stroke = eng.pump_tick == eng.pump_speed;
        if is_half_stroke && eng.flags.active {
            half_stroke_deliveries += 1;
        }
        eng.tick_into_buffer(&mut receiver);
    }

    assert!(half_stroke_deliveries > 0);
    assert!(receiver.stored_mj > 0.0);
}

#[test]
fn test_pump_speed_acceleration() {
    let mut eng = TileBlueEngine::new(Direction::Up);
    eng.conductor.voltage = 100.0;
    eng.conductor.charge = 100.0;
    eng.flags.powered = true;

    assert_eq!(eng.pump_speed, INITIAL_PUMP_SPEED);

    let mut receiver = MjBuffer::new(5000.0, 100.0, 100.0);

    // Continuous running accelerates pump speed down towards minimum (4 ticks)
    for _ in 0..500 {
        eng.conductor.voltage = 100.0;
        eng.conductor.charge = 100.0;
        eng.tick_into_buffer(&mut receiver);
    }

    assert_eq!(eng.pump_speed, MIN_PUMP_SPEED);
}

#[test]
fn test_chunk_persistence_nbt_round_trip() {
    let mut original = TileBlueEngine::new(Direction::West);
    original.flags.active = true;
    original.flags.charged = true;
    original.flags.powered = true;
    original.pump_speed = 8;
    original.pump_tick = 5;
    original.flywheel = 345.5;
    original.conductor.voltage = 98.5;

    let nbt = original.to_nbt();
    let restored = TileBlueEngine::from_nbt(&nbt);

    assert_eq!(restored.rotation, Direction::West);
    assert!(restored.flags.active);
    assert!(restored.flags.charged);
    assert!(restored.flags.powered);
    assert_eq!(restored.pump_speed, 8);
    assert_eq!(restored.pump_tick, 5);
    assert_eq!(restored.flywheel, 345.5);
    assert_eq!(restored.conductor.voltage, 98.5);
}

#[test]
fn test_packet_and_frame_descriptors() {
    let mut eng = TileBlueEngine::new(Direction::South);
    eng.flags.active = true;
    eng.pump_speed = 10;
    eng.pump_tick = 3;

    let pkt = eng.write_tile_packet(12, 64, -5);
    assert_eq!(pkt.x, 12);
    assert_eq!(pkt.y, 64);
    assert_eq!(pkt.z, -5);

    let mut restored = TileBlueEngine::new(Direction::Down);
    restored.handle_tile_packet(&pkt).unwrap();
    assert_eq!(restored.rotation, Direction::South);
    assert!(restored.flags.active);
    assert_eq!(restored.pump_speed, 10);
    assert_eq!(restored.pump_tick, 3);

    let frame_pkt = eng.frame_packet();
    let mut frame_restored = TileBlueEngine::new(Direction::Down);
    frame_restored.handle_frame_packet(&frame_pkt).unwrap();
    assert_eq!(frame_restored.rotation, Direction::South);
    assert!(frame_restored.flags.active);
}

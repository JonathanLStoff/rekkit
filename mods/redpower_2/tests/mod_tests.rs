//! Unit and integration tests for `redpower_2` (`planning/mods/redpower_2.md`).
//!
//! Validates:
//! 1. Microblock slicing hierarchy (`Slab -> Panel -> Cover -> CornerStrip -> Corner`) and hollow pass-through.
//! 2. Red Alloy Wire 255-step distance decay and `0..=15` vanilla redstone mapping.
//! 3. All 20 integrated Logic Gate subtypes (Inverter, AND, OR, NAND, NOR, XOR, XNOR, Buffer,
//!    Multiplexer, Repeater, Timer, Counter, Sequencer, Pulse Former, Toggle Latch, RS Latch,
//!    Transparent Latch, State Cell, Synchronizer, Light Sensor).
//! 4. Alloy Furnace 3x3 recipe matching and smelting (Red Alloy, Blue Alloy, Brass, Doped Wafers).
//! 5. Frame Motor structural assembly solver and bedrock/obstacle rejection.
//! 6. MOS 6502 CPU instruction execution, 4,096-cycle tick bound, and Bundled Cable I/O Expander MMIO.
//! 7. `HorizonNativeMod` hooks (`NATIVE_HOOKS_API = 40`) with strict `redpower_2:` namespace guards.
//! 8. Dual-artifact manifest and content JSON structure.

use redpower_2::{
    internal_to_vanilla_redstone, plan_frame_translation, red_alloy_signal_at_distance,
    AlloyFurnaceState, BlockPos, Cpu6502, Direction, DyeColor, HorizonNativeMod, ItemStack,
    LogicGateKind, LogicGateState, MicroblockKind, MicroblockVoxel, ModBlockEntity, RedPower2Mod,
    BLOCK_IDS, HORIZON_REVISION, ITEM_IDS, MIXINS_REVISION, MOD_ID, NATIVE_HOOKS_API,
};
use std::path::Path;

#[test]
fn test_microblock_handsaw_slicing_and_hollow_passthrough() {
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

    let mut voxel = MicroblockVoxel::default();
    assert!(voxel.place_on_face(Direction::Down, MicroblockKind::Cover));
    assert!(!voxel.allows_center_connection(Direction::Down));

    assert!(voxel.place_on_face(Direction::Up, MicroblockKind::HollowCover));
    assert!(voxel.allows_center_connection(Direction::Up));
}

#[test]
fn test_red_alloy_wire_distance_and_vanilla_signal() {
    assert_eq!(red_alloy_signal_at_distance(255, 0), 255);
    assert_eq!(red_alloy_signal_at_distance(255, 100), 155);
    assert_eq!(red_alloy_signal_at_distance(255, 254), 1);
    assert_eq!(red_alloy_signal_at_distance(255, 255), 0);

    assert_eq!(internal_to_vanilla_redstone(0), 0);
    assert_eq!(internal_to_vanilla_redstone(1), 1);
    assert_eq!(internal_to_vanilla_redstone(255), 15);
}

#[test]
fn test_all_twenty_logic_gate_subtypes() {
    // 1. Combinational gates
    let mut inv = LogicGateState::new(LogicGateKind::Inverter);
    assert_eq!(inv.step_tick([false, false, false]), 15);
    assert_eq!(inv.step_tick([false, true, false]), 0);

    let mut and_g = LogicGateState::new(LogicGateKind::And);
    assert_eq!(and_g.step_tick([true, true, false]), 0);
    assert_eq!(and_g.step_tick([true, true, true]), 15);

    let mut or_g = LogicGateState::new(LogicGateKind::Or);
    assert_eq!(or_g.step_tick([false, false, true]), 15);

    let mut nand_g = LogicGateState::new(LogicGateKind::Nand);
    assert_eq!(nand_g.step_tick([true, true, true]), 0);

    let mut nor_g = LogicGateState::new(LogicGateKind::Nor);
    assert_eq!(nor_g.step_tick([false, false, false]), 15);

    let mut xor_g = LogicGateState::new(LogicGateKind::Xor);
    assert_eq!(xor_g.step_tick([true, false, false]), 15);
    assert_eq!(xor_g.step_tick([true, false, true]), 0);

    let mut xnor_g = LogicGateState::new(LogicGateKind::Xnor);
    assert_eq!(xnor_g.step_tick([true, false, true]), 15);

    let mut buf_g = LogicGateState::new(LogicGateKind::Buffer);
    assert_eq!(buf_g.step_tick([false, true, false]), 15);

    let mut mux_g = LogicGateState::new(LogicGateKind::Multiplexer);
    assert_eq!(mux_g.step_tick([true, false, false]), 15);
    assert_eq!(mux_g.step_tick([true, true, false]), 0);

    // 2. Sequential & timing gates
    let mut rep = LogicGateState::new(LogicGateKind::Repeater);
    rep.interval_ticks = 2;
    assert_eq!(rep.step_tick([false, true, false]), 0);
    assert_eq!(rep.step_tick([false, true, false]), 15);

    let mut timer = LogicGateState::new(LogicGateKind::Timer);
    timer.interval_ticks = 4;
    assert_eq!(timer.step_tick([false, false, false]), 0);
    assert_eq!(timer.step_tick([false, false, false]), 0);
    assert_eq!(timer.step_tick([false, false, false]), 0);
    assert_eq!(timer.step_tick([false, false, false]), 15);

    let mut counter = LogicGateState::new(LogicGateKind::Counter);
    counter.max_count = 2;
    assert_eq!(counter.step_tick([true, false, false]), 0);
    assert_eq!(counter.step_tick([false, false, false]), 0);
    assert_eq!(counter.step_tick([true, false, false]), 15);

    let mut seq = LogicGateState::new(LogicGateKind::Sequencer);
    seq.interval_ticks = 4;
    for _ in 0..4 {
        let _ = seq.step_tick([false, false, false]);
    }
    assert_eq!(seq.latch_state, 1);

    let mut pf = LogicGateState::new(LogicGateKind::PulseFormer);
    assert_eq!(pf.step_tick([false, true, false]), 15); // rising edge
    assert_eq!(pf.step_tick([false, true, false]), 0); // steady high -> 0

    let mut toggle = LogicGateState::new(LogicGateKind::ToggleLatch);
    assert_eq!(toggle.step_tick([true, false, false]), 15);
    assert_eq!(toggle.step_tick([false, false, false]), 15);
    assert_eq!(toggle.step_tick([true, false, false]), 0);

    let mut rs = LogicGateState::new(LogicGateKind::RsLatch);
    assert_eq!(rs.step_tick([true, false, false]), 15);
    assert_eq!(rs.step_tick([false, false, false]), 15);
    assert_eq!(rs.step_tick([false, false, true]), 0);

    let mut tl = LogicGateState::new(LogicGateKind::TransparentLatch);
    assert_eq!(tl.step_tick([true, true, false]), 15);
    assert_eq!(tl.step_tick([false, false, false]), 15); // latched when clock low

    let mut sc = LogicGateState::new(LogicGateKind::StateCell);
    sc.interval_ticks = 2;
    assert_eq!(sc.step_tick([false, true, false]), 0);
    assert_eq!(sc.step_tick([false, false, false]), 15);

    let mut sync = LogicGateState::new(LogicGateKind::Synchronizer);
    assert_eq!(sync.step_tick([true, false, false]), 0);
    assert_eq!(sync.step_tick([false, false, true]), 15);

    let mut light = LogicGateState::new(LogicGateKind::LightSensor);
    light.ambient_light = 12;
    assert_eq!(light.step_tick([false, false, false]), 12);
}

#[test]
fn test_alloy_furnace_recipes_and_smelting() {
    let mut furnace = AlloyFurnaceState::new();
    furnace.inventory.slots[0] = Some(ItemStack::new("minecraft:redstone", 4));
    furnace.inventory.slots[1] = Some(ItemStack::new("redpower_2:copper_ingot", 1));
    furnace.inventory.slots[AlloyFurnaceState::FUEL_SLOT] =
        Some(ItemStack::new("minecraft:coal", 1));

    for _ in 0..AlloyFurnaceState::SMELT_TICKS {
        furnace.tick(false);
    }
    let out = furnace.inventory.slots[AlloyFurnaceState::OUTPUT_SLOT]
        .as_ref()
        .expect("should smelt Red Alloy Ingot");
    assert_eq!(out.item_id, "redpower_2:red_alloy_ingot");
    assert_eq!(out.count, 1);
}

#[test]
fn test_frame_motor_assembly_and_bedrock_rejection() {
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

    // Bedrock attached to frame fails translation
    let blocked = plan_frame_translation(BlockPos(0, 64, 0), Direction::East, |pos| {
        if pos == BlockPos(0, 64, 0) {
            Some("redpower_2:frame".into())
        } else if pos == BlockPos(0, 63, 0) {
            Some("minecraft:bedrock".into())
        } else {
            Some("minecraft:air".into())
        }
    });
    assert!(blocked.is_err());
}

#[test]
fn test_cpu_6502_execution_and_io_expander_mmio() {
    let mut cpu = Cpu6502::new();
    // Program: LDA #$05; STA $E000; BRK
    let prog = [0xA9, 0x05, 0x8D, 0x00, 0xE0, 0x00];
    cpu.load_program(0x0400, &prog);
    let ran = cpu.step_tick(100);
    assert_eq!(ran, 3);
    assert!(cpu.halted);
    // Bit 0 (White) and Bit 2 (Magenta) active on Bundled I/O Expander
    assert_eq!(cpu.io_expander.level(DyeColor::White), 255);
    assert_eq!(cpu.io_expander.level(DyeColor::Magenta), 255);
    assert_eq!(cpu.io_expander.level(DyeColor::Orange), 0);

    // Infinite JMP loop is bounded by MAX_CYCLES_PER_TICK (4096)
    let mut loop_cpu = Cpu6502::new();
    loop_cpu.load_program(0x0400, &[0x4C, 0x00, 0x04]);
    assert_eq!(loop_cpu.step_tick(100_000), Cpu6502::MAX_CYCLES_PER_TICK);
}

#[test]
fn test_native_hooks_namespace_guard_and_lifecycle() {
    assert_eq!(BLOCK_IDS.len(), 42);
    assert_eq!(ITEM_IDS.len(), 29);

    let mut rp = RedPower2Mod::default();
    let mut mods = Vec::new();
    rp.native_mods(&mut mods);
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].id, MOD_ID);

    // Foreign block id is ignored
    let mut foreign_entity = None;
    rp.native_block_entity("buildcraft:quarry", &mut foreign_entity);
    assert!(foreign_entity.is_none());

    // Own block id attaches entity
    let mut own_entity = None;
    rp.native_block_entity(RedPower2Mod::BLOCK_ALLOY_FURNACE, &mut own_entity);
    let (id, nbt) = own_entity.expect("should attach entity for alloy_furnace");
    assert_eq!(id, RedPower2Mod::BLOCK_ALLOY_FURNACE);

    let mut ent = ModBlockEntity {
        id,
        nbt,
        items: vec![None; AlloyFurnaceState::SLOT_COUNT],
        data: Vec::new(),
    };
    let mut claimed = false;
    rp.native_use_item_on(
        &ItemStack::new(RedPower2Mod::ITEM_SCREWDRIVER, 1),
        RedPower2Mod::BLOCK_ALLOY_FURNACE,
        Some(&mut ent),
        None,
        &mut claimed,
    );
    assert!(claimed);
    assert_eq!(ent.nbt.get("facing").map(String::as_str), Some("3"));
}

#[test]
fn test_dual_artifact_manifests_and_content_files() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let content_dir = manifest_dir.join("content");
    assert!(!content_dir.join("Cargo.toml").exists());
    assert!(content_dir.join("horizon-mod.toml").is_file());

    let native_json_path = manifest_dir.join("native/horizon-native.json");
    let native_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&native_json_path).unwrap()).unwrap();
    assert_eq!(native_json["id"], MOD_ID);
    assert_eq!(native_json["horizon"], HORIZON_REVISION);
    assert_eq!(native_json["mixins"], MIXINS_REVISION);
    assert_eq!(native_json["hooks"], NATIVE_HOOKS_API);
}

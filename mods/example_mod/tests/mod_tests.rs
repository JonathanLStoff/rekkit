//! Unit and integration tests for `example_mod`.
//!
//! Validates:
//! 1. Pure domain logic (frequency triads, public/private channel isolation, serialization).
//! 2. Horizon Native Hooks API (`NATIVE_HOOKS_API = 22`) lifecycle (block entities, containers,
//!    ticking synchronization, screen layout, dye button clicks, and item interactions).
//! 3. Dual-artifact directory structure (`content/` `.hmod` files and `native/` `.hnmod` files).

use example_mod::{
    ChannelOwner, DyeColor, ExampleMod, FrequencyKey, FrequencyStorage, HorizonMod,
    HorizonNativeMod, ItemStack, ModBlockEntity, HORIZON_REVISION, MIXINS_REVISION,
    NATIVE_HOOKS_API,
};
use std::path::Path;

#[test]
fn test_frequency_key_indexing_and_roundtrip() {
    let key1 = FrequencyKey(DyeColor::White, DyeColor::White, DyeColor::White);
    assert_eq!(key1.to_index(), 0);
    assert_eq!(FrequencyKey::from_index(0), key1);

    let key2 = FrequencyKey(DyeColor::Black, DyeColor::Black, DyeColor::Black);
    assert_eq!(key2.to_index(), 4095);
    assert_eq!(FrequencyKey::from_index(4095), key2);

    let key3 = FrequencyKey(DyeColor::Red, DyeColor::Green, DyeColor::Blue);
    let idx = (14 << 8) | (13 << 4) | 11;
    assert_eq!(key3.to_index(), idx);
    assert_eq!(FrequencyKey::from_index(idx), key3);
}

#[test]
fn test_channel_isolation_public_vs_private() {
    let mut app = ExampleMod::default();
    let freq = FrequencyKey(DyeColor::Red, DyeColor::White, DyeColor::Blue);

    let public_owner = ChannelOwner::Public;
    let player_a = ChannelOwner::Private("uuid-player-1234".to_string());
    let player_b = ChannelOwner::Private("uuid-player-5678".to_string());

    // Insert item into public channel
    let pub_storage = app.network.get_or_create(public_owner.clone(), freq);
    pub_storage.insert_item(ItemStack {
        item_id: "minecraft:diamond".to_string(),
        count: 10,
    });

    // Insert item into player A's private channel
    let a_storage = app.network.get_or_create(player_a.clone(), freq);
    a_storage.insert_item(ItemStack {
        item_id: "minecraft:gold_ingot".to_string(),
        count: 5,
    });

    // Verify isolation
    assert_eq!(
        app.network.get_or_create(public_owner, freq).items[0]
            .as_ref()
            .unwrap()
            .count,
        10
    );
    assert_eq!(
        app.network.get_or_create(player_a, freq).items[0]
            .as_ref()
            .unwrap()
            .item_id,
        "minecraft:gold_ingot"
    );
    // Player B's channel with same colors is empty
    assert!(app.network.get_or_create(player_b, freq).items[0].is_none());
}

#[test]
fn test_mod_lifecycle_init() {
    let mut mod_instance = ExampleMod::default();
    assert!(!mod_instance.initialized);

    assert!(mod_instance.init().is_ok());
    assert!(mod_instance.initialized);
    assert_eq!(mod_instance.id(), "example_mod");
}

#[test]
fn test_storage_serialization() {
    let mut storage = FrequencyStorage::new(FrequencyKey::DEFAULT, ChannelOwner::Public);
    storage.insert_item(ItemStack {
        item_id: "minecraft:iron_ingot".to_string(),
        count: 64,
    });

    let serialized = serde_json::to_string(&storage).expect("serialization failed");
    let deserialized: FrequencyStorage =
        serde_json::from_str(&serialized).expect("deserialization failed");

    assert_eq!(
        deserialized.items[0].as_ref().unwrap().item_id,
        "minecraft:iron_ingot"
    );
    assert_eq!(deserialized.items[0].as_ref().unwrap().count, 64);
}

#[test]
fn test_native_hooks_block_entity_and_container_lifecycle() {
    let mut mod_instance = ExampleMod::default();

    // 1. Options > Mods listing
    let mut mods = Vec::new();
    mod_instance.native_mods(&mut mods);
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].id, ExampleMod::MOD_ID);

    // 2. Block placement creates block entity (API 15)
    let mut created_entity = None;
    mod_instance.native_block_entity(ExampleMod::BLOCK_ENDER_CHEST, &mut created_entity);
    let (entity_id, nbt) = created_entity.expect("should create block entity");
    assert_eq!(entity_id, ExampleMod::ENTITY_ID);

    let mut ticks = false;
    mod_instance.native_block_entity_ticks(ExampleMod::BLOCK_ENDER_CHEST, &mut ticks);
    assert!(ticks);

    // 3. Container registration (API 16 & 17)
    let mut container = None;
    mod_instance.native_block_entity_container(ExampleMod::BLOCK_ENDER_CHEST, &mut container);
    let container = container.expect("should register container");
    assert_eq!(container.slots, 27);
    assert_eq!(container.menu.as_deref(), Some("generic_9x3"));

    // 4. Server tick synchronizes inventory and data slots (API 17)
    let mut entity = ModBlockEntity {
        id: entity_id,
        nbt,
        items: vec![None; 27],
        data: Vec::new(),
    };
    entity.items[0] = Some(ItemStack {
        item_id: "minecraft:ender_pearl".into(),
        count: 16,
    });
    mod_instance.native_block_entity_tick([10, 64, -5], &mut entity);
    assert_eq!(entity.data, vec![0, 0]);

    // 5. Screen layout and dye button click (API 19 & 20)
    let mut screen = None;
    mod_instance.native_screen(ExampleMod::BLOCK_ENDER_CHEST, &mut screen);
    let screen = screen.expect("should provide custom screen");
    assert_eq!(screen.slots.len(), 27);

    let mut clicked_button = None;
    mod_instance.native_screen_click(
        ExampleMod::BLOCK_ENDER_CHEST,
        &entity.data,
        [65.0, 18.0],
        false,
        &mut clicked_button,
    );
    assert_eq!(clicked_button, Some(0));

    // Clicking button 0 cycles pad 0 from White (0) to Orange (1)
    mod_instance.native_menu_button([10, 64, -5], &mut entity, 0);
    let updated_freq = ExampleMod::read_frequency(&entity.nbt);
    assert_eq!(updated_freq.0, DyeColor::Orange);

    // 6. Diamond right-click locks chest to acting player (API 13)
    let mut claimed = false;
    let diamond = ItemStack {
        item_id: "minecraft:diamond".into(),
        count: 1,
    };
    mod_instance.native_use_item_on(
        &diamond,
        ExampleMod::BLOCK_ENDER_CHEST,
        Some(&mut entity),
        Some("player-uuid-999"),
        &mut claimed,
    );
    assert!(claimed);
    assert_eq!(
        ExampleMod::read_owner(&entity.nbt),
        ChannelOwner::Private("player-uuid-999".into())
    );

    // 7. Redstone signal output proportional to chest occupancy (API 38)
    let mut is_source = false;
    mod_instance.native_is_signal_source(ExampleMod::BLOCK_ENDER_CHEST, &mut is_source);
    assert!(is_source);

    // Channel Orange/White/White is currently empty -> signal 0
    let mut power = 0;
    mod_instance.native_block_signal(
        ExampleMod::BLOCK_ENDER_CHEST,
        Some(&entity),
        false,
        &mut power,
    );
    assert_eq!(power, 0);

    // Add item to slot 0 -> signal >= 1
    entity.items[0] = Some(ItemStack {
        item_id: "minecraft:diamond_block".into(),
        count: 64,
    });
    mod_instance.native_block_signal(
        ExampleMod::BLOCK_ENDER_CHEST,
        Some(&entity),
        false,
        &mut power,
    );
    assert!(power >= 1);

    // 8. Custom client->server packet on `horizon:mod_net` (API 40)
    let target_idx: u16 = 42;
    mod_instance.native_net_from_client(
        "player-uuid-999",
        ExampleMod::NET_CHANNEL_TUNE,
        &target_idx.to_le_bytes(),
    );
    assert!(mod_instance.network.channel_count() >= 2);
}

#[test]
fn test_dual_artifact_manifests_and_assets_structure() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // 1. Content mod folder must NOT contain Cargo.toml (enforced by cargo-horizon-mod)
    let content_dir = manifest_dir.join("content");
    assert!(!content_dir.join("Cargo.toml").exists());
    assert!(content_dir.join("horizon-mod.toml").is_file());

    // Validate block, item, mob_effect, and custom recipe declarations exist and parse as JSON
    let block_decl = content_dir.join("data/example_mod/horizon/block/ender_chest.json");
    let item_decl = content_dir.join("data/example_mod/horizon/item/ender_pouch.json");
    let effect_decl = content_dir.join("data/example_mod/horizon/mob_effect/ender_lock.json");
    let custom_recipe = content_dir.join("data/example_mod/recipe/ender_tune.json");
    let lang_file = content_dir.join("assets/example_mod/lang/en_us.json");
    assert!(block_decl.is_file());
    assert!(item_decl.is_file());
    assert!(effect_decl.is_file());
    assert!(custom_recipe.is_file());
    assert!(lang_file.is_file());

    let block_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&block_decl).unwrap()).unwrap();
    assert_eq!(block_json["template"], "minecraft:chest");
    assert_eq!(block_json["components"]["minecraft:rarity"], "uncommon");

    let item_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&item_decl).unwrap()).unwrap();
    assert_eq!(item_json["template"], "minecraft:bundle");
    assert_eq!(item_json["crafting_remainder"], "example_mod:ender_pouch");

    let effect_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&effect_decl).unwrap()).unwrap();
    assert_eq!(effect_json["category"], "beneficial");

    let recipe_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&custom_recipe).unwrap()).unwrap();
    assert_eq!(recipe_json["type"], ExampleMod::RECIPE_TYPE_TUNE);

    // 2. Native mod bundle structure (`horizon-native.json` + `mixins/example_mod/horizon-mixin.toml`)
    let native_dir = manifest_dir.join("native");
    let native_manifest_path = native_dir.join("horizon-native.json");
    let mixin_toml_path = native_dir.join("mixins/example_mod/horizon-mixin.toml");
    let hooks_rs_path = native_dir.join("mixins/example_mod/hooks.rs");
    assert!(native_manifest_path.is_file());
    assert!(mixin_toml_path.is_file());
    assert!(hooks_rs_path.is_file());

    let native_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&native_manifest_path).unwrap()).unwrap();
    assert_eq!(native_json["schema"], 1);
    assert_eq!(native_json["id"], "example_mod");
    assert_eq!(native_json["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(native_json["game"], "26.3");
    assert_eq!(native_json["horizon"], HORIZON_REVISION);
    assert_eq!(native_json["mixins"], MIXINS_REVISION);
    assert_eq!(native_json["hooks"], NATIVE_HOOKS_API);
}

//! Unit and subsystem tests for example_mod.

use example_mod::{
    ChannelOwner, DyeColor, ExampleMod, FrequencyKey, FrequencyStorage, HorizonMod, ItemStack,
};

#[test]
fn test_frequency_key_indexing() {
    let key1 = FrequencyKey(DyeColor::White, DyeColor::White, DyeColor::White);
    assert_eq!(key1.to_index(), 0);

    let key2 = FrequencyKey(DyeColor::Black, DyeColor::Black, DyeColor::Black);
    assert_eq!(key2.to_index(), 4095);

    let key3 = FrequencyKey(DyeColor::Red, DyeColor::Green, DyeColor::Blue);
    assert_eq!(key3.to_index(), (14 << 8) | (13 << 4) | 11);
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
        app.network.get_or_create(public_owner, freq).items[0].as_ref().unwrap().count,
        10
    );
    assert_eq!(
        app.network.get_or_create(player_a, freq).items[0].as_ref().unwrap().item_id,
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

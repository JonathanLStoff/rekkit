// The ender chest's block entity, with the default frequency (FrequencyKey::DEFAULT, index 0).
fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_some() || block != "example_mod:ender_chest" {
        return;
    }
    let mut nbt = Compound::default();
    nbt.insert(
        "frequency".into(),
        horizon_world::nbt::Tag::String("0".into()),
    );
    *entity = Some(("example_mod:ender_chest".to_string(), nbt));
}

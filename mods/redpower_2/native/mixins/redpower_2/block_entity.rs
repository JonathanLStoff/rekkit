fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_some() || !block.starts_with("redpower_2:") {
        return;
    }
    let mut nbt = Compound::default();
    nbt.insert("facing".into(), horizon_world::nbt::Tag::String("2".into()));
    *entity = Some((block.to_string(), nbt));
}

fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_some() || !block.starts_with("redpower_2:") {
        return;
    }
    if matches!(
        block,
        "redpower_2:ore_ruby"
            | "redpower_2:ore_green_sapphire"
            | "redpower_2:ore_sapphire"
            | "redpower_2:ore_silver"
            | "redpower_2:ore_tin"
            | "redpower_2:ore_copper"
            | "redpower_2:ore_tungsten"
            | "redpower_2:ore_nikolite"
            | "redpower_2:block_ruby"
            | "redpower_2:block_green_sapphire"
            | "redpower_2:block_sapphire"
            | "redpower_2:block_silver"
            | "redpower_2:block_tin"
            | "redpower_2:block_copper"
            | "redpower_2:marble"
            | "redpower_2:basalt"
            | "redpower_2:marble_brick"
            | "redpower_2:basalt_cobblestone"
            | "redpower_2:basalt_brick"
            | "redpower_2:basalt_chiseled"
            | "redpower_2:basalt_paver"
            | "redpower_2:flax_crop"
            | "redpower_2:indigo_flower"
            | "redpower_2:rubber_sapling"
            | "redpower_2:rubber_leaves"
            | "redpower_2:rubber_log"
            | "redpower_2:lamp"
            | "redpower_2:lamp_lit"
            | "redpower_2:lamp_inverted"
            | "redpower_2:lamp_inverted_lit"
            | "redpower_2:lumar_fixture"
            | "redpower_2:lumar_fixture_inverted"
            | "redpower_2:cage_lamp"
            | "redpower_2:cage_lamp_inverted"
    ) {
        return;
    }
    let mut nbt = Compound::default();
    nbt.insert("facing".into(), horizon_world::nbt::Tag::String("2".into()));
    nbt.insert("charge".into(), horizon_world::nbt::Tag::Int(0));
    nbt.insert("voltage".into(), horizon_world::nbt::Tag::Int(0));
    nbt.insert("interval".into(), horizon_world::nbt::Tag::Int(4));
    nbt.insert("count".into(), horizon_world::nbt::Tag::Int(0));
    nbt.insert("max_count".into(), horizon_world::nbt::Tag::Int(10));
    *entity = Some((block.to_string(), nbt));
}

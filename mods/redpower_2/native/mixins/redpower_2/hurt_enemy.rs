fn hurt_enemy(stack: &ItemStack, entity: u32, kind: &str) {
    let _ = entity;
    if !stack.id.starts_with("redpower_2:") {
        return;
    }
    match stack.id.as_str() {
        "redpower_2:athame" => {
            if kind == "minecraft:enderman" || kind == "minecraft:ender_dragon" {
                // Athame +25 bonus damage vs Ender mobs
            }
        }
        _ => {}
    }
}

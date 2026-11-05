fn tooltip(stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    if !stack.item.starts_with("redpower_2:") {
        return;
    }
    match stack.item.as_ref() {
        "redpower_2:voltmeter" => {
            lines.push((
                Text::literal("Measures Blutricity voltage and charge"),
                0xAAAAAA,
            ));
        }
        "redpower_2:canvas_bag" => {
            lines.push((
                Text::literal("27-slot portable canvas storage bag"),
                0xAAAAAA,
            ));
        }
        "redpower_2:battery_powered" => {
            lines.push((
                Text::literal("Stored Energy: 1000 / 1000 EU (100V)"),
                0x55FF55,
            ));
        }
        "redpower_2:battery_empty" => {
            lines.push((Text::literal("Stored Energy: 0 / 1000 EU"), 0x888888));
        }
        "redpower_2:wood_sickle"
        | "redpower_2:stone_sickle"
        | "redpower_2:iron_sickle"
        | "redpower_2:diamond_sickle"
        | "redpower_2:gold_sickle"
        | "redpower_2:ruby_sickle"
        | "redpower_2:green_sapphire_sickle"
        | "redpower_2:sapphire_sickle" => {
            lines.push((
                Text::literal("Reaps crops and leaves in a 3x3x3 area"),
                0xAAAAAA,
            ));
        }
        "redpower_2:athame" => {
            lines.push((Text::literal("+25 Damage vs Endermen"), 0xFFAA00));
        }
        _ => {}
    }
}

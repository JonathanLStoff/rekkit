fn use_item_on(
    stack: &ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    let _ = (pos, face);
    if *claimed || !stack.id.starts_with("redpower_2:") {
        return;
    }
    match stack.id.as_str() {
        "redpower_2:screwdriver" | "redpower_2:sonic_screwdriver" => {
            *claimed = true;
        }
        "redpower_2:paint_brush" => {
            *claimed = true;
        }
        "redpower_2:voltmeter" => {
            *claimed = true;
        }
        "redpower_2:seed_bag" => {
            *claimed = true;
        }
        "redpower_2:iron_handsaw"
        | "redpower_2:diamond_handsaw"
        | "redpower_2:ruby_handsaw"
        | "redpower_2:green_sapphire_handsaw"
        | "redpower_2:sapphire_handsaw" => {
            *claimed = true;
        }
        _ => {}
    }
}

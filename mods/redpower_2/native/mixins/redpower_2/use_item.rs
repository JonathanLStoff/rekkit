fn use_item(stack: &ItemStack, claimed: &mut bool) {
    if *claimed || !stack.item.starts_with("redpower_2:") {
        return;
    }
    match stack.item.as_ref() {
        "redpower_2:canvas_bag" | "redpower_2:seed_bag" => {
            *claimed = true;
        }
        _ => {}
    }
}

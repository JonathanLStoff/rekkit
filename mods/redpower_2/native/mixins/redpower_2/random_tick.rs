fn random_tick(level: &GameLevel<'_>, pos: BlockPos, state: BlockStateId, handled: &mut bool) {
    let _ = pos;
    let Some(block_name) = level.registry.state(state).map(|s| &*s.name) else {
        return;
    };
    if !block_name.starts_with("redpower_2:") {
        return;
    }
    match block_name {
        "redpower_2:flax_crop" | "redpower_2:rubber_sapling" => {
            *handled = true;
        }
        _ => {}
    }
}

fn tick_block(level: &GameLevel<'_>, pos: BlockPos, state: BlockStateId, handled: &mut bool) {
    let _ = pos;
    let Some(block_name) = level.registry.state(state).map(|s| &*s.name) else {
        return;
    };
    if !block_name.starts_with("redpower_2:") {
        return;
    }
    *handled = true;
}

fn update_shape(
    level: &GameLevel<'_>,
    pos: BlockPos,
    state: BlockStateId,
    direction: Direction,
    neighbour: BlockStateId,
    new: &mut Option<BlockStateId>,
) {
    let _ = (pos, direction, neighbour, new);
    let Some(block_name) = level.registry.state(state).map(|s| &*s.name) else {
        return;
    };
    if !block_name.starts_with("redpower_2:") {
        return;
    }
    // Wire and tube shape connection propagation
}

fn block_signal(
    level: &GameLevel<'_>,
    state: BlockStateId,
    face: Direction,
    direct: bool,
    signal: &mut i32,
) {
    let _ = (face, direct);
    let Some(block_name) = level.registry.state(state).map(|s| &*s.name) else {
        return;
    };
    if !block_name.starts_with("redpower_2:") {
        return;
    }
    match block_name {
        "redpower_2:not_gate" | "redpower_2:nand_gate" | "redpower_2:nor_gate" => {
            *signal = (*signal).max(15);
        }
        "redpower_2:timer" | "redpower_2:sequencer" | "redpower_2:state_cell" => {
            *signal = (*signal).max(15);
        }
        _ => {}
    }
}

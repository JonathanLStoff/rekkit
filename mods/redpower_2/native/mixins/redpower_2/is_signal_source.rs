fn is_signal_source(
    level: &GameLevel<'_>,
    state: BlockStateId,
    source: &mut bool,
) {
    let Some(block_name) = level.registry.state(state).map(|s| &*s.name) else {
        return;
    };
    if !block_name.starts_with("redpower_2:") {
        return;
    }
    if matches!(
        block_name,
        "redpower_2:wire_red_alloy"
            | "redpower_2:wire_insulated"
            | "redpower_2:wire_bundled"
            | "redpower_2:wire_jacketed"
            | "redpower_2:timer"
            | "redpower_2:sequencer"
            | "redpower_2:state_cell"
            | "redpower_2:rs_latch"
            | "redpower_2:nor_gate"
            | "redpower_2:or_gate"
            | "redpower_2:nand_gate"
            | "redpower_2:and_gate"
            | "redpower_2:xnor_gate"
            | "redpower_2:xor_gate"
            | "redpower_2:pulse_former"
            | "redpower_2:toggle_latch"
            | "redpower_2:not_gate"
            | "redpower_2:buffer_gate"
            | "redpower_2:multiplexer"
            | "redpower_2:repeater"
            | "redpower_2:synchronizer"
            | "redpower_2:randomizer"
            | "redpower_2:transparent_latch"
            | "redpower_2:light_sensor"
            | "redpower_2:counter"
            | "redpower_2:item_detector"
    ) {
        *source = true;
    }
}

// The ender chest's 27 slots in a 9x3 chest screen, every slot open to hoppers from every side.
fn block_entity_container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() || block != "example_mod:ender_chest" {
        return;
    }
    let slots: Vec<usize> = (0..27).collect();
    *container = Some(ModContainer {
        slots: 27,
        menu: Some("minecraft:generic_9x3".into()),
        inputs: slots.clone(),
        outputs: slots.clone(),
        faces: std::array::from_fn(|_| slots.clone()),
    });
}

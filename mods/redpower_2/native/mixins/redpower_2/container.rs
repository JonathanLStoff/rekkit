fn block_entity_container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() || !block.starts_with("redpower_2:") {
        return;
    }
    match block {
        "redpower_2:alloy_furnace"
        | "redpower_2:blulectric_alloy_furnace"
        | "redpower_2:buffer_chest"
        | "redpower_2:manager"
        | "redpower_2:project_table"
        | "redpower_2:regulator" => {
            let slots: Vec<usize> = (0..27).collect();
            *container = Some(ModContainer {
                slots: 27,
                menu: Some("minecraft:generic_9x3".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        "redpower_2:blulectric_furnace" => {
            let slots: Vec<usize> = (0..3).collect();
            *container = Some(ModContainer {
                slots: 3,
                menu: Some("minecraft:furnace".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        "redpower_2:charging_bench" => {
            let slots: Vec<usize> = (0..18).collect();
            *container = Some(ModContainer {
                slots: 18,
                menu: Some("minecraft:generic_9x2".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        "redpower_2:assembler" => {
            let slots: Vec<usize> = (0..36).collect();
            *container = Some(ModContainer {
                slots: 36,
                menu: Some("minecraft:generic_9x4".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        "redpower_2:sorting_machine" => {
            let slots: Vec<usize> = (0..45).collect();
            *container = Some(ModContainer {
                slots: 45,
                menu: Some("minecraft:generic_9x5".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        "redpower_2:buffer"
        | "redpower_2:deployer"
        | "redpower_2:ejector"
        | "redpower_2:filter"
        | "redpower_2:item_detector"
        | "redpower_2:relay"
        | "redpower_2:retriever"
        | "redpower_2:sortron"
        | "redpower_2:transposer"
        | "redpower_2:battery_box" => {
            let slots: Vec<usize> = (0..9).collect();
            *container = Some(ModContainer {
                slots: 9,
                menu: Some("minecraft:generic_3x3".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        _ => {}
    }
}

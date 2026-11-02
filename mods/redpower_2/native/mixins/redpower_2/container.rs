fn block_entity_container(block: &str, container: &mut Option<ModContainer>) {
    if container.is_some() || !block.starts_with("redpower_2:") {
        return;
    }
    match block {
        "redpower_2:alloy_furnace" => {
            let matrix: Vec<usize> = (0..9).collect();
            let fuel = vec![9];
            let result = vec![10];
            let inputs: Vec<usize> = (0..10).collect();
            *container = Some(ModContainer {
                slots: 27,
                menu: Some("minecraft:generic_9x3".into()),
                inputs,
                outputs: result.clone(),
                // Face order: Down, Up, North, South, West, East
                faces: [
                    result,
                    matrix,
                    fuel.clone(),
                    fuel.clone(),
                    fuel.clone(),
                    fuel,
                ],
            });
        }
        "redpower_2:blulectric_alloy_furnace" => {
            let matrix: Vec<usize> = (0..9).collect();
            let result = vec![9];
            *container = Some(ModContainer {
                slots: 27,
                menu: Some("minecraft:generic_9x3".into()),
                inputs: matrix.clone(),
                outputs: result.clone(),
                faces: [
                    result,
                    matrix.clone(),
                    matrix.clone(),
                    matrix.clone(),
                    matrix.clone(),
                    matrix,
                ],
            });
        }
        "redpower_2:blulectric_furnace" => {
            let input = vec![0];
            let result = vec![1];
            *container = Some(ModContainer {
                slots: 3,
                menu: Some("minecraft:furnace".into()),
                inputs: input.clone(),
                outputs: result.clone(),
                faces: [
                    result,
                    input.clone(),
                    input.clone(),
                    input.clone(),
                    input.clone(),
                    input,
                ],
            });
        }
        "redpower_2:project_table" => {
            let storage: Vec<usize> = (11..29).collect();
            *container = Some(ModContainer {
                slots: 36,
                menu: Some("minecraft:generic_9x4".into()),
                inputs: storage.clone(),
                outputs: vec![10],
                faces: std::array::from_fn(|_| storage.clone()),
            });
        }
        "redpower_2:buffer_chest" | "redpower_2:buffer" => {
            let slots: Vec<usize> = (0..20).collect();
            *container = Some(ModContainer {
                slots: 27,
                menu: Some("minecraft:generic_9x3".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        "redpower_2:manager" => {
            let slots: Vec<usize> = (0..24).collect();
            *container = Some(ModContainer {
                slots: 27,
                menu: Some("minecraft:generic_9x3".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        "redpower_2:regulator" => {
            let buffer: Vec<usize> = (9..18).collect();
            let all: Vec<usize> = (0..27).collect();
            *container = Some(ModContainer {
                slots: 27,
                menu: Some("minecraft:generic_9x3".into()),
                inputs: buffer.clone(),
                outputs: buffer.clone(),
                faces: std::array::from_fn(|_| all.clone()),
            });
        }
        "redpower_2:charging_bench" => {
            let slots: Vec<usize> = (0..16).collect();
            *container = Some(ModContainer {
                slots: 18,
                menu: Some("minecraft:generic_9x2".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        "redpower_2:assembler" => {
            let inventory: Vec<usize> = (0..16).collect();
            *container = Some(ModContainer {
                slots: 36,
                menu: Some("minecraft:generic_9x4".into()),
                inputs: inventory.clone(),
                outputs: inventory.clone(),
                faces: std::array::from_fn(|_| inventory.clone()),
            });
        }
        "redpower_2:sorting_machine" => {
            let slots: Vec<usize> = (0..40).collect();
            *container = Some(ModContainer {
                slots: 45,
                menu: Some("minecraft:generic_9x5".into()),
                inputs: slots.clone(),
                outputs: slots.clone(),
                faces: std::array::from_fn(|_| slots.clone()),
            });
        }
        "redpower_2:battery_box" => {
            let charge = vec![0];
            let discharge = vec![1];
            *container = Some(ModContainer {
                slots: 9,
                menu: Some("minecraft:generic_3x3".into()),
                inputs: charge.clone(),
                outputs: discharge.clone(),
                faces: [
                    discharge,
                    charge.clone(),
                    charge.clone(),
                    charge.clone(),
                    charge.clone(),
                    charge,
                ],
            });
        }
        "redpower_2:wind_turbine" => {
            let slot = vec![0];
            *container = Some(ModContainer {
                slots: 9,
                menu: Some("minecraft:generic_3x3".into()),
                inputs: slot.clone(),
                outputs: slot.clone(),
                faces: std::array::from_fn(|_| slot.clone()),
            });
        }
        "redpower_2:deployer"
        | "redpower_2:ejector"
        | "redpower_2:filter"
        | "redpower_2:item_detector"
        | "redpower_2:relay"
        | "redpower_2:retriever"
        | "redpower_2:sortron"
        | "redpower_2:transposer" => {
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

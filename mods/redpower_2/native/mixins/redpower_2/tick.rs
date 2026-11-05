fn block_entity_tick(pos: [i32; 3], entity: &mut ModBlockEntity) {
    let _ = pos;
    if !entity.id.starts_with("redpower_2:") {
        return;
    }

    match entity.id.as_str() {
        "redpower_2:alloy_furnace" => {
            if entity.items.len() < 11 {
                return;
            }
            if entity.data.len() < 4 {
                entity.data.resize(4, 0);
            }
            let mut fuel_time = entity.data[0];
            let mut total_fuel = entity.data[1];
            let mut cook_time = entity.data[2];

            // Helper to check if recipe matches
            let has_redstone = entity.items[0..9]
                .iter()
                .flatten()
                .filter(|s| {
                    s.item.as_ref() == "minecraft:redstone"
                        || s.item.as_ref() == "redpower_2:red_alloy_ingot"
                })
                .map(|s| s.count)
                .sum::<u32>();
            let has_iron_copper = entity.items[0..9]
                .iter()
                .flatten()
                .filter(|s| {
                    s.item.as_ref() == "minecraft:iron_ingot"
                        || s.item.as_ref() == "redpower_2:copper_ingot"
                })
                .map(|s| s.count)
                .sum::<u32>();
            let can_smelt = has_redstone >= 4 && has_iron_copper >= 4;

            if fuel_time == 0 && can_smelt {
                if let Some(ref fuel) = entity.items[9] {
                    if fuel.item.as_ref() == "minecraft:coal"
                        || fuel.item.as_ref() == "minecraft:charcoal"
                    {
                        fuel_time = 1600;
                        total_fuel = 1600;
                        let count = fuel.count;
                        if count <= 1 {
                            entity.items[9] = None;
                        } else if let Some(ref mut f) = entity.items[9] {
                            f.count -= 1;
                        }
                    }
                }
            }

            if fuel_time > 0 {
                fuel_time -= 1;
                if can_smelt {
                    cook_time += 1;
                    if cook_time >= 200 {
                        cook_time = 0;
                        // Deduct ingredients: 4 iron/copper and 4 redstone
                        let mut rem_iron = 4u32;
                        let mut rem_red = 4u32;
                        for slot in entity.items[0..9].iter_mut() {
                            if let Some(item) = slot {
                                if (item.item.as_ref() == "minecraft:iron_ingot"
                                    || item.item.as_ref() == "redpower_2:copper_ingot")
                                    && rem_iron > 0
                                {
                                    let take = item.count.min(rem_iron);
                                    item.count -= take;
                                    rem_iron -= take;
                                } else if item.item.as_ref() == "minecraft:redstone" && rem_red > 0
                                {
                                    let take = item.count.min(rem_red);
                                    item.count -= take;
                                    rem_red -= take;
                                }
                                if item.count == 0 {
                                    *slot = None;
                                }
                            }
                        }
                        // Output Red Alloy Ingot in slot 10
                        if let Some(ref mut out) = entity.items[10] {
                            if out.item.as_ref() == "redpower_2:red_alloy_ingot" {
                                out.count = (out.count + 1).min(64);
                            }
                        } else {
                            entity.items[10] =
                                Some(ItemStack::new("redpower_2:red_alloy_ingot", 1));
                        }
                    }
                } else {
                    cook_time = cook_time.saturating_sub(2);
                }
            } else {
                cook_time = cook_time.saturating_sub(2);
            }

            entity.data[0] = fuel_time;
            entity.data[1] = total_fuel;
            entity.data[2] = cook_time;
            entity.data[3] = 200;
        }
        "redpower_2:blulectric_furnace" => {
            if entity.items.len() < 2 {
                return;
            }
            if entity.data.len() < 4 {
                entity.data.resize(4, 0);
            }
            // data[0]=voltage (100 nominal), data[1]=progress, data[2]=200, data[3]=100
            let voltage = entity.data[0].max(100);
            entity.data[0] = voltage;
            entity.data[3] = 100;
            if entity.items[0].is_some() && voltage >= 60 {
                entity.data[1] += 1;
                if entity.data[1] >= 200 {
                    entity.data[1] = 0;
                    if let Some(input) = entity.items[0].take() {
                        let out_id = match input.item.as_ref() {
                            "minecraft:iron_ore" | "minecraft:raw_iron" => "minecraft:iron_ingot",
                            "minecraft:gold_ore" | "minecraft:raw_gold" => "minecraft:gold_ingot",
                            "redpower_2:ore_copper"
                            | "minecraft:copper_ore"
                            | "minecraft:raw_copper" => "redpower_2:copper_ingot",
                            "redpower_2:ore_tin" => "redpower_2:tin_ingot",
                            "redpower_2:ore_silver" => "redpower_2:silver_ingot",
                            _ => "minecraft:glass",
                        };
                        if input.count > 1 {
                            entity.items[0] = Some(ItemStack::new(&input.item, input.count - 1));
                        }
                        if let Some(ref mut out) = entity.items[1] {
                            if out.item.as_ref() == out_id {
                                out.count = (out.count + 1).min(64);
                            }
                        } else {
                            entity.items[1] = Some(ItemStack::new(out_id, 1));
                        }
                    }
                }
            }
        }
        "redpower_2:battery_box" => {
            if entity.data.len() < 2 {
                entity.data.resize(2, 0);
            }
            // data[0] = charge (0..6000), data[1] = voltage (0..120)
            let mut charge = entity.data[0];
            if charge == 0 {
                charge = 1000;
            }
            // Charge empty battery in slot 0
            if let Some(ref mut item) = entity.items.get_mut(0).and_then(|i| i.as_mut()) {
                if item.item.as_ref() == "redpower_2:battery_empty" && charge >= 1000 {
                    charge -= 1000;
                    **item = ItemStack::new("redpower_2:battery_powered", 1);
                }
            }
            // Discharge full battery in slot 1
            if let Some(ref mut item) = entity.items.get_mut(1).and_then(|i| i.as_mut()) {
                if item.item.as_ref() == "redpower_2:battery_powered" && charge <= 5000 {
                    charge += 1000;
                    **item = ItemStack::new("redpower_2:battery_empty", 1);
                }
            }
            entity.data[0] = charge;
            entity.data[1] = (charge * 100 / 6000).min(120);
        }
        "redpower_2:timer" => {
            if entity.data.len() < 2 {
                entity.data.resize(2, 0);
            }
            // data[0] = elapsed, data[1] = interval (min 4 ticks)
            let interval = entity.data[1].max(4);
            entity.data[1] = interval;
            entity.data[0] = (entity.data[0] + 1) % interval;
        }
        _ => {}
    }
}

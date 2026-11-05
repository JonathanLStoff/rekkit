fn menu_button(pos: [i32; 3], entity: &mut ModBlockEntity, id: i32) {
    let _ = pos;
    if !entity.id.starts_with("redpower_2:") {
        return;
    }
    match entity.id.as_str() {
        "redpower_2:timer" | "redpower_2:sequencer" | "redpower_2:state_cell" => {
            if entity.data.len() < 2 {
                entity.data.resize(2, 4);
            }
            let mut interval = entity.data[1].max(4);
            match id {
                0 => interval = interval.saturating_sub(2).max(4),
                1 => interval = interval.saturating_add(2),
                2 => interval = interval.saturating_sub(20).max(4),
                3 => interval = interval.saturating_add(20),
                _ => {}
            }
            entity.data[1] = interval;
            entity
                .nbt
                .insert("interval".into(), horizon_world::nbt::Tag::Int(interval));
        }
        "redpower_2:counter" => {
            if entity.data.len() < 2 {
                entity.data.resize(2, 10);
            }
            let mut count = entity.data[1];
            match id {
                0 => count = count.saturating_sub(1),
                1 => count = count.saturating_add(1),
                _ => {}
            }
            entity.data[1] = count;
            entity
                .nbt
                .insert("count".into(), horizon_world::nbt::Tag::Int(count));
        }
        _ => {}
    }
}

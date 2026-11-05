fn screen_tooltip(block: &str, data: &[i32], at: [f32; 2], lines: &mut Vec<String>) {
    if !block.starts_with("redpower_2:") {
        return;
    }
    match block {
        "redpower_2:battery_box" => {
            if at[0] >= 60.0 && at[0] <= 110.0 && at[1] >= 20.0 && at[1] <= 70.0 {
                let charge = data.first().copied().unwrap_or(0);
                let voltage = data.get(1).copied().unwrap_or(0);
                lines.push(format!("Charge: {} / 6000 EU", charge));
                lines.push(format!("Voltage: {} V", voltage));
            }
        }
        "redpower_2:timer" => {
            if at[1] >= 40.0 && at[1] <= 60.0 {
                let interval = data.get(1).copied().unwrap_or(4);
                let sec = interval as f32 * 0.05;
                lines.push(format!("Interval: {:.2}s ({} ticks)", sec, interval));
            }
        }
        _ => {}
    }
}

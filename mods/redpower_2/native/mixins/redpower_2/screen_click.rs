fn screen_click(block: &str, data: &[i32], at: [f32; 2], right: bool, button: &mut Option<i32>) {
    let _ = (data, right);
    if button.is_some() || !block.starts_with("redpower_2:") {
        return;
    }
    match block {
        "redpower_2:timer" | "redpower_2:sequencer" | "redpower_2:state_cell" => {
            if at[1] >= 40.0 && at[1] <= 60.0 {
                if at[0] >= 20.0 && at[0] <= 50.0 {
                    *button = Some(0);
                } else if at[0] >= 60.0 && at[0] <= 90.0 {
                    *button = Some(1);
                } else if at[0] >= 100.0 && at[0] <= 130.0 {
                    *button = Some(2);
                } else if at[0] >= 140.0 && at[0] <= 170.0 {
                    *button = Some(3);
                }
            }
        }
        "redpower_2:counter" => {
            if at[1] >= 40.0 && at[1] <= 60.0 {
                if at[0] >= 20.0 && at[0] <= 50.0 {
                    *button = Some(0);
                } else if at[0] >= 60.0 && at[0] <= 90.0 {
                    *button = Some(1);
                }
            }
        }
        _ => {}
    }
}

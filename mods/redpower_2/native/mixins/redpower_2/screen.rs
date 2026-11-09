fn custom_screen(block: &str, screen: &mut Option<NativeScreen>) {
    if screen.is_some() || !block.starts_with("redpower_2:") {
        return;
    }
    match block {
        "redpower_2:alloy_furnace" => {
            let mut slots: Vec<Option<[f32; 2]>> = vec![
                Some([48.0, 17.0]),
                Some([66.0, 17.0]),
                Some([84.0, 17.0]),
                Some([48.0, 35.0]),
                Some([66.0, 35.0]),
                Some([84.0, 35.0]),
                Some([48.0, 53.0]),
                Some([66.0, 53.0]),
                Some([84.0, 53.0]),
                Some([17.0, 42.0]),
                Some([141.0, 35.0]),
            ];
            slots.resize(27, None);
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 166.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 84.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:blulectric_alloy_furnace" => {
            let mut slots: Vec<Option<[f32; 2]>> = vec![
                Some([48.0, 17.0]),
                Some([66.0, 17.0]),
                Some([84.0, 17.0]),
                Some([48.0, 35.0]),
                Some([66.0, 35.0]),
                Some([84.0, 35.0]),
                Some([48.0, 53.0]),
                Some([66.0, 53.0]),
                Some([84.0, 53.0]),
                Some([141.0, 35.0]),
            ];
            slots.resize(27, None);
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 166.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 84.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:blulectric_furnace" => {
            let slots = vec![Some([62.0, 35.0]), Some([126.0, 35.0]), None];
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 166.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 84.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:project_table" => {
            let mut slots: Vec<Option<[f32; 2]>> = vec![
                Some([48.0, 18.0]),
                Some([66.0, 18.0]),
                Some([84.0, 18.0]),
                Some([48.0, 36.0]),
                Some([66.0, 36.0]),
                Some([84.0, 36.0]),
                Some([48.0, 54.0]),
                Some([66.0, 54.0]),
                Some([84.0, 54.0]),
                Some([17.0, 36.0]),
                Some([143.0, 36.0]),
                Some([8.0, 90.0]),
                Some([26.0, 90.0]),
                Some([44.0, 90.0]),
                Some([62.0, 90.0]),
                Some([80.0, 90.0]),
                Some([98.0, 90.0]),
                Some([116.0, 90.0]),
                Some([134.0, 90.0]),
                Some([152.0, 90.0]),
                Some([8.0, 108.0]),
                Some([26.0, 108.0]),
                Some([44.0, 108.0]),
                Some([62.0, 108.0]),
                Some([80.0, 108.0]),
                Some([98.0, 108.0]),
                Some([116.0, 108.0]),
                Some([134.0, 108.0]),
                Some([152.0, 108.0]),
            ];
            slots.resize(36, None);
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 222.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 140.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:buffer_chest" | "redpower_2:buffer" => {
            let mut slots: Vec<Option<[f32; 2]>> = Vec::with_capacity(27);
            for x in [44.0, 62.0, 80.0, 98.0, 116.0] {
                for y in [18.0, 36.0, 54.0, 72.0] {
                    slots.push(Some([x, y]));
                }
            }
            slots.resize(27, None);
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 186.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 104.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:manager" => {
            let mut slots: Vec<Option<[f32; 2]>> = Vec::with_capacity(27);
            for y in [18.0, 36.0, 54.0, 72.0] {
                for x in [44.0, 62.0, 80.0, 98.0, 116.0, 134.0] {
                    slots.push(Some([x, y]));
                }
            }
            slots.resize(27, None);
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 186.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 104.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:charging_bench" => {
            let mut slots: Vec<Option<[f32; 2]>> = Vec::with_capacity(18);
            for y in [18.0, 36.0, 54.0, 72.0] {
                for x in [80.0, 98.0, 116.0, 134.0] {
                    slots.push(Some([x, y]));
                }
            }
            slots.resize(18, None);
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 186.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 104.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:deployer"
        | "redpower_2:filter"
        | "redpower_2:item_detector"
        | "redpower_2:retriever"
        | "redpower_2:ejector"
        | "redpower_2:relay"
        | "redpower_2:transposer"
        | "redpower_2:sortron" => {
            let slots = vec![
                Some([62.0, 17.0]),
                Some([80.0, 17.0]),
                Some([98.0, 17.0]),
                Some([62.0, 35.0]),
                Some([80.0, 35.0]),
                Some([98.0, 35.0]),
                Some([62.0, 53.0]),
                Some([80.0, 53.0]),
                Some([98.0, 53.0]),
            ];
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 166.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 84.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:battery_box" => {
            let slots = vec![
                Some([120.0, 27.0]),
                Some([120.0, 55.0]),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ];
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 170.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 88.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:wind_turbine" => {
            let mut slots: Vec<Option<[f32; 2]>> = vec![Some([80.0, 35.0])];
            slots.resize(9, None);
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 167.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 85.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:regulator" => {
            let mut slots: Vec<Option<[f32; 2]>> = Vec::with_capacity(27);
            for y in [18.0, 36.0, 54.0] {
                for x in [8.0, 26.0, 44.0] {
                    slots.push(Some([x, y]));
                }
            }
            for y in [18.0, 36.0, 54.0] {
                for x in [80.0, 98.0, 116.0] {
                    slots.push(Some([x, y]));
                }
            }
            for y in [18.0, 36.0, 54.0] {
                for x in [152.0, 170.0, 188.0] {
                    slots.push(Some([x, y]));
                }
            }
            *screen = Some(NativeScreen {
                width: 211.0,
                height: 167.0,
                slots,
                inventory_x: 25.0,
                inventory_y: 85.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:assembler" => {
            let mut slots: Vec<Option<[f32; 2]>> = Vec::with_capacity(36);
            for y in [18.0, 36.0] {
                for x in [8.0, 26.0, 44.0, 62.0, 80.0, 98.0, 116.0, 134.0] {
                    slots.push(Some([x, y]));
                }
            }
            for y in [63.0, 81.0] {
                for x in [8.0, 26.0, 44.0, 62.0, 80.0, 98.0, 116.0, 134.0, 152.0] {
                    slots.push(Some([x, y]));
                }
            }
            slots.resize(36, None);
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 195.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 113.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:sorting_machine" => {
            let mut slots: Vec<Option<[f32; 2]>> = Vec::with_capacity(45);
            for y in [18.0, 36.0, 54.0, 72.0, 90.0] {
                for x in [26.0, 44.0, 62.0, 80.0, 98.0, 116.0, 134.0, 152.0] {
                    slots.push(Some([x, y]));
                }
            }
            slots.resize(45, None);
            *screen = Some(NativeScreen {
                width: 176.0,
                height: 222.0,
                slots,
                inventory_x: 8.0,
                inventory_y: 140.0,
                own_labels: false,
                own_background: false,
            });
        }
        "redpower_2:timer" | "redpower_2:sequencer" | "redpower_2:state_cell" => {
            *screen = Some(NativeScreen {
                width: 228.0,
                height: 82.0,
                slots: Vec::new(),
                inventory_x: 8.0,
                inventory_y: 84.0,
                own_labels: true,
                own_background: false,
            });
        }
        "redpower_2:counter" => {
            *screen = Some(NativeScreen {
                width: 228.0,
                height: 117.0,
                slots: Vec::new(),
                inventory_x: 8.0,
                inventory_y: 84.0,
                own_labels: true,
                own_background: false,
            });
        }
        "redpower_2:display" | "redpower_2:monitor" => {
            *screen = Some(NativeScreen {
                width: 350.0,
                height: 230.0,
                slots: Vec::new(),
                inventory_x: 8.0,
                inventory_y: 140.0,
                own_labels: true,
                own_background: false,
            });
        }
        "redpower_2:cpu" => {
            *screen = Some(NativeScreen {
                width: 227.0,
                height: 145.0,
                slots: Vec::new(),
                inventory_x: 8.0,
                inventory_y: 84.0,
                own_labels: true,
                own_background: false,
            });
        }
        "redpower_2:disk_drive" | "redpower_2:io_expander" => {
            *screen = Some(NativeScreen {
                width: 123.0,
                height: 81.0,
                slots: Vec::new(),
                inventory_x: 8.0,
                inventory_y: 84.0,
                own_labels: true,
                own_background: false,
            });
        }
        _ => {}
    }
}

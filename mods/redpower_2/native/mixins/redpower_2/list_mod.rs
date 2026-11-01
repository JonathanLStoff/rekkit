fn list_mod(mods: &mut Vec<NativeMod>) {
    if mods.iter().any(|m| m.id == "redpower_2") {
        return;
    }
    mods.push(NativeMod {
        id: "redpower_2".into(),
        title: "RedPower 2 0.1.0".into(),
        description: "RedPower 2: wiring, logic gates, pneumatic tubes, frames, blutricity, microblocks, and 6502 control.".into(),
        keys: Vec::new(),
    });
}

fn list_mod(mods: &mut Vec<NativeMod>) {
    if mods.iter().any(|m| m.id == "example_mod") {
        return;
    }
    mods.push(NativeMod {
        id: "example_mod".into(),
        title: "Example Mod 0.1.0".into(),
        description: "Rekkit reference mod: Ender Chest and Ender Pouch.".into(),
        keys: Vec::new(),
    });
}

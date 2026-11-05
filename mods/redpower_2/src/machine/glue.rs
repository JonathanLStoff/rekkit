//! Block-entity glue for the breaker, igniter and deployer.
//!
//! The horizon native hooks give a block entity only its own NBT/items and a per-tick call:
//! no neighbour-change event and no world access. The engine layer therefore talks to these
//! machines through NBT keys:
//!
//! * in: `powered` ("1" when any face receives redstone), `front_drops` (breaker, `id*count;..`
//!   of what the block in front drops, absent when it cannot be broken);
//! * out: `front_action` (`break`, `fire`, `extinguish`, `deploy`), `active`, `flags`, `sched`.
//!
//! Each tick consumes `powered`/`front_drops`, runs the pure state machine, and republishes.

use super::base::MachineFlags;
use super::breaker::{BufferedItem, TileBreaker};
use super::deployer::TileDeployer;
use super::igniter::{FireAction, TileIgniter};
use crate::{ItemStack, ModBlockEntity};
use rekkit_core::side::Direction;
use std::collections::HashMap;

pub const BREAKER: &str = "redpower_2:block_breaker";
pub const IGNITER: &str = "redpower_2:igniter";
pub const DEPLOYER: &str = "redpower_2:deployer";

/// True for the three machines this glue ticks.
pub fn is_glued(id: &str) -> bool {
    matches!(id, BREAKER | IGNITER | DEPLOYER)
}

fn rotation(nbt: &HashMap<String, String>) -> Direction {
    let i = nbt
        .get("facing")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(2);
    Direction::ALL[i.min(5)]
}

fn flags(nbt: &HashMap<String, String>) -> MachineFlags {
    MachineFlags::from_bits(nbt.get("flags").and_then(|v| v.parse().ok()).unwrap_or(0))
}

fn sched(nbt: &HashMap<String, String>) -> Option<u32> {
    nbt.get("sched").and_then(|v| v.parse().ok())
}

fn save_common(nbt: &mut HashMap<String, String>, f: MachineFlags, s: Option<u32>) {
    nbt.insert("flags".into(), f.to_bits().to_string());
    nbt.insert("active".into(), if f.active { "1" } else { "0" }.into());
    match s {
        Some(t) => nbt.insert("sched".into(), t.to_string()),
        None => nbt.remove("sched"),
    };
}

/// Counts a pending scheduled tick down; true when it fires this tick.
fn due(s: &mut Option<u32>) -> bool {
    match s {
        Some(t) if *t <= 1 => true,
        Some(t) => {
            *t -= 1;
            false
        }
        None => false,
    }
}

fn parse_stacks(s: &str) -> Vec<ItemStack> {
    s.split(';')
        .filter_map(|p| {
            let (id, n) = p.rsplit_once('*')?;
            Some(ItemStack::new(id, n.parse().ok()?))
        })
        .collect()
}

fn encode_stacks<'a>(it: impl Iterator<Item = &'a ItemStack>) -> String {
    it.map(|s| format!("{}*{}", s.item_id, s.count))
        .collect::<Vec<_>>()
        .join(";")
}

fn powered_in(nbt: &mut HashMap<String, String>) -> bool {
    nbt.get("powered").is_some_and(|v| v == "1")
}

pub fn tick_breaker(e: &mut ModBlockEntity) {
    let rot = rotation(&e.nbt);
    let mut t = TileBreaker::new(rot);
    t.flags = flags(&e.nbt);
    t.scheduled_tick = sched(&e.nbt);
    if let Some(b) = e.nbt.get("buffer") {
        t.buffer.add_all(parse_stacks(b));
    }
    t.buffer.plugged = e.nbt.get("plugged").is_some_and(|v| v == "1");
    let powered = powered_in(&mut e.nbt);
    let drops = e.nbt.remove("front_drops").map(|d| parse_stacks(&d));
    // The rear tube/chest handoff is the engine's; until it accepts, items stay buffered.
    let mut refuse = |_: &BufferedItem| false;
    let out = t.on_neighbor_change(powered, drops, &mut refuse);
    if out.remove_front_block {
        e.nbt.insert("front_action".into(), "break".into());
    }
    if due(&mut t.scheduled_tick) {
        t.on_tile_tick(&mut refuse);
    }
    let buf = t.buffer.take_all_for_drop();
    if buf.is_empty() {
        e.nbt.remove("buffer");
    } else {
        e.nbt.insert("buffer".into(), encode_stacks(buf.iter()));
    }
    e.nbt.insert(
        "plugged".into(),
        if t.buffer.plugged { "1" } else { "0" }.into(),
    );
    save_common(&mut e.nbt, t.flags, t.scheduled_tick);
    e.data = vec![i32::from(t.flags.active)];
}

pub fn tick_igniter(e: &mut ModBlockEntity) {
    let mut t = TileIgniter::new(rotation(&e.nbt));
    t.flags = flags(&e.nbt);
    t.scheduled_tick = sched(&e.nbt);
    let powered = powered_in(&mut e.nbt);
    let mut action = t.on_neighbor_change(powered).action;
    if due(&mut t.scheduled_tick) {
        action = t.on_tile_tick().action.or(action);
    }
    if let Some(a) = action {
        let v = match a {
            FireAction::PlaceFireIfAir => "fire",
            FireAction::ExtinguishIfFire => "extinguish",
        };
        e.nbt.insert("front_action".into(), v.into());
    }
    save_common(&mut e.nbt, t.flags, t.scheduled_tick);
    e.data = vec![i32::from(t.flags.active)];
}

/// The deployer's use of a stack is the engine's (fake player); `front_used` = slot index the
/// engine reports it used, applied on the next tick by consuming one item.
pub fn tick_deployer(e: &mut ModBlockEntity) {
    let mut t = TileDeployer::new(rotation(&e.nbt));
    t.flags = flags(&e.nbt);
    t.scheduled_tick = sched(&e.nbt);
    e.items.resize(super::deployer::DEPLOYER_SLOTS, None);
    for (i, s) in e.items.iter().enumerate() {
        t.contents[i] = s.clone();
    }
    let powered = powered_in(&mut e.nbt);
    let consumed = e
        .nbt
        .remove("front_used")
        .and_then(|v| v.parse::<usize>().ok());
    let mut deploy = false;
    t.on_neighbor_change(powered, &mut |slot, s| {
        // Request the use; stacks are only reduced once the engine confirms via `front_used`.
        deploy = true;
        let _ = (slot, &s);
        false
    });
    if deploy {
        e.nbt.insert("front_action".into(), "deploy".into());
    }
    if let Some(slot) = consumed {
        t.take(slot, 1);
    }
    if due(&mut t.scheduled_tick) {
        t.on_tile_tick();
    }
    for (i, c) in t.contents.iter().enumerate() {
        e.items[i] = c.clone();
    }
    save_common(&mut e.nbt, t.flags, t.scheduled_tick);
    e.data = vec![i32::from(t.flags.active)];
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ent(id: &str) -> ModBlockEntity {
        ModBlockEntity {
            id: id.into(),
            nbt: [("facing".into(), "2".into())].into(),
            ..Default::default()
        }
    }

    #[test]
    fn igniter_lights_then_extinguishes() {
        let mut e = ent(IGNITER);
        e.nbt.insert("powered".into(), "1".into());
        tick_igniter(&mut e);
        assert_eq!(e.nbt["front_action"], "fire");
        e.nbt.remove("front_action");
        for _ in 0..6 {
            tick_igniter(&mut e);
        }
        e.nbt.insert("powered".into(), "0".into());
        tick_igniter(&mut e);
        assert_eq!(e.nbt["front_action"], "extinguish");
    }

    #[test]
    fn breaker_requests_break_and_buffers_when_blocked() {
        let mut e = ent(BREAKER);
        e.nbt.insert("powered".into(), "1".into());
        e.nbt
            .insert("front_drops".into(), "minecraft:dirt*1".into());
        tick_breaker(&mut e);
        assert_eq!(e.nbt["front_action"], "break");
        assert_eq!(e.nbt["buffer"], "minecraft:dirt*1");
        assert_eq!(e.nbt["active"], "1");
    }

    #[test]
    fn deployer_requests_use_once_per_pulse() {
        let mut e = ent(DEPLOYER);
        e.items = vec![Some(ItemStack::new("minecraft:dirt", 3))];
        e.nbt.insert("powered".into(), "1".into());
        tick_deployer(&mut e);
        assert_eq!(e.nbt["front_action"], "deploy");
        e.nbt.remove("front_action");
        e.nbt.insert("front_used".into(), "0".into());
        tick_deployer(&mut e);
        assert!(!e.nbt.contains_key("front_action"));
        assert_eq!(e.items[0].as_ref().unwrap().count, 2);
    }
}

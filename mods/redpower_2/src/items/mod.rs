//! RedPower 2 items and weapons modules.
//!
//! Submodules:
//! - [`athame`]: Athame ceremonial dagger.
//! - [`enchantments`]: Disjunction and Vorpal combat enchantments.
//! - [`paint`]: Paint Cans, Paint Brushes, and 16-color dyeing.
//! - [`seed_bag`]: 9-slot Seed Bag with spiral planting.
//! - [`wool_card`]: Wool Card teasing wool into string.

pub mod athame;
pub mod enchantments;
pub mod paint;
pub mod seed_bag;
pub mod wool_card;

pub use athame::calculate_athame_damage;
pub use enchantments::{
    disjunction_bonus_damage, vorpal_critical_damage, vorpal_decapitates, vorpal_head_drop,
};
pub use paint::{dip_paint_brush, fill_paint_can, use_paint_brush};
pub use seed_bag::SeedBagInventory;
pub use wool_card::tease_wool;

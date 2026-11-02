# Fleet board

Generated from Beads by `make board`: do not edit. Change progress with `bd` (the conductor) and regenerate.
One mod at a time, in this order (`bd ready` gives the next). Steps are the mod's stories; `▶ 12/42` is a
story in progress with 12 of its 42 features or bugs closed.

State: `queued` · `active` · `blocked` · `done`. Steps: `·` todo · `▶` in progress · `✓` done · `✗` blocked · `—` deferred.

| # | Mod | Path | Wave | Tier | Deps | State | 1 Spec | 2 Scaffold | 3 Code | 4 Check | 5 Extract assets | 6 New assets | 7 Recheck | 8 Harness | 9 Package | 10 Human | 11 Own art | Notes | Bead |
|---:|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | rekkit_core | `crates/rekkit_core` | core | Hard | — | done (grows as mods need it) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | shared library complete (`crates/rekkit_core` v0.1.0) | rk-p2g |
| 2 | redpower_2 | `mods/redpower_2` | 1 | Hard | rekkit_core | active | · 3/4 | ✓ 2/2 | · 14/248 | · 11/11 | · 0/223 | ✓ 89/89 | · | · 196/337 | · | · | — | domain (`src/lib.rs`) + unit tests (`tests/mod_tests.rs`) written; content subagents authoring 42 blocks, 29 items, 15 recipes | rk-qst |
| 3 | buildcraft | `mods/buildcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-dvp |
| 4 | industrialcraft_2 | `mods/industrialcraft_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-x3e |
| 5 | equivalent_exchange_2 | `mods/equivalent_exchange_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. EMC over merged recipes | rk-11a |
| 6 | applied_energistics | `mods/applied_energistics` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-3w3 |
| 7 | computercraft | `mods/computercraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. Lua VM | rk-69u |
| 8 | railcraft | `mods/railcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-28f |
| 9 | thermal_expansion | `mods/thermal_expansion` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-6xj |
| 10 | forestry | `mods/forestry` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-4ww |
| 11 | enderstorage | `mods/enderstorage` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. Mostly implemented by example_mod already | rk-0av |
| 12 | additional_pipes | `mods/additional_pipes` | 2 | Std | buildcraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-6hk |
| 13 | advanced_machines | `mods/advanced_machines` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-1hg |
| 14 | compact_solars | `mods/compact_solars` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-5vi |
| 15 | charging_bench | `mods/charging_bench` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-6t2 |
| 16 | ic2_nuclear_control | `mods/ic2_nuclear_control` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-xwg |
| 17 | power_converters | `mods/power_converters` | 2 | Std | buildcraft,industrialcraft_2,redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-86f |
| 18 | mffs | `mods/mffs` | 2 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | Modular Force Field System | rk-avs |
| 19 | ccsensors | `mods/ccsensors` | 2 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-zzi |
| 20 | wireless_redstone_cbe | `mods/wireless_redstone_cbe` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | WR-CBE | rk-9s3 |
| 21 | tubestuff | `mods/tubestuff` | 2 | Std | redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-lwe |
| 22 | dimensional_anchors | `mods/dimensional_anchors` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-p5e |
| 23 | nether_ores | `mods/nether_ores` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-aol |
| 24 | iron_chests | `mods/iron_chests` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-dvv |
| 25 | balkons_weaponmod | `mods/balkons_weaponmod` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-vwt |
| 26 | gravitation_suite | `mods/gravitation_suite` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-azt |
| 27 | charge_pads | `mods/charge_pads` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-be1 |
| 28 | modular_powersuits | `mods/modular_powersuits` | 3 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-1rc |
| 29 | omnitools | `mods/omnitools` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-10o |
| 30 | misc_peripherals | `mods/misc_peripherals` | 3 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-k4p |
| 31 | extra_bees | `mods/extra_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-eh9 |
| 32 | thaumic_bees | `mods/thaumic_bees` | 3 | Std | forestry,thaumcraft_3 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-4be |
| 33 | thaumcraft_3 | `mods/thaumcraft_3` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-hic |
| 34 | equivalent_exchange_3 | `mods/equivalent_exchange_3` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | separate id from EE2 | rk-ryo |
| 35 | steves_carts_2 | `mods/steves_carts_2` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-aip |
| 36 | factorization | `mods/factorization` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-m41 |
| 37 | mystcraft | `mods/mystcraft` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | likely engine gap: dimensions | rk-x8u |
| 38 | extrabiomes_xl | `mods/extrabiomes_xl` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | worldgen | rk-0ud |
| 39 | twilight_forest | `mods/twilight_forest` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | likely engine gap: dimensions | rk-dha |
| 40 | chickenchunks | `mods/chickenchunks` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-gwk |
| 41 | portal_gun | `mods/portal_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-0hi |
| 42 | gravity_gun | `mods/gravity_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-8po |
| 43 | soul_shards | `mods/soul_shards` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-3d4 |
| 44 | xenos_reliquary | `mods/xenos_reliquary` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-c59 |
| 45 | obsidian_pressure_plates | `mods/obsidian_pressure_plates` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-dy0 |
| 46 | xycraft | `mods/xycraft` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-doz |
| 47 | agrarian_skies_2_triage | `planning/agrarianskies2_triage.md` | 4 | — | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | triage only; you approve the port list | rk-ivw |

# Fleet board

Generated from Beads by `make board`: do not edit. Change progress with `bd` (the conductor) and regenerate.
One mod at a time, in this order (`bd ready` gives the next). Steps are the mod's stories; `▶ 12/42` is a
story in progress with 12 of its 42 features or bugs closed.

State: `queued` · `active` · `blocked` · `done`. Steps: `·` todo · `▶` in progress · `✓` done · `✗` blocked · `—` deferred.

| # | Mod | Path | Wave | Tier | Deps | State | 1 Spec | 2 Scaffold | 3 Code | 4 Check | 5 Extract assets | 6 New assets | 7 Recheck | 8 Harness | 9 Package | 10 Human | 11 Own art | Notes | Bead |
|---:|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | rekkit_core | `crates/rekkit_core` | core | Hard | — | growing: 11 for redpower_2 | · | · | · | · | · | · | · | · | · | · | · | shared library complete (`crates/rekkit_core` v0.1.0) | rk-p2g |
| 2 | rekkit_core | `crates/rekkit_core` | core | Hard | — | growing: 11 for redpower_2 | · | · | · | · | · | · | · | · | · | · | · | rk-p2g | rk-lc52 |
| 3 | redpower_2 | `mods/redpower_2` | 1 | Hard | rekkit_core | active | ✓ 4/4 | ✓ 2/2 | · 234/514 | · 11/12 | · 75/225 | ✓ 89/89 | · | · 196/337 | · | · | — | domain (`src/lib.rs`) + unit tests (`tests/mod_tests.rs`) written; content subagents authoring 42 blocks, 29 items, 15 recipes | rk-qst |
| 4 | buildcraft | `mods/buildcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-dvp |
| 5 | industrialcraft_2 | `mods/industrialcraft_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-x3e |
| 6 | equivalent_exchange_2 | `mods/equivalent_exchange_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. EMC over merged recipes | rk-11a |
| 7 | applied_energistics | `mods/applied_energistics` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-3w3 |
| 8 | computercraft | `mods/computercraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. Lua VM | rk-69u |
| 9 | railcraft | `mods/railcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-28f |
| 10 | thermal_expansion | `mods/thermal_expansion` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-6xj |
| 11 | forestry | `mods/forestry` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-4ww |
| 12 | enderstorage | `mods/enderstorage` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. Mostly implemented by example_mod already | rk-0av |
| 13 | redpower_2 | `mods/redpower_2` | 1 | Hard | rekkit_core | queued | ✓ 4/4 | ✓ 2/2 | · 234/514 | · 11/12 | · 75/225 | ✓ 89/89 | · | · 196/337 | · | · | — | rk-qst | rk-a9xh |
| 14 | buildcraft | `mods/buildcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dvp | rk-woni |
| 15 | industrialcraft_2 | `mods/industrialcraft_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-x3e | rk-xwq7 |
| 16 | equivalent_exchange_2 | `mods/equivalent_exchange_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-11a | rk-34iq |
| 17 | applied_energistics | `mods/applied_energistics` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-3w3 | rk-1ki1 |
| 18 | computercraft | `mods/computercraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-69u | rk-eba1 |
| 19 | railcraft | `mods/railcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-28f | rk-1arx |
| 20 | thermal_expansion | `mods/thermal_expansion` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6xj | rk-31d4 |
| 21 | forestry | `mods/forestry` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-4ww | rk-mulq |
| 22 | enderstorage | `mods/enderstorage` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0av | rk-e4da |
| 23 | additional_pipes | `mods/additional_pipes` | 2 | Std | buildcraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-6hk |
| 24 | advanced_machines | `mods/advanced_machines` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-1hg |
| 25 | compact_solars | `mods/compact_solars` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-5vi |
| 26 | charging_bench | `mods/charging_bench` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-6t2 |
| 27 | ic2_nuclear_control | `mods/ic2_nuclear_control` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-xwg |
| 28 | power_converters | `mods/power_converters` | 2 | Std | buildcraft,industrialcraft_2,redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-86f |
| 29 | mffs | `mods/mffs` | 2 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | Modular Force Field System | rk-avs |
| 30 | ccsensors | `mods/ccsensors` | 2 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-zzi |
| 31 | wireless_redstone_cbe | `mods/wireless_redstone_cbe` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | WR-CBE | rk-9s3 |
| 32 | tubestuff | `mods/tubestuff` | 2 | Std | redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-lwe |
| 33 | dimensional_anchors | `mods/dimensional_anchors` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-p5e |
| 34 | nether_ores | `mods/nether_ores` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-aol |
| 35 | iron_chests | `mods/iron_chests` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-dvv |
| 36 | balkons_weaponmod | `mods/balkons_weaponmod` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-vwt |
| 37 | additional_pipes | `mods/additional_pipes` | 2 | Std | buildcraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6hk | rk-0mcm |
| 38 | advanced_machines | `mods/advanced_machines` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-1hg | rk-3avg |
| 39 | compact_solars | `mods/compact_solars` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-5vi | rk-finn |
| 40 | charging_bench | `mods/charging_bench` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6t2 | rk-0tma |
| 41 | ic2_nuclear_control | `mods/ic2_nuclear_control` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-xwg | rk-pueh |
| 42 | power_converters | `mods/power_converters` | 2 | Std | buildcraft,industrialcraft_2,redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-86f | rk-cgnk |
| 43 | mffs | `mods/mffs` | 2 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-avs | rk-440b |
| 44 | ccsensors | `mods/ccsensors` | 2 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-zzi | rk-1no3 |
| 45 | wireless_redstone_cbe | `mods/wireless_redstone_cbe` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-9s3 | rk-n1tl |
| 46 | tubestuff | `mods/tubestuff` | 2 | Std | redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-lwe | rk-ng0l |
| 47 | dimensional_anchors | `mods/dimensional_anchors` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-p5e | rk-apwd |
| 48 | nether_ores | `mods/nether_ores` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-aol | rk-goyi |
| 49 | iron_chests | `mods/iron_chests` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dvv | rk-c0e8 |
| 50 | balkons_weaponmod | `mods/balkons_weaponmod` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-vwt | rk-7zhf |
| 51 | gravitation_suite | `mods/gravitation_suite` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-azt |
| 52 | charge_pads | `mods/charge_pads` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-be1 |
| 53 | modular_powersuits | `mods/modular_powersuits` | 3 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-1rc |
| 54 | omnitools | `mods/omnitools` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-10o |
| 55 | misc_peripherals | `mods/misc_peripherals` | 3 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-k4p |
| 56 | extra_bees | `mods/extra_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-eh9 |
| 57 | thaumic_bees | `mods/thaumic_bees` | 3 | Std | forestry,thaumcraft_3 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-4be |
| 58 | thaumcraft_3 | `mods/thaumcraft_3` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-hic |
| 59 | equivalent_exchange_3 | `mods/equivalent_exchange_3` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | separate id from EE2 | rk-ryo |
| 60 | steves_carts_2 | `mods/steves_carts_2` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-aip |
| 61 | factorization | `mods/factorization` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-m41 |
| 62 | mystcraft | `mods/mystcraft` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | likely engine gap: dimensions | rk-x8u |
| 63 | extrabiomes_xl | `mods/extrabiomes_xl` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | worldgen | rk-0ud |
| 64 | twilight_forest | `mods/twilight_forest` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | likely engine gap: dimensions | rk-dha |
| 65 | chickenchunks | `mods/chickenchunks` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-gwk |
| 66 | portal_gun | `mods/portal_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-0hi |
| 67 | gravity_gun | `mods/gravity_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-8po |
| 68 | soul_shards | `mods/soul_shards` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-3d4 |
| 69 | xenos_reliquary | `mods/xenos_reliquary` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-c59 |
| 70 | obsidian_pressure_plates | `mods/obsidian_pressure_plates` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-dy0 |
| 71 | xycraft | `mods/xycraft` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-doz |
| 72 | gravitation_suite | `mods/gravitation_suite` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-azt | rk-l8xb |
| 73 | charge_pads | `mods/charge_pads` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-be1 | rk-b4ik |
| 74 | modular_powersuits | `mods/modular_powersuits` | 3 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-1rc | rk-gi4y |
| 75 | omnitools | `mods/omnitools` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-10o | rk-w1j6 |
| 76 | misc_peripherals | `mods/misc_peripherals` | 3 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-k4p | rk-t0ht |
| 77 | extra_bees | `mods/extra_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-eh9 | rk-2kkh |
| 78 | thaumic_bees | `mods/thaumic_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-4be | rk-ryx4 |
| 79 | thaumcraft_3 | `mods/thaumcraft_3` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-hic | rk-umou |
| 80 | equivalent_exchange_3 | `mods/equivalent_exchange_3` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-ryo | rk-lj2w |
| 81 | steves_carts_2 | `mods/steves_carts_2` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-aip | rk-rako |
| 82 | factorization | `mods/factorization` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-m41 | rk-y8zi |
| 83 | mystcraft | `mods/mystcraft` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-x8u | rk-12jc |
| 84 | extrabiomes_xl | `mods/extrabiomes_xl` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0ud | rk-wcoo |
| 85 | twilight_forest | `mods/twilight_forest` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dha | rk-yg0t |
| 86 | chickenchunks | `mods/chickenchunks` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-gwk | rk-vbv8 |
| 87 | portal_gun | `mods/portal_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0hi | rk-03nu |
| 88 | gravity_gun | `mods/gravity_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-8po | rk-0pfm |
| 89 | soul_shards | `mods/soul_shards` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-3d4 | rk-q66c |
| 90 | xenos_reliquary | `mods/xenos_reliquary` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-c59 | rk-6rr8 |
| 91 | obsidian_pressure_plates | `mods/obsidian_pressure_plates` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dy0 | rk-ngyf |
| 92 | xycraft | `mods/xycraft` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-doz | rk-x5hb |
| 93 | agrarian_skies_2_triage | `planning/agrarianskies2_triage.md` | 4 | — | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | triage only; you approve the port list | rk-ivw |
| 94 | agrarian_skies_2_triage | `planning/agrarianskies2_triage.md` | 4 | — | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-ivw | rk-5h5q |

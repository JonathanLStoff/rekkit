# Fleet board

Generated from Beads by `make board`: do not edit. Change progress with `bd` (the conductor) and regenerate.
One mod at a time, in this order (`bd ready` gives the next). Steps are the mod's stories; `▶ 12/42` is a
story in progress with 12 of its 42 features or bugs closed.

State: `queued` · `active` · `blocked` · `done`. Steps: `·` todo · `▶` in progress · `✓` done · `✗` blocked · `—` deferred.

| # | Mod | Path | Wave | Tier | Deps | State | 1 Spec | 2 Scaffold | 3 Code | 4 Check | 5 Extract assets | 6 New assets | 7 Recheck | 8 Harness | 9 Package | 10 Human | 11 Own art | Notes | Bead |
|---:|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | rekkit_core | `crates/rekkit_core` | core | Hard | — | growing: 11 for redpower_2 | · | · | · | · | · | · | · | · | · | · | · | shared library complete (`crates/rekkit_core` v0.1.0) | rk-p2g |
| 2 | rekkit_core | `crates/rekkit_core` | core | Hard | — | growing: 11 for redpower_2 | · | · | · | · | · | · | · | · | · | · | · | rk-p2g | rk-lc52 |
| 3 | rekkit_core | `crates/rekkit_core` | core | Hard | — | growing: 11 for redpower_2 | · | · | · | · | · | · | · | · | · | · | · | rk-p2g | rk-skdj |
| 4 | redpower_2 | `mods/redpower_2` | 1 | Hard | rekkit_core | active | ✓ 4/4 | ✓ 2/2 | ✓ 503/503 | ✓ 12/12 | ✓ 225/225 | ✓ 89/89 | ✓ | ✓ 337/337 | ✓ | · | — | domain (`src/lib.rs`) + unit tests (`tests/mod_tests.rs`) written; content subagents authoring 42 blocks, 29 items, 15 recipes | rk-qst |
| 5 | buildcraft | `mods/buildcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-dvp |
| 6 | industrialcraft_2 | `mods/industrialcraft_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-x3e |
| 7 | equivalent_exchange_2 | `mods/equivalent_exchange_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. EMC over merged recipes | rk-11a |
| 8 | applied_energistics | `mods/applied_energistics` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-3w3 |
| 9 | computercraft | `mods/computercraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. Lua VM | rk-69u |
| 10 | railcraft | `mods/railcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-28f |
| 11 | thermal_expansion | `mods/thermal_expansion` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-6xj |
| 12 | forestry | `mods/forestry` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | rk-4ww |
| 13 | enderstorage | `mods/enderstorage` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. Mostly implemented by example_mod already | rk-0av |
| 14 | redpower_2 | `mods/redpower_2` | 1 | Hard | rekkit_core | queued | ✓ 4/4 | ✓ 2/2 | ✓ 503/503 | ✓ 12/12 | ✓ 225/225 | ✓ 89/89 | ✓ | ✓ 337/337 | ✓ | · | — | rk-qst | rk-a9xh |
| 15 | buildcraft | `mods/buildcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dvp | rk-woni |
| 16 | industrialcraft_2 | `mods/industrialcraft_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-x3e | rk-xwq7 |
| 17 | equivalent_exchange_2 | `mods/equivalent_exchange_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-11a | rk-34iq |
| 18 | applied_energistics | `mods/applied_energistics` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-3w3 | rk-1ki1 |
| 19 | computercraft | `mods/computercraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-69u | rk-eba1 |
| 20 | railcraft | `mods/railcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-28f | rk-1arx |
| 21 | thermal_expansion | `mods/thermal_expansion` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6xj | rk-31d4 |
| 22 | forestry | `mods/forestry` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-4ww | rk-mulq |
| 23 | enderstorage | `mods/enderstorage` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0av | rk-e4da |
| 24 | redpower_2 | `mods/redpower_2` | 1 | Hard | rekkit_core | queued | ✓ 4/4 | ✓ 2/2 | ✓ 503/503 | ✓ 12/12 | ✓ 225/225 | ✓ 89/89 | ✓ | ✓ 337/337 | ✓ | · | — | rk-qst | rk-23va |
| 25 | buildcraft | `mods/buildcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dvp | rk-juev |
| 26 | industrialcraft_2 | `mods/industrialcraft_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-x3e | rk-03c1 |
| 27 | equivalent_exchange_2 | `mods/equivalent_exchange_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-11a | rk-90tw |
| 28 | applied_energistics | `mods/applied_energistics` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-3w3 | rk-pvin |
| 29 | computercraft | `mods/computercraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-69u | rk-5gyc |
| 30 | railcraft | `mods/railcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-28f | rk-f80l |
| 31 | thermal_expansion | `mods/thermal_expansion` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6xj | rk-7n26 |
| 32 | forestry | `mods/forestry` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-4ww | rk-a542 |
| 33 | enderstorage | `mods/enderstorage` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0av | rk-q2az |
| 34 | additional_pipes | `mods/additional_pipes` | 2 | Std | buildcraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-6hk |
| 35 | advanced_machines | `mods/advanced_machines` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-1hg |
| 36 | compact_solars | `mods/compact_solars` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-5vi |
| 37 | charging_bench | `mods/charging_bench` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-6t2 |
| 38 | ic2_nuclear_control | `mods/ic2_nuclear_control` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-xwg |
| 39 | power_converters | `mods/power_converters` | 2 | Std | buildcraft,industrialcraft_2,redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-86f |
| 40 | mffs | `mods/mffs` | 2 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | Modular Force Field System | rk-avs |
| 41 | ccsensors | `mods/ccsensors` | 2 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-zzi |
| 42 | wireless_redstone_cbe | `mods/wireless_redstone_cbe` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | WR-CBE | rk-9s3 |
| 43 | tubestuff | `mods/tubestuff` | 2 | Std | redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-lwe |
| 44 | dimensional_anchors | `mods/dimensional_anchors` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-p5e |
| 45 | nether_ores | `mods/nether_ores` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-aol |
| 46 | iron_chests | `mods/iron_chests` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-dvv |
| 47 | balkons_weaponmod | `mods/balkons_weaponmod` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-vwt |
| 48 | additional_pipes | `mods/additional_pipes` | 2 | Std | buildcraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6hk | rk-0mcm |
| 49 | advanced_machines | `mods/advanced_machines` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-1hg | rk-3avg |
| 50 | compact_solars | `mods/compact_solars` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-5vi | rk-finn |
| 51 | charging_bench | `mods/charging_bench` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6t2 | rk-0tma |
| 52 | ic2_nuclear_control | `mods/ic2_nuclear_control` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-xwg | rk-pueh |
| 53 | power_converters | `mods/power_converters` | 2 | Std | buildcraft,industrialcraft_2,redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-86f | rk-cgnk |
| 54 | mffs | `mods/mffs` | 2 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-avs | rk-440b |
| 55 | ccsensors | `mods/ccsensors` | 2 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-zzi | rk-1no3 |
| 56 | wireless_redstone_cbe | `mods/wireless_redstone_cbe` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-9s3 | rk-n1tl |
| 57 | tubestuff | `mods/tubestuff` | 2 | Std | redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-lwe | rk-ng0l |
| 58 | dimensional_anchors | `mods/dimensional_anchors` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-p5e | rk-apwd |
| 59 | nether_ores | `mods/nether_ores` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-aol | rk-goyi |
| 60 | iron_chests | `mods/iron_chests` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dvv | rk-c0e8 |
| 61 | balkons_weaponmod | `mods/balkons_weaponmod` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-vwt | rk-7zhf |
| 62 | additional_pipes | `mods/additional_pipes` | 2 | Std | buildcraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6hk | rk-4hdf |
| 63 | advanced_machines | `mods/advanced_machines` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-1hg | rk-rtd7 |
| 64 | compact_solars | `mods/compact_solars` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-5vi | rk-dlb5 |
| 65 | charging_bench | `mods/charging_bench` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6t2 | rk-m3mc |
| 66 | ic2_nuclear_control | `mods/ic2_nuclear_control` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-xwg | rk-3swx |
| 67 | power_converters | `mods/power_converters` | 2 | Std | buildcraft,industrialcraft_2,redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-86f | rk-9uee |
| 68 | mffs | `mods/mffs` | 2 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-avs | rk-c9nn |
| 69 | ccsensors | `mods/ccsensors` | 2 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-zzi | rk-0gjx |
| 70 | wireless_redstone_cbe | `mods/wireless_redstone_cbe` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-9s3 | rk-8wip |
| 71 | tubestuff | `mods/tubestuff` | 2 | Std | redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-lwe | rk-fmhi |
| 72 | dimensional_anchors | `mods/dimensional_anchors` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-p5e | rk-rwir |
| 73 | nether_ores | `mods/nether_ores` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-aol | rk-vs29 |
| 74 | iron_chests | `mods/iron_chests` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dvv | rk-ru3g |
| 75 | balkons_weaponmod | `mods/balkons_weaponmod` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-vwt | rk-vifh |
| 76 | gravitation_suite | `mods/gravitation_suite` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-azt |
| 77 | charge_pads | `mods/charge_pads` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-be1 |
| 78 | modular_powersuits | `mods/modular_powersuits` | 3 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-1rc |
| 79 | omnitools | `mods/omnitools` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-10o |
| 80 | misc_peripherals | `mods/misc_peripherals` | 3 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-k4p |
| 81 | extra_bees | `mods/extra_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-eh9 |
| 82 | thaumic_bees | `mods/thaumic_bees` | 3 | Std | forestry,thaumcraft_3 | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-4be |
| 83 | thaumcraft_3 | `mods/thaumcraft_3` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-hic |
| 84 | equivalent_exchange_3 | `mods/equivalent_exchange_3` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | separate id from EE2 | rk-ryo |
| 85 | steves_carts_2 | `mods/steves_carts_2` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-aip |
| 86 | factorization | `mods/factorization` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-m41 |
| 87 | mystcraft | `mods/mystcraft` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | likely engine gap: dimensions | rk-x8u |
| 88 | extrabiomes_xl | `mods/extrabiomes_xl` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | worldgen | rk-0ud |
| 89 | twilight_forest | `mods/twilight_forest` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | likely engine gap: dimensions | rk-dha |
| 90 | chickenchunks | `mods/chickenchunks` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-gwk |
| 91 | portal_gun | `mods/portal_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-0hi |
| 92 | gravity_gun | `mods/gravity_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-8po |
| 93 | soul_shards | `mods/soul_shards` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-3d4 |
| 94 | xenos_reliquary | `mods/xenos_reliquary` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-c59 |
| 95 | obsidian_pressure_plates | `mods/obsidian_pressure_plates` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-dy0 |
| 96 | xycraft | `mods/xycraft` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | rk-doz |
| 97 | gravitation_suite | `mods/gravitation_suite` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-azt | rk-l8xb |
| 98 | charge_pads | `mods/charge_pads` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-be1 | rk-b4ik |
| 99 | modular_powersuits | `mods/modular_powersuits` | 3 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-1rc | rk-gi4y |
| 100 | omnitools | `mods/omnitools` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-10o | rk-w1j6 |
| 101 | misc_peripherals | `mods/misc_peripherals` | 3 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-k4p | rk-t0ht |
| 102 | extra_bees | `mods/extra_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-eh9 | rk-2kkh |
| 103 | thaumic_bees | `mods/thaumic_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-4be | rk-ryx4 |
| 104 | thaumcraft_3 | `mods/thaumcraft_3` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-hic | rk-umou |
| 105 | equivalent_exchange_3 | `mods/equivalent_exchange_3` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-ryo | rk-lj2w |
| 106 | steves_carts_2 | `mods/steves_carts_2` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-aip | rk-rako |
| 107 | factorization | `mods/factorization` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-m41 | rk-y8zi |
| 108 | mystcraft | `mods/mystcraft` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-x8u | rk-12jc |
| 109 | extrabiomes_xl | `mods/extrabiomes_xl` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0ud | rk-wcoo |
| 110 | twilight_forest | `mods/twilight_forest` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dha | rk-yg0t |
| 111 | chickenchunks | `mods/chickenchunks` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-gwk | rk-vbv8 |
| 112 | portal_gun | `mods/portal_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0hi | rk-03nu |
| 113 | gravity_gun | `mods/gravity_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-8po | rk-0pfm |
| 114 | soul_shards | `mods/soul_shards` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-3d4 | rk-q66c |
| 115 | xenos_reliquary | `mods/xenos_reliquary` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-c59 | rk-6rr8 |
| 116 | obsidian_pressure_plates | `mods/obsidian_pressure_plates` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dy0 | rk-ngyf |
| 117 | xycraft | `mods/xycraft` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-doz | rk-x5hb |
| 118 | gravitation_suite | `mods/gravitation_suite` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-azt | rk-usei |
| 119 | charge_pads | `mods/charge_pads` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-be1 | rk-l0n3 |
| 120 | modular_powersuits | `mods/modular_powersuits` | 3 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-1rc | rk-cvoe |
| 121 | omnitools | `mods/omnitools` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-10o | rk-jy7k |
| 122 | misc_peripherals | `mods/misc_peripherals` | 3 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-k4p | rk-qzwr |
| 123 | extra_bees | `mods/extra_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-eh9 | rk-55k0 |
| 124 | thaumic_bees | `mods/thaumic_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-4be | rk-ycvf |
| 125 | thaumcraft_3 | `mods/thaumcraft_3` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-hic | rk-g99v |
| 126 | equivalent_exchange_3 | `mods/equivalent_exchange_3` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-ryo | rk-sn8c |
| 127 | steves_carts_2 | `mods/steves_carts_2` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-aip | rk-h93n |
| 128 | factorization | `mods/factorization` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-m41 | rk-w3hh |
| 129 | mystcraft | `mods/mystcraft` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-x8u | rk-hb4h |
| 130 | extrabiomes_xl | `mods/extrabiomes_xl` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0ud | rk-ptoq |
| 131 | twilight_forest | `mods/twilight_forest` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dha | rk-ifa0 |
| 132 | chickenchunks | `mods/chickenchunks` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-gwk | rk-hop0 |
| 133 | portal_gun | `mods/portal_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0hi | rk-xot3 |
| 134 | gravity_gun | `mods/gravity_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-8po | rk-924p |
| 135 | soul_shards | `mods/soul_shards` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-3d4 | rk-elfh |
| 136 | xenos_reliquary | `mods/xenos_reliquary` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-c59 | rk-xzjl |
| 137 | obsidian_pressure_plates | `mods/obsidian_pressure_plates` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dy0 | rk-z9kk |
| 138 | xycraft | `mods/xycraft` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-doz | rk-f4ai |
| 139 | agrarian_skies_2_triage | `planning/agrarianskies2_triage.md` | 4 | — | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | triage only; you approve the port list | rk-ivw |
| 140 | agrarian_skies_2_triage | `planning/agrarianskies2_triage.md` | 4 | — | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-ivw | rk-5h5q |
| 141 | agrarian_skies_2_triage | `planning/agrarianskies2_triage.md` | 4 | — | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-ivw | rk-c2uo |

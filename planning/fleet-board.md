# Fleet board

Generated from `issues/` by `make board`: do not edit. Change progress with `issuectl` (the conductor) and regenerate.
One mod at a time, in this order (`issuectl dag` gives the next). Steps are the mod's stories; `▶ 12/42` is a
story in progress with 12 of its 42 features or bugs closed.

State: `queued` · `active` · `blocked` · `done`. Steps: `·` todo · `▶` in progress · `✓` done · `✗` blocked · `—` deferred.

| # | Mod | Path | Wave | Tier | Deps | State | 1 Spec | 2 Scaffold | 3 Code | 4 Check | 5 Extract assets | 6 New assets | 7 Recheck | 8 Harness | 9 Package | 10 Human | 11 Own art | Notes | Issue |
|---:|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | computercraft | `mods/computercraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. Lua VM | absolutely-strange-point |
| 2 | buildcraft | `mods/buildcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | absurdly-gabby-corn |
| 3 | applied_energistics | `mods/applied_energistics` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-3w3 | applied-energistics |
| 4 | forestry | `mods/forestry` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | spec exists; gap check | downright-abject-wax |
| 5 | equivalent_exchange_2 | `mods/equivalent_exchange_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-11a | equivalent-exchange-2 |
| 6 | industrialcraft_2 | `mods/industrialcraft_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-x3e | industrialcraft-2 |
| 7 | railcraft | `mods/railcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-28f | profoundly-pastoral-force |
| 8 | redpower_2 | `mods/redpower_2` | 1 | Hard | rekkit_core | blocked | ✓ 4/4 | ✓ 2/2 | ✓ 503/503 | ✓ 12/12 | ✓ 225/225 | ✓ 89/89 | ✓ | ✓ 337/337 | ✓ | · | — | domain (`src/lib.rs`) + unit tests (`tests/mod_tests.rs`) written; content subagents authoring 42 blocks, 29 items, 15 recipes | redpower-2-3 |
| 9 | rekkit_core | `crates/rekkit_core` | core | Hard | — | growing: 11 for redpower_2 | · | · | · | · | · | · | · | · | · | · | · | shared library complete (`crates/rekkit_core` v0.1.0) | rekkit-core-3 |
| 10 | enderstorage | `mods/enderstorage` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0av | seriously-plastic-glass |
| 11 | thermal_expansion | `mods/thermal_expansion` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6xj | thermal-expansion |
| 12 | mffs | `mods/mffs` | 2 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-avs | acceptably-demonic-door |
| 13 | ccsensors | `mods/ccsensors` | 2 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · |  | acceptably-dirty-gold |
| 14 | additional_pipes | `mods/additional_pipes` | 2 | Std | buildcraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6hk | additional-pipes |
| 15 | advanced_machines | `mods/advanced_machines` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-1hg | advanced-machines |
| 16 | balkons_weaponmod | `mods/balkons_weaponmod` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-vwt | balkons-weaponmod |
| 17 | charging_bench | `mods/charging_bench` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-6t2 | charging-bench |
| 18 | compact_solars | `mods/compact_solars` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-5vi | compact-solars |
| 19 | dimensional_anchors | `mods/dimensional_anchors` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-p5e | dimensional-anchors |
| 20 | ic2_nuclear_control | `mods/ic2_nuclear_control` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-xwg | ic2-nuclear-control |
| 21 | iron_chests | `mods/iron_chests` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dvv | iron-chests |
| 22 | nether_ores | `mods/nether_ores` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-aol | nether-ores |
| 23 | power_converters | `mods/power_converters` | 2 | Std | buildcraft,industrialcraft_2,redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-86f | power-converters |
| 24 | tubestuff | `mods/tubestuff` | 2 | Std | redpower_2 | queued | · | · | · | · | · | · | · | · | · | · | · |  | quite-dead-walk |
| 25 | wireless_redstone_cbe | `mods/wireless_redstone_cbe` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-9s3 | wireless-redstone-cbe |
| 26 | factorization | `mods/factorization` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-m41 | absolutely-wanting-sail |
| 27 | agrarian_skies_2_triage | `planning/agrarianskies2_triage.md` | 4 | — | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-ivw | agrarian-skies-2 |
| 28 | charge_pads | `mods/charge_pads` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-be1 | charge-pads |
| 29 | xycraft | `mods/xycraft` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-doz | downright-obtainable-carpenter |
| 30 | equivalent_exchange_3 | `mods/equivalent_exchange_3` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-ryo | equivalent-exchange-3 |
| 31 | chickenchunks | `mods/chickenchunks` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | essentially-bawdy-note |
| 32 | omnitools | `mods/omnitools` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · |  | essentially-faded-organization |
| 33 | extra_bees | `mods/extra_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-eh9 | extra-bees |
| 34 | extrabiomes_xl | `mods/extrabiomes_xl` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0ud | extrabiomes-xl |
| 35 | gravitation_suite | `mods/gravitation_suite` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-azt | gravitation-suite |
| 36 | gravity_gun | `mods/gravity_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-8po | gravity-gun |
| 37 | misc_peripherals | `mods/misc_peripherals` | 3 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · | · | rk-k4p | misc-peripherals |
| 38 | modular_powersuits | `mods/modular_powersuits` | 3 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | · | rk-1rc | modular-powersuits |
| 39 | obsidian_pressure_plates | `mods/obsidian_pressure_plates` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dy0 | obsidian-pressure-plates |
| 40 | portal_gun | `mods/portal_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-0hi | portal-gun |
| 41 | mystcraft | `mods/mystcraft` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-x8u | pretty-right-country |
| 42 | soul_shards | `mods/soul_shards` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-3d4 | soul-shards |
| 43 | steves_carts_2 | `mods/steves_carts_2` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-aip | steves-carts-2 |
| 44 | thaumcraft_3 | `mods/thaumcraft_3` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-hic | thaumcraft-3 |
| 45 | thaumic_bees | `mods/thaumic_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · | · | rk-4be | thaumic-bees |
| 46 | twilight_forest | `mods/twilight_forest` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-dha | twilight-forest |
| 47 | xenos_reliquary | `mods/xenos_reliquary` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | · | rk-c59 | xenos-reliquary |

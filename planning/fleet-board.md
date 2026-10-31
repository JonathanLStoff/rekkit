# Fleet board

Owned by the conductor. Order: `rekkit_core` first, then Wave 1 in `planning/priority.md` order (if the two disagree, priority.md wins and the conductor reorders this table), then the other Tekkit mods, then Direwolf20, then Agrarian Skies 2. One mod at a time: at most one row is `active`. `scripts/fleet/run-next.sh` resumes the active row, else starts the first `queued` row whose Deps are all `done`.

Step 7 (E2E) is `✓` only after a live run. Until the machine has Minecraft 26.3 resources, a dry run passing is `◐` (dry run only) and does not block the next steps; re-run live later.

State: `queued` · `active` · `blocked` · `done`. Steps: `·` todo · `▶` in progress · `✓` done · `✗` failed · `—` n/a.

| # | Mod | Path | Wave | Tier | Deps | State | 1 Spec | 2 Scaffold | 3 Tests | 4 Extract | 5 Assets | 6 Validate | 7 E2E | 8 Version | 9 Push | 10 Release | Notes |
|---:|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | rekkit_core | `crates/rekkit_core` | core | Hard | — | queued | · | · | · | — | — | · | — | · | · | — | shared library; no content, native, E2E or release |
| 2 | redpower_2 | `mods/redpower_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check |
| 3 | buildcraft | `mods/buildcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check |
| 4 | industrialcraft_2 | `mods/industrialcraft_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check |
| 5 | equivalent_exchange_2 | `mods/equivalent_exchange_2` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. EMC over merged recipes |
| 6 | computercraft | `mods/computercraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. Lua VM |
| 7 | applied_energistics | `mods/applied_energistics` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check |
| 8 | railcraft | `mods/railcraft` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check |
| 9 | thermal_expansion | `mods/thermal_expansion` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check |
| 10 | forestry | `mods/forestry` | 1 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check |
| 11 | enderstorage | `mods/enderstorage` | 1 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | spec exists; gap check. Mostly implemented by example_mod already |
| 12 | additional_pipes | `mods/additional_pipes` | 2 | Std | buildcraft | queued | · | · | · | · | · | · | · | · | · | · |  |
| 13 | advanced_machines | `mods/advanced_machines` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · |  |
| 14 | charging_bench | `mods/charging_bench` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · |  |
| 15 | compact_solars | `mods/compact_solars` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · |  |
| 16 | ic2_nuclear_control | `mods/ic2_nuclear_control` | 2 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · |  |
| 17 | mffs | `mods/mffs` | 2 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · | Modular Force Field System |
| 18 | power_converters | `mods/power_converters` | 2 | Std | buildcraft,industrialcraft_2,redpower_2 | queued | · | · | · | · | · | · | · | · | · | · |  |
| 19 | ccsensors | `mods/ccsensors` | 2 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · |  |
| 20 | tubestuff | `mods/tubestuff` | 2 | Std | redpower_2 | queued | · | · | · | · | · | · | · | · | · | · |  |
| 21 | wireless_redstone_cbe | `mods/wireless_redstone_cbe` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | WR-CBE |
| 22 | dimensional_anchors | `mods/dimensional_anchors` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 23 | iron_chests | `mods/iron_chests` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 24 | nether_ores | `mods/nether_ores` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 25 | balkons_weaponmod | `mods/balkons_weaponmod` | 2 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 26 | charge_pads | `mods/charge_pads` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · |  |
| 27 | gravitation_suite | `mods/gravitation_suite` | 3 | Std | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · |  |
| 28 | modular_powersuits | `mods/modular_powersuits` | 3 | Hard | industrialcraft_2 | queued | · | · | · | · | · | · | · | · | · | · |  |
| 29 | omnitools | `mods/omnitools` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 30 | misc_peripherals | `mods/misc_peripherals` | 3 | Std | computercraft | queued | · | · | · | · | · | · | · | · | · | · |  |
| 31 | extra_bees | `mods/extra_bees` | 3 | Std | forestry | queued | · | · | · | · | · | · | · | · | · | · |  |
| 32 | thaumcraft_3 | `mods/thaumcraft_3` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 33 | thaumic_bees | `mods/thaumic_bees` | 3 | Std | thaumcraft_3,forestry | queued | · | · | · | · | · | · | · | · | · | · |  |
| 34 | equivalent_exchange_3 | `mods/equivalent_exchange_3` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | separate id from EE2 |
| 35 | factorization | `mods/factorization` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 36 | steves_carts_2 | `mods/steves_carts_2` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 37 | mystcraft | `mods/mystcraft` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | likely engine gap: dimensions |
| 38 | twilight_forest | `mods/twilight_forest` | 3 | Hard | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | likely engine gap: dimensions |
| 39 | extrabiomes_xl | `mods/extrabiomes_xl` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · | worldgen |
| 40 | chickenchunks | `mods/chickenchunks` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 41 | gravity_gun | `mods/gravity_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 42 | portal_gun | `mods/portal_gun` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 43 | soul_shards | `mods/soul_shards` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 44 | obsidian_pressure_plates | `mods/obsidian_pressure_plates` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 45 | xenos_reliquary | `mods/xenos_reliquary` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 46 | xycraft | `mods/xycraft` | 3 | Std | rekkit_core | queued | · | · | · | · | · | · | · | · | · | · |  |
| 47 | agrarian_skies_2_triage | `planning/agrarianskies2_triage.md` | 4 | — | forestry | queued | · | — | — | — | — | — | — | — | — | — | triage only; you approve the port list |

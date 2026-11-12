---
created: 2026-11-12
updated: 2026-11-12
type: task
status: done
priority: normal
labels:
- key:part02-blutricity-conductor
- mod
- mod:redpower_2
- needs:rekkit_core
- path:mods/redpower_2
- role:domain-builder-pro
- step:3
- ticket
- tier:Hard
- wave:1
beads_id: rk-7brh
epic: redpower-2-3-2
blocked_by: ['@redpower-2-core-5']
closed: 2026-11-12
---

# redpower_2: Blutricity Dynamic RLC Circuit Simulation Engine

Implement the complete RedPower 2 Blutricity physical model (BluePowerConductor and BluePowerEndpoint): 100.0V nominal, 60.0V minimum machine threshold, 120.0V safe operating limit, R=0.01 ohm conductor resistance, inductive scaling step factor 0.07, parallel conductance 0.5, capacitance (endpoint 4.0F, panel 0.25F), and NBT charge persistence. Done: unit tests validating exact dynamic current iteration step, voltage calculation, parallel flow, and endpoint charge storage pass.

Size: ~110k tokens.
Read first: spec 2.5 Blutricity Power Grid, 11. Uses from `rekkit_core`
Java: BluePowerConductor#iterate, BluePowerConductor#recache, BluePowerConductor#getVoltage, BluePowerConductor#applyCurrent, BluePowerConductor#drawPower, BluePowerEndpoint#iterate
Files: mods/redpower_2/src/core/conductor.rs

## Beads import

- Beads id: `rk-7brh`
- Beads status: `closed`
- Beads priority: `2`
- Beads type: `task`
- Created (Beads): 2026-11-03T03:29:43Z
- Closed (Beads): 2026-11-03T22:05:05Z
- Close reason (Beads): Faithful BluePowerConductor/BluePowerEndpoint RLC engine now lives at mods/redpower_2/src/core/conductor.rs with 14 unit tests covering the exact iteration step, the once-per-tick voltage update, the

## Comments

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T03:35:43Z · Jonathan Stoff] [domain-builder-pro] partial: Faithful BluePowerConductor/Endpoint RLC engine ported to mods/redpower_2/src/wiring/blutricity.rs (lazy per-tick voltage, per-slot inductor currents, recache, iterate over a BlueGrid, apply/draw power, NBT). Not yet wired into lib.rs device structs.
Files: mods/redpower_2/src/wiring/blutricity.rs, mods/redpower_2/src/wiring/mod.rs
cargo test blutricity: 8 pass; clippy clean for new file (one preexisting warning in world/athame.rs). Deliberately did not reuse rekkit_core BlutricityNode: it is a static charge/ohm model with no per-connection inductor current or lazy tick voltage, so it cannot express the original. The older simplified BluePowerConductor in lib.rs (step_iterate) is left in place because solar/thermopile/battery/transformer and tests use it.

### 2026-11-12T16:48:02Z · @beads-import

[2026-11-03T22:04:55Z · Jonathan Stoff] [domain-builder-pro] done: Faithful BluePowerConductor/BluePowerEndpoint RLC engine now lives at mods/redpower_2/src/core/conductor.rs with 14 unit tests covering the exact iteration step, the once-per-tick voltage update, the pre-update parallel flow term, and endpoint charge/flow storage plus persistence.
Files: mods/redpower_2/src/core/conductor.rs, mods/redpower_2/src/core/mod.rs, mods/redpower_2/src/wiring/blutricity.rs, mods/redpower_2/src/lib.rs, target/fleet/progress/rk-7brh.md
ENGINE: core/conductor.rs holds BlueConductor (imask/currents/Vcap/Icap/Veff/lastTick/It1/Itot/invCap), BlueGrid (links + iterate + tick) and the endpoint step, ported method by method from com.eloraam.redpower.core.BluePowerConductor (2.0pr6, redpowercore) and BluePowerEndpoint. recache now uses the real 30-bit connectable mask: CON_DIR_MASK[6] = getConDirMask values (bits 0..23 edges, 24..29 faces) selects the three axis slots and DIRMAP selects the twelve edge slots; the old code treated conm as a 6-bit face mask. Added charge_scaled/flow_scaled (endpoint gauges), remove_slot, BlueGrid::remove, endpoint NBT keys chg/flw, and load now keeps the original's 'currents must match the mask' guard. ITERATION: I_flow = I(pre-update) + V*0.5, matching the Java (see follow_up about the spec formula). The previous agent's wiring/blutricity.rs was moved here and is now a re-export, so machine::magtube and machine::frame keep compiling; lib.rs re-exports BlueConductor/BlueGrid. TESTS (14 in core::conductor::tests): exact_current_iteration_step (7.0 A first step, 13.8152 A second with a live inductor), voltage_calculation_is_exact_and_settles_once_per_tick, voltage_wraps_at_65536_ticks, parallel_flow_uses_pre_update_current (60 A, not 66.986), parallel_links_carry_separate_currents, endpoint_charge_storage_and_flow_history (800/400/600 and Flow bits 1/2/5), endpoint_charge_persists, load_ignores_scalars_when_current_list_is_short, recache_maps_axis_and_edge_bits, draw_and_apply_power_match_the_formulas, energy_above_threshold, missing_neighbour_skips_its_slot, grid_tick_runs_every_node_and_endpoint_step, remove_drops_links. NOT DONE, deliberately: the device states in lib.rs (SolarPanelState, ThermopileState, WindTurbineState, BatteryBoxState, TransformerState) still use the older simplified BluePowerConductor in lib.rs; those structs are the subject of rk-30hm / rk-2mws / rk-8tpx / rk-as8b and switching them here would collide with those beads (follow_up).

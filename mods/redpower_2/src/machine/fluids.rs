//! RedPower 2 fluid transport system: Fluid Pipe, Blulectric Pump, and Grate.
//!
//! Unifies:
//! - Fluid Pipes (`TilePipe`, `PipeWorld`): pressure-driven fluid flow, flanges, and cover occlusion.
//! - Blulectric Pump (`TilePump`): creates +1000 outlet and -1000 inlet pressure, pumping up to 1 bucket/tick at >= 60V.
//! - Grate (`TileGrate`, `GratePathfinder`): suction-driven liquid block intake and pressure-driven liquid pool drainage.
//! - Voltmeter pipe pressure measurement.

pub use super::grate::{GratePathfinder, GrateWorld, TileGrate, FULL_BLOCK, GRATE_THRESHOLD};
pub use super::pipe::{
    average_pressure, FluidBuffer, Pipe, PipeWorld, PIPE_MAX_LEVEL, PRESSURE_FLOW, TANK_PRESSURE,
};
pub use crate::core::math::rot_to_side;
pub use crate::fluid::pump::TilePump;
use crate::WorldCoord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block IDs for fluid devices.
pub const PIPE_BLOCK_ID: &str = "redpower_2:pipe";
pub const PUMP_BLOCK_ID: &str = "redpower_2:pump";
pub const GRATE_BLOCK_ID: &str = "redpower_2:grate";

/// Integrated Fluid Network simulating pipes, pumps, and grates together.
#[derive(Debug, Default, Clone)]
pub struct FluidNetwork {
    pub pipes: PipeWorld,
    pub pumps: HashMap<WorldCoord, TilePump>,
    pub grates: HashMap<WorldCoord, TileGrate>,
}

impl FluidNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pipe to the network.
    pub fn add_pipe(&mut self, pos: WorldCoord, pipe: Pipe) {
        self.pipes.pipes.insert(pos, pipe);
    }

    /// Add a pump to the network.
    pub fn add_pump(&mut self, pos: WorldCoord, pump: TilePump) {
        self.pumps.insert(pos, pump);
    }

    /// Add a grate to the network.
    pub fn add_grate(&mut self, pos: WorldCoord, grate: TileGrate) {
        self.grates.insert(pos, grate);
    }

    /// Measure pressure at a pipe coordinate (for Voltmeter L2 scenario).
    pub fn measure_pressure(&self, pos: WorldCoord) -> Option<i32> {
        self.pipes.pipes.get(&pos).map(|p| p.pressure)
    }

    /// Calculate pressure propagation across all connected pipes, pumps, and grates.
    pub fn update_pressures(&mut self) {
        // Collect pressures from pumps
        let mut fixed_pressures: HashMap<WorldCoord, [Option<i32>; 6]> = HashMap::new();

        for (&pump_pos, pump) in &self.pumps {
            if pump.active {
                let rt = rot_to_side(pump.rotation);
                let outlet_side = rt as usize;
                let inlet_side = (rt ^ 1) as usize;

                if outlet_side < 6 {
                    let out_pos = pump_pos.coord_step(rt);
                    let entry = fixed_pressures.entry(out_pos).or_insert([None; 6]);
                    entry[inlet_side] = Some(1000);
                }
                if inlet_side < 6 {
                    let in_pos = pump_pos.coord_step(rt ^ 1);
                    let entry = fixed_pressures.entry(in_pos).or_insert([None; 6]);
                    entry[outlet_side] = Some(-1000);
                }
            }
        }

        // Snapshot current pipe pressures to prevent simultaneous borrow conflicts
        let pipe_snapshots: HashMap<WorldCoord, i32> = self
            .pipes
            .pipes
            .iter()
            .map(|(&pos, p)| (pos, p.pressure))
            .collect();

        // Propagate pressures into pipes
        for (&pos, pipe) in self.pipes.pipes.iter_mut() {
            let mut neighbor_pressures = [None; 6];
            for i in 0..6 {
                let neighbor_pos = pos.coord_step(i as i32);
                if let Some(fixed) = fixed_pressures.get(&pos) {
                    if let Some(p) = fixed[i] {
                        neighbor_pressures[i] = Some(p);
                        continue;
                    }
                }
                if let Some(&p) = pipe_snapshots.get(&neighbor_pos) {
                    neighbor_pressures[i] = Some(p);
                }
            }
            pipe.pressure = average_pressure(&neighbor_pressures);
        }

        // Feed updated pipe pressures to adjacent grates
        for (&grate_pos, grate) in self.grates.iter_mut() {
            let pipe_side = grate.rotation;
            let pipe_pos = grate_pos.coord_step(pipe_side);
            if let Some(pipe) = self.pipes.pipes.get(&pipe_pos) {
                grate.pressure = pipe.pressure;
            }
        }
    }

    /// Advance one simulation tick: run pump mechanics, pipe transfers, and grates.
    pub fn tick(&mut self, world: &dyn GrateWorld, world_tick: i64) {
        // 1. Tick pumps
        for pump in self.pumps.values_mut() {
            pump.tick();
        }

        // 2. Transfer between pumps and attached pipes
        for (&pump_pos, pump) in self.pumps.iter_mut() {
            let rt = rot_to_side(pump.rotation);

            // Sucking from inlet pipe into pump inbuf
            let in_pos = pump_pos.coord_step(rt ^ 1);
            if let Some(in_pipe) = self.pipes.pipes.get_mut(&in_pos) {
                let lvl = in_pipe.buffer.get_level(world_tick);
                if lvl > 0
                    && (pump.inbuf.fluid_type == 0
                        || pump.inbuf.fluid_type == in_pipe.buffer.fluid_type)
                {
                    let transfer = lvl
                        .min(PIPE_MAX_LEVEL - pump.inbuf.level)
                        .min(PRESSURE_FLOW * 4);
                    if transfer > 0 {
                        let ft = in_pipe.buffer.fluid_type;
                        pump.inbuf.add_level(ft, transfer);
                        in_pipe.buffer.add_level(ft, -transfer);
                    }
                }
            }

            // Pushing from pump outbuf into outlet pipe
            let out_pos = pump_pos.coord_step(rt);
            if let Some(out_pipe) = self.pipes.pipes.get_mut(&out_pos) {
                let lvl = pump.outbuf.get_level(world_tick);
                if lvl > 0
                    && (out_pipe.buffer.fluid_type == 0
                        || out_pipe.buffer.fluid_type == pump.outbuf.fluid_type)
                {
                    let transfer = lvl
                        .min(PIPE_MAX_LEVEL - out_pipe.buffer.level)
                        .min(PRESSURE_FLOW * 4);
                    if transfer > 0 {
                        let ft = pump.outbuf.fluid_type;
                        out_pipe.buffer.add_level(ft, transfer);
                        pump.outbuf.add_level(ft, -transfer);
                    }
                }
            }
        }

        // 3. Update pressure gradients
        self.update_pressures();

        // 4. Move liquid between adjacent pipes
        let pipe_positions: Vec<WorldCoord> = self.pipes.pipes.keys().copied().collect();
        for &pos in &pipe_positions {
            let con = self.pipes.connections(pos);
            self.pipes.move_liquid(pos, con, world_tick);
        }

        // 5. Tick grates against world blocks
        for (&grate_pos, grate) in self.grates.iter_mut() {
            let front_pos = grate_pos.coord_step(grate.rotation ^ 1);
            let front_pressure = self.pipes.pipes.get(&front_pos).map(|p| p.pressure);
            grate.tick(front_pressure, world);
        }
    }
}

/// Chunk reload serialization record for fluid systems (Level 5 contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidPersistenceState {
    pub pump_states: HashMap<String, TilePump>,
    pub pipe_buffers: HashMap<String, FluidBuffer>,
}

impl FluidPersistenceState {
    pub fn capture(net: &FluidNetwork) -> Self {
        let mut pump_states = HashMap::new();
        for (pos, pump) in &net.pumps {
            pump_states.insert(format!("{},{},{}", pos.x, pos.y, pos.z), pump.clone());
        }

        let mut pipe_buffers = HashMap::new();
        for (pos, pipe) in &net.pipes.pipes {
            pipe_buffers.insert(format!("{},{},{}", pos.x, pos.y, pos.z), pipe.buffer);
        }

        Self {
            pump_states,
            pipe_buffers,
        }
    }

    pub fn restore(&self, net: &mut FluidNetwork) {
        for (pos_str, pump) in &self.pump_states {
            let coords: Vec<i32> = pos_str.split(',').filter_map(|s| s.parse().ok()).collect();
            if coords.len() == 3 {
                let pos = WorldCoord {
                    x: coords[0],
                    y: coords[1],
                    z: coords[2],
                };
                net.pumps.insert(pos, pump.clone());
            }
        }

        for (pos_str, &buf) in &self.pipe_buffers {
            let coords: Vec<i32> = pos_str.split(',').filter_map(|s| s.parse().ok()).collect();
            if coords.len() == 3 {
                let pos = WorldCoord {
                    x: coords[0],
                    y: coords[1],
                    z: coords[2],
                };
                if let Some(pipe) = net.pipes.pipes.get_mut(&pos) {
                    pipe.buffer = buf;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[derive(Default)]
    struct MockWorld {
        pub blocks: HashMap<WorldCoord, (i32, i32)>, // fluid_id, level
        pub air: HashSet<WorldCoord>,
    }

    impl GrateWorld for MockWorld {
        fn liquid_id(&self, pos: WorldCoord) -> i32 {
            self.blocks.get(&pos).map(|(id, _)| *id).unwrap_or(0)
        }
        fn fluid_id_at(&self, fluid_id: i32, pos: WorldCoord) -> i32 {
            self.blocks
                .get(&pos)
                .filter(|(id, _)| *id == fluid_id)
                .map(|(id, _)| *id)
                .unwrap_or(0)
        }
        fn fluid_level(&self, _fluid_id: i32, pos: WorldCoord) -> i32 {
            self.blocks.get(&pos).map(|(_, l)| *l).unwrap_or(0)
        }
        fn set_fluid_level(&mut self, _fluid_id: i32, _pos: WorldCoord, _level: i32) -> bool {
            true
        }
        fn quanta(&self, _fluid_id: i32) -> i32 {
            1000
        }
        fn is_air(&self, pos: WorldCoord) -> bool {
            self.air.contains(&pos) || !self.blocks.contains_key(&pos)
        }
        fn remove_block(&mut self, pos: WorldCoord) {
            self.blocks.remove(&pos);
            self.air.insert(pos);
        }
    }

    #[test]
    fn test_fluid_pipe_pressure_conduction() {
        let mut net = FluidNetwork::new();
        let p1 = WorldCoord { x: 0, y: 0, z: 0 };
        let p2 = WorldCoord { x: 1, y: 0, z: 0 };

        let mut pipe1 = Pipe::default();
        let pipe2 = Pipe::default();
        pipe1.pressure = 500;

        net.add_pipe(p1, pipe1);
        net.add_pipe(p2, pipe2);

        // Voltmeter pressure measurement (L2 scenario)
        assert_eq!(net.measure_pressure(p1), Some(500));
        assert_eq!(net.measure_pressure(p2), Some(0));

        net.update_pressures();
        // Pipe 2 pressure rises due to conduction
        assert!(net.measure_pressure(p2).unwrap() > 0);
    }

    #[test]
    fn test_pump_generates_suction_and_pressure() {
        let mut net = FluidNetwork::new();
        let pump_pos = WorldCoord { x: 0, y: 0, z: 0 };

        let mut pump = TilePump::new(0); // rot 0: rot_to_side is 5 (East / +x), inlet is 4 (West / -x)
        pump.cond.add_charge(400.0); // 100V
        pump.powered = true;
        pump.tick();
        assert!(pump.active);

        net.add_pump(pump_pos, pump);

        let inlet_pipe_pos = WorldCoord { x: -1, y: 0, z: 0 };
        let outlet_pipe_pos = WorldCoord { x: 1, y: 0, z: 0 };

        net.add_pipe(inlet_pipe_pos, Pipe::default());
        net.add_pipe(outlet_pipe_pos, Pipe::default());

        net.update_pressures();

        // Outlet pipe has positive pressure, inlet has negative suction
        let p_out = net.measure_pressure(outlet_pipe_pos).unwrap();
        let p_in = net.measure_pressure(inlet_pipe_pos).unwrap();

        assert!(p_out > 0, "outlet pressure must be positive, got {}", p_out);
        assert!(
            p_in < 0,
            "inlet pressure must be negative (suction), got {}",
            p_in
        );
    }

    #[test]
    fn test_grate_and_chunk_persistence() {
        let mut net = FluidNetwork::new();
        let pump_pos = WorldCoord { x: 0, y: 0, z: 0 };
        let mut pump = TilePump::new(0);
        pump.inbuf.fluid_type = 1;
        pump.inbuf.level = 500;
        net.add_pump(pump_pos, pump);

        let grate_pos = WorldCoord { x: 2, y: 0, z: 0 };
        let grate = TileGrate::new(4); // Facing West
        net.add_grate(grate_pos, grate);

        let snap = FluidPersistenceState::capture(&net);
        let json = serde_json::to_string(&snap).expect("serialize fluid state");
        let restored: FluidPersistenceState = serde_json::from_str(&json).expect("deserialize");

        let mut new_net = FluidNetwork::new();
        restored.restore(&mut new_net);

        assert!(new_net.pumps.contains_key(&pump_pos));
        assert_eq!(new_net.pumps[&pump_pos].inbuf.level, 500);

        let mock_world = MockWorld::default();
        new_net.tick(&mock_world, 1);
    }
}

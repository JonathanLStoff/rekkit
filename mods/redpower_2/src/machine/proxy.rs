//! Machine proxy registration for RedPower 2 (`redpower_2`).
//!
//! Mirrors `MachineProxy` (GUI id -> tile/container/screen) and
//! `MachineProxyClient#registerRenderers` (block meta -> renderer, special
//! tile renderers). Pure data tables; the engine glue reads them.

use super::base::{Machine2Kind, MachineKind, MachinePanelKind};

/// GUI ids handled by `MachineProxy#getClientGuiElement` / `getServerGuiElement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GuiId {
    Deployer = 1,
    Filter = 2,
    BlueFurnace = 3,
    BufferChest = 4,
    Sorter = 5,
    ItemDetector = 6,
    Retriever = 7,
    BatteryBox = 8,
    Regulator = 9,
    BlueAlloyFurnace = 10,
    Assembler = 11,
    Ejector = 12,
    Relay = 13,
    ChargingBench = 14,
    WindTurbine = 15,
    Manager = 16,
}

impl GuiId {
    pub const ALL: [Self; 16] = [
        Self::Deployer,
        Self::Filter,
        Self::BlueFurnace,
        Self::BufferChest,
        Self::Sorter,
        Self::ItemDetector,
        Self::Retriever,
        Self::BatteryBox,
        Self::Regulator,
        Self::BlueAlloyFurnace,
        Self::Assembler,
        Self::Ejector,
        Self::Relay,
        Self::ChargingBench,
        Self::WindTurbine,
        Self::Manager,
    ];

    /// Unknown ids yield `None` (Java returns null).
    pub const fn from_id(id: u8) -> Option<Self> {
        if id >= 1 && id <= 16 {
            Some(Self::ALL[(id - 1) as usize])
        } else {
            None
        }
    }

    pub const fn id(self) -> u8 {
        self as u8
    }

    /// The GUI opened by a `BlockMachine` meta, where the original has one.
    pub const fn for_machine(kind: MachineKind) -> Option<Self> {
        Some(match kind {
            MachineKind::Deployer => Self::Deployer,
            MachineKind::Filter => Self::Filter,
            MachineKind::ItemDetector => Self::ItemDetector,
            MachineKind::SortingMachine => Self::Sorter,
            MachineKind::BatteryBox => Self::BatteryBox,
            MachineKind::Retriever => Self::Retriever,
            MachineKind::Regulator => Self::Regulator,
            MachineKind::Assembler => Self::Assembler,
            MachineKind::Ejector => Self::Ejector,
            MachineKind::Relay => Self::Relay,
            MachineKind::WindTurbine => Self::WindTurbine,
            _ => return None,
        })
    }

    pub const fn for_machine2(kind: Machine2Kind) -> Self {
        match kind {
            Machine2Kind::Manager => Self::Manager,
            Machine2Kind::Sortron => Self::Sorter,
        }
    }
}

/// Renderer classes bound in `registerRenderers`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RendererKind {
    Machine,
    Machine2,
    Breaker,
    BatteryBox,
    Motor,
    WindTurbine,
    Thermopile,
    BlueFurnace,
    BufferChest,
    ChargingBench,
    SolarPanel,
    Pump,
    Accel,
    Grate,
    Transformer,
    Frame,
    FrameMoving,
    FrameTube,
    FrameRedstoneTube,
    Pipe,
    Tube,
    RedstoneTube,
}

/// Renderer for a `BlockMachine` meta (0..15).
pub const fn machine_renderer(kind: MachineKind) -> RendererKind {
    match kind {
        MachineKind::BlockBreaker => RendererKind::Breaker,
        MachineKind::BatteryBox => RendererKind::BatteryBox,
        MachineKind::FrameMotor => RendererKind::Motor,
        MachineKind::WindTurbine => RendererKind::WindTurbine,
        MachineKind::Thermopile => RendererKind::Thermopile,
        _ => RendererKind::Machine,
    }
}

/// Renderer for `BlockMachine2` metas (both use `RenderMachine2`).
pub const fn machine2_renderer(_kind: Machine2Kind) -> RendererKind {
    RendererKind::Machine2
}

/// Renderer for a `BlockMachinePanel` meta.
pub const fn panel_renderer(kind: MachinePanelKind) -> RendererKind {
    match kind {
        MachinePanelKind::SolarPanel => RendererKind::SolarPanel,
        MachinePanelKind::Pump => RendererKind::Pump,
        MachinePanelKind::Accelerator => RendererKind::Accel,
        MachinePanelKind::Grate => RendererKind::Grate,
        MachinePanelKind::Transformer => RendererKind::Transformer,
    }
}

/// Renderer for a `BlockAppliance` meta (`None` where none is registered).
pub const fn appliance_renderer(meta: u8) -> Option<RendererKind> {
    match meta {
        1 | 4 => Some(RendererKind::BlueFurnace),
        2 => Some(RendererKind::BufferChest),
        5 => Some(RendererKind::ChargingBench),
        _ => None,
    }
}

/// Renderer for a `BlockFrame` meta (0..3).
pub const fn frame_renderer(meta: u8) -> Option<RendererKind> {
    match meta {
        0 => Some(RendererKind::Frame),
        1 => Some(RendererKind::FrameMoving),
        2 => Some(RendererKind::FrameTube),
        3 => Some(RendererKind::FrameRedstoneTube),
        _ => None,
    }
}

/// "High" renderer for a `BlockMicro` meta (pipes and tubes, metas 7..11).
pub const fn micro_high_renderer(meta: u8) -> Option<RendererKind> {
    match meta {
        7 => Some(RendererKind::Pipe),
        8 | 10 | 11 => Some(RendererKind::Tube),
        9 => Some(RendererKind::RedstoneTube),
        _ => None,
    }
}

/// Animated texture bindings from `registerRenderers`: (slot, first-frame set).
pub const TEXTURE_ANIMS: [(u32, u32, [u32; 4]); 2] = [
    (239, 2, [146, 147, 148, 149]),
    (255, 2, [151, 152, 153, 154]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gui_ids_round_trip() {
        for g in GuiId::ALL {
            assert_eq!(GuiId::from_id(g.id()), Some(g));
        }
        assert_eq!(GuiId::from_id(0), None);
        assert_eq!(GuiId::from_id(17), None);
    }

    #[test]
    fn machine_guis_match_has_gui() {
        for k in MachineKind::ALL {
            if k.has_gui() {
                assert!(GuiId::for_machine(k).is_some(), "{k:?}");
            }
        }
        assert_eq!(GuiId::for_machine(MachineKind::Relay), Some(GuiId::Relay));
    }

    #[test]
    fn renderers() {
        assert_eq!(machine_renderer(MachineKind::BatteryBox), RendererKind::BatteryBox);
        assert_eq!(machine_renderer(MachineKind::Filter), RendererKind::Machine);
        assert_eq!(appliance_renderer(3), None);
        assert_eq!(micro_high_renderer(9), Some(RendererKind::RedstoneTube));
        assert_eq!(frame_renderer(4), None);
    }
}

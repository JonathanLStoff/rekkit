//! Routing GUIs & Containers: Filter, Sorter, Retriever.
//!
//! Provides unified access and contract-backed level 3 GUI definitions for:
//! - Filter (`redpower_2:filter`, `ContainerFilter` / `GuiFilter`):
//!   9 ghost filter slots, tube color tag assignment button (0..=16).
//! - Sorting Machine (`redpower_2:sorting_machine`, `ContainerSorter` / `GuiSorter`):
//!   40 filter slots across 8 color channels, mode selection (0..=6), auto mode (0..=2),
//!   default color (0..=16), and per-column color selectors.
//! - Retriever (`redpower_2:retriever`, `ContainerRetriever` / `GuiRetriever`):
//!   9 request filter template slots, mode selection, routing color tag selector (0..=16),
//!   and Blutricity buffer monitoring.

pub use super::logistics_gui::{
    apply_button as retriever_apply_button, retriever_click, retriever_layout, BTN_COLOR_NEXT,
    BTN_COLOR_PREV, BTN_MODE_NEXT, BTN_MODE_PREV,
};
pub use super::simple_gui::{filter_apply, filter_click, grid9_layout as filter_layout};
pub use super::sorter_gui::{
    click as sorter_click, layout as sorter_layout, SorterSettings, AUTOMODE_MAX, BTN_AUTO_NEXT,
    BTN_AUTO_PREV, BTN_COLUMN_NEXT, BTN_COLUMN_PREV, BTN_DEFCOLOR_NEXT, BTN_DEFCOLOR_PREV,
    BTN_MODE_NEXT as BTN_SORTER_MODE_NEXT, BTN_MODE_PREV as BTN_SORTER_MODE_PREV,
    COLOR_MAX as SORTER_COLOR_MAX, COLUMNS as SORTER_COLUMNS, MODE_MAX as SORTER_MODE_MAX,
    SLOT_COUNT as SORTER_SLOT_COUNT,
};

/// Container layout specification for routing machine screens.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutingGuiLayout {
    pub title: &'static str,
    pub size: [f32; 2],
    pub slots: Vec<[f32; 2]>,
    pub inventory: [f32; 2],
}

/// Retrieve the GUI layout for Filter (`redpower_2:filter`).
pub fn filter_gui_layout() -> RoutingGuiLayout {
    let base = filter_layout();
    RoutingGuiLayout {
        title: "Filter",
        size: base.size,
        slots: base.slots,
        inventory: base.inventory,
    }
}

/// Retrieve the GUI layout for Sorting Machine (`redpower_2:sorting_machine`).
pub fn sorter_gui_layout() -> RoutingGuiLayout {
    let base = sorter_layout();
    RoutingGuiLayout {
        title: "Sorting Machine",
        size: base.size,
        slots: base.slots,
        inventory: base.inventory,
    }
}

/// Retrieve the GUI layout for Retriever (`redpower_2:retriever`).
pub fn retriever_gui_layout() -> RoutingGuiLayout {
    let base = retriever_layout();
    RoutingGuiLayout {
        title: "Retriever",
        size: base.size,
        slots: base.slots,
        inventory: base.inventory,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_gui_layout_and_color_tag() {
        let l = filter_gui_layout();
        assert_eq!(l.title, "Filter");
        assert_eq!(l.size, [176.0, 166.0]);
        assert_eq!(l.slots.len(), 9);
        assert_eq!(l.slots[0], [62.0, 17.0]);
        assert_eq!(l.slots[4], [80.0, 35.0]);
        assert_eq!(l.slots[8], [98.0, 53.0]);

        // Color button click and wrap 0..=16
        assert_eq!(filter_click([120.0, 60.0], false), Some(BTN_COLOR_NEXT));
        assert_eq!(filter_click([120.0, 60.0], true), Some(BTN_COLOR_PREV));
        assert_eq!(filter_apply(0, BTN_COLOR_NEXT), 1);
        assert_eq!(filter_apply(16, BTN_COLOR_NEXT), 0);
        assert_eq!(filter_apply(0, BTN_COLOR_PREV), 16);
    }

    #[test]
    fn test_sorter_gui_layout_and_modes() {
        let l = sorter_gui_layout();
        assert_eq!(l.title, "Sorting Machine");
        assert_eq!(l.size, [176.0, 222.0]);
        assert_eq!(l.slots.len(), 40);
        assert_eq!(l.slots[0], [26.0, 18.0]);
        assert_eq!(l.slots[39], [152.0, 90.0]);

        let mut s = SorterSettings::default();
        // Mode 0..=6
        s.apply(BTN_SORTER_MODE_NEXT);
        assert_eq!(s.mode, 1);
        s.apply(BTN_SORTER_MODE_PREV);
        assert_eq!(s.mode, 0);
        s.apply(BTN_SORTER_MODE_PREV);
        assert_eq!(s.mode, 6);

        // Columns 0..8 colors
        s.apply(BTN_COLUMN_NEXT + 4);
        assert_eq!(s.colors[4], 1);
        s.apply(BTN_COLUMN_PREV + 4);
        assert_eq!(s.colors[4], 0);
        s.apply(BTN_COLUMN_PREV + 4);
        assert_eq!(s.colors[4], 16);
    }

    #[test]
    fn test_retriever_gui_layout_modes_and_colors() {
        let l = retriever_gui_layout();
        assert_eq!(l.title, "Retriever");
        assert_eq!(l.size, [176.0, 166.0]);
        assert_eq!(l.slots.len(), 9);
        assert_eq!(l.slots[0], [62.0, 17.0]);
        assert_eq!(l.slots[8], [98.0, 53.0]);

        // Clicks for color (x118..129) and mode (x45..58)
        assert_eq!(retriever_click([120.0, 60.0], false), Some(BTN_COLOR_NEXT));
        assert_eq!(retriever_click([50.0, 60.0], false), Some(BTN_MODE_NEXT));

        assert_eq!(retriever_apply_button(0, 0, BTN_COLOR_NEXT), (1, 0));
        assert_eq!(retriever_apply_button(16, 0, BTN_COLOR_NEXT), (0, 0));
        assert_eq!(retriever_apply_button(0, 0, BTN_MODE_NEXT), (0, 1));
        assert_eq!(retriever_apply_button(0, 1, BTN_MODE_NEXT), (0, 0));
    }
}

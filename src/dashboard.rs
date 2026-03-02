use crate::trading_panel::TradingPanel;

pub const GRID_PRESETS: &[(usize, usize)] = &[
    (1, 1), (2, 1), (3, 1), (4, 1), (8, 1),
    (2, 2), (3, 2), (4, 2),
    (2, 3), (3, 3),
];

pub struct Dashboard {
    pub name: String,
    pub cols: usize,
    pub rows: usize,
    pub panels: Vec<Option<TradingPanel>>, // len = cols * rows, row-major
}

impl Dashboard {
    pub fn new(name: impl Into<String>, cols: usize, rows: usize) -> Self {
        let n = cols * rows;
        Self {
            name: name.into(),
            cols,
            rows,
            panels: (0..n).map(|_| None).collect(),
        }
    }

    /// Set the grid dimensions, preserving existing panels by flat index order.
    /// Panels with index < new_n are kept; panels beyond new_n are dropped (their
    /// subscriptions will be removed and WS connections will be closed by iced).
    pub fn resize_grid(&mut self, cols: usize, rows: usize) {
        let new_n = cols * rows;
        let old_panels: Vec<Option<TradingPanel>> = self.panels.drain(..).collect();
        let mut new_panels: Vec<Option<TradingPanel>> = (0..new_n).map(|_| None).collect();
        for (i, panel) in old_panels.into_iter().enumerate() {
            if i < new_n {
                new_panels[i] = panel;
            }
            // panels beyond new_n are dropped here — iced stops their subscriptions
        }
        self.cols = cols;
        self.rows = rows;
        self.panels = new_panels;
    }

    pub fn panel_at(&self, col: usize, row: usize) -> Option<&TradingPanel> {
        let idx = row * self.cols + col;
        self.panels.get(idx)?.as_ref()
    }

    pub fn panel_at_mut(&mut self, col: usize, row: usize) -> Option<&mut TradingPanel> {
        let idx = row * self.cols + col;
        self.panels.get_mut(idx)?.as_mut()
    }

    /// Panel index in flat Vec for a given col/row
    pub fn panel_idx(cols: usize, col: usize, row: usize) -> usize {
        row * cols + col
    }
}

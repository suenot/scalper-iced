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

    /// Set the grid dimensions, preserving existing panels where possible
    pub fn resize_grid(&mut self, cols: usize, rows: usize) {
        let new_n = cols * rows;
        let mut new_panels: Vec<Option<TradingPanel>> = (0..new_n).map(|_| None).collect();
        // Copy panels that fit in the new grid (row-major)
        for row in 0..rows.min(self.rows) {
            for col in 0..cols.min(self.cols) {
                let old_idx = row * self.cols + col;
                let new_idx = row * cols + col;
                if old_idx < self.panels.len() {
                    new_panels[new_idx] = self.panels[old_idx].take();
                }
            }
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

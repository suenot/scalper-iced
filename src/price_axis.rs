use crate::theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FollowMode {
    Auto,
    Locked,
    Manual,
}

#[derive(Debug, Clone)]
pub struct PriceAxis {
    pub last_price: f64,
    pub tick_size: f64,
    pub display_step: f64,
    pub center_price: f64,
    pub row_height: f32,
    pub follow_mode: FollowMode,
}

impl PriceAxis {
    pub fn new(display_step: f64) -> Self {
        Self {
            last_price: 0.0,
            tick_size: 0.01,
            display_step,
            center_price: 0.0,
            row_height: theme::DEFAULT_ROW_HEIGHT,
            follow_mode: FollowMode::Auto,
        }
    }

    /// Convert a price to Y coordinate on canvas.
    /// Higher prices are at the top (lower Y).
    pub fn price_to_y(&self, price: f64, canvas_height: f32) -> f32 {
        let center_y = canvas_height / 2.0;
        let price_delta = self.center_price - price;
        center_y + (price_delta / self.display_step) as f32 * self.row_height
    }

    /// Convert Y coordinate back to price.
    pub fn y_to_price(&self, y: f32, canvas_height: f32) -> f64 {
        let center_y = canvas_height / 2.0;
        let row_delta = (y - center_y) / self.row_height;
        self.center_price - row_delta as f64 * self.display_step
    }

    /// Get the row index relative to center for a given price.
    pub fn price_to_row(&self, price: f64) -> i32 {
        ((self.center_price - price) / self.display_step).round() as i32
    }

    /// Get the number of visible rows for a given canvas height.
    pub fn visible_rows(&self, canvas_height: f32) -> i32 {
        (canvas_height / self.row_height) as i32
    }

    /// Get the visible price range (high, low) for a given canvas height.
    pub fn visible_price_range(&self, canvas_height: f32) -> (f64, f64) {
        let half_rows = self.visible_rows(canvas_height) as f64 / 2.0;
        let high = self.center_price + half_rows * self.display_step;
        let low = self.center_price - half_rows * self.display_step;
        (high, low)
    }

    /// Update the last price and adjust center in Auto mode.
    pub fn update_last_price(&mut self, price: f64) {
        self.last_price = price;
        if self.follow_mode == FollowMode::Auto {
            self.center_price = price;
        }
    }

    /// Handle scroll input.
    pub fn on_scroll(&mut self, delta_rows: f32) {
        if self.follow_mode == FollowMode::Auto {
            self.follow_mode = FollowMode::Manual;
        }
        self.center_price += delta_rows as f64 * self.display_step;
    }

    /// Handle zoom input (change row height).
    pub fn on_zoom(&mut self, delta: f32) {
        let new_height = self.row_height * (1.0 + delta * 0.1);
        self.row_height = new_height.clamp(theme::MIN_ROW_HEIGHT, theme::MAX_ROW_HEIGHT);
    }

    /// Handle price step change (Ctrl+scroll — change grouping).
    /// Smooth sequence with intermediate steps: 1, 1.5, 2, 3, 5, 7, 10, 15, 20, 30, 50, ...
    pub fn on_change_price_step(&mut self, delta: f32) {
        let steps: &[f64] = &[
            1.0, 1.5, 2.0, 3.0, 5.0, 7.0,
            10.0, 15.0, 20.0, 30.0, 50.0, 70.0,
            100.0, 150.0, 200.0, 300.0, 500.0, 700.0,
            1000.0, 1500.0, 2000.0, 3000.0, 5000.0, 7000.0, 10000.0,
        ];
        let current_mult = self.display_step / self.tick_size;

        if delta > 0.0 {
            if let Some(&next) = steps.iter().find(|&&s| s > current_mult * 1.01) {
                self.display_step = self.tick_size * next;
            }
        } else {
            if let Some(&prev) = steps.iter().rev().find(|&&s| s < current_mult * 0.99) {
                self.display_step = self.tick_size * prev;
            }
        }
    }

    /// Snap back to following the last price.
    pub fn snap_to_price(&mut self) {
        self.center_price = self.last_price;
        self.follow_mode = FollowMode::Auto;
    }

    /// Toggle between Auto and Locked modes.
    pub fn toggle_follow_mode(&mut self) {
        self.follow_mode = match self.follow_mode {
            FollowMode::Auto => FollowMode::Locked,
            FollowMode::Locked => FollowMode::Auto,
            FollowMode::Manual => FollowMode::Auto,
        };
        if self.follow_mode == FollowMode::Auto {
            self.center_price = self.last_price;
        }
    }
}

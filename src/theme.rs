use iced::Color;

pub const BACKGROUND: Color = Color::from_rgb(
    0x1a as f32 / 255.0,
    0x1a as f32 / 255.0,
    0x2e as f32 / 255.0,
);

pub const PANEL_BG: Color = Color::from_rgb(
    0x16 as f32 / 255.0,
    0x16 as f32 / 255.0,
    0x28 as f32 / 255.0,
);

pub const BID_GREEN: Color = Color::from_rgb(
    0x26 as f32 / 255.0,
    0xa6 as f32 / 255.0,
    0x9a as f32 / 255.0,
);

pub const ASK_RED: Color = Color::from_rgb(
    0xef as f32 / 255.0,
    0x53 as f32 / 255.0,
    0x50 as f32 / 255.0,
);

pub const SPREAD_YELLOW: Color = Color::from_rgb(
    0xff as f32 / 255.0,
    0xeb as f32 / 255.0,
    0x3b as f32 / 255.0,
);

pub const TEXT_PRIMARY: Color = Color::from_rgb(
    0xe0 as f32 / 255.0,
    0xe0 as f32 / 255.0,
    0xe0 as f32 / 255.0,
);

pub const TEXT_BRIGHT: Color = Color::WHITE;

pub const TEXT_DIM: Color = Color::from_rgb(
    0x80 as f32 / 255.0,
    0x80 as f32 / 255.0,
    0x90 as f32 / 255.0,
);

pub const GRID_LINE: Color = Color::from_rgb(
    0x2a as f32 / 255.0,
    0x2a as f32 / 255.0,
    0x3e as f32 / 255.0,
);

pub const LAST_PRICE_LINE: Color = Color::from_rgb(
    0xff as f32 / 255.0,
    0xd7 as f32 / 255.0,
    0x00 as f32 / 255.0,
);

pub const HEADER_BG: Color = Color::from_rgb(
    0x12 as f32 / 255.0,
    0x12 as f32 / 255.0,
    0x22 as f32 / 255.0,
);

pub const DEFAULT_ROW_HEIGHT: f32 = 20.0;
pub const MIN_ROW_HEIGHT: f32 = 10.0;
pub const MAX_ROW_HEIGHT: f32 = 48.0;

/// Compute a heatmap color for a volume bar.
/// `intensity` is 0.0..1.0 where 1.0 = max volume.
pub fn bid_heatmap(intensity: f32) -> Color {
    let base_r = 0x26 as f32 / 255.0;
    let base_g = 0xa6 as f32 / 255.0;
    let base_b = 0x9a as f32 / 255.0;
    let alpha = 0.15 + intensity * 0.65;
    Color::from_rgba(base_r, base_g, base_b, alpha)
}

pub fn ask_heatmap(intensity: f32) -> Color {
    let base_r = 0xef as f32 / 255.0;
    let base_g = 0x53 as f32 / 255.0;
    let base_b = 0x50 as f32 / 255.0;
    let alpha = 0.15 + intensity * 0.65;
    Color::from_rgba(base_r, base_g, base_b, alpha)
}

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke, Text};
use iced::{Element, Length, Point, Rectangle, Renderer, Theme};

use crate::panel_message::PanelMessage;
use crate::model::TickCandle;
use crate::price_axis::PriceAxis;
use crate::theme as t;

/// Bubble chart: each tick candle rendered as a circle.
/// X = time (right-to-left), Y = price (shared PriceAxis), radius = volume.
pub struct BubbleChartCanvas {
    pub candles: Vec<TickCandle>,
    pub price_axis: PriceAxis,
    cache: canvas::Cache,
}

impl BubbleChartCanvas {
    pub fn new(price_axis: PriceAxis) -> Self {
        Self {
            candles: Vec::new(),
            price_axis,
            cache: canvas::Cache::new(),
        }
    }

    pub fn update_data(&mut self, candles: Vec<TickCandle>) {
        self.candles = candles;
        self.cache.clear();
    }

    pub fn update_price_axis(&mut self, price_axis: &PriceAxis) {
        self.price_axis = price_axis.clone();
        self.cache.clear();
    }

    pub fn view(&self) -> Element<'_, PanelMessage> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl canvas::Program<PanelMessage> for &BubbleChartCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let size = frame.size();
            frame.fill_rectangle(Point::ORIGIN, size, t::PANEL_BG);

            if self.candles.is_empty() {
                let text = Text {
                    content: "Bubble Chart...".to_string(),
                    position: Point::new(size.width / 2.0, size.height / 2.0),
                    color: t::TEXT_DIM,
                    size: 14.0.into(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    ..Text::default()
                };
                frame.fill_text(text);
                return;
            }

            let row_h = self.price_axis.row_height;
            let label_width = 60.0_f32;
            let chart_width = size.width - label_width;

            // Find max volume for radius scaling
            let max_vol = self
                .candles
                .iter()
                .map(|c| c.volume)
                .fold(0.0_f64, f64::max)
                .max(0.001);

            let num_candles = self.candles.len();
            let col_width = (chart_width / num_candles.max(1) as f32).min(40.0).max(6.0);
            let max_radius = (col_width / 2.0).min(row_h * 2.0).min(20.0);

            for (i, candle) in self.candles.iter().enumerate() {
                let x = chart_width - (num_candles - i) as f32 * col_width + col_width / 2.0;
                let price = (candle.open + candle.close) / 2.0;
                let y = self.price_axis.price_to_y(price, size.height) + row_h / 2.0;

                if y < -max_radius || y > size.height + max_radius {
                    continue;
                }

                let vol_ratio = (candle.volume / max_vol) as f32;
                let radius = (vol_ratio.sqrt() * max_radius).max(2.0);

                let is_buy = candle.is_bullish();
                let (fill_color, stroke_color) = if is_buy {
                    (
                        t::bid_heatmap(vol_ratio * 0.5),
                        t::BID_GREEN,
                    )
                } else {
                    (
                        t::ask_heatmap(vol_ratio * 0.5),
                        t::ASK_RED,
                    )
                };

                // Fill circle
                let circle = Path::circle(Point::new(x, y), radius);
                frame.fill(&circle, fill_color);
                frame.stroke(
                    &circle,
                    Stroke::default().with_color(stroke_color).with_width(1.0),
                );

                // Volume label inside large bubbles
                if radius > 12.0 {
                    let vol_text = Text {
                        content: format_volume_short(candle.volume),
                        position: Point::new(x, y),
                        color: t::TEXT_BRIGHT,
                        size: (radius * 0.6).max(7.0).min(11.0).into(),
                        align_x: iced::alignment::Horizontal::Center.into(),
                        align_y: iced::alignment::Vertical::Center,
                        ..Text::default()
                    };
                    frame.fill_text(vol_text);
                }
            }

            // Last price line
            let last_y = self
                .price_axis
                .price_to_y(self.price_axis.last_price, size.height);
            let last_y_center = last_y + row_h / 2.0;
            if last_y_center > 0.0 && last_y_center < size.height {
                let line = Path::line(
                    Point::new(0.0, last_y_center),
                    Point::new(chart_width, last_y_center),
                );
                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_color(t::LAST_PRICE_LINE)
                        .with_width(1.0),
                );
            }

            // Price scale labels on the right
            let visible_rows = self.price_axis.visible_rows(size.height);
            let step = (visible_rows / 8).max(1);
            for row_offset in (-visible_rows / 2..=visible_rows / 2).step_by(step as usize) {
                let y = size.height / 2.0 + row_offset as f32 * row_h;
                let price = self.price_axis.y_to_price(y, size.height);
                if y > 10.0 && y < size.height - 10.0 {
                    let grid = Path::line(
                        Point::new(0.0, y),
                        Point::new(chart_width, y),
                    );
                    frame.stroke(
                        &grid,
                        Stroke::default()
                            .with_color(t::GRID_LINE)
                            .with_width(0.5),
                    );

                    let label = Text {
                        content: format_price(price),
                        position: Point::new(size.width - 3.0, y),
                        color: t::TEXT_DIM,
                        size: 9.0.into(),
                        align_x: iced::alignment::Horizontal::Right.into(),
                        align_y: iced::alignment::Vertical::Center,
                        ..Text::default()
                    };
                    frame.fill_text(label);
                }
            }

            // Separator
            let sep = Path::line(
                Point::new(chart_width, 0.0),
                Point::new(chart_width, size.height),
            );
            frame.stroke(
                &sep,
                Stroke::default()
                    .with_color(t::GRID_LINE)
                    .with_width(0.5),
            );
        });

        vec![geometry]
    }
}

fn format_price(price: f64) -> String {
    if price >= 1000.0 {
        format!("{:.1}", price)
    } else if price >= 1.0 {
        format!("{:.2}", price)
    } else {
        format!("{:.4}", price)
    }
}

fn format_volume_short(vol: f64) -> String {
    if vol >= 1000.0 {
        format!("{:.0}", vol)
    } else if vol >= 1.0 {
        format!("{:.1}", vol)
    } else {
        format!("{:.3}", vol)
    }
}

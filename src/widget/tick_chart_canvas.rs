use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke, Text};
use iced::{Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::panel_message::PanelMessage;
use crate::model::TickCandle;
use crate::price_axis::PriceAxis;
use crate::theme as t;

pub struct TickChartCanvas {
    pub candles: Vec<TickCandle>,
    pub price_axis: PriceAxis,
    cache: canvas::Cache,
}

impl TickChartCanvas {
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

impl canvas::Program<PanelMessage> for &TickChartCanvas {
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
                    content: "Tick Chart...".to_string(),
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

            // Use shared PriceAxis for Y mapping — keeps yellow line aligned
            // with orderbook and cluster chart
            let label_width = 60.0_f32;
            let chart_width = size.width - label_width;
            let row_h = self.price_axis.row_height;

            let num_candles = self.candles.len();
            let candle_width = (chart_width / num_candles.max(1) as f32).min(20.0).max(3.0);
            let body_width = (candle_width * 0.7).max(2.0);
            let wick_width = 1.0_f32;

            for (i, candle) in self.candles.iter().enumerate() {
                let x = chart_width - (num_candles - i) as f32 * candle_width;
                let center_x = x + candle_width / 2.0;

                let open_y = self.price_axis.price_to_y(candle.open, size.height);
                let close_y = self.price_axis.price_to_y(candle.close, size.height);
                let high_y = self.price_axis.price_to_y(candle.high, size.height);
                let low_y = self.price_axis.price_to_y(candle.low, size.height);

                let color = if candle.is_bullish() {
                    t::BID_GREEN
                } else {
                    t::ASK_RED
                };

                // Draw wick
                let wick = Path::line(
                    Point::new(center_x, high_y),
                    Point::new(center_x, low_y),
                );
                frame.stroke(
                    &wick,
                    Stroke::default().with_color(color).with_width(wick_width),
                );

                // Draw body (min 2px so thin candles are still visible)
                let body_top = open_y.min(close_y);
                let body_height = (open_y - close_y).abs().max(2.0);
                frame.fill_rectangle(
                    Point::new(center_x - body_width / 2.0, body_top),
                    Size::new(body_width, body_height),
                    color,
                );
            }

            // Price scale labels on the right
            let visible_rows = self.price_axis.visible_rows(size.height);
            let step = (visible_rows / 8).max(1);
            for row_offset in (-visible_rows / 2..=visible_rows / 2).step_by(step as usize) {
                let y = size.height / 2.0 + row_offset as f32 * row_h;
                let price = self.price_axis.y_to_price(y, size.height);
                if y > 10.0 && y < size.height - 10.0 {
                    // Grid line
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

            // Separator line
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

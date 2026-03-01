use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke, Text};
use iced::{Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::message::Message;
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

    pub fn view(&self) -> Element<'_, Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl canvas::Program<Message> for &TickChartCanvas {
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

            // Compute local price range from candle data for auto-scaling
            let mut price_min = f64::MAX;
            let mut price_max = f64::MIN;
            for candle in &self.candles {
                if candle.low < price_min {
                    price_min = candle.low;
                }
                if candle.high > price_max {
                    price_max = candle.high;
                }
            }

            // Add padding (10% on each side)
            let range = (price_max - price_min).max(self.price_axis.display_step * 2.0);
            let padding = range * 0.1;
            price_min -= padding;
            price_max += padding;
            let total_range = price_max - price_min;

            // Reserve space for price labels on the right
            let label_width = 60.0_f32;
            let chart_width = size.width - label_width;
            let chart_height = size.height - 20.0; // top margin
            let top_margin = 10.0_f32;

            // Local price-to-Y mapping (auto-scaled to fit data)
            let local_price_to_y = |price: f64| -> f32 {
                let ratio = (price_max - price) / total_range;
                top_margin + ratio as f32 * chart_height
            };

            let num_candles = self.candles.len();
            let candle_width = (chart_width / num_candles.max(1) as f32).min(20.0).max(3.0);
            let body_width = (candle_width * 0.7).max(2.0);
            let wick_width = 1.0_f32;

            for (i, candle) in self.candles.iter().enumerate() {
                let x = chart_width - (num_candles - i) as f32 * candle_width;
                let center_x = x + candle_width / 2.0;

                let open_y = local_price_to_y(candle.open);
                let close_y = local_price_to_y(candle.close);
                let high_y = local_price_to_y(candle.high);
                let low_y = local_price_to_y(candle.low);

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

                // Draw body
                let body_top = open_y.min(close_y);
                let body_height = (open_y - close_y).abs().max(2.0);
                frame.fill_rectangle(
                    Point::new(center_x - body_width / 2.0, body_top),
                    Size::new(body_width, body_height),
                    color,
                );
            }

            // Draw last price horizontal line
            let last_y = local_price_to_y(self.price_axis.last_price);
            if last_y > 0.0 && last_y < size.height {
                let line = Path::line(
                    Point::new(0.0, last_y),
                    Point::new(chart_width, last_y),
                );
                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_color(t::LAST_PRICE_LINE)
                        .with_width(1.0),
                );
            }

            // Draw price scale labels on the right side
            let num_labels = 6;
            for i in 0..=num_labels {
                let price = price_max - (total_range * i as f64 / num_labels as f64);
                let y = local_price_to_y(price);
                if y > 5.0 && y < size.height - 5.0 {
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

            // Separator line between chart and labels
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

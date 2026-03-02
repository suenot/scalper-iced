use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke, Text};
use iced::{Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::panel_message::PanelMessage;
use crate::model::ClusterCandle;
use crate::price_axis::PriceAxis;
use crate::theme as t;

pub struct ClusterCanvas {
    pub candles: Vec<ClusterCandle>,
    pub price_axis: PriceAxis,
    cache: canvas::Cache,
}

impl ClusterCanvas {
    pub fn new(price_axis: PriceAxis) -> Self {
        Self {
            candles: Vec::new(),
            price_axis,
            cache: canvas::Cache::new(),
        }
    }

    pub fn update_data(&mut self, candles: Vec<ClusterCandle>) {
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

impl canvas::Program<PanelMessage> for &ClusterCanvas {
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
                    content: "Clusters...".to_string(),
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

            let num_candles = self.candles.len();
            let candle_width = (size.width / num_candles.max(1) as f32).min(80.0);
            let row_h = self.price_axis.row_height;

            // Aggregate cluster points when display_step > tick_size
            let step = self.price_axis.display_step;
            let tick = self.price_axis.tick_size;
            let need_agg = step > tick * 1.5;
            let agg_candles: Vec<_> = if need_agg {
                self.candles.iter().map(|c| c.aggregated(step)).collect()
            } else {
                Vec::new()
            };
            let candles_ref: &[crate::model::ClusterCandle] = if need_agg {
                &agg_candles
            } else {
                &self.candles
            };

            let global_max_vol = candles_ref
                .iter()
                .flat_map(|c| c.cluster_points.iter())
                .map(|p| p.volume)
                .fold(0.0_f64, f64::max)
                .max(0.001);

            for (i, candle) in candles_ref.iter().enumerate() {
                let x = size.width - (num_candles - i) as f32 * candle_width;

                for point in &candle.cluster_points {
                    let y = self.price_axis.price_to_y(point.price, size.height);
                    if y < -row_h || y > size.height + row_h {
                        continue;
                    }

                    let intensity = (point.volume / global_max_vol) as f32;

                    let cell_color = if point.percent > 50.0 {
                        t::bid_heatmap(intensity)
                    } else {
                        t::ask_heatmap(intensity)
                    };

                    frame.fill_rectangle(
                        Point::new(x + 1.0, y + 1.0),
                        Size::new(candle_width - 2.0, row_h - 2.0),
                        cell_color,
                    );

                    if candle_width > 30.0 && row_h > 12.0 {
                        let vol_text = Text {
                            content: format_volume(point.volume),
                            position: Point::new(x + candle_width / 2.0, y + row_h / 2.0),
                            color: t::TEXT_PRIMARY,
                            size: (row_h * 0.5).max(8.0).min(12.0).into(),
                            align_x: iced::alignment::Horizontal::Center.into(),
                            align_y: iced::alignment::Vertical::Center,
                            ..Text::default()
                        };
                        frame.fill_text(vol_text);
                    }
                }

                let sep = Path::line(Point::new(x, 0.0), Point::new(x, size.height));
                frame.stroke(
                    &sep,
                    Stroke::default()
                        .with_color(t::GRID_LINE)
                        .with_width(0.5),
                );
            }

            let last_y = self
                .price_axis
                .price_to_y(self.price_axis.last_price, size.height);
            if last_y > 0.0 && last_y < size.height {
                let line = Path::line(
                    Point::new(0.0, last_y + row_h / 2.0),
                    Point::new(size.width, last_y + row_h / 2.0),
                );
                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_color(t::LAST_PRICE_LINE)
                        .with_width(1.0),
                );
            }
        });

        vec![geometry]
    }
}

fn format_volume(vol: f64) -> String {
    if vol >= 1000.0 {
        format!("{:.0}", vol)
    } else if vol >= 100.0 {
        format!("{:.1}", vol)
    } else if vol >= 1.0 {
        format!("{:.2}", vol)
    } else if vol >= 0.01 {
        format!("{:.3}", vol)
    } else if vol >= 0.001 {
        format!("{:.4}", vol)
    } else {
        format!("{:.5}", vol)
    }
}

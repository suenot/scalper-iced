use iced::event::Event;
use iced::keyboard;
use iced::mouse;
use iced::widget::canvas::{self, Action, Canvas, Geometry, Path, Stroke, Text};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::message::{Message, Side};
use crate::model::OrderBookSnapshot;
use crate::price_axis::PriceAxis;
use crate::theme as t;

/// Tracks keyboard modifiers for Ctrl+Scroll / Shift+Scroll detection.
#[derive(Debug, Default)]
pub struct CanvasState {
    modifiers: keyboard::Modifiers,
}

pub struct OrderBookCanvas {
    pub orderbook: Option<OrderBookSnapshot>,
    pub price_axis: PriceAxis,
    cache: canvas::Cache,
}

impl OrderBookCanvas {
    pub fn new(price_axis: PriceAxis) -> Self {
        Self {
            orderbook: None,
            price_axis,
            cache: canvas::Cache::new(),
        }
    }

    pub fn update_data(&mut self, orderbook: OrderBookSnapshot) {
        self.orderbook = Some(orderbook);
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

impl canvas::Program<Message> for &OrderBookCanvas {
    type State = CanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            // Track modifier keys (Ctrl, Shift, etc.)
            Event::Keyboard(keyboard::Event::ModifiersChanged(mods)) => {
                state.modifiers = *mods;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let price = self.price_axis.y_to_price(pos.y, bounds.height);
                    let mid = self.price_axis.center_price;
                    let side = if price > mid { Side::Sell } else { Side::Buy };
                    return Some(
                        Action::publish(Message::OrderBookClicked { price, side })
                            .and_capture(),
                    );
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 20.0,
                };
                if cursor.is_over(bounds) {
                    if state.modifiers.control() {
                        // Ctrl+Scroll: change price step (grouping)
                        return Some(Action::publish(Message::ChangePriceStep(dy)).and_capture());
                    } else if state.modifiers.shift() {
                        // Shift+Scroll: zoom (change row height)
                        return Some(Action::publish(Message::Zoom(dy)).and_capture());
                    } else {
                        // Plain scroll: move price axis
                        return Some(Action::publish(Message::Scroll(dy)).and_capture());
                    }
                }
            }
            _ => {}
        }
        None
    }

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

            // Background
            frame.fill_rectangle(Point::ORIGIN, size, t::PANEL_BG);

            let Some(raw_ob) = &self.orderbook else {
                let text = Text {
                    content: "Waiting for data...".to_string(),
                    position: Point::new(size.width / 2.0, size.height / 2.0),
                    color: t::TEXT_DIM,
                    size: 16.0.into(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    ..Text::default()
                };
                frame.fill_text(text);
                return;
            };

            // Aggregate price levels when display_step > tick_size
            let step = self.price_axis.display_step;
            let tick = self.price_axis.tick_size;
            let ob = if step > tick * 1.5 {
                raw_ob.aggregated(step)
            } else {
                raw_ob.clone()
            };

            let max_vol = ob.max_volume().max(0.001);
            let row_h = self.price_axis.row_height;
            let half_width = size.width / 2.0;
            let price_label_width = 90.0_f32;
            let bar_max_width = half_width - price_label_width / 2.0 - 5.0;

            // Draw grid lines
            let visible_rows = self.price_axis.visible_rows(size.height);
            let half_rows = visible_rows / 2;

            for row_offset in -half_rows..=half_rows {
                let y = size.height / 2.0 + row_offset as f32 * row_h;
                if y < 0.0 || y > size.height {
                    continue;
                }
                let line = Path::line(Point::new(0.0, y), Point::new(size.width, y));
                frame.stroke(
                    &line,
                    Stroke::default()
                        .with_color(t::GRID_LINE)
                        .with_width(0.5),
                );
            }

            // Draw asks
            for level in &ob.asks {
                let y = self.price_axis.price_to_y(level.price, size.height);
                if y < -row_h || y > size.height {
                    continue;
                }

                let intensity = (level.amount / max_vol) as f32;
                let bar_width = intensity * bar_max_width;

                frame.fill_rectangle(
                    Point::new(0.0, y),
                    Size::new(size.width, row_h),
                    t::ask_heatmap(intensity * 0.3),
                );

                let bar_x = half_width + price_label_width / 2.0;
                frame.fill_rectangle(
                    Point::new(bar_x, y + 1.0),
                    Size::new(bar_width, row_h - 2.0),
                    t::ask_heatmap(intensity),
                );

                // Volume text for asks — left-aligned right of price column
                let vol_text = Text {
                    content: format_volume(level.amount),
                    position: Point::new(
                        half_width + price_label_width / 2.0 + 4.0,
                        y + row_h / 2.0,
                    ),
                    color: t::TEXT_PRIMARY,
                    size: (row_h * 0.5).max(8.0).min(12.0).into(),
                    align_y: iced::alignment::Vertical::Center,
                    ..Text::default()
                };
                frame.fill_text(vol_text);

                let price_text = Text {
                    content: format_price(level.price),
                    position: Point::new(half_width, y + row_h / 2.0),
                    color: t::ASK_RED,
                    size: (row_h * 0.5).max(9.0).min(12.0).into(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    ..Text::default()
                };
                frame.fill_text(price_text);
            }

            // Draw bids
            for level in &ob.bids {
                let y = self.price_axis.price_to_y(level.price, size.height);
                if y < -row_h || y > size.height {
                    continue;
                }

                let intensity = (level.amount / max_vol) as f32;
                let bar_width = intensity * bar_max_width;

                frame.fill_rectangle(
                    Point::new(0.0, y),
                    Size::new(size.width, row_h),
                    t::bid_heatmap(intensity * 0.3),
                );

                let bar_x = half_width - price_label_width / 2.0 - bar_width;
                frame.fill_rectangle(
                    Point::new(bar_x, y + 1.0),
                    Size::new(bar_width, row_h - 2.0),
                    t::bid_heatmap(intensity),
                );

                // Volume text for bids — right-aligned left of price column
                let vol_text = Text {
                    content: format_volume(level.amount),
                    position: Point::new(
                        half_width - price_label_width / 2.0 - 4.0,
                        y + row_h / 2.0,
                    ),
                    color: t::TEXT_PRIMARY,
                    size: (row_h * 0.5).max(8.0).min(12.0).into(),
                    align_x: iced::alignment::Horizontal::Right.into(),
                    align_y: iced::alignment::Vertical::Center,
                    ..Text::default()
                };
                frame.fill_text(vol_text);

                let price_text = Text {
                    content: format_price(level.price),
                    position: Point::new(half_width, y + row_h / 2.0),
                    color: t::BID_GREEN,
                    size: (row_h * 0.5).max(9.0).min(12.0).into(),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    ..Text::default()
                };
                frame.fill_text(price_text);
            }

            // Draw spread area
            if let (Some(best_bid), Some(best_ask)) = (ob.best_bid(), ob.best_ask()) {
                let bid_y = self.price_axis.price_to_y(best_bid, size.height);
                let ask_y = self.price_axis.price_to_y(best_ask, size.height);
                let spread_height = bid_y - ask_y;

                if spread_height > 0.0 {
                    frame.fill_rectangle(
                        Point::new(0.0, ask_y + row_h),
                        Size::new(size.width, spread_height - row_h),
                        Color::from_rgba(0.1, 0.1, 0.2, 0.5),
                    );

                    let spread_text = Text {
                        content: format!(
                            "Spread: {:.2} ({:.4}%)",
                            best_ask - best_bid,
                            ob.spread_percent * 100.0
                        ),
                        position: Point::new(
                            half_width,
                            ask_y + row_h + (spread_height - row_h) / 2.0,
                        ),
                        color: t::SPREAD_YELLOW,
                        size: 11.0.into(),
                        align_x: iced::alignment::Horizontal::Center.into(),
                        align_y: iced::alignment::Vertical::Center,
                        ..Text::default()
                    };
                    frame.fill_text(spread_text);
                }
            }

            // Draw last price line
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
                        .with_width(1.5),
                );
            }
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

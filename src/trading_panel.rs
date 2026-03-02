use std::collections::HashMap;

use iced::mouse;
use iced::widget::{button, column, container, row, text, Row};
use iced::{Color, Element, Length, Subscription};

use crate::message::{Message, PanelId, Side};
use crate::panel::order::OrderPanel;
use crate::panel::pnl::StubPnL;
use crate::panel::position::{PositionSide, StubPosition};
use crate::panel_message::PanelMessage;
use crate::price_axis::{FollowMode, PriceAxis};
use crate::theme as t;
use crate::widget::bubble_chart_canvas::BubbleChartCanvas;
use crate::widget::cluster_canvas::ClusterCanvas;
use crate::widget::orderbook_canvas::OrderBookCanvas;
use crate::widget::tape::Tape;
use crate::widget::tick_chart_canvas::TickChartCanvas;
use crate::ws;

pub struct TradingPanel {
    // Config
    pub ws_url: String,
    pub symbol: String,

    // Shared state
    pub price_axis: PriceAxis,

    // Widgets
    pub orderbook_canvas: OrderBookCanvas,
    pub cluster_canvas: ClusterCanvas,
    pub tick_chart_canvas: TickChartCanvas,
    pub bubble_chart_canvas: BubbleChartCanvas,
    pub tape: Tape,

    // Stub panels
    pub position: StubPosition,
    pub order_panel: OrderPanel,
    pub pnl: StubPnL,

    // Connection state
    pub ws_connected: bool,
    pub last_price: f64,
    pub message_count: u64,

    // WS messages per second (tracked per-panel; App reads this for MPS display)
    pub ws_msg_count: u32,

    // Panel order and visibility
    pub panel_order: Vec<PanelId>,
    pub panel_visible: HashMap<PanelId, bool>,

    // Drag/drop state (button reordering)
    pub dragging: Option<PanelId>,
    pub drag_did_move: bool,

    // Panel resize state (draggable dividers)
    pub panel_widths: HashMap<PanelId, u16>,
    pub resizing_divider: Option<usize>, // index of divider being dragged
    pub resize_start_x: f32,
    pub resize_px_per_unit: f32, // calculated on first move for 1:1 tracking
    pub resize_start_widths: Vec<(PanelId, u16)>,
}

impl TradingPanel {
    /// Construct a new TradingPanel.
    ///
    /// The App is responsible for loading settings and applying them after
    /// construction via the public fields.  This constructor applies defaults
    /// only so that the panel is immediately usable if no saved config exists.
    pub fn new(
        symbol: String,
        price_step: f64,
        ws_base_url: String,
        clusters_n: usize,
        ticks_n: usize,
    ) -> Self {
        let ws_url = format!(
            "{}/ws?symbol={}&priceStep={}&clustersN={}&ticksN={}",
            ws_base_url, symbol, price_step, clusters_n, ticks_n
        );

        let price_axis = PriceAxis::new(price_step);

        let panel_order = PanelId::default_order();
        let panel_visible = HashMap::from([
            (PanelId::ClusterChart, true),
            (PanelId::TickChart, false),
            (PanelId::BubbleChart, true),
            (PanelId::OrderBook, true),
            (PanelId::Tape, false),
            (PanelId::BottomBar, false),
        ]);
        let panel_widths = HashMap::from([
            (PanelId::ClusterChart, 20u16),
            (PanelId::TickChart, 15u16),
            (PanelId::BubbleChart, 15u16),
            (PanelId::OrderBook, 30u16),
            (PanelId::Tape, 20u16),
        ]);

        Self {
            ws_url,
            symbol,
            price_axis: price_axis.clone(),
            orderbook_canvas: OrderBookCanvas::new(price_axis.clone()),
            cluster_canvas: ClusterCanvas::new(price_axis.clone()),
            tick_chart_canvas: TickChartCanvas::new(price_axis.clone()),
            bubble_chart_canvas: BubbleChartCanvas::new(price_axis.clone()),
            tape: Tape::new(),
            position: StubPosition::default(),
            order_panel: OrderPanel::default(),
            pnl: StubPnL::default(),
            ws_connected: false,
            last_price: 0.0,
            message_count: 0,
            ws_msg_count: 0,
            panel_order,
            panel_visible,
            dragging: None,
            drag_did_move: false,
            panel_widths,
            resizing_divider: None,
            resize_start_x: 0.0,
            resize_px_per_unit: 10.0,
            resize_start_widths: Vec::new(),
        }
    }

    // ------------------------------------------------------------------ //
    //  Update
    // ------------------------------------------------------------------ //

    pub fn update(&mut self, msg: PanelMessage) {
        match msg {
            PanelMessage::WsEvent(event) => match event {
                ws::WsEvent::Connected => {
                    self.ws_connected = true;
                    println!("[WS][{}] Connected", self.symbol);
                }
                ws::WsEvent::Disconnected => {
                    self.ws_connected = false;
                    println!("[WS][{}] Disconnected", self.symbol);
                }
                ws::WsEvent::MessageReceived(msg) => {
                    self.message_count += 1;
                    self.ws_msg_count += 1;

                    let mut price_axis_changed = false;

                    if let Some(ob) = msg.orderbook {
                        let mid = ob.mid_price();
                        if (mid - self.last_price).abs() > f64::EPSILON {
                            self.last_price = mid;
                            self.price_axis.update_last_price(mid);
                            self.position.mark_price = mid;
                            price_axis_changed = true;
                        }
                        self.orderbook_canvas.update_data(ob);
                        self.orderbook_canvas.update_price_axis(&self.price_axis);
                    }

                    if let Some(clusters) = msg.clusters {
                        self.cluster_canvas.update_data(clusters);
                        if price_axis_changed {
                            self.cluster_canvas.update_price_axis(&self.price_axis);
                        }
                    }

                    if let Some(ticks) = msg.ticks {
                        self.tick_chart_canvas.update_data(ticks.clone());
                        self.bubble_chart_canvas.update_data(ticks.clone());
                        self.tape.update_ticks(ticks);
                        if price_axis_changed {
                            self.tick_chart_canvas.update_price_axis(&self.price_axis);
                            self.bubble_chart_canvas.update_price_axis(&self.price_axis);
                        }
                    }
                }
                ws::WsEvent::Error(e) => {
                    eprintln!("[WS][{}] Error: {}", self.symbol, e);
                }
            },

            PanelMessage::OrderBookClicked { price, side } => {
                let side_str = match side {
                    Side::Buy => "BUY LIMIT",
                    Side::Sell => "SELL LIMIT",
                };
                println!("[TRADE][{}] {} @ {:.2} (stub)", self.symbol, side_str, price);
            }

            PanelMessage::Scroll(delta) => {
                self.price_axis.on_scroll(delta);
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
            }

            PanelMessage::Zoom(delta) => {
                self.price_axis.on_zoom(delta);
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
            }

            PanelMessage::ChangePriceStep(delta) => {
                self.price_axis.on_change_price_step(delta);
                println!("[UI][{}] Price step: {:.4}", self.symbol, self.price_axis.display_step);
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
            }

            PanelMessage::SnapToPrice => {
                self.price_axis.snap_to_price();
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
            }

            PanelMessage::ToggleFollowMode => {
                self.price_axis.toggle_follow_mode();
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
                println!("[UI][{}] Follow mode: {:?}", self.symbol, self.price_axis.follow_mode);
            }

            PanelMessage::BuyMarket => {
                println!("[TRADE][{}] BUY MARKET @ {:.2} (stub)", self.symbol, self.last_price);
            }
            PanelMessage::SellMarket => {
                println!("[TRADE][{}] SELL MARKET @ {:.2} (stub)", self.symbol, self.last_price);
            }
            PanelMessage::ClosePosition => {
                println!("[TRADE][{}] CLOSE POSITION (stub)", self.symbol);
                self.position.side = PositionSide::None;
                self.position.size = 0.0;
                self.position.unrealized_pnl = 0.0;
            }
            PanelMessage::CancelAllOrders => {
                println!("[TRADE][{}] CANCEL ALL ORDERS (stub)", self.symbol);
            }
            PanelMessage::EmergencyCloseAll => {
                println!("[TRADE][{}] EMERGENCY CLOSE ALL (stub)", self.symbol);
                self.position.side = PositionSide::None;
                self.position.size = 0.0;
                self.position.unrealized_pnl = 0.0;
            }

            PanelMessage::VolumeFilterChanged(val) => {
                self.tape.volume_filter = val;
            }

            PanelMessage::QuantityChanged(qty) => {
                self.order_panel.quantity = qty;
            }

            PanelMessage::TogglePanel(panel_id) => {
                let visible = self.panel_visible.entry(panel_id).or_insert(true);
                *visible = !*visible;
            }

            PanelMessage::DragStart(panel_id) => {
                self.dragging = Some(panel_id);
                self.drag_did_move = false;
            }
            PanelMessage::DragOver(target) => {
                if let Some(dragged) = self.dragging {
                    if dragged != target {
                        let from = self.panel_order.iter().position(|&p| p == dragged);
                        let to = self.panel_order.iter().position(|&p| p == target);
                        if let (Some(from_idx), Some(to_idx)) = (from, to) {
                            let item = self.panel_order.remove(from_idx);
                            self.panel_order.insert(to_idx, item);
                            self.drag_did_move = true;
                        }
                    }
                }
            }
            PanelMessage::DragEnd => {
                if let Some(dragged) = self.dragging {
                    if !self.drag_did_move {
                        let visible = self.panel_visible.entry(dragged).or_insert(true);
                        *visible = !*visible;
                    }
                }
                self.dragging = None;
                self.drag_did_move = false;
            }

            PanelMessage::ResizeStart { divider_index, x } => {
                let visible: Vec<(PanelId, u16)> = self
                    .panel_order
                    .iter()
                    .filter(|p| {
                        p.is_main_panel()
                            && self.panel_visible.get(p).copied().unwrap_or(true)
                    })
                    .map(|&p| (p, *self.panel_widths.get(&p).unwrap_or(&20)))
                    .collect();
                self.resizing_divider = Some(divider_index);
                self.resize_start_x = x;
                self.resize_start_widths = visible;
            }
            PanelMessage::ResizeMove(x) => {
                if let Some(div_idx) = self.resizing_divider {
                    if self.resize_start_x == 0.0 {
                        self.resize_start_x = x;
                        let left_sum: f32 = self.resize_start_widths[..=div_idx]
                            .iter()
                            .map(|(_, w)| *w as f32)
                            .sum();
                        if left_sum > 0.1 {
                            self.resize_px_per_unit = x / left_sum;
                        }
                        return;
                    }

                    let delta_px = x - self.resize_start_x;
                    let delta_units = (delta_px / self.resize_px_per_unit).round() as i32;

                    if div_idx < self.resize_start_widths.len()
                        && div_idx + 1 < self.resize_start_widths.len()
                    {
                        let (left_id, left_w) = self.resize_start_widths[div_idx];
                        let (right_id, right_w) = self.resize_start_widths[div_idx + 1];

                        let pair_total = left_w + right_w;
                        let new_left =
                            ((left_w as i32 + delta_units).max(5) as u16).min(pair_total - 5);
                        let new_right = pair_total - new_left;

                        self.panel_widths.insert(left_id, new_left);
                        self.panel_widths.insert(right_id, new_right);
                    }
                }
            }
            PanelMessage::ResizeEnd => {
                self.resizing_divider = None;
            }
        }
    }

    // ------------------------------------------------------------------ //
    //  View
    // ------------------------------------------------------------------ //

    pub fn view(&self) -> Element<'_, PanelMessage> {
        // Per-panel header bar
        let ws_indicator = if self.ws_connected {
            text("WS: OK").size(11).color(t::BID_GREEN)
        } else {
            text("WS: ...").size(11).color(t::ASK_RED)
        };

        let mode_text = match self.price_axis.follow_mode {
            FollowMode::Auto => text("AUTO").size(11).color(t::BID_GREEN),
            FollowMode::Locked => text("LOCKED").size(11).color(t::SPREAD_YELLOW),
            FollowMode::Manual => text("MANUAL").size(11).color(t::ASK_RED),
        };

        // Toggle buttons with drag/drop
        let mut toggle_buttons = Row::new().spacing(2);
        for &panel_id in &self.panel_order {
            let on = *self.panel_visible.get(&panel_id).unwrap_or(&true);
            let is_dragged = self.dragging == Some(panel_id);

            let (text_color, bg) = if is_dragged {
                (t::TEXT_BRIGHT, Color::from_rgba(0.4, 0.7, 1.0, 0.5))
            } else if on {
                (t::TEXT_BRIGHT, Color::from_rgba(1.0, 1.0, 1.0, 0.15))
            } else {
                (t::TEXT_DIM, Color::from_rgba(1.0, 1.0, 1.0, 0.04))
            };

            let label = panel_id.label().to_string();
            let btn: Element<'_, PanelMessage> = container(
                text(label).size(10).color(text_color),
            )
            .padding([2, 6])
            .style(move |_theme: &_| container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into();

            let wrapped: Element<'_, PanelMessage> = iced::widget::mouse_area(btn)
                .on_press(PanelMessage::DragStart(panel_id))
                .on_release(PanelMessage::DragEnd)
                .on_enter(PanelMessage::DragOver(panel_id))
                .into();

            toggle_buttons = toggle_buttons.push(wrapped);
        }

        let center_btn = button(text("Center").size(10).color(t::TEXT_BRIGHT))
            .on_press(PanelMessage::SnapToPrice)
            .padding([2, 6])
            .style(|_theme: &_, _status| button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
                text_color: t::TEXT_BRIGHT,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let header_bar = container(
            row![
                ws_indicator,
                text(format!("  {}  ", self.symbol))
                    .size(12)
                    .color(t::TEXT_BRIGHT),
                text(format!("Last: {:.1}", self.last_price))
                    .size(12)
                    .color(t::SPREAD_YELLOW),
                text("  Mode: ").size(11).color(t::TEXT_DIM),
                mode_text,
                center_btn,
                text(format!("  Step: {}", format_step(self.price_axis.display_step)))
                    .size(10)
                    .color(t::TEXT_DIM),
                text(format!("  Msgs: {}  ", self.message_count))
                    .size(10)
                    .color(t::TEXT_DIM),
            ]
            .push(toggle_buttons)
            .spacing(4)
            .align_y(iced::Alignment::Center)
            .padding(4),
        )
        .style(|_theme: &_| container::Style {
            background: Some(t::HEADER_BG.into()),
            ..Default::default()
        })
        .width(Length::Fill);

        // Main panels — rendered in panel_order with draggable dividers
        let visible_panels = self.visible_main_panels();
        let mut main_panels = Row::new().spacing(0).height(Length::Fill);

        for (i, &panel_id) in visible_panels.iter().enumerate() {
            // Add divider before each panel (except first)
            if i > 0 {
                let div_idx = i - 1;
                let is_active = self.resizing_divider == Some(div_idx);
                let divider_bg = if is_active {
                    Color::from_rgba(0.4, 0.7, 1.0, 0.8)
                } else {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.08)
                };
                let divider: Element<'_, PanelMessage> = iced::widget::mouse_area(
                    container(text("").size(1))
                        .width(Length::Fixed(4.0))
                        .height(Length::Fill)
                        .style(move |_theme: &_| container::Style {
                            background: Some(iced::Background::Color(divider_bg)),
                            ..Default::default()
                        }),
                )
                .on_press(PanelMessage::ResizeStart {
                    divider_index: div_idx,
                    x: 0.0,
                })
                .on_release(PanelMessage::ResizeEnd)
                .interaction(mouse::Interaction::ResizingHorizontally)
                .into();

                main_panels = main_panels.push(divider);
            }

            let width = *self.panel_widths.get(&panel_id).unwrap_or(&20);
            let panel_widget: Element<'_, PanelMessage> = match panel_id {
                PanelId::TickChart => container(self.tick_chart_canvas.view())
                    .width(Length::FillPortion(width))
                    .height(Length::Fill)
                    .into(),
                PanelId::ClusterChart => container(self.cluster_canvas.view())
                    .width(Length::FillPortion(width))
                    .height(Length::Fill)
                    .into(),
                PanelId::BubbleChart => container(self.bubble_chart_canvas.view())
                    .width(Length::FillPortion(width))
                    .height(Length::Fill)
                    .into(),
                PanelId::OrderBook => container(self.orderbook_canvas.view())
                    .width(Length::FillPortion(width))
                    .height(Length::Fill)
                    .into(),
                PanelId::Tape => container(self.tape.view())
                    .width(Length::FillPortion(width))
                    .height(Length::Fill)
                    .into(),
                PanelId::BottomBar => continue,
            };

            main_panels = main_panels.push(panel_widget);
        }

        // Full panel layout
        let mut content = column![header_bar, main_panels].spacing(2);

        // Bottom panels row — only if BottomBar is visible
        if self
            .panel_visible
            .get(&PanelId::BottomBar)
            .copied()
            .unwrap_or(true)
        {
            let bottom_panels = row![
                crate::panel::position::view(&self.position),
                crate::panel::order::view(&self.order_panel),
                crate::panel::pnl::view(&self.pnl),
            ]
            .spacing(2)
            .height(Length::Fixed(90.0));

            content = content.push(bottom_panels);
        }

        container(content)
            .style(|_theme: &_| container::Style {
                background: Some(t::BACKGROUND.into()),
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(2)
            .into()
    }

    // ------------------------------------------------------------------ //
    //  Subscription
    // ------------------------------------------------------------------ //

    /// Returns subscriptions for this panel, routing all events through
    /// `Message::ForPanel { dash, panel, msg }` so the outer App can
    /// dispatch them to the correct TradingPanel.
    ///
    /// Uses `Subscription::run_with` + a named function to satisfy iced 0.14's
    /// requirement that closures passed to `Subscription::map` are zero-sized.
    pub fn subscription(&self, dash: usize, panel: usize) -> Vec<Subscription<Message>> {
        // `Subscription::map` requires a zero-sized (non-capturing) closure.
        // We use `with((dash, panel))` to embed the tag into the stream output,
        // then map with a non-capturing closure that destructures the tuple.
        vec![ws::client::connect(self.ws_url.clone())
            .with((dash, panel))
            .map(|((d, p), ev)| Message::ForPanel {
                dash: d,
                panel: p,
                msg: PanelMessage::WsEvent(ev),
            })]
    }

    // ------------------------------------------------------------------ //
    //  Helpers
    // ------------------------------------------------------------------ //

    /// Returns the visible main panels in display order.
    pub fn visible_main_panels(&self) -> Vec<PanelId> {
        self.panel_order
            .iter()
            .filter(|p| {
                p.is_main_panel() && self.panel_visible.get(p).copied().unwrap_or(true)
            })
            .copied()
            .collect()
    }
}

// ------------------------------------------------------------------ //
//  Free helpers
// ------------------------------------------------------------------ //

fn format_step(step: f64) -> String {
    if step >= 1.0 {
        format!("{:.0}", step)
    } else if step >= 0.1 {
        format!("{:.1}", step)
    } else if step >= 0.01 {
        format!("{:.2}", step)
    } else {
        format!("{:.4}", step)
    }
}

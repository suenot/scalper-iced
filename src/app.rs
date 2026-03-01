use iced::widget::{column, container, row, text};
use iced::{Element, Length, Subscription};

use crate::message::{Message, Side};
// models imported via ws module
use crate::panel::order::OrderPanel;
use crate::panel::pnl::StubPnL;
use crate::panel::position::{StubPosition, PositionSide};
use crate::price_axis::{FollowMode, PriceAxis};
use crate::theme as t;
use crate::widget::cluster_canvas::ClusterCanvas;
use crate::widget::orderbook_canvas::OrderBookCanvas;
use crate::widget::tape::Tape;
use crate::widget::tick_chart_canvas::TickChartCanvas;
use crate::ws;

pub struct App {
    // Config
    ws_url: String,
    symbol: String,

    // Shared state
    price_axis: PriceAxis,

    // Widgets
    orderbook_canvas: OrderBookCanvas,
    cluster_canvas: ClusterCanvas,
    tick_chart_canvas: TickChartCanvas,
    tape: Tape,

    // Stub panels
    position: StubPosition,
    order_panel: OrderPanel,
    pnl: StubPnL,

    // Connection state
    ws_connected: bool,
    last_price: f64,
    message_count: u64,
}

impl App {
    pub fn new(ws_url: String, symbol: String, price_step: f64) -> (Self, iced::Task<Message>) {
        let price_axis = PriceAxis::new(price_step);

        let app = Self {
            ws_url,
            symbol,
            price_axis: price_axis.clone(),
            orderbook_canvas: OrderBookCanvas::new(price_axis.clone()),
            cluster_canvas: ClusterCanvas::new(price_axis.clone()),
            tick_chart_canvas: TickChartCanvas::new(price_axis.clone()),
            tape: Tape::new(),
            position: StubPosition::default(),
            order_panel: OrderPanel::default(),
            pnl: StubPnL::default(),
            ws_connected: false,
            last_price: 0.0,
            message_count: 0,
        };

        (app, iced::Task::none())
    }

    pub fn title(&self) -> String {
        let status = if self.ws_connected { "OK" } else { "..." };
        let mode = match self.price_axis.follow_mode {
            FollowMode::Auto => "AUTO",
            FollowMode::Locked => "LOCK",
            FollowMode::Manual => "MAN",
        };
        format!(
            "Scalper | {} | {:.1} | [{}] [{}]",
            self.symbol, self.last_price, status, mode
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::WsEvent(event) => match event {
                ws::WsEvent::Connected => {
                    self.ws_connected = true;
                    println!("[WS] Connected");
                }
                ws::WsEvent::Disconnected => {
                    self.ws_connected = false;
                    println!("[WS] Disconnected");
                }
                ws::WsEvent::MessageReceived(msg) => {
                    self.message_count += 1;

                    if let Some(ob) = msg.orderbook {
                        let mid = ob.mid_price();
                        self.last_price = mid;
                        self.price_axis.update_last_price(mid);
                        self.position.mark_price = mid;

                        self.orderbook_canvas.update_data(ob);
                        self.orderbook_canvas.update_price_axis(&self.price_axis);
                    }

                    if let Some(clusters) = msg.clusters {
                        self.cluster_canvas.update_data(clusters);
                        self.cluster_canvas.update_price_axis(&self.price_axis);
                    }

                    if let Some(ticks) = msg.ticks {
                        self.tick_chart_canvas.update_data(ticks.clone());
                        self.tick_chart_canvas.update_price_axis(&self.price_axis);
                        self.tape.update_ticks(ticks);
                    }
                }
                ws::WsEvent::Error(e) => {
                    eprintln!("[WS] Error: {}", e);
                }
            },

            Message::OrderBookClicked { price, side } => {
                let side_str = match side {
                    Side::Buy => "BUY LIMIT",
                    Side::Sell => "SELL LIMIT",
                };
                println!("[TRADE] {} @ {:.2} (stub)", side_str, price);
            }

            Message::Scroll(delta) => {
                self.price_axis.on_scroll(delta);
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
            }

            Message::Zoom(delta) => {
                self.price_axis.on_zoom(delta);
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
            }

            Message::SnapToPrice => {
                self.price_axis.snap_to_price();
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
            }

            Message::ToggleFollowMode => {
                self.price_axis.toggle_follow_mode();
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                println!(
                    "[UI] Follow mode: {:?}",
                    self.price_axis.follow_mode
                );
            }

            Message::BuyMarket => {
                println!("[TRADE] BUY MARKET @ {:.2} (stub)", self.last_price);
            }
            Message::SellMarket => {
                println!("[TRADE] SELL MARKET @ {:.2} (stub)", self.last_price);
            }
            Message::ClosePosition => {
                println!("[TRADE] CLOSE POSITION (stub)");
                self.position.side = PositionSide::None;
                self.position.size = 0.0;
                self.position.unrealized_pnl = 0.0;
            }
            Message::CancelAllOrders => {
                println!("[TRADE] CANCEL ALL ORDERS (stub)");
            }
            Message::EmergencyCloseAll => {
                println!("[TRADE] EMERGENCY CLOSE ALL (stub)");
                self.position.side = PositionSide::None;
                self.position.size = 0.0;
                self.position.unrealized_pnl = 0.0;
            }

            Message::VolumeFilterChanged(val) => {
                self.tape.volume_filter = val;
            }

            Message::QuantityChanged(qty) => {
                self.order_panel.quantity = qty;
            }

            Message::NoOp => {}
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Status bar
        let ws_indicator = if self.ws_connected {
            text("WS: Connected").size(11).color(t::BID_GREEN)
        } else {
            text("WS: Disconnected").size(11).color(t::ASK_RED)
        };

        let mode_text = match self.price_axis.follow_mode {
            FollowMode::Auto => text("AUTO").size(11).color(t::BID_GREEN),
            FollowMode::Locked => text("LOCKED").size(11).color(t::SPREAD_YELLOW),
            FollowMode::Manual => text("MANUAL").size(11).color(t::ASK_RED),
        };

        let status_bar = container(
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
                text(format!("  Msgs: {}", self.message_count))
                    .size(10)
                    .color(t::TEXT_DIM),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center)
            .padding(4),
        )
        .style(|_theme: &_| container::Style {
            background: Some(t::HEADER_BG.into()),
            ..Default::default()
        })
        .width(Length::Fill);

        // Main panels row: Tick Chart | Cluster Chart | OrderBook | Tape
        let main_panels = row![
            // Tick chart (left, 15% width)
            container(self.tick_chart_canvas.view())
                .width(Length::FillPortion(15))
                .height(Length::Fill),
            // Cluster chart (20% width)
            container(self.cluster_canvas.view())
                .width(Length::FillPortion(20))
                .height(Length::Fill),
            // OrderBook (main, 40% width)
            container(self.orderbook_canvas.view())
                .width(Length::FillPortion(40))
                .height(Length::Fill),
            // Tape (right, 25% width)
            container(self.tape.view())
                .width(Length::FillPortion(25))
                .height(Length::Fill),
        ]
        .spacing(2)
        .height(Length::Fill);

        // Bottom panels row
        let bottom_panels = row![
            crate::panel::position::view(&self.position),
            crate::panel::order::view(&self.order_panel),
            crate::panel::pnl::view(&self.pnl),
        ]
        .spacing(2)
        .height(Length::Fixed(90.0));

        // Full layout
        let content = column![status_bar, main_panels, bottom_panels].spacing(2);

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

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            ws::client::connect(self.ws_url.clone()).map(Message::WsEvent),
            crate::hotkeys::hotkey_subscription(),
        ])
    }
}

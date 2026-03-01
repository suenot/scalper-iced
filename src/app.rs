use std::collections::HashMap;
use std::time::Instant;

use iced::event::Event;
use iced::mouse;
use iced::widget::{button, column, container, row, scrollable, stack, text, Row};
use iced::{Color, Element, Length, Subscription};

use crate::message::{Message, PanelId, Side};
use crate::panel::order::OrderPanel;
use crate::panel::pnl::StubPnL;
use crate::panel::position::{PositionSide, StubPosition};
use crate::price_axis::{FollowMode, PriceAxis};
use crate::settings::Settings;
use crate::theme as t;
use crate::widget::bubble_chart_canvas::BubbleChartCanvas;
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
    bubble_chart_canvas: BubbleChartCanvas,
    tape: Tape,

    // Stub panels
    position: StubPosition,
    order_panel: OrderPanel,
    pnl: StubPnL,

    // Connection state
    ws_connected: bool,
    last_price: f64,
    message_count: u64,

    // Panel order and visibility
    panel_order: Vec<PanelId>,
    panel_visible: HashMap<PanelId, bool>,

    // Drag/drop state (button reordering)
    dragging: Option<PanelId>,
    drag_did_move: bool,

    // Panel resize state (draggable dividers)
    panel_widths: HashMap<PanelId, u16>,
    resizing_divider: Option<usize>,  // index of divider being dragged
    resize_start_x: f32,
    resize_px_per_unit: f32,          // calculated on first move for 1:1 tracking
    resize_start_widths: Vec<(PanelId, u16)>,

    // FPS (all update calls = renders) + MPS (WS messages/sec)
    render_count: u32,
    ws_msg_count: u32,
    fps: u32,
    mps: u32,
    last_fps_instant: Instant,

    // Help overlay
    show_help: bool,
}

impl App {
    pub fn new(ws_url: String, symbol: String, price_step: f64) -> (Self, iced::Task<Message>) {
        let price_axis = PriceAxis::new(price_step);

        // Defaults
        let mut panel_order = PanelId::default_order();
        let mut panel_visible = HashMap::from([
            (PanelId::ClusterChart, true),
            (PanelId::TickChart, false),
            (PanelId::BubbleChart, true),
            (PanelId::OrderBook, true),
            (PanelId::Tape, false),
            (PanelId::BottomBar, false),
        ]);
        let mut panel_widths = HashMap::from([
            (PanelId::ClusterChart, 20),
            (PanelId::TickChart, 15),
            (PanelId::BubbleChart, 15),
            (PanelId::OrderBook, 30),
            (PanelId::Tape, 20),
        ]);

        // Override from saved settings
        if let Some(saved) = Settings::load() {
            let loaded_order = saved.to_panel_order();
            if !loaded_order.is_empty() {
                panel_order = loaded_order;
            }
            let loaded_vis = saved.to_panel_visible();
            if !loaded_vis.is_empty() {
                panel_visible = loaded_vis;
            }
            let loaded_widths = saved.to_panel_widths();
            if !loaded_widths.is_empty() {
                panel_widths = loaded_widths;
            }
        }

        let app = Self {
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
            panel_order,
            panel_visible,
            dragging: None,
            drag_did_move: false,
            panel_widths,
            resizing_divider: None,
            resize_start_x: 0.0,
            resize_px_per_unit: 10.0,
            resize_start_widths: Vec::new(),
            render_count: 0,
            ws_msg_count: 0,
            fps: 0,
            mps: 0,
            last_fps_instant: Instant::now(),
            show_help: false,
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
        self.render_count += 1;

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
                    self.ws_msg_count += 1;

                    // Track if price axis changed (only orderbook updates it)
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
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
            }

            Message::Zoom(delta) => {
                self.price_axis.on_zoom(delta);
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
            }

            Message::ChangePriceStep(delta) => {
                self.price_axis.on_change_price_step(delta);
                println!("[UI] Price step: {:.4}", self.price_axis.display_step);
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
            }

            Message::SnapToPrice => {
                self.price_axis.snap_to_price();
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
            }

            Message::ToggleFollowMode => {
                self.price_axis.toggle_follow_mode();
                self.orderbook_canvas.update_price_axis(&self.price_axis);
                self.cluster_canvas.update_price_axis(&self.price_axis);
                self.tick_chart_canvas.update_price_axis(&self.price_axis);
                self.bubble_chart_canvas.update_price_axis(&self.price_axis);
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

            Message::TogglePanel(panel_id) => {
                let visible = self.panel_visible.entry(panel_id).or_insert(true);
                *visible = !*visible;
                self.save_settings();
            }

            Message::DragStart(panel_id) => {
                self.dragging = Some(panel_id);
                self.drag_did_move = false;
            }
            Message::DragOver(target) => {
                // Live reorder: as cursor enters another button while dragging,
                // immediately shift the dragged item to that position (like dnd-kit)
                if let Some(dragged) = self.dragging {
                    if dragged != target {
                        let from = self.panel_order.iter().position(|&p| p == dragged);
                        let to = self.panel_order.iter().position(|&p| p == target);
                        if let (Some(from_idx), Some(to_idx)) = (from, to) {
                            // Remove from old position and insert at new position
                            let item = self.panel_order.remove(from_idx);
                            self.panel_order.insert(to_idx, item);
                            self.drag_did_move = true;
                        }
                    }
                }
            }
            Message::DragEnd => {
                if let Some(dragged) = self.dragging {
                    if !self.drag_did_move {
                        let visible = self.panel_visible.entry(dragged).or_insert(true);
                        *visible = !*visible;
                    }
                }
                let did_change = self.dragging.is_some();
                self.dragging = None;
                self.drag_did_move = false;
                if did_change {
                    self.save_settings();
                }
            }

            Message::ResizeStart { divider_index, x } => {
                // Snapshot current visible panels + widths so we can calculate delta
                let visible: Vec<(PanelId, u16)> = self
                    .panel_order
                    .iter()
                    .filter(|p| p.is_main_panel() && self.panel_visible.get(p).copied().unwrap_or(true))
                    .map(|&p| (p, *self.panel_widths.get(&p).unwrap_or(&20)))
                    .collect();
                self.resizing_divider = Some(divider_index);
                self.resize_start_x = x;
                self.resize_start_widths = visible;
            }
            Message::ResizeMove(x) => {
                if let Some(div_idx) = self.resizing_divider {
                    // On first move: capture start_x and compute pixels-per-unit
                    if self.resize_start_x == 0.0 {
                        self.resize_start_x = x;
                        // Estimate: divider is at x pixels from left edge
                        // In unit space, divider is at left_sum units out of total
                        let left_sum: f32 = self.resize_start_widths[..=div_idx]
                            .iter()
                            .map(|(_, w)| *w as f32)
                            .sum();
                        if left_sum > 0.1 {
                            // x ≈ left_sum * px_per_unit → px_per_unit ≈ x / left_sum
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
                        let new_left = ((left_w as i32 + delta_units).max(5) as u16).min(pair_total - 5);
                        let new_right = pair_total - new_left;

                        self.panel_widths.insert(left_id, new_left);
                        self.panel_widths.insert(right_id, new_right);
                    }
                }
            }
            Message::ResizeEnd => {
                if self.resizing_divider.is_some() {
                    self.resizing_divider = None;
                    self.save_settings();
                }
            }

            Message::ToggleHelp => {
                self.show_help = !self.show_help;
            }

            Message::FpsTick => {
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_fps_instant);
                if elapsed.as_millis() > 0 {
                    let secs = elapsed.as_secs_f64();
                    self.fps = (self.render_count as f64 / secs).round() as u32;
                    self.mps = (self.ws_msg_count as f64 / secs).round() as u32;
                }
                self.render_count = 0;
                self.ws_msg_count = 0;
                self.last_fps_instant = now;
            }

            Message::NoOp => {}
        }
    }

    fn save_settings(&self) {
        Settings::save(&self.panel_order, &self.panel_visible, &self.panel_widths);
    }

    /// Returns visible main panels in display order.
    fn visible_main_panels(&self) -> Vec<PanelId> {
        self.panel_order
            .iter()
            .filter(|p| p.is_main_panel() && self.panel_visible.get(p).copied().unwrap_or(true))
            .copied()
            .collect()
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

        // Build toggle buttons in panel_order with drag/drop via mouse_area
        // Like dnd-kit: press → grab, drag over → items shift live, release → finalize
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
            let btn: Element<'_, Message> = container(
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

            let wrapped: Element<'_, Message> = iced::widget::mouse_area(btn)
                .on_press(Message::DragStart(panel_id))
                .on_release(Message::DragEnd)
                .on_enter(Message::DragOver(panel_id))
                .into();

            toggle_buttons = toggle_buttons.push(wrapped);
        }

        let mps_color = if self.mps >= 8 {
            t::BID_GREEN
        } else if self.mps >= 4 {
            t::SPREAD_YELLOW
        } else {
            t::ASK_RED
        };

        let center_btn = button(text("Center").size(10).color(t::TEXT_BRIGHT))
            .on_press(Message::SnapToPrice)
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

        let left_bar = row![
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
        .align_y(iced::Alignment::Center);

        let stats_label = row![
            text(format!("{} mps", self.mps)).size(10).color(mps_color),
            text(" | ").size(10).color(t::TEXT_DIM),
            text(format!("{} fps", self.fps)).size(10).color(t::TEXT_DIM),
        ].spacing(0);

        let help_btn = button(text("?").size(10).color(t::TEXT_BRIGHT))
            .on_press(Message::ToggleHelp)
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

        let status_bar = container(
            row![
                container(left_bar).width(Length::Fill),
                stats_label,
                help_btn,
            ]
            .spacing(8)
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
                let divider: Element<'_, Message> = iced::widget::mouse_area(
                    container(text("").size(1))
                        .width(Length::Fixed(4.0))
                        .height(Length::Fill)
                        .style(move |_theme: &_| container::Style {
                            background: Some(iced::Background::Color(divider_bg)),
                            ..Default::default()
                        }),
                )
                .on_press(Message::ResizeStart { divider_index: div_idx, x: 0.0 })
                .on_release(Message::ResizeEnd)
                .interaction(mouse::Interaction::ResizingHorizontally)
                .into();

                main_panels = main_panels.push(divider);
            }

            let width = *self.panel_widths.get(&panel_id).unwrap_or(&20);
            let panel_widget: Element<'_, Message> = match panel_id {
                PanelId::TickChart => {
                    container(self.tick_chart_canvas.view())
                        .width(Length::FillPortion(width))
                        .height(Length::Fill)
                        .into()
                }
                PanelId::ClusterChart => {
                    container(self.cluster_canvas.view())
                        .width(Length::FillPortion(width))
                        .height(Length::Fill)
                        .into()
                }
                PanelId::BubbleChart => {
                    container(self.bubble_chart_canvas.view())
                        .width(Length::FillPortion(width))
                        .height(Length::Fill)
                        .into()
                }
                PanelId::OrderBook => {
                    container(self.orderbook_canvas.view())
                        .width(Length::FillPortion(width))
                        .height(Length::Fill)
                        .into()
                }
                PanelId::Tape => {
                    container(self.tape.view())
                        .width(Length::FillPortion(width))
                        .height(Length::Fill)
                        .into()
                }
                PanelId::BottomBar => continue,
            };

            main_panels = main_panels.push(panel_widget);
        }

        // Full layout
        let mut content = column![status_bar, main_panels].spacing(2);

        // Bottom panels row — only if BottomBar is visible
        if self.panel_visible.get(&PanelId::BottomBar).copied().unwrap_or(true) {
            let bottom_panels = row![
                crate::panel::position::view(&self.position),
                crate::panel::order::view(&self.order_panel),
                crate::panel::pnl::view(&self.pnl),
            ]
            .spacing(2)
            .height(Length::Fixed(90.0));

            content = content.push(bottom_panels);
        }

        let base = container(content)
            .style(|_theme: &_| container::Style {
                background: Some(t::BACKGROUND.into()),
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(2);

        if self.show_help {
            let help_overlay = self.help_overlay();
            stack![base, help_overlay]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            base.into()
        }
    }

    fn help_overlay(&self) -> Element<'_, Message> {
        let hk_row = |key: &'static str, desc: &'static str| -> Element<'_, Message> {
            row![
                container(text(key).size(12).color(t::SPREAD_YELLOW))
                    .width(Length::Fixed(120.0)),
                text(desc).size(12).color(t::TEXT_DIM),
            ]
            .spacing(8)
            .into()
        };

        let help_content = column![
            row![
                text("Hotkeys").size(16).color(t::TEXT_BRIGHT),
                container(text("").size(1)).width(Length::Fill),
                button(text("X").size(12).color(t::TEXT_BRIGHT))
                    .on_press(Message::ToggleHelp)
                    .padding([2, 8])
                    .style(|_theme: &_, _status| button::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(1.0, 0.3, 0.3, 0.3))),
                        text_color: t::TEXT_BRIGHT,
                        border: iced::Border { radius: 3.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
            ]
            .align_y(iced::Alignment::Center),
            text("").size(6),

            text("Navigation").size(13).color(t::BID_GREEN),
            hk_row("LShift", "Center orderbook on spread"),
            hk_row("R", "Toggle follow mode (Auto/Manual)"),
            hk_row("Scroll", "Scroll price axis"),
            hk_row("Ctrl+Scroll", "Change price step (grouping)"),
            hk_row("Shift+Scroll", "Zoom (change row height)"),
            text("").size(4),

            text("Trading (stub)").size(13).color(t::BID_GREEN),
            hk_row("T", "Buy at market"),
            hk_row("Y", "Sell at market"),
            hk_row("D", "Close position"),
            hk_row("Space", "Cancel all limit orders"),
            hk_row("Escape", "Emergency close all"),
            text("").size(4),

            text("Panels").size(13).color(t::BID_GREEN),
            hk_row("Ctrl+1", "Toggle Clusters"),
            hk_row("Ctrl+2", "Toggle Ticks"),
            hk_row("Ctrl+3", "Toggle Bubbles"),
            hk_row("Ctrl+4", "Toggle OrderBook"),
            hk_row("Ctrl+5", "Toggle Tape"),
            hk_row("Ctrl+6", "Toggle Bottom Bar"),
            text("").size(4),

            text("Other").size(13).color(t::BID_GREEN),
            hk_row("F1", "Toggle this help"),
            hk_row("Click btn", "Show/hide panel"),
            hk_row("Drag btn", "Reorder panels"),
            hk_row("Drag divider", "Resize panels"),
        ]
        .spacing(3)
        .padding(20)
        .width(Length::Fixed(340.0));

        // Semi-transparent backdrop + centered help card
        iced::widget::mouse_area(
            container(
                container(scrollable(help_content))
                    .style(|_theme: &_| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.18, 0.97))),
                        border: iced::Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                        },
                        ..Default::default()
                    })
                    .max_height(500.0),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(|_theme: &_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                ..Default::default()
            }),
        )
        .on_press(Message::ToggleHelp)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            ws::client::connect(self.ws_url.clone()).map(Message::WsEvent),
            crate::hotkeys::hotkey_subscription(),
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::FpsTick),
        ];

        // While resizing, listen for global mouse events
        if self.resizing_divider.is_some() {
            subs.push(
                iced::event::listen_with(|event, _status, _id| match event {
                    Event::Mouse(mouse::Event::CursorMoved { position }) => {
                        Some(Message::ResizeMove(position.x))
                    }
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        Some(Message::ResizeEnd)
                    }
                    _ => None,
                }),
            );
        }

        Subscription::batch(subs)
    }
}

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

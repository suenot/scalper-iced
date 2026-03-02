use std::time::Instant;

use iced::event::Event;
use iced::mouse;
use iced::widget::{button, column, container, row, scrollable, stack, text, Column, Row};
use iced::{Color, Element, Length, Subscription};

use crate::dashboard::{Dashboard, GRID_PRESETS};
use crate::message::Message;
use crate::settings::{panel_id_to_str, str_to_panel_id, DashboardConfig, PanelConfig, Settings};
use crate::theme as t;
use crate::trading_panel::TradingPanel;

pub struct App {
    // Multi-dashboard state
    pub dashboards: Vec<Dashboard>,
    pub active_dashboard: usize,
    pub active_panel: usize, // flat index in active dashboard

    // Config for creating new panels
    ws_base_url: String,
    clusters_n: usize,
    ticks_n: usize,
    default_price_step: f64,
    default_symbol: String,

    // Global counters
    render_count: u32,
    fps: u32,
    mps: u32,
    last_fps_instant: Instant,

    // UI state
    show_help: bool,
}

impl App {
    pub fn new(
        ws_base_url: String,
        default_symbol: String,
        default_price_step: f64,
        clusters_n: usize,
        ticks_n: usize,
    ) -> (Self, iced::Task<Message>) {
        let mut dashboards: Vec<Dashboard> = Vec::new();
        let mut active_dashboard = 0;

        // Try to restore from saved settings
        if let Some(saved) = Settings::load() {
            active_dashboard =
                saved.active_dashboard.min(saved.dashboards.len().saturating_sub(1));
            for dash_cfg in saved.dashboards {
                let mut d = Dashboard::new(&dash_cfg.name, dash_cfg.cols, dash_cfg.rows);
                for (i, panel_cfg) in dash_cfg.panels.into_iter().enumerate() {
                    if let (Some(cfg), Some(slot)) = (panel_cfg, d.panels.get_mut(i)) {
                        let price_step =
                            if cfg.price_step > 0.0 { cfg.price_step } else { default_price_step };
                        let mut tp = TradingPanel::new(
                            cfg.symbol,
                            price_step,
                            ws_base_url.clone(),
                            clusters_n,
                            ticks_n,
                        );
                        // Restore panel order
                        let order: Vec<_> =
                            cfg.panel_order.iter().filter_map(|s| str_to_panel_id(s)).collect();
                        if !order.is_empty() {
                            tp.panel_order = order;
                        }
                        // Restore visibility
                        for (k, v) in &cfg.panel_visible {
                            if let Some(pid) = str_to_panel_id(k) {
                                tp.panel_visible.insert(pid, *v);
                            }
                        }
                        // Restore widths
                        for (k, v) in &cfg.panel_widths {
                            if let Some(pid) = str_to_panel_id(k) {
                                tp.panel_widths.insert(pid, *v);
                            }
                        }
                        *slot = Some(tp);
                    }
                }
                dashboards.push(d);
            }
        }

        // Default: one dashboard with a 1×1 grid and the default symbol
        if dashboards.is_empty() {
            let mut d = Dashboard::new("Tab 1", 1, 1);
            d.panels[0] = Some(TradingPanel::new(
                default_symbol.clone(),
                default_price_step,
                ws_base_url.clone(),
                clusters_n,
                ticks_n,
            ));
            dashboards.push(d);
        }

        let app = Self {
            dashboards,
            active_dashboard,
            active_panel: 0,
            ws_base_url,
            clusters_n,
            ticks_n,
            default_price_step,
            default_symbol,
            render_count: 0,
            fps: 0,
            mps: 0,
            last_fps_instant: Instant::now(),
            show_help: false,
        };

        (app, iced::Task::none())
    }

    pub fn title(&self) -> String {
        let sym = self
            .dashboards
            .get(self.active_dashboard)
            .and_then(|d| d.panels.iter().flatten().next())
            .map(|p| p.symbol.as_str())
            .unwrap_or("---");
        format!("Scalper | {}", sym)
    }

    // ------------------------------------------------------------------ //
    //  Update
    // ------------------------------------------------------------------ //

    pub fn update(&mut self, message: Message) {
        self.render_count += 1;

        match message {
            Message::ForPanel { dash, panel, msg } => {
                if let Some(dashboard) = self.dashboards.get_mut(dash) {
                    if let Some(Some(tp)) = dashboard.panels.get_mut(panel) {
                        tp.update(msg);
                    }
                }
            }

            Message::ActivePanelAction(msg) => {
                let dash = self.active_dashboard;
                let panel_idx = self.active_panel;
                if let Some(dashboard) = self.dashboards.get_mut(dash) {
                    // Try the tracked active panel first; fallback to first non-None
                    let found = if let Some(Some(tp)) = dashboard.panels.get_mut(panel_idx) {
                        tp.update(msg.clone());
                        true
                    } else {
                        false
                    };
                    if !found {
                        if let Some(tp) = dashboard.panels.iter_mut().flatten().next() {
                            tp.update(msg);
                        }
                    }
                }
            }

            Message::AddDashboard => {
                let name = format!("Tab {}", self.dashboards.len() + 1);
                self.dashboards.push(Dashboard::new(name, 1, 1));
                self.active_dashboard = self.dashboards.len() - 1;
                self.active_panel = 0;
                self.save_settings();
            }

            Message::RemoveDashboard(idx) => {
                if self.dashboards.len() > 1 && idx < self.dashboards.len() {
                    self.dashboards.remove(idx);
                    if self.active_dashboard >= self.dashboards.len() {
                        self.active_dashboard = self.dashboards.len() - 1;
                    }
                    self.active_panel = 0;
                    self.save_settings();
                }
            }

            Message::SwitchDashboard(idx) => {
                if idx < self.dashboards.len() {
                    self.active_dashboard = idx;
                    self.active_panel = 0;
                }
            }

            Message::SetGridLayout { dash, cols, rows } => {
                if let Some(dashboard) = self.dashboards.get_mut(dash) {
                    dashboard.resize_grid(cols, rows);
                }
                self.active_panel = 0;
                self.save_settings();
            }

            Message::SetPanelSymbol { dash, panel, symbol } => {
                if let Some(dashboard) = self.dashboards.get_mut(dash) {
                    if panel < dashboard.panels.len() {
                        dashboard.panels[panel] = Some(TradingPanel::new(
                            symbol,
                            self.default_price_step,
                            self.ws_base_url.clone(),
                            self.clusters_n,
                            self.ticks_n,
                        ));
                    }
                }
                self.save_settings();
            }

            // Global mouse tracking for panel resize (listen_with can't capture vars)
            Message::GlobalMouseMove(x) => {
                use crate::panel_message::PanelMessage;
                for dashboard in &mut self.dashboards {
                    for tp in dashboard.panels.iter_mut().flatten() {
                        if tp.resizing_divider.is_some() {
                            tp.update(PanelMessage::ResizeMove(x));
                        }
                    }
                }
            }
            Message::GlobalMouseRelease => {
                use crate::panel_message::PanelMessage;
                for dashboard in &mut self.dashboards {
                    for tp in dashboard.panels.iter_mut().flatten() {
                        if tp.resizing_divider.is_some() {
                            tp.update(PanelMessage::ResizeEnd);
                        }
                    }
                }
                self.save_settings();
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

                    let total_ws: u32 = self
                        .dashboards
                        .iter()
                        .flat_map(|d| d.panels.iter().flatten())
                        .map(|p| p.ws_msg_count)
                        .sum();
                    self.mps = (total_ws as f64 / secs).round() as u32;

                    for tp in self
                        .dashboards
                        .iter_mut()
                        .flat_map(|d| d.panels.iter_mut().flatten())
                    {
                        tp.ws_msg_count = 0;
                    }
                }
                self.render_count = 0;
                self.last_fps_instant = now;
            }

            Message::NoOp => {}
        }
    }

    // ------------------------------------------------------------------ //
    //  Settings persistence
    // ------------------------------------------------------------------ //

    fn save_settings(&self) {
        let dashboards: Vec<DashboardConfig> = self
            .dashboards
            .iter()
            .map(|d| DashboardConfig {
                name: d.name.clone(),
                cols: d.cols,
                rows: d.rows,
                panels: d
                    .panels
                    .iter()
                    .map(|p| {
                        p.as_ref().map(|tp| PanelConfig {
                            symbol: tp.symbol.clone(),
                            price_step: tp.price_axis.display_step,
                            panel_order: tp
                                .panel_order
                                .iter()
                                .map(|pid| panel_id_to_str(*pid))
                                .collect(),
                            panel_visible: tp
                                .panel_visible
                                .iter()
                                .map(|(k, v)| (panel_id_to_str(*k), *v))
                                .collect(),
                            panel_widths: tp
                                .panel_widths
                                .iter()
                                .map(|(k, v)| (panel_id_to_str(*k), *v))
                                .collect(),
                        })
                    })
                    .collect(),
            })
            .collect();

        Settings::save(&Settings {
            dashboards,
            active_dashboard: self.active_dashboard,
        });
    }

    // ------------------------------------------------------------------ //
    //  View
    // ------------------------------------------------------------------ //

    pub fn view(&self) -> Element<'_, Message> {
        let dash = self.active_dashboard;
        let dashboard = match self.dashboards.get(dash) {
            Some(d) => d,
            None => return text("No dashboard").into(),
        };

        // === Tab bar (left side) ===
        let mut tabs = Row::new().spacing(2).align_y(iced::Alignment::Center);
        for (i, d) in self.dashboards.iter().enumerate() {
            let is_active = i == self.active_dashboard;
            let tab_btn = button(
                text(d.name.as_str())
                    .size(11)
                    .color(if is_active { t::TEXT_BRIGHT } else { t::TEXT_DIM }),
            )
            .on_press(Message::SwitchDashboard(i))
            .padding([2, 8])
            .style(move |_theme: &_, _status| button::Style {
                background: Some(iced::Background::Color(if is_active {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.15)
                } else {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.04)
                })),
                text_color: if is_active { t::TEXT_BRIGHT } else { t::TEXT_DIM },
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
            tabs = tabs.push(tab_btn);

            // "×" close button (hidden when only one tab)
            if self.dashboards.len() > 1 {
                let close_btn = button(text("×").size(11).color(t::TEXT_DIM))
                    .on_press(Message::RemoveDashboard(i))
                    .padding([2, 4])
                    .style(|_theme: &_, _status| button::Style {
                        background: None,
                        text_color: t::TEXT_DIM,
                        ..Default::default()
                    });
                tabs = tabs.push(close_btn);
            }
        }

        // "+" add-dashboard button
        let add_tab_btn = button(text("+").size(11).color(t::TEXT_DIM))
            .on_press(Message::AddDashboard)
            .padding([2, 6])
            .style(|_theme: &_, _status| button::Style {
                background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.04))),
                text_color: t::TEXT_DIM,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        tabs = tabs.push(add_tab_btn);

        // === Grid preset picker (right side) ===
        let mut preset_btns = Row::new().spacing(2).align_y(iced::Alignment::Center);
        for &(cols, rows) in GRID_PRESETS {
            let is_current = dashboard.cols == cols && dashboard.rows == rows;
            let label = format!("{}×{}", cols, rows);
            let preset_btn = button(
                text(label)
                    .size(10)
                    .color(if is_current { t::TEXT_BRIGHT } else { t::TEXT_DIM }),
            )
            .on_press(Message::SetGridLayout { dash, cols, rows })
            .padding([2, 5])
            .style(move |_theme: &_, _status| button::Style {
                background: Some(iced::Background::Color(if is_current {
                    Color::from_rgba(0.3, 0.6, 1.0, 0.3)
                } else {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.04)
                })),
                text_color: if is_current { t::TEXT_BRIGHT } else { t::TEXT_DIM },
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
            preset_btns = preset_btns.push(preset_btn);
        }

        // === Stats + help ===
        let mps_color = if self.mps >= 8 {
            t::BID_GREEN
        } else if self.mps >= 4 {
            t::SPREAD_YELLOW
        } else {
            t::ASK_RED
        };
        let stats = row![
            text(format!("{} mps", self.mps)).size(10).color(mps_color),
            text(" | ").size(10).color(t::TEXT_DIM),
            text(format!("{} fps", self.fps)).size(10).color(t::TEXT_DIM),
        ]
        .spacing(0);

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

        // === Top bar ===
        let top_bar = container(
            row![
                tabs,
                container(text("")).width(Length::Fill), // spacer
                preset_btns,
                stats,
                help_btn,
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding([2, 4]),
        )
        .style(|_theme: &_| container::Style {
            background: Some(t::HEADER_BG.into()),
            ..Default::default()
        })
        .width(Length::Fill);

        // === Grid ===
        let cols = dashboard.cols;
        let rows = dashboard.rows;

        let grid_widget: Element<'_, Message> = if rows == 1 {
            let mut r = Row::new().spacing(2).height(Length::Fill);
            for col in 0..cols {
                let panel_idx = col;
                let cell = render_cell(dash, panel_idx, dashboard);
                r = r.push(cell);
            }
            r.into()
        } else {
            let mut col_widget = Column::new().spacing(2).height(Length::Fill);
            for row_idx in 0..rows {
                let mut r = Row::new().spacing(2).height(Length::Fill);
                for col in 0..cols {
                    let panel_idx = row_idx * cols + col;
                    let cell = render_cell(dash, panel_idx, dashboard);
                    r = r.push(cell);
                }
                col_widget = col_widget.push(r);
            }
            col_widget.into()
        };

        let content = column![top_bar, grid_widget].spacing(0);

        let base = container(content)
            .style(|_theme: &_| container::Style {
                background: Some(t::BACKGROUND.into()),
                ..Default::default()
            })
            .width(Length::Fill)
            .height(Length::Fill);

        if self.show_help {
            let help_overlay = help_overlay();
            stack![base, help_overlay]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            base.into()
        }
    }

    // ------------------------------------------------------------------ //
    //  Subscription
    // ------------------------------------------------------------------ //

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subs: Vec<Subscription<Message>> = vec![
            crate::hotkeys::hotkey_subscription(),
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::FpsTick),
        ];

        // Global mouse events for panel resize (listen_with requires fn pointer, no capture)
        let any_resizing = self
            .dashboards
            .iter()
            .flat_map(|d| d.panels.iter().flatten())
            .any(|p| p.resizing_divider.is_some());
        if any_resizing {
            subs.push(iced::event::listen_with(|event, _status, _id| match event {
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    Some(Message::GlobalMouseMove(position.x))
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Some(Message::GlobalMouseRelease)
                }
                _ => None,
            }));
        }

        // Collect panel subscriptions (WS) from all panels
        for (di, dashboard) in self.dashboards.iter().enumerate() {
            for (pi, panel_slot) in dashboard.panels.iter().enumerate() {
                if let Some(tp) = panel_slot {
                    subs.extend(tp.subscription(di, pi));
                }
            }
        }

        Subscription::batch(subs)
    }
}

// ------------------------------------------------------------------ //
//  Free functions (no &self borrow so no conflict with dashboard borrow)
// ------------------------------------------------------------------ //

fn render_cell(dash: usize, panel_idx: usize, dashboard: &Dashboard) -> Element<'_, Message> {
    match dashboard.panels.get(panel_idx) {
        Some(Some(tp)) => tp
            .view()
            .map(move |msg| Message::ForPanel { dash, panel: panel_idx, msg }),
        _ => container(
            column![
                text("Empty").size(12).color(t::TEXT_DIM),
                text("Add symbol via CLI --symbol").size(10).color(t::TEXT_DIM),
            ]
            .spacing(4)
            .align_x(iced::alignment::Horizontal::Center)
            .padding(16),
        )
        .style(|_theme: &_| container::Style {
            background: Some(t::PANEL_BG.into()),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into(),
    }
}

fn help_overlay<'a>() -> Element<'a, Message> {
    let hk_row = |key: &'static str, desc: &'static str| -> Element<'a, Message> {
        row![
            container(text(key).size(12).color(t::SPREAD_YELLOW)).width(Length::Fixed(140.0)),
            text(desc).size(12).color(t::TEXT_DIM),
        ]
        .spacing(8)
        .into()
    };

    let help_content = column![
        row![
            text("Hotkeys").size(16).color(t::TEXT_BRIGHT),
            container(text("")).width(Length::Fill),
            button(text("X").size(12).color(t::TEXT_BRIGHT))
                .on_press(Message::ToggleHelp)
                .padding([2, 8])
                .style(|_theme: &_, _status| button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(1.0, 0.3, 0.3, 0.3))),
                    text_color: t::TEXT_BRIGHT,
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
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
        text("Dashboards").size(13).color(t::BID_GREEN),
        hk_row("Tab bar", "Switch / add / remove dashboards"),
        hk_row("1×1 … 3×3 picker", "Change grid layout"),
        text("").size(4),
        text("Other").size(13).color(t::BID_GREEN),
        hk_row("F1", "Toggle this help"),
        hk_row("Click panel btn", "Show/hide sub-panel"),
        hk_row("Drag panel btn", "Reorder sub-panels"),
        hk_row("Drag divider", "Resize sub-panels"),
    ]
    .spacing(3)
    .padding(20)
    .width(Length::Fixed(360.0));

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
                .max_height(520.0),
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

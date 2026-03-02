use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::panel_message::PanelMessage;
use crate::theme as t;

#[derive(Debug, Clone)]
pub struct StubPnL {
    pub realized_today: f64,
    pub unrealized: f64,
    pub total_today: f64,
    pub trades: i32,
    pub win_rate: f64,
    pub commissions: f64,
}

impl Default for StubPnL {
    fn default() -> Self {
        Self {
            realized_today: 0.0,
            unrealized: 0.0,
            total_today: 0.0,
            trades: 0,
            win_rate: 0.0,
            commissions: 0.0,
        }
    }
}

pub fn view(pnl: &StubPnL) -> Element<'_, PanelMessage> {
    let total_color = if pnl.total_today >= 0.0 {
        t::BID_GREEN
    } else {
        t::ASK_RED
    };

    container(
        column![
            text("P&L").size(11).color(t::TEXT_DIM),
            row![
                text(format!("Today: {:.2}", pnl.total_today))
                    .size(13)
                    .color(total_color),
            ],
            row![
                text(format!("Realized: {:.2}", pnl.realized_today))
                    .size(10)
                    .color(t::TEXT_DIM),
                text(format!("  Unreal: {:.2}", pnl.unrealized))
                    .size(10)
                    .color(t::TEXT_DIM),
            ]
            .spacing(4),
            row![
                text(format!("Trades: {}", pnl.trades))
                    .size(10)
                    .color(t::TEXT_DIM),
                text(format!("  WR: {:.0}%", pnl.win_rate))
                    .size(10)
                    .color(t::TEXT_DIM),
                text(format!("  Fee: {:.2}", pnl.commissions))
                    .size(10)
                    .color(t::TEXT_DIM),
            ]
            .spacing(4),
        ]
        .spacing(4)
        .padding(8),
    )
    .style(|_theme: &_| container::Style {
        background: Some(t::PANEL_BG.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

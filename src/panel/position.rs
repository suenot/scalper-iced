use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

use crate::panel_message::PanelMessage;
use crate::theme as t;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PositionSide {
    Long,
    Short,
    None,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StubPosition {
    pub symbol: String,
    pub side: PositionSide,
    pub size: f64,
    pub entry_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
}

impl Default for StubPosition {
    fn default() -> Self {
        Self {
            symbol: "BTCUSDT".into(),
            side: PositionSide::None,
            size: 0.0,
            entry_price: 0.0,
            mark_price: 0.0,
            unrealized_pnl: 0.0,
        }
    }
}

pub fn view(position: &StubPosition) -> Element<'_, PanelMessage> {
    let side_text = match position.side {
        PositionSide::Long => text("LONG").color(t::BID_GREEN).size(12),
        PositionSide::Short => text("SHORT").color(t::ASK_RED).size(12),
        PositionSide::None => text("FLAT").color(t::TEXT_DIM).size(12),
    };

    let pnl_color = if position.unrealized_pnl >= 0.0 {
        t::BID_GREEN
    } else {
        t::ASK_RED
    };

    container(
        column![
            text("Position").size(11).color(t::TEXT_DIM),
            row![
                side_text,
                text(format!(" {:.4}", position.size))
                    .size(12)
                    .color(t::TEXT_PRIMARY),
            ]
            .spacing(4),
            row![
                text(format!("Entry: {:.1}", position.entry_price))
                    .size(11)
                    .color(t::TEXT_DIM),
                text(format!("  PnL: {:.2}", position.unrealized_pnl))
                    .size(11)
                    .color(pnl_color),
            ]
            .spacing(4),
            row![
                button(text("Close All").size(11))
                    .on_press(PanelMessage::ClosePosition)
                    .padding([2, 8]),
                button(text("Cancel All").size(11))
                    .on_press(PanelMessage::CancelAllOrders)
                    .padding([2, 8]),
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

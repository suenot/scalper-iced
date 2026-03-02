use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length};

use crate::panel_message::PanelMessage;
use crate::theme as t;

pub struct OrderPanel {
    pub quantity: String,
}

impl Default for OrderPanel {
    fn default() -> Self {
        Self {
            quantity: "0.001".into(),
        }
    }
}

pub fn view(panel: &OrderPanel) -> Element<'_, PanelMessage> {
    container(
        column![
            text("Order").size(11).color(t::TEXT_DIM),
            row![
                text("Qty:").size(11).color(t::TEXT_PRIMARY),
                text_input("0.001", &panel.quantity)
                    .on_input(PanelMessage::QuantityChanged)
                    .size(11)
                    .width(Length::Fixed(80.0)),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
            row![
                button(text("BUY").size(12).color(iced::Color::WHITE))
                    .on_press(PanelMessage::BuyMarket)
                    .padding([4, 16])
                    .style(button::success),
                button(text("SELL").size(12).color(iced::Color::WHITE))
                    .on_press(PanelMessage::SellMarket)
                    .padding([4, 16])
                    .style(button::danger),
            ]
            .spacing(8),
        ]
        .spacing(6)
        .padding(8),
    )
    .style(|_theme: &_| container::Style {
        background: Some(t::PANEL_BG.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

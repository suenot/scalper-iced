use iced::keyboard;
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::Subscription;

use crate::message::Message;

pub fn hotkey_subscription() -> Subscription<Message> {
    keyboard::listen().map(|event| match event {
        keyboard::Event::KeyPressed { key, .. } => match key {
            Key::Named(Named::F1) => Message::BuyMarket,
            Key::Named(Named::F2) => Message::SellMarket,
            Key::Named(Named::F3) => Message::ClosePosition,
            Key::Named(Named::F5) => Message::CancelAllOrders,
            Key::Named(Named::Escape) => Message::EmergencyCloseAll,
            Key::Named(Named::Space) => Message::ToggleFollowMode,
            _ => Message::NoOp,
        },
        _ => Message::NoOp,
    })
}

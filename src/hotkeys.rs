use iced::keyboard;
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::Subscription;

use crate::message::Message;

pub fn hotkey_subscription() -> Subscription<Message> {
    keyboard::listen().map(|event| match event {
        keyboard::Event::KeyPressed {
            key, modifiers, ..
        } => {
            // Ctrl+1..4 toggle panels
            if modifiers.control() {
                match &key {
                    Key::Character(c) if c.as_str() == "1" => return Message::ToggleTickChart,
                    Key::Character(c) if c.as_str() == "2" => return Message::ToggleClusterChart,
                    Key::Character(c) if c.as_str() == "3" => return Message::ToggleOrderBook,
                    Key::Character(c) if c.as_str() == "4" => return Message::ToggleTape,
                    _ => {}
                }
            }
            match key {
                Key::Named(Named::F1) => Message::BuyMarket,
                Key::Named(Named::F2) => Message::SellMarket,
                Key::Named(Named::F3) => Message::ClosePosition,
                Key::Named(Named::F5) => Message::CancelAllOrders,
                Key::Named(Named::Escape) => Message::EmergencyCloseAll,
                Key::Named(Named::Space) => Message::ToggleFollowMode,
                _ => Message::NoOp,
            }
        }
        _ => Message::NoOp,
    })
}

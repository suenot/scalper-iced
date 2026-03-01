use iced::keyboard;
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::Subscription;

use crate::message::{Message, PanelId};

pub fn hotkey_subscription() -> Subscription<Message> {
    keyboard::listen().map(|event| match event {
        keyboard::Event::KeyPressed {
            key, modifiers, ..
        } => {
            // Ctrl+1..6 toggle panels
            if modifiers.control() {
                match &key {
                    Key::Character(c) if c.as_str() == "1" => return Message::TogglePanel(PanelId::ClusterChart),
                    Key::Character(c) if c.as_str() == "2" => return Message::TogglePanel(PanelId::TickChart),
                    Key::Character(c) if c.as_str() == "3" => return Message::TogglePanel(PanelId::BubbleChart),
                    Key::Character(c) if c.as_str() == "4" => return Message::TogglePanel(PanelId::OrderBook),
                    Key::Character(c) if c.as_str() == "5" => return Message::TogglePanel(PanelId::Tape),
                    Key::Character(c) if c.as_str() == "6" => return Message::TogglePanel(PanelId::BottomBar),
                    _ => {}
                }
            }
            match key {
                // CScalp-style hotkeys
                Key::Named(Named::F1) => Message::ToggleHelp,
                Key::Named(Named::Shift) => Message::SnapToPrice,  // Center orderbook on spread
                Key::Named(Named::Space) => Message::CancelAllOrders, // CScalp: Space = cancel limits
                Key::Character(ref c) if c.as_str() == "t" => Message::BuyMarket,
                Key::Character(ref c) if c.as_str() == "y" => Message::SellMarket,
                Key::Character(ref c) if c.as_str() == "d" => Message::ClosePosition,
                Key::Character(ref c) if c.as_str() == "r" => Message::ToggleFollowMode,
                Key::Named(Named::Escape) => Message::EmergencyCloseAll,
                _ => Message::NoOp,
            }
        }
        _ => Message::NoOp,
    })
}

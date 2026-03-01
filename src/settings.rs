use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::message::PanelId;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub panel_order: Vec<String>,
    pub panel_visible: HashMap<String, bool>,
    pub panel_widths: HashMap<String, u16>,
}

impl Settings {
    fn settings_path() -> PathBuf {
        // Store next to the executable, or in CWD
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push(SETTINGS_FILE);
        path
    }

    pub fn load() -> Option<Settings> {
        let path = Self::settings_path();
        let data = fs::read_to_string(&path).ok()?;
        let settings: Settings = serde_json::from_str(&data).ok()?;
        println!("[settings] Loaded from {}", path.display());
        Some(settings)
    }

    pub fn save(
        panel_order: &[PanelId],
        panel_visible: &HashMap<PanelId, bool>,
        panel_widths: &HashMap<PanelId, u16>,
    ) {
        let settings = Settings {
            panel_order: panel_order.iter().map(|p| panel_id_to_str(*p)).collect(),
            panel_visible: panel_visible
                .iter()
                .map(|(k, v)| (panel_id_to_str(*k), *v))
                .collect(),
            panel_widths: panel_widths
                .iter()
                .map(|(k, v)| (panel_id_to_str(*k), *v))
                .collect(),
        };

        let path = Self::settings_path();
        match serde_json::to_string_pretty(&settings) {
            Ok(json) => {
                if let Err(e) = fs::write(&path, json) {
                    eprintln!("[settings] Failed to save: {}", e);
                }
            }
            Err(e) => eprintln!("[settings] Failed to serialize: {}", e),
        }
    }

    pub fn to_panel_order(&self) -> Vec<PanelId> {
        self.panel_order
            .iter()
            .filter_map(|s| str_to_panel_id(s))
            .collect()
    }

    pub fn to_panel_visible(&self) -> HashMap<PanelId, bool> {
        self.panel_visible
            .iter()
            .filter_map(|(k, v)| str_to_panel_id(k).map(|id| (id, *v)))
            .collect()
    }

    pub fn to_panel_widths(&self) -> HashMap<PanelId, u16> {
        self.panel_widths
            .iter()
            .filter_map(|(k, v)| str_to_panel_id(k).map(|id| (id, *v)))
            .collect()
    }
}

fn panel_id_to_str(id: PanelId) -> String {
    match id {
        PanelId::ClusterChart => "cluster".into(),
        PanelId::TickChart => "tick".into(),
        PanelId::BubbleChart => "bubble".into(),
        PanelId::OrderBook => "orderbook".into(),
        PanelId::Tape => "tape".into(),
        PanelId::BottomBar => "bottom".into(),
    }
}

fn str_to_panel_id(s: &str) -> Option<PanelId> {
    match s {
        "cluster" => Some(PanelId::ClusterChart),
        "tick" => Some(PanelId::TickChart),
        "bubble" => Some(PanelId::BubbleChart),
        "orderbook" => Some(PanelId::OrderBook),
        "tape" => Some(PanelId::Tape),
        "bottom" => Some(PanelId::BottomBar),
        _ => None,
    }
}

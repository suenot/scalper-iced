use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::message::PanelId;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PanelConfig {
    pub symbol: String,
    pub price_step: f64,
    pub panel_order: Vec<String>,
    pub panel_visible: HashMap<String, bool>,
    pub panel_widths: HashMap<String, u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DashboardConfig {
    pub name: String,
    pub cols: usize,
    pub rows: usize,
    pub panels: Vec<Option<PanelConfig>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub dashboards: Vec<DashboardConfig>,
    pub active_dashboard: usize,
}

impl Settings {
    fn settings_path() -> PathBuf {
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

    pub fn save(settings: &Settings) {
        let path = Self::settings_path();
        match serde_json::to_string_pretty(settings) {
            Ok(json) => {
                if let Err(e) = fs::write(&path, json) {
                    eprintln!("[settings] Failed to save: {}", e);
                }
            }
            Err(e) => eprintln!("[settings] Failed to serialize: {}", e),
        }
    }
}

pub fn panel_id_to_str(id: PanelId) -> String {
    match id {
        PanelId::ClusterChart => "cluster".into(),
        PanelId::TickChart => "tick".into(),
        PanelId::BubbleChart => "bubble".into(),
        PanelId::OrderBook => "orderbook".into(),
        PanelId::Tape => "tape".into(),
        PanelId::BottomBar => "bottom".into(),
    }
}

pub fn str_to_panel_id(s: &str) -> Option<PanelId> {
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

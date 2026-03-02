use crate::panel_message::PanelMessage;

#[derive(Debug, Clone)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    ClusterChart,
    TickChart,
    BubbleChart,
    OrderBook,
    Tape,
    BottomBar,
}

impl PanelId {
    pub fn label(&self) -> &'static str {
        match self {
            PanelId::ClusterChart => "Clst",
            PanelId::TickChart => "Tick",
            PanelId::BubbleChart => "Bbl",
            PanelId::OrderBook => "OB",
            PanelId::Tape => "Tape",
            PanelId::BottomBar => "Bot",
        }
    }

    pub fn default_order() -> Vec<PanelId> {
        vec![
            PanelId::ClusterChart,
            PanelId::TickChart,
            PanelId::BubbleChart,
            PanelId::OrderBook,
            PanelId::Tape,
            PanelId::BottomBar,
        ]
    }

    pub fn is_main_panel(&self) -> bool {
        !matches!(self, PanelId::BottomBar)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Routes to a specific panel inside a dashboard
    ForPanel { dash: usize, panel: usize, msg: PanelMessage },

    /// Hotkey action routed to the currently active/focused panel
    ActivePanelAction(PanelMessage),

    // Dashboard / tab management
    AddDashboard,
    RemoveDashboard(usize),
    SwitchDashboard(usize),
    SetGridLayout { dash: usize, cols: usize, rows: usize },

    /// Set a symbol for a panel cell (creates/replaces TradingPanel)
    SetPanelSymbol { dash: usize, panel: usize, symbol: String },

    // Global mouse events (used for panel resize tracking)
    GlobalMouseMove(f32),
    GlobalMouseRelease,

    // Global UI
    ToggleHelp,
    FpsTick,
    NoOp,
}

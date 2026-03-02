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

/// Exchange selector in instrument picker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exchange {
    Bybit,
    Binance,
    OKX,
    Gate,
    BingX,
}

impl Exchange {
    pub fn as_str(self) -> &'static str {
        match self {
            Exchange::Bybit => "bybit",
            Exchange::Binance => "binance",
            Exchange::OKX => "okx",
            Exchange::Gate => "gate",
            Exchange::BingX => "bingx",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Exchange::Bybit => "Bybit",
            Exchange::Binance => "Binance",
            Exchange::OKX => "OKX",
            Exchange::Gate => "Gate",
            Exchange::BingX => "BingX",
        }
    }
}

/// Market type selector in instrument picker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketType {
    Linear,
    Inverse,
    Spot,
}

impl MarketType {
    pub fn as_str(self) -> &'static str {
        match self {
            MarketType::Linear => "linear",
            MarketType::Inverse => "inverse",
            MarketType::Spot => "spot",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MarketType::Linear => "Linear",
            MarketType::Inverse => "Inverse",
            MarketType::Spot => "Spot",
        }
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

    // Instrument picker (for empty grid cells)
    BeginEditCell { dash: usize, panel: usize },
    CancelEditCell,
    CellSymbolInput(String),

    // Instrument picker — exchange / market type selectors
    SetPickerExchange(Exchange),
    SetPickerMarket(MarketType),

    // Symbol list fetch results
    SymbolsFetched(Vec<String>),
    SymbolsFetchError(String),

    // Global mouse events (used for panel resize tracking)
    GlobalMouseMove(f32),
    GlobalMouseRelease,

    // Global UI
    ToggleHelp,
    FpsTick,
    NoOp,
}

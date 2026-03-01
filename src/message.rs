use crate::ws::WsEvent;

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

    /// Whether this panel goes in the main (center) row vs bottom bar
    pub fn is_main_panel(&self) -> bool {
        !matches!(self, PanelId::BottomBar)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // WebSocket
    WsEvent(WsEvent),

    // OrderBook interaction
    OrderBookClicked { price: f64, side: Side },
    Scroll(f32),
    Zoom(f32),
    SnapToPrice,

    // Trading (stubbed)
    BuyMarket,
    SellMarket,
    ClosePosition,
    CancelAllOrders,
    EmergencyCloseAll,

    // UI
    ToggleFollowMode,
    VolumeFilterChanged(f64),
    TogglePanel(PanelId),

    // Drag/drop panel reordering
    DragStart(PanelId),
    DragOver(PanelId),
    DragEnd,

    // Panel resize (draggable dividers)
    ResizeStart { divider_index: usize, x: f32 },
    ResizeMove(f32),
    ResizeEnd,

    // Order panel
    QuantityChanged(String),

    // Help overlay
    ToggleHelp,

    // FPS counter
    FpsTick,

    // No-op (for unhandled hotkeys)
    NoOp,
}

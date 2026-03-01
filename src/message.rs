use crate::ws::WsEvent;

#[derive(Debug, Clone)]
pub enum Side {
    Buy,
    Sell,
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

    // Order panel
    QuantityChanged(String),

    // No-op (for unhandled hotkeys)
    NoOp,
}

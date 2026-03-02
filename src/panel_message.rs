use crate::message::{PanelId, Side};
use crate::ws::WsEvent;

#[derive(Debug, Clone)]
pub enum PanelMessage {
    // WebSocket
    WsEvent(WsEvent),

    // OrderBook interaction
    OrderBookClicked { price: f64, side: Side },
    Scroll(f32),
    Zoom(f32),
    ChangePriceStep(f32),
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
}

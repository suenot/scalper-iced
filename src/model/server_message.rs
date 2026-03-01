use serde::Deserialize;
use super::{OrderBookSnapshot, ClusterCandle, TickCandle};

#[derive(Deserialize, Debug, Clone)]
pub struct ServerMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub timestamp: i64,
    pub orderbook: Option<OrderBookSnapshot>,
    pub clusters: Option<Vec<ClusterCandle>>,
    pub ticks: Option<Vec<TickCandle>>,
}

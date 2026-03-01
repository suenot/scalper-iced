use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct TickCandle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub count: i32,
    #[serde(rename = "buyVolume")]
    pub buy_volume: f64,
    #[serde(rename = "sellVolume")]
    pub sell_volume: f64,
}

impl TickCandle {
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }

    pub fn delta(&self) -> f64 {
        self.buy_volume - self.sell_volume
    }
}

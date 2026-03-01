use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct PriceLevel {
    pub price: f64,
    pub amount: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OrderBookSnapshot {
    pub asks: Vec<PriceLevel>,
    pub bids: Vec<PriceLevel>,
    pub spread: f64,
    #[serde(rename = "spreadPercent")]
    pub spread_percent: f64,
    pub timestamp: i64,
}

impl OrderBookSnapshot {
    pub fn mid_price(&self) -> f64 {
        if let (Some(best_bid), Some(best_ask)) = (self.bids.first(), self.asks.first()) {
            (best_bid.price + best_ask.price) / 2.0
        } else {
            self.spread
        }
    }

    pub fn max_volume(&self) -> f64 {
        let max_bid = self.bids.iter().map(|l| l.amount).fold(0.0_f64, f64::max);
        let max_ask = self.asks.iter().map(|l| l.amount).fold(0.0_f64, f64::max);
        max_bid.max(max_ask)
    }

    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|l| l.price)
    }

    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|l| l.price)
    }
}

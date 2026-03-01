use std::collections::BTreeMap;

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

    /// Aggregate price levels into buckets of the given step size.
    /// Each bucket sums the volumes of all levels within it.
    /// bucket_price = floor(price / step) * step
    pub fn aggregated(&self, step: f64) -> OrderBookSnapshot {
        let agg_asks = aggregate_levels(&self.asks, step, true);
        let agg_bids = aggregate_levels(&self.bids, step, false);

        let mut snap = OrderBookSnapshot {
            asks: agg_asks,
            bids: agg_bids,
            spread: self.spread,
            spread_percent: self.spread_percent,
            timestamp: self.timestamp,
        };

        // Recalculate spread from aggregated best bid/ask
        if let (Some(bb), Some(ba)) = (snap.best_bid(), snap.best_ask()) {
            snap.spread = ba - bb;
            let mid = (ba + bb) / 2.0;
            snap.spread_percent = if mid > 0.0 { snap.spread / mid * 100.0 } else { 0.0 };
        }

        snap
    }
}

/// Aggregate price levels into buckets.
/// Uses integer keys to avoid floating-point comparison issues.
fn aggregate_levels(levels: &[PriceLevel], step: f64, is_ask: bool) -> Vec<PriceLevel> {
    let mut buckets: BTreeMap<i64, f64> = BTreeMap::new();

    for level in levels {
        let key = (level.price / step).floor() as i64;
        *buckets.entry(key).or_insert(0.0) += level.amount;
    }

    let mut result: Vec<PriceLevel> = buckets
        .into_iter()
        .map(|(key, amount)| PriceLevel {
            price: key as f64 * step,
            amount,
        })
        .collect();

    if is_ask {
        result.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
    } else {
        result.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
    }

    result
}

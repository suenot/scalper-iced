use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ClusterPoint {
    pub price: f64,
    pub volume: f64,
    pub percent: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClusterCandle {
    pub timestamp: i64,
    #[serde(rename = "clusterPoints")]
    pub cluster_points: Vec<ClusterPoint>,
}

impl ClusterCandle {
    pub fn total_volume(&self) -> f64 {
        self.cluster_points.iter().map(|p| p.volume).sum()
    }

    pub fn max_volume(&self) -> f64 {
        self.cluster_points.iter().map(|p| p.volume).fold(0.0_f64, f64::max)
    }

    /// Aggregate cluster points into buckets of the given step size.
    /// Volumes are summed, percent is volume-weighted average.
    pub fn aggregated(&self, step: f64) -> ClusterCandle {
        let mut buckets: BTreeMap<i64, (f64, f64)> = BTreeMap::new(); // key → (total_vol, weighted_pct)

        for point in &self.cluster_points {
            let key = (point.price / step).floor() as i64;
            let entry = buckets.entry(key).or_insert((0.0, 0.0));
            entry.0 += point.volume;
            entry.1 += point.volume * point.percent; // for weighted average
        }

        let cluster_points: Vec<ClusterPoint> = buckets
            .into_iter()
            .map(|(key, (vol, weighted_pct))| ClusterPoint {
                price: key as f64 * step,
                volume: vol,
                percent: if vol > 0.0 { weighted_pct / vol } else { 50.0 },
            })
            .collect();

        ClusterCandle {
            timestamp: self.timestamp,
            cluster_points,
        }
    }
}

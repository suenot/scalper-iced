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
}

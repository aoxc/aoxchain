use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearMiniModel {
    pub weight: f64,
    pub bias: f64,
}

impl Default for LinearMiniModel {
    fn default() -> Self {
        Self {
            weight: 0.0,
            bias: 0.0,
        }
    }
}

impl LinearMiniModel {
    pub fn predict(&self, x: f64) -> f64 {
        self.weight * x + self.bias
    }

    pub fn mean_squared_error(&self, batch: &[(f64, f64)]) -> f64 {
        let loss_sum = batch
            .iter()
            .map(|(x, target)| {
                let err = self.predict(*x) - target;
                err * err
            })
            .sum::<f64>();
        loss_sum / batch.len() as f64
    }
}

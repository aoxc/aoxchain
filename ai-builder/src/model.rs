use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Recommendation {
    Allow,
    Observe,
    Throttle,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AoxcanQt01Model {
    pub name: String,
    pub weights: [f64; 4],
    pub bias: f64,
    pub threshold: f64,
}

impl Default for AoxcanQt01Model {
    fn default() -> Self {
        Self {
            name: "aoxcan-QT01".to_string(),
            weights: [0.0; 4],
            bias: 0.0,
            threshold: 0.65,
        }
    }
}

impl AoxcanQt01Model {
    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
    }

    pub fn predict_probability(&self, features: [f64; 4]) -> f64 {
        let linear = self
            .weights
            .iter()
            .zip(features.iter())
            .fold(self.bias, |acc, (w, x)| acc + (w * x));

        1.0 / (1.0 + (-linear).exp())
    }

    pub fn classify(&self, features: [f64; 4]) -> Recommendation {
        let risk = self.predict_probability(features);

        if risk >= 0.9 {
            Recommendation::Reject
        } else if risk >= self.threshold {
            Recommendation::Throttle
        } else if risk >= 0.45 {
            Recommendation::Observe
        } else {
            Recommendation::Allow
        }
    }

    pub fn binary_cross_entropy(&self, batch: &[([f64; 4], f64)]) -> f64 {
        let epsilon = 1e-9;
        let total = batch
            .iter()
            .map(|(features, label)| {
                let p = self
                    .predict_probability(*features)
                    .clamp(epsilon, 1.0 - epsilon);
                -(label * p.ln() + (1.0 - label) * (1.0 - p).ln())
            })
            .sum::<f64>();

        total / batch.len() as f64
    }
}

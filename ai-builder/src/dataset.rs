use rand::{RngExt, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

use crate::AiBuilderError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSample {
    pub features: [f64; 4],
    pub label: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub samples: Vec<RiskSample>,
}

impl Dataset {
    pub fn synthetic_chain_risk(size: usize, seed: u64) -> Result<Self, AiBuilderError> {
        if size < 200 {
            return Err(AiBuilderError::Dataset(
                "synthetic_chain_risk size must be >= 200".to_string(),
            ));
        }

        let mut rng = StdRng::seed_from_u64(seed);
        let mut samples = Vec::with_capacity(size);

        for _ in 0..size {
            let tx_frequency = rng.random_range(0.0..1.0);
            let gas_spike = rng.random_range(0.0..1.0);
            let address_entropy = rng.random_range(0.0..1.0);
            let policy_mismatch = rng.random_range(0.0..1.0);

            let risk_signal = 1.6 * tx_frequency
                + 1.2 * gas_spike
                + 1.0 * address_entropy
                + 1.8 * policy_mismatch
                - 2.0;

            let label = if risk_signal > 0.0 { 1.0 } else { 0.0 };
            samples.push(RiskSample {
                features: [tx_frequency, gas_spike, address_entropy, policy_mismatch],
                label,
            });
        }

        Ok(Self { samples })
    }

    pub fn split(&self, train_split: f32) -> (Vec<RiskSample>, Vec<RiskSample>) {
        let train_size = ((self.samples.len() as f32) * train_split).floor() as usize;
        let train = self.samples[..train_size].to_vec();
        let eval = self.samples[train_size..].to_vec();
        (train, eval)
    }
}

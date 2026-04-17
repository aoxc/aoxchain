use serde::{Deserialize, Serialize};

use crate::AiBuilderError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub samples: Vec<Sample>,
}

impl Dataset {
    pub fn synthetic_linear(size: usize) -> Result<Self, AiBuilderError> {
        if size < 10 {
            return Err(AiBuilderError::Dataset(
                "synthetic dataset size must be >= 10".to_string(),
            ));
        }

        let samples = (0..size)
            .map(|i| {
                let x = i as f64 / size as f64;
                let y = 2.5 * x + 0.4;
                Sample { x, y }
            })
            .collect();

        Ok(Self { samples })
    }

    pub fn split(&self, train_split: f32) -> (Vec<Sample>, Vec<Sample>) {
        let train_size = ((self.samples.len() as f32) * train_split).floor() as usize;
        let train = self.samples[..train_size].to_vec();
        let eval = self.samples[train_size..].to_vec();
        (train, eval)
    }
}

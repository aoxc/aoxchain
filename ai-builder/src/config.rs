use serde::{Deserialize, Serialize};

use crate::AiBuilderError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub project_name: String,
    pub seed: u64,
    pub epochs: usize,
    pub learning_rate: f64,
    pub checkpoint_every: usize,
    pub train_split: f32,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            project_name: "aoxc-mini-ai".to_string(),
            seed: 42,
            epochs: 250,
            learning_rate: 0.01,
            checkpoint_every: 25,
            train_split: 0.8,
        }
    }
}

impl TrainingConfig {
    pub fn validate(&self) -> Result<(), AiBuilderError> {
        if self.epochs == 0 {
            return Err(AiBuilderError::Config(
                "epochs must be greater than zero".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.train_split) || self.train_split == 0.0 {
            return Err(AiBuilderError::Config(
                "train_split must be in (0.0, 1.0]".to_string(),
            ));
        }
        if self.learning_rate <= 0.0 {
            return Err(AiBuilderError::Config(
                "learning_rate must be positive".to_string(),
            ));
        }
        Ok(())
    }
}

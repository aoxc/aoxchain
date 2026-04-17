use serde::{Deserialize, Serialize};

use crate::AiBuilderError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub project_name: String,
    pub model_name: String,
    pub seed: u64,
    pub epochs: usize,
    pub learning_rate: f64,
    pub checkpoint_every: usize,
    pub train_split: f32,
    pub dataset_size: usize,
    pub class_threshold: f64,
    pub l2_regularization: f64,
    pub feature_count: usize,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            project_name: "aoxc-ai-lab".to_string(),
            model_name: "aoxcan-QT01".to_string(),
            seed: 42,
            epochs: 300,
            learning_rate: 0.08,
            checkpoint_every: 25,
            train_split: 0.8,
            dataset_size: 1200,
            class_threshold: 0.65,
            l2_regularization: 0.0005,
            feature_count: 4,
        }
    }
}

impl TrainingConfig {
    pub fn validate(&self) -> Result<(), AiBuilderError> {
        if self.model_name.trim().is_empty() {
            return Err(AiBuilderError::Config(
                "model_name cannot be empty".to_string(),
            ));
        }
        if self.epochs == 0 {
            return Err(AiBuilderError::Config(
                "epochs must be greater than zero".to_string(),
            ));
        }
        if self.dataset_size < 200 {
            return Err(AiBuilderError::Config(
                "dataset_size must be >= 200 for stable risk training".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.train_split) || self.train_split <= 0.1 {
            return Err(AiBuilderError::Config(
                "train_split must be in (0.1, 1.0]".to_string(),
            ));
        }
        if self.learning_rate <= 0.0 {
            return Err(AiBuilderError::Config(
                "learning_rate must be positive".to_string(),
            ));
        }
        if !(0.5..=0.99).contains(&self.class_threshold) {
            return Err(AiBuilderError::Config(
                "class_threshold must be between 0.50 and 0.99".to_string(),
            ));
        }
        if self.feature_count != 4 {
            return Err(AiBuilderError::Config(
                "feature_count must be 4 for aoxcan-QT01".to_string(),
            ));
        }
        Ok(())
    }
}

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{config::TrainingConfig, error::AiBuilderError, model::LinearMiniModel};

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    root: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckpointArtifact {
    epoch: usize,
    eval_loss: f64,
    config: TrainingConfig,
    model: LinearMiniModel,
}

impl ModelRegistry {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, AiBuilderError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn save_checkpoint(
        &self,
        epoch: usize,
        model: &LinearMiniModel,
        eval_loss: f64,
        cfg: &TrainingConfig,
    ) -> Result<(), AiBuilderError> {
        let artifact = CheckpointArtifact {
            epoch,
            eval_loss,
            config: cfg.clone(),
            model: model.clone(),
        };

        let path = self.root.join(format!("checkpoint-epoch-{epoch:04}.json"));
        let json = serde_json::to_string_pretty(&artifact)?;
        fs::write(path, json)?;
        Ok(())
    }
}

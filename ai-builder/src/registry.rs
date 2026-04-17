use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{config::TrainingConfig, error::AiBuilderError, model::AoxcanQt01Model};

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCheckpoint {
    pub epoch: usize,
    pub loss: f64,
    pub accuracy: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckpointArtifact {
    model_name: String,
    checkpoint: TrainingCheckpoint,
    config: TrainingConfig,
    model: AoxcanQt01Model,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportManifest {
    pub model_name: String,
    pub version: String,
    pub threshold: f64,
    pub chain_safe_note: String,
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
        model: &AoxcanQt01Model,
        cfg: &TrainingConfig,
        checkpoint: TrainingCheckpoint,
    ) -> Result<(), AiBuilderError> {
        let artifact = CheckpointArtifact {
            model_name: cfg.model_name.clone(),
            checkpoint,
            config: cfg.clone(),
            model: model.clone(),
        };

        let path = self.root.join(format!("checkpoint-epoch-{epoch:04}.json"));
        let json = serde_json::to_string_pretty(&artifact)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn export_model(
        &self,
        model: &AoxcanQt01Model,
        version: &str,
    ) -> Result<ExportManifest, AiBuilderError> {
        let manifest = ExportManifest {
            model_name: model.name.clone(),
            version: version.to_string(),
            threshold: model.threshold,
            chain_safe_note: "This model is advisory only and cannot mutate consensus state."
                .to_string(),
        };

        let model_path = self
            .root
            .join(format!("{}-{}.model.json", model.name, version));
        let manifest_path = self
            .root
            .join(format!("{}-{}.manifest.json", model.name, version));

        fs::write(model_path, serde_json::to_string_pretty(model)?)?;
        fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        Ok(manifest)
    }
}

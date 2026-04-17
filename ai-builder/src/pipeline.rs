use std::path::Path;

use crate::{
    config::TrainingConfig,
    dataset::Dataset,
    error::AiBuilderError,
    model::AoxcanQt01Model,
    registry::{ExportManifest, ModelRegistry},
    training::{EvalMetrics, Trainer},
};

#[derive(Debug)]
pub struct TrainingOutcome {
    pub model: AoxcanQt01Model,
    pub metrics: EvalMetrics,
    pub manifest: ExportManifest,
}

pub fn run_training_pipeline<P: AsRef<Path>>(
    cfg: TrainingConfig,
    registry_dir: P,
) -> Result<TrainingOutcome, AiBuilderError> {
    let dataset = Dataset::synthetic_chain_risk(cfg.dataset_size, cfg.seed)?;
    let (train, eval) = dataset.split(cfg.train_split);

    let mut model = AoxcanQt01Model {
        name: cfg.model_name.clone(),
        ..AoxcanQt01Model::default()
    };
    model.set_threshold(cfg.class_threshold);

    let registry = ModelRegistry::new(registry_dir)?;
    let trainer = Trainer::new(cfg, registry.clone())?;
    let metrics = trainer.fit(&mut model, &train, &eval)?;
    let manifest = registry.export_model(&model, "v1")?;

    Ok(TrainingOutcome {
        model,
        metrics,
        manifest,
    })
}

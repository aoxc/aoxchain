use std::path::Path;

use crate::{
    config::TrainingConfig, dataset::Dataset, error::AiBuilderError, model::LinearMiniModel,
    registry::ModelRegistry, training::Trainer,
};

pub fn run_training_pipeline<P: AsRef<Path>>(
    cfg: TrainingConfig,
    registry_dir: P,
) -> Result<LinearMiniModel, AiBuilderError> {
    let data = Dataset::synthetic_linear(500)?;
    let (train, eval) = data.split(cfg.train_split);

    let mut model = LinearMiniModel::default();
    let registry = ModelRegistry::new(registry_dir)?;
    let trainer = Trainer::new(cfg, registry)?;

    trainer.fit(&mut model, &train, &eval)?;
    Ok(model)
}

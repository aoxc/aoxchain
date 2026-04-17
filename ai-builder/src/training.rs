use crate::{
    config::TrainingConfig, dataset::Sample, error::AiBuilderError, model::LinearMiniModel,
    registry::ModelRegistry,
};

#[derive(Debug)]
pub struct Trainer {
    cfg: TrainingConfig,
    registry: ModelRegistry,
}

impl Trainer {
    pub fn new(cfg: TrainingConfig, registry: ModelRegistry) -> Result<Self, AiBuilderError> {
        cfg.validate()?;
        Ok(Self { cfg, registry })
    }

    pub fn fit(
        &self,
        model: &mut LinearMiniModel,
        train: &[Sample],
        eval: &[Sample],
    ) -> Result<(), AiBuilderError> {
        if train.is_empty() {
            return Err(AiBuilderError::Training(
                "training split is empty; increase dataset size or train_split".to_string(),
            ));
        }

        for epoch in 1..=self.cfg.epochs {
            let n = train.len() as f64;
            let mut grad_w = 0.0;
            let mut grad_b = 0.0;

            for sample in train {
                let pred = model.predict(sample.x);
                let err = pred - sample.y;
                grad_w += (2.0 / n) * err * sample.x;
                grad_b += (2.0 / n) * err;
            }

            model.weight -= self.cfg.learning_rate * grad_w;
            model.bias -= self.cfg.learning_rate * grad_b;

            if epoch % self.cfg.checkpoint_every == 0 || epoch == self.cfg.epochs {
                let eval_pairs = eval.iter().map(|s| (s.x, s.y)).collect::<Vec<(f64, f64)>>();
                let eval_loss = if eval_pairs.is_empty() {
                    0.0
                } else {
                    model.mean_squared_error(&eval_pairs)
                };

                self.registry
                    .save_checkpoint(epoch, model, eval_loss, &self.cfg)?;
            }
        }

        Ok(())
    }
}

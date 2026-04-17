use crate::{
    config::TrainingConfig,
    dataset::RiskSample,
    error::AiBuilderError,
    model::AoxcanQt01Model,
    registry::{ModelRegistry, TrainingCheckpoint},
};

#[derive(Debug, Clone, Copy)]
pub struct EvalMetrics {
    pub loss: f64,
    pub accuracy: f64,
}

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
        model: &mut AoxcanQt01Model,
        train: &[RiskSample],
        eval: &[RiskSample],
    ) -> Result<EvalMetrics, AiBuilderError> {
        if train.is_empty() {
            return Err(AiBuilderError::Training(
                "training split is empty; increase dataset size or train_split".to_string(),
            ));
        }

        let mut final_metrics = EvalMetrics {
            loss: 0.0,
            accuracy: 0.0,
        };

        for epoch in 1..=self.cfg.epochs {
            let n = train.len() as f64;
            let mut grad_w = [0.0; 4];
            let mut grad_b = 0.0;

            for sample in train {
                let p = model.predict_probability(sample.features);
                let error = p - sample.label;

                for (i, grad_item) in grad_w.iter_mut().enumerate() {
                    *grad_item += (error * sample.features[i]) / n;
                }
                grad_b += error / n;
            }

            for (i, weight) in model.weights.iter_mut().enumerate() {
                let reg_term = self.cfg.l2_regularization * *weight;
                *weight -= self.cfg.learning_rate * (grad_w[i] + reg_term);
            }
            model.bias -= self.cfg.learning_rate * grad_b;

            let metrics = self.evaluate(model, eval);
            final_metrics = metrics;

            if epoch % self.cfg.checkpoint_every == 0 || epoch == self.cfg.epochs {
                let artifact = TrainingCheckpoint {
                    epoch,
                    loss: metrics.loss,
                    accuracy: metrics.accuracy,
                };
                self.registry
                    .save_checkpoint(epoch, model, &self.cfg, artifact)?;
            }
        }

        Ok(final_metrics)
    }

    fn evaluate(&self, model: &AoxcanQt01Model, eval: &[RiskSample]) -> EvalMetrics {
        if eval.is_empty() {
            return EvalMetrics {
                loss: 0.0,
                accuracy: 0.0,
            };
        }

        let mut correct = 0usize;
        let pairs = eval
            .iter()
            .map(|sample| {
                let p = model.predict_probability(sample.features);
                let predicted = if p >= model.threshold { 1.0 } else { 0.0 };
                if (predicted - sample.label).abs() < f64::EPSILON {
                    correct += 1;
                }
                (sample.features, sample.label)
            })
            .collect::<Vec<([f64; 4], f64)>>();

        let loss = model.binary_cross_entropy(&pairs);
        let accuracy = correct as f64 / eval.len() as f64;

        EvalMetrics { loss, accuracy }
    }
}

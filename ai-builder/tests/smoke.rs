use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use aoxc_ai_builder::{Recommendation, TrainingConfig, run_training_pipeline};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn qt01_pipeline_trains_and_exports_manifest() {
    let cfg = TrainingConfig {
        epochs: 60,
        checkpoint_every: 20,
        dataset_size: 600,
        ..TrainingConfig::default()
    };

    let output = unique_temp_dir("qt01-train");
    let outcome = run_training_pipeline(cfg, &output).expect("training pipeline");

    assert!(outcome.metrics.accuracy >= 0.70);
    assert_eq!(outcome.manifest.model_name, outcome.model.name);

    fs::remove_dir_all(&output).ok();
}

#[test]
fn qt01_classification_produces_chain_safe_actions() {
    let cfg = TrainingConfig {
        epochs: 40,
        checkpoint_every: 20,
        dataset_size: 500,
        ..TrainingConfig::default()
    };

    let output = unique_temp_dir("qt01-action");
    let outcome = run_training_pipeline(cfg, &output).expect("training pipeline");

    let action = outcome.model.classify([0.95, 0.90, 0.88, 0.98]);
    assert!(matches!(
        action,
        Recommendation::Throttle | Recommendation::Reject
    ));

    fs::remove_dir_all(&output).ok();
}

use aoxc_ai_builder::{TrainingConfig, run_training_pipeline};

#[test]
fn pipeline_trains_and_updates_weights() {
    let cfg = TrainingConfig {
        epochs: 50,
        checkpoint_every: 25,
        ..TrainingConfig::default()
    };

    let output = tempfile::tempdir().expect("tempdir");
    let model = run_training_pipeline(cfg, output.path()).expect("training pipeline");

    assert!(model.weight > 0.0);
}

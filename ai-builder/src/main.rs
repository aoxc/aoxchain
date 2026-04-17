use std::path::PathBuf;

use aoxc_ai_builder::{TrainingConfig, run_training_pipeline};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "aoxc-ai-builder",
    about = "Chain-safe mini AI trainer for advisory risk scoring"
)]
struct Cli {
    #[arg(long, default_value = "aoxc-ai-lab")]
    project_name: String,
    #[arg(long, default_value = "aoxcan-QT01")]
    model_name: String,
    #[arg(long, default_value_t = 300)]
    epochs: usize,
    #[arg(long, default_value_t = 0.08)]
    learning_rate: f64,
    #[arg(long, default_value_t = 25)]
    checkpoint_every: usize,
    #[arg(long, default_value_t = 0.8)]
    train_split: f32,
    #[arg(long, default_value_t = 1200)]
    dataset_size: usize,
    #[arg(long, default_value_t = 0.65)]
    class_threshold: f64,
    #[arg(long, default_value = "models/checkpoints")]
    output_dir: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let cfg = TrainingConfig {
        project_name: cli.project_name,
        model_name: cli.model_name,
        seed: 42,
        epochs: cli.epochs,
        learning_rate: cli.learning_rate,
        checkpoint_every: cli.checkpoint_every,
        train_split: cli.train_split,
        dataset_size: cli.dataset_size,
        class_threshold: cli.class_threshold,
        l2_regularization: 0.0005,
        feature_count: 4,
    };

    match run_training_pipeline(cfg, cli.output_dir) {
        Ok(outcome) => {
            println!(
                "Training complete. model={} threshold={:.2} accuracy={:.4} loss={:.6}",
                outcome.model.name,
                outcome.model.threshold,
                outcome.metrics.accuracy,
                outcome.metrics.loss
            );
            println!(
                "Export manifest: {} {}",
                outcome.manifest.model_name, outcome.manifest.version
            );
            println!("Chain safety: {}", outcome.manifest.chain_safe_note);
        }
        Err(err) => {
            eprintln!("Training failed: {err}");
            std::process::exit(1);
        }
    }
}

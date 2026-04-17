use std::path::PathBuf;

use aoxc_ai_builder::{TrainingConfig, run_training_pipeline};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "aoxc-ai-builder",
    about = "Mini AI training scaffold for Rust-native experimentation"
)]
struct Cli {
    #[arg(long, default_value = "aoxc-mini-ai")]
    project_name: String,
    #[arg(long, default_value_t = 250)]
    epochs: usize,
    #[arg(long, default_value_t = 0.01)]
    learning_rate: f64,
    #[arg(long, default_value_t = 25)]
    checkpoint_every: usize,
    #[arg(long, default_value_t = 0.8)]
    train_split: f32,
    #[arg(long, default_value = "models/checkpoints")]
    checkpoints_dir: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let cfg = TrainingConfig {
        project_name: cli.project_name,
        seed: 42,
        epochs: cli.epochs,
        learning_rate: cli.learning_rate,
        checkpoint_every: cli.checkpoint_every,
        train_split: cli.train_split,
    };

    match run_training_pipeline(cfg, cli.checkpoints_dir) {
        Ok(model) => {
            println!(
                "Training complete. model={{weight: {:.6}, bias: {:.6}}}",
                model.weight, model.bias
            );
        }
        Err(err) => {
            eprintln!("Training failed: {err}");
            std::process::exit(1);
        }
    }
}

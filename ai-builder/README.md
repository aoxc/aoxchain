# AOXC AI Builder

`ai-builder/` is a professional, extensible mini-AI development surface designed for Rust-first experimentation.

## Goals

- Keep AI training and model artifacts in one predictable root layout.
- Provide a maintainable baseline for future model types beyond a linear starter model.
- Support repeatable training runs, checkpointing, and future export/serving workflows.

## Directory Layout

- `src/` — core Rust modules (`config`, `dataset`, `model`, `training`, `registry`, `pipeline`).
- `tests/` — integration tests for pipeline behavior.
- `configs/` — training profiles and future hyperparameter packs.
- `data/raw/` — external or generated source datasets.
- `data/processed/` — normalized datasets ready for training.
- `models/checkpoints/` — epoch-based checkpoint artifacts.
- `models/export/` — stable exported models for inference.
- `artifacts/logs/` — training run logs.
- `artifacts/metrics/` — evaluation metrics and comparison outputs.

## Quick Start

```bash
cargo run --manifest-path ai-builder/Cargo.toml -- \
  --project-name mini-ai-v1 \
  --epochs 200 \
  --learning-rate 0.02 \
  --checkpoint-every 20 \
  --checkpoints-dir ai-builder/models/checkpoints
```

## Next Expansion Points

- Add richer dataset connectors (CSV/Parquet/stream ingestion).
- Replace the starter linear model with tensor-backed model types.
- Add experiment tracking metadata and model versioning policies.
- Add deterministic seed control at the full pipeline level.
- Integrate with `crates/aoxcai` for unified runtime strategy.

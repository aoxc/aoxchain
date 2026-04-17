# AOXC AI Builder

`ai-builder/` is a Rust-native mini AI lab focused on **chain-safe advisory models**.

Current reference model: **`aoxcan-QT01`**

- Purpose: classify transaction-context risk and return advisory actions.
- Scope: training + checkpointing + export manifests.
- Safety: model is non-consensus and cannot mutate chain state.

## Why this model does not hurt the chain

`aoxcan-QT01` only outputs recommendations (`Allow`, `Observe`, `Throttle`, `Reject`).
It is designed as an off-chain decision-support component and does **not** change validator or kernel execution logic by itself.

## Directory Layout

- `src/` — model, training, registry, pipeline, CLI.
- `tests/` — integration tests for training and recommendation behavior.
- `configs/` — default training profile templates.
- `data/` — raw and processed dataset spaces.
- `models/` — checkpoints and exported model+manifest files.
- `artifacts/` — logs and metric reports.

## Quick Start

```bash
cargo run --manifest-path ai-builder/Cargo.toml -- \
  --project-name aoxc-risk-lab \
  --model-name aoxcan-QT01 \
  --epochs 300 \
  --learning-rate 0.08 \
  --dataset-size 1200 \
  --class-threshold 0.65 \
  --output-dir ai-builder/models/checkpoints
```

## Model Contract (QT01)

Inputs (4 normalized features):
1. transaction frequency
2. gas spike level
3. address entropy
4. policy mismatch signal

Outputs:
- risk probability (0..1)
- advisory class (`Allow`, `Observe`, `Throttle`, `Reject`)

## Extension Path

- Add real dataset adapters (CSV/Parquet/indexer streams).
- Add calibration and drift monitors.
- Add signed lineage manifest and model promotion gates.
- Add ensemble variants (`QT02`, `QT03`) with the same chain-safe contract.

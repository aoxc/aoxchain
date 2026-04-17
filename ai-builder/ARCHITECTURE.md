# AI Builder Architecture

## Objective

Provide a professional Rust training surface for `aoxcan-QT01`, a chain-safe advisory risk model.

## Component Flow

`config -> dataset -> trainer -> registry -> exported model + manifest`

### Module Roles

- `config`: run parameters and validation gates.
- `dataset`: deterministic synthetic risk data generator and train/eval splitting.
- `model`: logistic-risk model + advisory class mapping.
- `training`: gradient descent, regularization, evaluation metrics.
- `registry`: checkpoint and model export manifest persistence.
- `pipeline`: end-to-end orchestration.
- `main`: CLI entrypoint.

## Trust and Safety Boundaries

- All CLI and config values are untrusted until validated.
- Model outputs are advisory only.
- No consensus mutation path exists in this crate.
- Export manifest includes explicit chain-safe statement to avoid misuse.

## Operational Guarantees

- Invalid hyperparameters fail fast.
- Every checkpoint is versionable JSON.
- Exported model includes threshold and policy note.
- Training returns explicit accuracy and loss metrics.

## Roadmap

- Real historical dataset loader.
- Data quality checks and schema versioning.
- Model drift detection and automatic rollback recommendation.
- Signed export manifests for audit traceability.

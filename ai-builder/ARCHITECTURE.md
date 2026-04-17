# AI Builder Architecture

## Component Graph

`config -> dataset -> training -> registry -> artifacts`

Supporting modules:

- `model`: forward pass and loss implementation.
- `pipeline`: orchestration entrypoint for end-to-end training.
- `main`: CLI surface for local operators.

## Trust and Validation Boundaries

- CLI input is untrusted until `TrainingConfig::validate` succeeds.
- Dataset generation/loading must enforce minimum viability constraints.
- Checkpoint serialization is append-only per epoch artifact file.

## Operational Contract

- No training run starts with invalid hyperparameters.
- A failed training run exits non-zero and emits explicit error class.
- Checkpoints are versionable JSON artifacts to simplify audit and diff workflows.

## Future Hardening Roadmap

- Add schema version to checkpoint artifacts.
- Add signed artifact manifests for model lineage verification.
- Add deterministic run manifest (`seed`, config hash, dataset hash).
- Add benchmark gates for regression detection before export promotion.

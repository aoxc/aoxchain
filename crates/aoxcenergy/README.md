# aoxcenergy

`aoxcenergy` provides a deterministic economic floor engine for AOXC.

This crate does **not** attempt to predict speculative market price. Instead, it
computes a governance-auditable **full network cost floor** derived from:

- energy expenditure
- infrastructure and operations expenditure
- continuity and risk buffers
- tax burden
- treasury formation policy
- target sustainability margin

## Core outputs

- `minimum_survival_floor`
- `sustainability_floor`
- `treasury_build_floor`
- `governance approval status`
- `audit summary`

## Validation goals

```bash
cargo fmt --all
cargo check -p aoxcenergy
cargo test -p aoxcenergy -- --nocapture
cargo clippy -p aoxcenergy --all-targets --all-features -- -D warnings
```

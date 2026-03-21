# aoxcenergy

## Purpose

`aoxcenergy` provides the **deterministic economic floor engine** for the AOXChain workspace. 

This crate does **not** attempt to predict speculative market prices. Instead, it computes a deterministic, governance-auditable **full network cost floor** derived from physical and policy realities:
- Energy expenditure & cooling overhead
- Infrastructure and validator operations expenditure
- Continuity and security risk buffers
- Network tax burden
- Treasury formation policy
- Target sustainability margins

## Core Components

The engine strictly evaluates the operational realities of the network to produce a verifiable economic state:
- **`EnergyAnchorEngine`**: The core deterministic calculator for computing the economic floor.
- **`UnitAmount`**: A zero-risk, fixed-point monetary wrapper that entirely avoids floating-point arithmetic to prevent consensus drifts.
- **`EconomicFloorReport`**: The comprehensive output detailing the `sustainable_cost`, `pre_tax_full_cost`, and `per_unit_floor`.
- **`EconomicZone`**: Classifies realized network value into `LossZone`, `SurvivalZone`, or `TreasuryBuildZone`.
- **`GovernanceDecision`**: Enforces strict protocol limits on period-over-period floor jumps, emitting `Approved`, `RequiresReview`, or `Rejected` statuses.

## Code Scope

- `src/lib.rs` - Core economic engine, zero-risk arithmetic structures, zone classifiers, and governance evaluators.

## Security & Operational Notes

- **Zero-Risk Arithmetic**: All monetary calculations (BPS application, unit addition, tax division) **must** use `checked_*` operations. Floating-point math is strictly forbidden in this crate to maintain absolute determinism across all validator nodes.
- **Basis Points (BPS) Strictness**: All percentage-based policies (tax, margins, reserves) are represented in BPS where `10_000 bps = 100.00%`. The engine will aggressively reject any input exceeding this denominator.
- **Governance Overrides**: Large economic floor adjustments are automatically rejected by the engine unless an explicit `emergency_override` is authorized by the sovereign network policy.

## Local Validation

Before submitting changes to the economic engine, ensure all deterministic tests and static analysis checks pass flawlessly:

```bash
cargo fmt --all -- --check
cargo check -p aoxcenergy
cargo clippy -p aoxcenergy --all-targets --all-features -- -D warnings
cargo test -p aoxcenergy -- --nocapture
Related Components
Top-level architecture: ../../README.md

Sovereign Consensus: ../aoxcunity/README.md

Execution Orchestrator: ../aoxcexec/README.md

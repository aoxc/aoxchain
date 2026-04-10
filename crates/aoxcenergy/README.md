# README.md


> Scope: `crates/aoxcenergy`

## Purpose
Carries gas/fee/economic costing and resource-governance rules.

## Contents at a glance
- The code and files in this directory define the runtime behavior of this scope.
- The folder contains modules and supporting assets bounded by this responsibility domain.
- Any change should be evaluated together with its testing and compatibility impact.

## Advanced Post-Quantum Cost Controls
- `PolicyInputs` supports dedicated post-quantum cost reserves:
  - `quantum_transition_reserve_bps` for migration and staged rollout impact.
  - `quantum_assurance_bps` for validation, audit, and cryptographic assurance overhead.
- `GovernancePolicy` enforces `max_quantum_reserve_bps` so combined post-quantum reserves cannot exceed ratified limits.
- `EconomicFloorReport` exposes explicit post-quantum components for audit and deterministic cost-share attribution.

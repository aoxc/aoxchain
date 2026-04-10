# README.md

> Scope: `crates/aoxconfig`

## Purpose
Provides type-safe configuration models and validation layers.

## Contents at a glance
- The code and files in this directory define the runtime behavior of this scope.
- The folder contains modules and supporting assets bounded by this responsibility domain.
- Any change should be evaluated together with its testing and compatibility impact.

## Notable configuration surfaces
- `AoxConfig` now includes `quantum` policy controls for post-quantum signatures, key-exchange constraints, hybrid mode control, and audit cadence.
- Quantum policy validates minimum security targets (`128`, `192`, `256`) and ensures at least one allowed scheme can satisfy configured thresholds.
- Quantum policy now integrates directly with kernel enforcement through `QuantumSecurityConfig::to_kernel_profile()`, producing `aoxcore::protocol::quantum::QuantumKernelProfile` with strict validation before admission.

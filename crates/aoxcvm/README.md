# AOXCVM

AOXCVM is the AOXChain L1-native virtual machine redesign baseline.

## Design posture
- L1-first execution model.
- Crypto agility with post-quantum migration surfaces.
- Deterministic execution, explicit host boundaries, and governance-controlled evolution.

## Kernel specification surfaces
- `docs/PHASE1_KERNEL_SPEC.md`: normative AOXC-VMachine-QX1 Phase 1 kernel completion criteria.
- `docs/VM_OVERVIEW.md`: component scope and high-level execution model.
- `docs/INVARIANTS.md`: deterministic and safety invariants expected at runtime.

This crate remains mostly scaffolded, with initial transaction-envelope primitives implemented for deterministic hashing, stateless validation, replay tracking, and dry-run request modeling.

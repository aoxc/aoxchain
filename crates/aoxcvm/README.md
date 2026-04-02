# AOXCVM

AOXCVM is the AOXChain L1-native virtual machine redesign baseline.

## Status
- Architecture status: **Phase 1 kernel baseline implemented**
- Delivery model: **Three-phase kernel-to-quantum roadmap**
- Current priority: **Phase 2 — Runtime Expansion (capability hardening + canonical receipts)**

## Design posture
- L1-first execution model.
- Deterministic execution with explicit host boundaries.
- Versioned execution rules and migration-aware governance.
- Crypto agility with hybrid and post-quantum authorization evolution.
- Auditability through canonical receipts and replay discipline.

## Canonical specification surfaces
- `ROADMAP.md`: AOXC-VMachine-QX1 three-phase architecture and delivery specification.
- `docs/PHASE1_KERNEL_SPEC.md`: normative Phase 1 kernel completion criteria.
- `docs/PHASE1_COMPLETION_REPORT.md`: implemented Phase 1 coverage and validation evidence matrix.
- `docs/VM_OVERVIEW.md`: component scope and high-level execution model.
- `docs/INVARIANTS.md`: deterministic and safety invariants expected at runtime.
- `docs/PRODUCTION_CLOSURE_MASTER_PLAN.md`: production-grade closure pillars, evidence model, and exit criteria.
- `docs/QRKF_KERNEL_ARCHITECTURE.md`: kernel-mandatory vs adapter-extensible key-fabric architecture for quantum-resilient identity and authority.

## Phase sequencing
- **Phase 1 — Kernel Completion:** deterministic, rollback-safe, gas-safe, versioned, authorization-aware execution kernel.
- **Phase 2 — Runtime Expansion:** package lifecycle, richer syscall and runtime capabilities without weakening kernel guarantees.
- **Phase 3 — Quantum Hardening and Proof Ecosystem:** governed hybrid/PQ migration and deterministic witness/proof artifacts.

## Immediate execution focus (Phase 1)
The repository now carries a kernel-complete baseline aligned with the Phase 1 specification.
Ongoing work is focused on expanding runtime surfaces without weakening deterministic replay,
rollback safety, admission strictness, or canonical receipt guarantees.

# AOXCVM

AOXCVM is the AOXChain L1-native virtual machine redesign baseline.

## Status
- Architecture status: **Draft**
- Delivery model: **Three-phase kernel-to-quantum roadmap**
- Current priority: **Phase 1 — Kernel Completion**

## Design posture
- L1-first execution model.
- Deterministic execution with explicit host boundaries.
- Versioned execution rules and migration-aware governance.
- Crypto agility with hybrid and post-quantum authorization evolution.
- Auditability through canonical receipts and replay discipline.

## Canonical specification surfaces
- `ROADMAP.md`: AOXC-VMachine-QX1 three-phase architecture and delivery specification.
- `docs/PHASE1_KERNEL_SPEC.md`: normative Phase 1 kernel completion criteria.
- `docs/VM_OVERVIEW.md`: component scope and high-level execution model.
- `docs/INVARIANTS.md`: deterministic and safety invariants expected at runtime.

## Phase sequencing
- **Phase 1 — Kernel Completion:** deterministic, rollback-safe, gas-safe, versioned, authorization-aware execution kernel.
- **Phase 2 — Runtime Expansion:** package lifecycle, richer syscall and runtime capabilities without weakening kernel guarantees.
- **Phase 3 — Quantum Hardening and Proof Ecosystem:** governed hybrid/PQ migration and deterministic witness/proof artifacts.

## Immediate execution focus (Phase 1)
Phase 1 is complete only when AOXCVM can reliably produce canonical AOXC state transitions and receipts under explicit trust boundaries, with deterministic replay and adversarial test coverage.

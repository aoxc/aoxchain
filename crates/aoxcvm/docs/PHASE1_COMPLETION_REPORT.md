# AOXC-VMachine-QX1 Phase 1 Completion Report

## Status

Phase 1 kernel baseline is implemented for `crates/aoxcvm` with deterministic execution,
strict admission, rollback-safe transactional journaling, bounded resources, and canonical
receipt commitment surfaces.

This report maps the normative `PHASE1_KERNEL_SPEC.md` requirements to concrete repository
surfaces so reviewers can audit completion claims against code and tests.

## Requirement-to-Implementation Matrix

| Phase 1 requirement | Implementation surface |
|---|---|
| Deterministic immutable execution context | `src/context/execution.rs`, `src/context/deterministic.rs` |
| Admission gate binding context + envelope + limits | `src/vm/admission.rs`, `src/tx/validation.rs` |
| Kernel orchestration with admission before execution | `src/vm/kernel.rs` |
| Deterministic verifier and bounded execution checks | `src/verifier/determinism.rs`, `src/verifier/invariants.rs` |
| Bounded memory model | `src/memory/heap.rs`, `src/memory/limits.rs`, `src/memory/safety.rs` |
| Gas and resource accounting | `src/gas/meter.rs`, `src/gas/limits.rs`, `src/gas/refunds.rs` |
| Transactional state journal (checkpoint/rollback/merge/commit) | `src/host/journal.rs` |
| Explicit host boundary and dispatch surfaces | `src/host/mod.rs`, `src/host/syscall.rs`, `src/host/dispatcher.rs` |
| Canonical receipt/outcome commitment | `src/receipts/outcome.rs`, `src/receipts/commitment.rs`, `src/receipts/proof.rs` |
| Versioned governance and feature activation | `src/governance/protocol_versions.rs`, `src/governance/feature_gates.rs`, `src/policy/features.rs` |
| Error taxonomy and explicit failure classes | `src/errors.rs`, `src/vm/admission.rs`, `src/verifier/*`, `src/context/execution.rs` |

## Phase 1 Validation Surfaces

The following automated checks are implemented as part of the crate:

- kernel orchestration and context-bound checks in `src/vm/kernel.rs` tests,
- admission consistency checks in `src/vm/admission.rs` tests,
- journal rollback/merge/conflict/atomicity checks in `src/host/journal.rs` tests,
- deterministic verifier and execution path checks in `src/verifier/*` and `src/vm/*` tests,
- workspace-level crate test execution via `cargo test -p aoxcvm`.


## Phase-1 Lockpoints (Canonical Freeze)

The following Phase-1 lockpoints are now explicitly frozen:

1. **Canonical kernel entry**: `execute(tx, descriptor, host, spec) -> ExecutionOutcome` in `src/vm/phase1.rs` (single public path).
2. **Execution contracts**: `Phase1Tx`, `ExecutionOutcome`, `VmError`, and `Receipt` are defined as stable execution surfaces in `src/vm/phase1.rs`.
3. **Host + state journal boundary**: `Host` trait requires `checkpoint`, `rollback`, and `commit`; VM persistence flows only through host commit.
4. **Gas + memory semantics**: Out-of-gas, failure rollback, memory expansion, and deterministic bounds are enforced by `src/vm/machine.rs`, `src/gas/meter.rs`, and `src/memory/heap.rs`.
5. **Auth + object admission split**: admission ordering is fixed as input -> auth -> descriptor/object verification -> execution.
6. **Determinism gate tests**: replay, rollback, OOG, malformed input, invalid auth, and invalid object tests are mandatory in `src/vm/phase1.rs` test suite.
7. **Config–Descriptor–VM three-way fail-closed resolution**: spec is resolved via `VmSpec::from_config(config, descriptor)` and execution is rejected when configuration disables descriptor target.

## Phase 2 Entry Constraint

Phase 2 runtime expansion work MUST preserve all Phase 1 invariants:

- deterministic replay equality,
- explicit host boundary discipline,
- bounded resource accounting,
- rollback-clean failure paths,
- canonical receipt commitment stability.

Any runtime expansion that weakens these guarantees is out of policy.

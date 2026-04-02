# AOXC-VMachine-QX1 Phase 1 Quality Pass

## Purpose

This document defines the concrete quality gates required before AOXCVM Phase 1 is treated as operationally frozen and Phase 2 expansion work begins.

## 1. Public Contract Freeze (Required)

The following execution surfaces are treated as compatibility-sensitive and must remain stable within Phase 1:

- `ExecutionContext`,
- `ExecutionOutcome`,
- `Receipt`,
- `VmError`,
- `Host` trait contract,
- state journal contract boundaries,
- `resolve_runtime_binding(...)`,
- canonical `execute(...)` entrypoint.

Any change to these surfaces requires an explicit compatibility review and targeted regression updates.

## 2. Kernel Boundary Discipline

Phase 1 code must keep strict responsibility boundaries:

- auth components perform authorization only;
- verifier components perform deterministic verification only;
- engine components execute;
- state components journal and finalize state transitions;
- host components mediate external world access;
- receipt components produce canonical execution outcomes.

Cross-boundary leakage is treated as a regression.

## 3. Fail-Closed Posture

Consensus-visible defaults must reject by default. The following classes must fail closed:

- unknown VM target,
- disabled feature,
- unsupported auth scheme,
- malformed object,
- invalid entrypoint,
- version mismatch,
- disallowed syscall.

## 4. Determinism and Canonicalization

The following outputs must be deterministic for identical inputs:

- receipt commitment/hash,
- state-diff ordering,
- touched-set ordering,
- log ordering,
- object hash,
- auth envelope hash,
- trace root.

Determinism regressions are release-blocking.

## 5. Required Test Classes

Phase 1 quality pass requires all of the following:

1. invariant tests (rollback equivalence, no mutation on failed execution, fail-closed admission);
2. replay/differential determinism tests (same input -> same outcome/gas/hash/root);
3. adversarial matrix tests (malformed, oversized, unsupported version/scheme, syscall misuse attempts);
4. transition boundary tests (admission -> auth -> verifier -> execute ordering);
5. canonical snapshot/golden tests for receipt and report encodings;
6. parser/decoder fuzz smoke targets for auth/object/receipt/context surfaces.

## 6. CI Gate Expectations

The minimum Phase 1 CI gate should include:

- `cargo fmt --all --check`,
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
- `cargo test --workspace`,
- doctest execution for touched crates,
- fuzz smoke execution for registered Phase 1 targets.

## 7. Documentation Parity

`PHASE1_KERNEL_SPEC.md`, implementation, and tests must describe the same behavior. Any invariant enforced in code should appear in spec/test documentation, and unsupported behavior should be listed explicitly.

## 8. Immediate Implementation Queue (Top-10)

1. freeze `ExecutionOutcome`, `Receipt`, and `VmError` API surfaces with explicit compatibility notes;
2. finalize `Host` and state journal trait-level contracts with cross-crate usage examples;
3. expand receipt commitment canonicalization tests across reordered but semantically equivalent inputs;
4. maintain a dedicated replay determinism suite (same input -> same gas, status, roots, and commitment);
5. add property-based tests for gas accounting, journal rollback equivalence, and memory bounds;
6. expand fail-closed matrix tests for auth/object/config combinations and disabled targets;
7. enforce unsupported/invalid version rejection tests on all version selectors;
8. add fuzz smoke targets for auth envelope/object decoder/receipt proof parser;
9. maintain an explicit invariant registry mapped to test names;
10. keep CI gates strict (`fmt`, `clippy`, crate tests, doctests, fuzz smoke) with no silent bypass.

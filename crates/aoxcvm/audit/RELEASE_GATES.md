# RELEASE GATES

This document defines release gating criteria for declaring AOXCVM as **full / production-complete**.

The objective is to prevent “feature-complete but evidence-incomplete” releases. A release can only be labeled full when required implementation, verification, and artifact publication surfaces are all green.

## Gate Classification

- **P0 (blocking):** must pass before any `full` claim.
- **P1 (production-hardening):** required for production-grade full posture.
- **P2 (institutional assurance):** required for enterprise-level confidence and long-term governance maturity.

A release marked `full` must satisfy **all P0 + P1** gates, and should declare explicit status for any open P2 items.

---

## P0 — Blocking gates (mandatory)

### P0.1 CI and readiness gates
Required outcomes:
- workspace tests pass,
- `fmt` and `clippy` pass,
- readiness gate passes,
- testnet/devnet runtime gate passes,
- no required merge checks are red.

Evidence:
- CI run URLs,
- per-gate status snapshot in the release evidence bundle.

### P0.2 Cross-platform determinism artifact
Required matrix:
- Linux x86_64,
- Linux ARM64,
- macOS runner when available,
- debug vs release parity checks.

Required artifact:
- `determinism-report.json` (or versioned equivalent) containing execution fingerprints and equality assertions for program, receipt, and state-root tuples.

### P0.3 Continuous fuzzing with corpus retention
Required surfaces:
- verifier fuzz,
- bytecode parser/decoder fuzz,
- syscall input fuzz,
- receipt/proof fuzz,
- authorization envelope and package-manifest fuzz.

Required operation:
- fuzzing runs in PR and/or nightly CI,
- crash corpus is retained and regression-tested.

### P0.4 Gas calibration and DoS resistance
Required outcomes:
- benchmarked gas schedule rationale,
- DoS-focused benchmark set,
- p50/p95/p99 execution metrics,
- regression gate for gas schedule changes.

Required artifact:
- versioned gas benchmark report linked in release evidence.

### P0.5 Chain-level integration rehearsal
Required scenarios:
- multi-transaction block execution,
- transaction-order determinism,
- mempool admission → execution → receipt → commit flow,
- governance and application transaction mixed batches,
- upgrade rehearsal and rollback rehearsal.

Required evidence:
- validator-node style E2E harness outputs,
- chain-flow trace and receipt consistency report.

---

## P1 — Production-hardening gates (required for full posture)

### P1.1 Call model specification closure
Must include:
- internal vs external call model,
- call-depth limits,
- gas forwarding semantics,
- revert bubbling and return-data contract,
- storage visibility across nested frames,
- reentrancy and static/read-only call policy.

Evidence package:
- canonical call spec,
- golden fixtures,
- adversarial test matrix.

### P1.2 Verifier coverage matrix
Must include:
- invalid CFG cases,
- malformed/duplicate/oversized section cases,
- immediate encoding rejection vectors,
- unsupported version rejection,
- section ordering invariants,
- capability/profile mismatch rejection.

Evidence:
- maintained verifier coverage matrix with fixture references.

### P1.3 Syscall ABI/versioning closure
Per syscall requirements:
- ABI version,
- canonical encoding,
- payload bounds,
- deterministic errors,
- gas cost,
- profile-based availability gate.

Evidence:
- syscall compatibility table,
- reject fixtures for unsupported syscalls,
- syscall benchmark report.

### P1.4 State stress and semantics evidence
Required scenarios:
- repeated writes,
- delete-and-recreate in same transaction,
- large map mutations,
- conflict ordering invariants,
- transient vs persistent semantics,
- complex trace receipt/state-root consistency.

Evidence:
- stress suite outputs,
- state-proof fixtures,
- scaling benchmark artifact.

### P1.5 Standardized release evidence bundle
Each release must publish one versioned bundle containing:
- test summary,
- determinism report,
- fuzz summary,
- benchmark report,
- verifier coverage report,
- artifacts manifest,
- compatibility statement,
- residual risk statement.

Bundle generation must be reproducible via a single documented command.

---

## P2 — Institutional assurance gates

### P2.1 Independent external review
Minimum expectation:
- one independent review covering VM semantics, gas economics, profile governance, and invariants-vs-implementation reconciliation.

### P2.2 Published limitations and non-goals
Release documentation must declare:
- unsupported opcode families,
- known stress ceilings,
- current performance limits,
- compatibility restrictions,
- rollout caveats.

Reference location:
- `crates/aoxcvm/audit/KNOWN_LIMITATIONS.md`.

### P2.3 Operational resilience package
Expected operational artifacts:
- versioned compatibility and migration guarantees,
- benchmark regression policy,
- operational playbook,
- rollback playbook.

---

## Release Decision Rule

AOXCVM can be labeled **full** only when:
1. all P0 gates pass,
2. all P1 gates pass,
3. any open P2 items are explicitly listed as residual risk in the release evidence bundle.

If a gate lacks machine-verifiable evidence, the gate is considered **not passed**.

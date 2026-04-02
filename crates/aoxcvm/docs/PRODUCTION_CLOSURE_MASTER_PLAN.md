# AOXCVM Production Closure Master Plan

## Objective

This plan defines the closure criteria required to move AOXCVM from phase-complete engineering baseline to production-grade sovereign runtime readiness.

## Current Baseline

- Deterministic execution and bounded runtime controls are implemented.
- Canonical receipt, governance, and constitutional execution surfaces are present.
- Phase closure documents exist through Phase 3 release closure.
- Hash/fingerprint naming and canonical digest posture are now conservative and audit-friendly.

## Closure Pillars

### 1) Cryptography Posture Closure

Required:
- canonical dual-digest framing and naming,
- fingerprint encoding specification,
- explicit upgrade/versioning posture,
- golden vectors retained as release evidence.

Status:
- framing/spec/naming are documented,
- golden vectors should be packaged in release evidence bundles.

### 2) Verifier Excellence Closure

Required:
- verifier rule-to-fixture coverage matrix,
- malformed bytecode corpus,
- capability/profile mismatch fixtures,
- golden verifier vectors with CI enforcement.

Status:
- phase closure tests exist; matrix and corpus discipline should be formalized as named evidence artifacts.

### 3) Execution Semantics Closure

Required:
- canonical call model (nested return/revert),
- event ordering invariants,
- failure taxonomy bound to tests/spec,
- syscall frame-transition invariants.

Status:
- execution model documents exist; cross-linking to golden fixtures should be tightened.

### 4) Gas Economics Closure

Required:
- opcode/storage/syscall cost references,
- benchmark-backed calibration reports,
- regression gates for cost drift,
- DoS envelope evidence.

Status:
- gas and phase artifacts exist; calibration reports should be release-gated per version.

### 5) Determinism Evidence Closure

Required:
- architecture matrix evidence (`x86_64`, `arm64`),
- debug/release parity artifacts,
- same input -> same receipt/state-root/fingerprint reports.

Status:
- determinism artifacts exist; release cadence retention policy should remain enforced.

### 6) Fuzzing Permanence Closure

Required:
- persistent fuzz targets for verifier/receipt/syscall/auth/package parsing,
- crash corpus retention,
- nightly fuzz execution and triage policy.

Status:
- should be treated as continuous operations surface, not one-time milestone.

### 7) Operations and Release Closure

Required:
- release evidence bundle schema,
- known limitations and residual risk statements,
- rollback/migration playbooks,
- release sign-off checklist.

Status:
- release evidence scripts exist; production sign-off package should be generated for each release tag.

### 8) External Assurance Closure

Required:
- independent audit,
- cryptography review,
- VM semantics review,
- explicit residual-risk acceptance record.

Status:
- external assurance remains a mandatory final gate.

## Exit Criteria (Production-Grade AOXCVM)

AOXCVM is considered production-grade when:

1. all eight pillars produce versioned evidence artifacts,
2. closure artifacts are reproducible from repository automation,
3. residual risks are explicitly documented and accepted,
4. at least one independent external review cycle is closed.

## Governance Note

This document is operational guidance, not a claim that production closure has already been completed.

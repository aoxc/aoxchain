# AOXCVM Phase-3 Release Closure

This document records the release-grade closure package for AOXCVM against the five hard requirements requested for final readiness.

## 1) Adversarial / fuzz closure

The closure gate executes `phase3_release_closure` tests in both debug and release profiles.

Coverage provided by this suite:

- fuzz-style receipt/proof mutation loops,
- malformed/tampered receipt corpus checks,
- property-style nonce verifier monotonicity checks,
- syscall-boundary equivalent stress via deterministic limits and call-context checks,
- receipt/proof replay consistency checks.

Canonical command:

- `cargo test -p aoxcvm --test phase3_release_closure`
- `cargo test -p aoxcvm --test phase3_release_closure --release`

## 2) Cross-platform determinism evidence

The gate generates deterministic probe outputs in debug and release modes and fails closed if outputs diverge.

Evidence artifact:

- `artifacts/aoxcvm-phase3/determinism-matrix.json`
- `artifacts/aoxcvm-phase3/evidence-bundle/artifacts-manifest.json`

The artifact includes:

- host fixture (OS, CPU, rustc),
- debug/release parity hash,
- heterogeneous validator matrix requirements for Linux/macOS/Windows and x86_64/aarch64 classes.

## 3) Call model and broader execution semantics

The closure tests include nested-call behavior and storage-journal interaction checks:

- nested call context creation,
- child revert propagation through rollback checkpoints,
- parent commit persistence,
- call-depth edge case bounds against deterministic limits.

## 4) Gas economics and benchmark evidence

The release closure includes a canonical gas envelope artifact:

- `artifacts/aoxcvm-phase3/gas-benchmark-envelope.json`

The artifact records:

- opcode/gas ordering justification (`add < storage_write < pq_verify`),
- deterministic out-of-gas envelope for expensive verification path,
- release policy requiring benchmark-diff review.

## 5) Release-grade CI gates

`full-gate` workflow now includes AOXCVM phase-3 gate execution and artifact retention.

Release-grade properties:

- readiness gate must pass before full-gate completion,
- workspace-wide hardening remains enforced by `quality_gate.sh full`,
- retained evidence artifacts are uploaded (`aoxcvm-phase3-artifacts`),
- benchmark and determinism evidence remain traceable by commit SHA.

## Operator command

To run the AOXCVM closure package locally:

```bash
make aoxcvm-phase3-gate
```

Optional artifact root override:

```bash
AOXCVM_ARTIFACT_DIR=/path/to/artifacts make aoxcvm-phase3-gate
```

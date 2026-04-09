# AOXChain Technical Contract

This document defines AOXChain’s repository-wide engineering contract for deterministic behavior, trust boundaries, and readiness claims.

## 1) System Intent

AOXChain is a deterministic Layer-1 system with:

- kernel-owned consensus and settlement policy,
- deterministic execution with bounded metering,
- profile-driven cryptographic agility,
- evidence-governed operational promotion.

## 2) Non-Negotiable Invariants

1. **Determinism:** identical canonical inputs produce identical canonical outputs.
2. **Fail-closed validation:** malformed or unsupported critical inputs are rejected before state transition.
3. **Boundary integrity:** non-kernel surfaces cannot override consensus truth.
4. **Profile explicitness:** consensus-critical cryptography is versioned and policy-bound.
5. **Evidence traceability:** readiness claims require reproducible commands and retained artifacts.
6. **Identity tuple integrity:** `chain_id`, `network_id`, and `network_serial` must remain registry-derived and mutually consistent.
7. **Version-axis separation:** brand/ticker, release line, workspace version, and cryptographic profile version are distinct and must not be conflated.
8. **Environment topology minimums:** bootstrap inputs must satisfy environment minimum validator/bootnode thresholds; stricter thresholds apply to `testnet` and `mainnet`.

## 3) Layer Responsibilities

### 3.1 Kernel and Consensus (`aoxcore`, `aoxcunity`)
- owns consensus truth, finality interpretation, and settlement admission;
- enforces consensus-visible cryptographic policy;
- preserves replay-protection semantics.

### 3.2 Execution (`aoxcvm`, `aoxcexec`, `aoxcenergy`)
- executes deterministic state transitions under kernel policy;
- enforces bounded metering and deterministic syscall behavior;
- does not redefine consensus trust rules.

### 3.3 Services (`aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`)
- provides transport, RPC, storage, and config delivery;
- treats ingress as untrusted until validated;
- cannot bypass kernel admission rules.

### 3.4 Operations (`aoxcmd`, `aoxckit`, `aoxchub`, `scripts/`)
- provides lifecycle orchestration and diagnostics;
- runs readiness gates and evidence collection;
- cannot mutate protocol truth outside approved policy surfaces.

## 4) Layer Extension Rule

When introducing a new layer, role class, or operational plane:

1. update `ARCHITECTURE.md` with responsibility and dependency direction;
2. update `configs/topology/*` and environment overlays for activation policy;
3. define trust boundary and validation ownership explicitly;
4. add readiness evidence requirements in testing and operational runbooks.

No new layer should be introduced through implementation-only changes without synchronized governance and operational documentation.

## 5) Required Readiness Gates

Readiness status is valid only when required gates pass and evidence is retained.

```bash
make build
make test
make quality
make audit
make os-compat-gate
make testnet-gate
make testnet-readiness-gate
```

## 6) High-Sensitivity Change Classes

The following classes require synchronized implementation, tests, and documentation:

- consensus/finality behavior,
- execution semantics and metering rules,
- cryptographic profile and key lifecycle policy,
- serialization and storage compatibility,
- RPC/API/operator control surfaces,
- release and rollback workflows,
- network identity and version policy definitions.

## 7) Program Trajectory

Program trajectory is governed by `ROADMAP.md`: production-grade testnet operation first, then controlled activation of a PQ-resilient mainnet.

Direct "quantum-first" migration requests are valid only when implemented as governed kernel policy changes with deterministic migration, fail-closed negotiation, and evidence-backed cutover rehearsal.

## 8) License and Liability Context

AOXChain is distributed under the MIT License on an **"as is"** basis, without warranty or liability assumptions by maintainers or contributors except where restricted by applicable law.

## 9) Production-Readiness Interpretation Rule

A passing compile/test subset is necessary but insufficient for production declaration.

Production-grade claims require:

- full execution of mandatory readiness gates listed in `TESTING.md`,
- environment-track consistency checks for the target deployment channel,
- retained evidence (commands, artifacts, and identifiers) linked to the candidate commit.

Point-in-time closure gaps and pending actions are tracked in:

- `docs/REPOSITORY_PRODUCTION_GAP_REPORT.md`

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

## 4) Required Readiness Gates

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

## 5) High-Sensitivity Change Classes

The following classes require synchronized implementation, tests, and documentation:

- consensus/finality behavior,
- execution semantics and metering rules,
- cryptographic profile and key lifecycle policy,
- serialization and storage compatibility,
- RPC/API/operator control surfaces,
- release and rollback workflows.

## 6) Program Trajectory

Program trajectory is governed by `ROADMAP.md`: production-grade testnet operation first, then controlled activation of a PQ-resilient mainnet.

## 7) License and Liability Context

AOXChain is distributed under the MIT License on an **"as is"** basis, without warranty or liability assumptions by maintainers or contributors except where restricted by applicable law.

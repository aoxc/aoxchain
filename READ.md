# AOXChain Canonical Technical Reference

This document is the repository-level technical contract for AOXChain.  
It defines non-negotiable behavior, boundary ownership, and validation expectations used for engineering review and release decisions.

---

## 1) System Intent

AOXChain is engineered as a deterministic Layer-1 program with:

- kernel-owned consensus and settlement policy,
- protocol-governed deterministic VM execution,
- explicit cryptographic profile evolution (including post-quantum migration),
- evidence-driven operational readiness decisions.

The system favors explicit policy, reproducibility, and fail-closed behavior over convenience.

---

## 2) Canonical Layer Responsibilities

### 2.1 Kernel and Consensus (`aoxcore`, `aoxcunity`)
- owns consensus truth, finality interpretation, and settlement admission;
- enforces profile-bound cryptographic policy at protocol boundaries;
- preserves replay protection and deterministic validity semantics.

### 2.2 Execution (`aoxcvm`, `aoxcexec`, `aoxcenergy`)
- executes deterministic state transitions under kernel-defined policy;
- enforces bounded metering and deterministic cryptographic syscall behavior;
- must not redefine consensus-level trust or finality meaning.

### 2.3 Services (`aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`)
- provides transport, RPC, storage, and typed configuration delivery;
- treats all external ingress as untrusted until validated;
- cannot bypass kernel validation decisions.

### 2.4 Operations (`aoxcmd`, `aoxckit`, `aoxchub`, `scripts/`)
- provides lifecycle orchestration, diagnostics, and evidence generation;
- drives release gates and closure workflows;
- cannot mutate protocol truth outside defined policy surfaces.

---

## 3) Non-Negotiable Invariants

1. **Determinism:** identical canonical inputs must yield identical canonical outputs.
2. **Fail-closed validation:** unknown or malformed critical inputs are rejected before state transition.
3. **Boundary integrity:** non-kernel surfaces cannot override consensus truth.
4. **Profile explicitness:** consensus-critical cryptographic behavior is versioned and policy-bound.
5. **Evidence traceability:** readiness claims require reproducible commands and retained artifacts.

---

## 4) Validation and Readiness Contract

Readiness claims are valid only when required validation gates pass with reviewable evidence.

Canonical quality surfaces:

```bash
make build
make test
make quality
make audit
make os-compat-gate
make testnet-gate
make testnet-readiness-gate
make quantum-readiness-gate
```

Runtime lifecycle surface:

```bash
make runtime-source-check AOXC_NETWORK_KIND=<env>
make runtime-install AOXC_NETWORK_KIND=<env>
make runtime-verify AOXC_NETWORK_KIND=<env>
make runtime-activate AOXC_NETWORK_KIND=<env>
make runtime-status AOXC_NETWORK_KIND=<env>
```

---

## 5) Change Impact Rules

A change is high-sensitivity and requires synchronized documentation + validation updates when it affects:

- consensus/finality behavior,
- VM semantics, metering, or syscall policy,
- cryptographic profiles or key workflows,
- storage or serialization contracts,
- release controls or operator procedures.

Non-trivial behavior changes must be explicitly described in review context.

---

## 6) Cross-Reference Map

- `README.md` — project purpose, repository surfaces, command entry points.
- `SCOPE.md` — scope boundaries and compatibility posture.
- `ARCHITECTURE.md` — component boundaries and dependency direction.
- `SECURITY.md` — private reporting and disclosure model.
- `TESTING.md` — mandatory validation policy and testnet go/no-go criteria.
- `WHITEPAPER.md` — end-to-end protocol design narrative and implementation blueprint.
- `docs/PRODUCTION_IMPLEMENTATION_BLUEPRINT.md` — production closure matrix.
- `QUANTUM_ROADMAP.md` / `QUANTUM_CHECKLIST.md` — profile migration execution and gate checklist.

---

## 7) License and Liability Context

AOXChain is distributed under the MIT License and provided **"as is"**, without warranties or liability assumptions by maintainers or contributors, except where restricted by applicable law.

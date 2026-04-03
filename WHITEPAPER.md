# AOXChain Whitepaper (Engineering Edition)

## Abstract

AOXChain is a deterministic Layer-1 protocol program designed around kernel-first trust interpretation, execution determinism, and cryptographic agility.  
This whitepaper defines the target architecture, threat-oriented boundary model, protocol governance direction, and production delivery controls used to move from active development to durable network operation.

---

## 1. Design Objectives

AOXChain prioritizes:

1. deterministic consensus and state transition behavior;
2. explicit trust boundaries and fail-closed ingress policy;
3. protocol-governed VM execution with bounded metering;
4. staged cryptographic profile migration, including post-quantum readiness;
5. evidence-based release and operational governance.

These objectives are enforced through repository-level controls (`TESTING.md`, `SECURITY.md`, `SCOPE.md`) and implementation-layer separation.

---

## 2. Architectural Model

### 2.1 Layering

- **Kernel layer:** `aoxcore`, `aoxcunity`
  - consensus truth, finality classification, settlement policy.
- **Execution layer:** `aoxcvm`, `aoxcexec`, `aoxcenergy`
  - deterministic execution, bounded metering, policy-constrained cryptographic syscalls.
- **Service layer:** `aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`
  - networking, API transport, storage, and typed configuration materialization.
- **Operations layer:** `aoxcmd`, `aoxckit`, `aoxchub`, `scripts/`
  - environment lifecycle, diagnostics, release gates, and evidence generation.

### 2.2 Boundary Doctrine

- kernel policy cannot be overridden by service or tooling convenience layers;
- externally sourced inputs are untrusted until validated;
- deterministic execution is a protocol property, not an implementation detail;
- release claims are accepted only with reproducible evidence.

---

## 3. Consensus and Finality Posture

AOXChain treats consensus safety and liveness as first-class invariants.  
Finality interpretation, replay protection, and settlement admission remain kernel responsibilities.

Core principles:

- explicit quorum/finality policies,
- deterministic tie-break and fork-choice behavior,
- fail-closed handling for malformed/unknown protocol envelopes,
- auditability of finality-critical decisions.

---

## 4. Deterministic Execution and VM Policy

The AOX VM is protocol-owned and policy-governed.

Execution requirements:

- deterministic opcode behavior across supported environments,
- bounded and explicit metering,
- deterministic cryptographic syscall behavior under active profile policy,
- stable admission rules for bytecode and syscall surfaces,
- no consensus-critical dependency on nondeterministic external IO.

---

## 5. Cryptographic Agility and Post-Quantum Migration

AOXChain follows versioned cryptographic profile governance:

- profile identifiers are consensus-visible and explicit;
- hybrid migration windows are supported for controlled transition;
- downgrade and profile-policy bypass paths are prohibited;
- rollback behavior must remain deterministic and operator-auditable.

Implementation and rollout controls are tracked by:

- `QUANTUM_ROADMAP.md`
- `QUANTUM_CHECKLIST.md`

---

## 6. Networking and API Trust Surfaces

Network and RPC layers provide transport and observability, not protocol truth.

Required controls:

- strict session/peer validation and admission policy,
- anti-downgrade profile negotiation controls,
- topology and endpoint hardening for testnet/mainnet profiles,
- no bypass of kernel validation via RPC convenience paths.

---

## 7. Governance, Testing, and Evidence

AOXChain enforces readiness through command-driven, reviewable gates.

Baseline command surfaces:

```bash
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

A network readiness claim is valid only when required gates pass and evidence is retained with commit linkage.

---

## 8. Production Closure Criteria

A branch is production-closable only when:

- required quality and readiness gates pass without unreviewed exceptions;
- architectural boundaries remain intact;
- security posture is documented and actively triaged;
- compatibility and migration impacts are explicitly declared;
- operational runbooks are aligned with current runtime behavior.

Recommended closure references:

- `docs/PRODUCTION_IMPLEMENTATION_BLUEPRINT.md`
- `docs/OS_COMPATIBILITY.md`
- `TESTING.md`

---

## 9. Scope and Liability Context

AOXChain is distributed under the MIT License on an **"as is"** basis.  
No implicit warranty of uninterrupted operation, production fitness, or jurisdiction-specific compliance is provided by repository state alone.

Governance decisions should therefore rely on retained evidence, explicit review outcomes, and reproducible validation.

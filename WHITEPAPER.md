# AOXChain Whitepaper (Engineering Edition)

## Abstract

AOXChain is a deterministic Layer-1 protocol program centered on kernel-first trust interpretation, deterministic execution, and profile-governed cryptographic agility. This whitepaper defines architectural intent, threat boundaries, governance posture, and evidence-based production controls.

## 1. Design Objectives

AOXChain prioritizes:

1. deterministic consensus and state transition semantics,
2. explicit trust boundaries and fail-closed ingress control,
3. policy-governed VM execution with bounded metering,
4. staged cryptographic migration including post-quantum readiness,
5. evidence-based release and operational governance.

## 2. Architectural Model

### 2.1 Layering

- **Kernel layer** (`aoxcore`, `aoxcunity`): consensus truth and finality policy.
- **Execution layer** (`aoxcvm`, `aoxcexec`, `aoxcenergy`): deterministic execution and metering.
- **Service layer** (`aoxcnet`, `aoxcrpc`, `aoxcdata`, `aoxconfig`): networking, API, storage, configuration.
- **Operations layer** (`aoxcmd`, `aoxckit`, `aoxchub`, `scripts/`): lifecycle control, diagnostics, and evidence.

### 2.2 Boundary Doctrine

- kernel policy cannot be overridden by service convenience layers,
- external input remains untrusted until validated,
- deterministic execution is a protocol property,
- release claims require reproducible evidence.

## 3. Consensus and Finality Posture

Consensus safety and liveness are first-class invariants.
Finality interpretation, replay protection, and settlement admission remain kernel responsibilities.

## 4. Deterministic VM Policy

Execution requirements include:

- deterministic opcode behavior,
- explicit bounded metering,
- deterministic cryptographic syscall behavior under active profile,
- stable bytecode and syscall admission policy,
- no consensus-critical dependency on nondeterministic external I/O.

## 5. Cryptographic Agility

AOXChain uses versioned profile governance:

- profile identifiers are consensus-visible,
- hybrid windows are explicit and bounded,
- downgrade and profile-bypass paths are prohibited,
- rollback behavior is deterministic and auditable.

## 6. Networking and API Surfaces

Network and RPC layers provide transport and observability, not protocol truth.
They must enforce admission policy, anti-downgrade controls, and strict separation from consensus-critical internals.

## 7. Governance, Testing, and Evidence

Readiness is command-driven and evidence-gated. Typical baseline commands:

```bash
make test
make quality
make audit
make testnet-gate
make testnet-readiness-gate
```

A readiness claim is valid only with retained artifacts linked to commit identity.

## 8. Production Closure Criteria

A branch is production-closable only when:

- required gates pass without unreviewed exceptions,
- architectural boundaries remain intact,
- security posture is documented and triaged,
- compatibility and migration impacts are explicit,
- operator runbooks are synchronized with runtime behavior.

## 9. Scope and Liability Context

AOXChain is distributed under MIT on an **"AS IS"** basis. Repository state alone provides no implicit warranty of production fitness or uninterrupted operation.

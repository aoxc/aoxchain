# AOXC-VMachine-QX1 Phase 1 Kernel Specification

## 1. Purpose and Scope

This document defines the **Phase 1 (Kernel Complete)** baseline for AOXC-VMachine-QX1.
Phase 1 is the minimum production-grade kernel surface that consensus can trust for deterministic,
rollback-safe execution under versioned policy control.

Phase 1 is **post-quantum migration-ready**, not a final post-quantum endpoint. The required posture is:

- PQ-aware,
- crypto-agile,
- hybrid-auth ready,
- migration-safe,
- deterministic at kernel level.

## 2. Phase 1 Mission Statement

AOXC-VMachine-QX1 Phase 1 SHALL provide one canonical execution kernel that:

1. accepts transaction envelopes,
2. verifies authorization,
3. admits verified objects/bytecode,
4. executes deterministically,
5. meters gas and resources,
6. journals state transition intent,
7. resolves commit/rollback,
8. emits canonical receipt and outcome hashes.

## 3. Non-Goals for Phase 1

The following are explicitly out of scope for Phase 1 completion:

- full compiler ecosystem maturity,
- package marketplace/runtime economy features,
- multi-engine plugin runtime,
- advanced zk witness/proof backend integration.

## 4. Normative Kernel Layers (Mandatory)

Phase 1 completeness requires all layers below.

### 4.1 Kernel Identity Layer

The kernel identity MUST include:

- `kernel_name = "AOXC-VMachine-QX1"`,
- `kernel_semver`,
- `kernel_api_version`,
- `execution_spec_id`,
- `state_transition_version`,
- `auth_policy_version`,
- `object_format_version`.

The layer MUST also produce:

- canonical kernel fingerprint,
- build reproducibility metadata,
- protocol compatibility matrix.

### 4.2 Execution Context Layer

Execution context MUST include deterministic, immutable fields:

- chain/network identifiers,
- epoch and block identifiers,
- transaction identifiers and index,
- caller/callee/origin/value,
- gas limits and pricing model reference,
- execution depth and readonly flag,
- active spec version and feature bitmap.

Context MUST be host-constructed and immutable throughout execution.

### 4.3 Input Validation Layer (Admission Gate)

Before execution begins, admission MUST validate:

- envelope structure,
- auth field shape,
- nonce and replay domain requirements,
- size/limit constraints,
- object/package integrity references,
- profile and feature compatibility,
- chain policy compatibility,
- version compatibility.

Invalid admission inputs MUST NOT enter the execution loop.

### 4.4 Authorization Kernel

Phase 1 authorization MUST support scheme abstraction at minimum:

- `Secp256k1`,
- `Ed25519`,
- `Hybrid`,
- `ReservedPQ`,
- `Multisig`,
- `PolicyBound`,
- `SessionDelegated`.

`ReservedPQ` means format and verifier interfaces are present even when specific PQ suites are not yet activated.

Authorization MUST provide:

- account auth descriptor versioning,
- scheme registry,
- threshold policy checks,
- replay protection,
- domain separation,
- intent binding,
- chain-bound authorization.

### 4.5 Object/Code Admission Layer

The kernel MUST execute only admitted canonical artifacts. Admission MUST validate:

- object header and section integrity,
- metadata and entrypoint declaration,
- import/capability declarations,
- section hash correctness,
- version/profile compatibility,
- deterministic entrypoint mapping.

### 4.6 Bytecode/Instruction Kernel

Execution loop MUST define deterministic semantics for:

- decode and dispatch,
- invalid opcode handling,
- trap/halt/return/revert classes,
- bounded control flow.

Phase 1 reference model SHOULD remain bounded frame-based (register-frame preferred for auditability).

### 4.7 Memory Kernel

Memory model MUST be explicit and bounded across regions:

- code memory,
- readonly data,
- call data,
- frame locals,
- heap,
- return buffer,
- scratch region.

Requirements:

- checked bounds on all accesses,
- explicit uninitialized-read policy,
- gas-charged expansion,
- no undefined memory behavior,
- no cross-frame unsafe aliasing,
- immutable readonly segments.

### 4.8 Gas and Resource Kernel

Gas accounting MUST be kernel-native and complete:

- base and dynamic instruction costs,
- memory expansion costs,
- syscall and verifier costs,
- auth/object verification costs,
- refund model with explicit ceiling,
- out-of-gas fail-closed behavior.

Gas underflow/overflow MUST be impossible by construction.

### 4.9 State Journal Kernel

Execution MUST write intent into journal, not persistent state directly.

Journal must support:

- nested checkpoints,
- rollback,
- merge,
- commit,
- deterministic diff ordering,
- canonical diff serialization.

### 4.10 Host Boundary Kernel

All external interactions MUST pass through explicit host interfaces.

Allowed host classes include:

- account and storage queries,
- code/object lookup,
- block/config queries,
- event/log sink,
- precompile dispatch,
- policy and feature checks.

Forbidden directly in kernel:

- filesystem access,
- direct network I/O,
- nondeterministic randomness,
- wall-clock time reads,
- direct database handle mutation,
- platform-dependent side effects.

### 4.11 Receipt/Outcome Kernel

Canonical receipt MUST include at minimum:

- status and status class,
- gas consumed/refunded,
- return or revert payload,
- logs/events,
- created/deleted/touched summaries,
- verifier/auth metadata,
- kernel/spec version,
- trace/journal roots,
- final receipt hash.

### 4.12 Versioned Rule Kernel

Execution MUST be profile-versioned rather than hardcoded to a single present-tense rule set.

Versioned controls include:

- instruction availability,
- gas table revision,
- auth scheme activation,
- PQ migration mode,
- object format evolution,
- syscall permissions,
- receipt schema evolution.

## 5. Required Core Trait Contracts

Phase 1 MUST define and stabilize trait-level contracts for:

- **Host**: state/query/event/precompile/policy interfaces,
- **StateJournal**: checkpoint/write/create/delete/rollback/merge/finalize,
- **AuthVerifier**: envelope/domain/replay/scheme/threshold verification,
- **ObjectVerifier**: structure/section/entrypoint/capability/version checks,
- **GasMeter**: charge/dynamic/refund/remaining/consumed/checkpoint/rollback,
- **KernelMachine**: prepare/verify/execute/finalize/receipt production.

## 6. Execution Lifecycle (Canonical)

1. Admission,
2. Validation,
3. Auth verification,
4. Object verification,
5. Context construction,
6. Gas initialization,
7. Journal checkpoint,
8. Execute,
9. Syscall/host mediation,
10. Outcome classification,
11. Commit/rollback resolution,
12. Receipt build,
13. Deterministic commitment hashing.

## 7. Failure Taxonomy (Required Separation)

Phase 1 MUST model distinct failure classes, including:

- `AdmissionError`,
- `AuthError`,
- `ObjectError`,
- `DecodeError`,
- `InstructionError`,
- `MemoryError`,
- `GasError`,
- `SyscallError`,
- `PolicyError`,
- `StateError`,
- `FatalKernelError`.

Consensus behavior and RPC semantics MUST be driven by this explicit classification, not by implicit mapping.

## 8. Quantum-Ready Requirements for Phase 1

Phase 1 quantum readiness means migration-safe surfaces, specifically:

1. versioned auth descriptors,
2. hybrid auth envelope support,
3. reserved PQ verifier slots,
4. crypto-agility registry (algorithm selection not hardcoded),
5. epoch-based policy activation modes (`hybrid_required`, `pq_preferred`, `pq_mandatory`).

## 9. Exit Criteria: “Phase 1 Complete”

Phase 1 is complete only when all below are satisfied:

### 9.1 Technical completeness

- deterministic execution spec finalized,
- receipt schema stabilized,
- journal rollback semantics validated,
- gas invariants enforced and tested,
- auth abstraction complete,
- object admission/verifier complete,
- versioned rules active.

### 9.2 Testing completeness

- determinism replay,
- malformed object adversarial testing,
- gas exhaustion paths,
- nested rollback coverage,
- auth replay and threshold tests,
- version migration compatibility tests.

### 9.3 Security completeness

- no hidden I/O paths,
- no state writes outside journal,
- no execution path without object verification,
- no auth bypass surface,
- no unchecked memory growth,
- no silent policy downgrade behavior.

## 10. Phase Separation After Phase 1

- **Phase 1**: Kernel complete (determinism, gas, rollback, auth abstraction, object admission, versioning).
- **Phase 2**: Runtime expansion (tooling, package ecosystem, richer syscall/user surfaces).
- **Phase 3**: Quantum hardening + proof ecosystem (activated PQ suites and advanced witness/proof paths).

## 11. Engineering Principle

> No feature before kernel invariants.

Phase 1 prioritizes invariant closure over feature breadth.

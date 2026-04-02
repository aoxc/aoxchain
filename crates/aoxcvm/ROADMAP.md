# AOXC-VMachine-QX1

## Three-Phase Architecture and Delivery Specification

### Status: Draft
### Scope: Kernel-to-Quantum Execution Roadmap
### Target: AOXChain `crates/aoxcvm`

---

# 1. Document Purpose

This document defines the complete three-phase architecture, security model, and delivery path for **AOXC-VMachine-QX1**, the AOXChain execution machine intended to serve as the deterministic, crypto-agile, and quantum-migration-ready virtual machine for the AOXC Layer 1 protocol.

The document is intentionally written at a kernel and protocol level. It does not describe a temporary prototype. It defines the intended long-term machine contract for AOXC execution.

The scope of this document includes:

- the mission of AOXC-VMachine-QX1,
- the complete architectural boundaries of Phase 1,
- the expansion objectives of Phase 2,
- the quantum-hardening and proof-oriented objectives of Phase 3,
- execution invariants,
- trust boundaries,
- state transition responsibilities,
- authorization evolution,
- object and bytecode admission,
- versioning and governance integration.

This document should be read as a **protocol-facing VM roadmap**, not merely as a crate-level note.

---

# 2. Design Philosophy

AOXC-VMachine-QX1 must not be designed as a simple bytecode interpreter. It must be designed as the **canonical execution kernel** that the AOXC consensus layer can trust.

The machine must be:

- deterministic across all honest nodes,
- explicit in all state-transition side effects,
- bounded in all resource consumption,
- strict in all external interaction boundaries,
- upgrade-aware through explicit versioning,
- crypto-agile by construction,
- ready for hybrid and post-quantum authorization evolution,
- testable, auditable, and replayable.

The machine must never rely on implicit host behavior, hidden nondeterminism, direct operating-system state, or implementation-defined execution semantics.

The machine must be able to say, for any accepted input:

> this input was admitted under a known execution specification, executed under a known authorization policy, charged under a known resource schedule, and finalized into a canonical receipt under deterministic state-transition rules.

That is the core mission.

---

# 3. Global Architectural Principles

## 3.1 Determinism First

Every valid execution must produce the same observable result on every honest node.

This includes:

- status code,
- return data,
- revert data,
- gas used,
- gas refunded,
- state diffs,
- emitted logs,
- touched account summaries,
- receipt hash,
- trace root, if enabled by protocol rules.

No execution path may depend on:

- wall-clock time,
- local filesystem state,
- host-specific randomness,
- thread race outcomes,
- undefined memory behavior,
- floating-point ambiguity,
- unordered map iteration that affects consensus-visible outputs.

---

## 3.2 Explicit Trust Boundaries

All external interaction must pass through explicit host interfaces.

The VM must not directly access:

- the database,
- the network,
- the filesystem,
- host entropy sources,
- OS process state,
- mutable global singletons.

All interaction with chain state, block metadata, policy, cryptography providers, and precompiles must occur through clearly defined traits or protocol interfaces.

---

## 3.3 Versioned Evolution

Execution rules must be versioned.

No critical rule may be encoded as an implicit and timeless assumption.

The following must be version-aware:

- execution semantics,
- gas schedule,
- object format,
- authorization policy,
- syscall surface,
- receipt schema,
- feature activation flags,
- post-quantum migration state.

---

## 3.4 Security-Sensitive Minimalism

The kernel must remain small in responsibility but complete in authority.

This means the kernel must fully own:

- execution lifecycle,
- state journal discipline,
- gas accounting,
- authorization binding,
- object admission boundaries,
- host boundary enforcement,
- receipt canonicalization.

It must not become a general application framework in Phase 1.

---

## 3.5 Crypto Agility

Cryptographic choices must not be hardcoded into the irreversible structure of the machine.

The machine must support:

- multiple authorization schemes,
- future hybrid authorization models,
- future post-quantum signature adoption,
- scheme versioning,
- protocol-governed activation and deprecation.

Crypto agility must exist in the machine design even before all future schemes are activated.

---

# 4. AOXC-VMachine-QX1 Mission

AOXC-VMachine-QX1 is the AOXC execution machine responsible for transforming protocol-admitted transactions and object invocations into deterministic state transitions and canonical receipts.

Its responsibilities include:

- transaction execution,
- object and bytecode invocation,
- authorization enforcement,
- resource accounting,
- state transition journaling,
- controlled interaction with chain state,
- failure classification,
- canonical receipt generation.

Its responsibilities do **not** include:

- direct storage engine ownership,
- direct network interaction,
- direct consensus logic,
- direct wallet policy management,
- direct explorer/indexing responsibilities,
- arbitrary plugin execution without governance control.

---

# 5. Three-Phase Delivery Model

AOXC-VMachine-QX1 is delivered in three phases:

- **Phase 1 — Kernel Completion**
- **Phase 2 — Runtime Expansion**
- **Phase 3 — Quantum Hardening and Proof Ecosystem**

These phases are sequential in trust importance.

The machine must not skip Phase 1 discipline in order to reach Phase 2 convenience or Phase 3 marketing claims.

---

# 6. Phase 1 — Kernel Completion

## 6.1 Phase 1 Objective

Phase 1 establishes the complete deterministic execution kernel of AOXC-VMachine-QX1.

Phase 1 must produce a VM kernel that is:

- execution-complete at the kernel level,
- deterministic,
- rollback-safe,
- gas-safe,
- authorization-abstracted,
- object-admission aware,
- versioned,
- future-ready for hybrid and post-quantum authorization migration.

Phase 1 does **not** need to deliver the entire ecosystem around the VM. It must deliver the full execution core that the chain can trust.

---

## 6.2 Phase 1 Deliverables

Phase 1 must deliver the following capabilities:

1. canonical kernel identity,
2. immutable execution context,
3. transaction and invocation admission rules,
4. authorization abstraction layer,
5. object and bytecode admission layer,
6. deterministic instruction execution engine,
7. bounded memory model,
8. complete gas and resource metering,
9. state journal with checkpoints and rollback,
10. explicit host boundary,
11. syscall routing and permission control,
12. canonical outcome and receipt generation,
13. versioned execution rule selection,
14. complete error taxonomy,
15. deterministic replay and adversarial test baseline.

---

## 6.3 Phase 1 Core Components

### 6.3.1 Kernel Identity Layer

The machine must expose explicit identity and compatibility descriptors.

Minimum required fields:

- kernel name,
- kernel semantic version,
- execution specification identifier,
- state-transition version,
- authorization policy version,
- object format version,
- receipt schema version,
- optional feature set fingerprint.

This layer exists to ensure that the execution environment is never ambiguous.

---

### 6.3.2 Execution Context Layer

Every execution must operate under an immutable context.

The execution context must include, at minimum:

- chain identifier,
- network identifier,
- epoch,
- block height,
- block timestamp,
- block hash or block reference identifier,
- transaction hash,
- transaction index,
- caller,
- callee,
- origin,
- transferred value,
- gas limit,
- execution depth,
- readonly mode flag,
- active spec version,
- active feature bitmap.

This context must be canonical and serializable.

---

### 6.3.3 Input Admission Layer

No execution may begin before admission checks succeed.

Admission checks must validate:

- transaction envelope shape,
- field sizes,
- version compatibility,
- authorization envelope structure,
- payload size,
- object reference format,
- declared execution mode,
- chain policy compatibility,
- feature activation compatibility.

Malformed or unsupported input must fail before entering the instruction engine.

---

### 6.3.4 Authorization Kernel

Authorization must be abstracted from the instruction engine.

Phase 1 must support an extensible authorization model including:

- classical single-signature schemes,
- threshold or multisignature models,
- policy-bound authorization descriptors,
- delegated or session-style authorization descriptors,
- hybrid authorization envelope support,
- reserved post-quantum scheme identifiers.

Even where full post-quantum verification is not activated in Phase 1, the machine must be designed so that future post-quantum activation does not require a state-model rewrite.

Authorization must include:

- replay protection,
- chain/domain separation,
- intent binding,
- version binding,
- threshold validation, where applicable,
- policy compatibility enforcement.

---

### 6.3.5 Object and Code Admission Layer

The machine must not execute arbitrary opaque payloads without validation.

Phase 1 must define and validate a canonical execution object or code admission format.

This layer must validate:

- object header,
- section boundaries,
- entrypoint declarations,
- capability flags,
- section hashes,
- version compatibility,
- integrity invariants,
- execution profile compatibility.

The purpose of this layer is to ensure that execution only proceeds on inputs that satisfy a canonical machine boundary.

---

### 6.3.6 Instruction Execution Engine

This is the kernel execution core.

Phase 1 must implement:

- deterministic decoding,
- deterministic dispatch,
- bounded execution loop,
- control flow validation,
- halt semantics,
- return semantics,
- revert semantics,
- trap/fault classification,
- invalid instruction handling.

The instruction engine must not directly mutate persistent chain state.

All durable changes must be journaled first.

---

### 6.3.7 Memory Kernel

The machine must define an explicit and bounded memory model.

The memory layer must define:

- code memory,
- readonly data regions,
- call data view,
- frame-local regions,
- heap or dynamic data region,
- scratch region,
- return buffer region.

All memory access must be:

- bounds checked,
- deterministic,
- gas-accounted where relevant,
- free from undefined behavior.

Uninitialized memory policy must be explicit.

Cross-frame memory aliasing rules must be explicit.

---

### 6.3.8 Gas and Resource Kernel

Resource accounting is a kernel responsibility.

Phase 1 must implement:

- base instruction costs,
- dynamic cost rules,
- memory expansion costs,
- syscall costs,
- object admission costs, if applicable,
- authorization verification costs, if protocol-visible,
- refund rules,
- out-of-gas semantics,
- fail-closed accounting behavior.

The gas meter must never allow ambiguity in:

- remaining gas,
- consumed gas,
- refund ceilings,
- rollback interactions,
- fatal vs revert charging behavior.

---

### 6.3.9 State Journal Kernel

State must be updated through journaling, not through direct mutation.

The journal must support:

- write recording,
- create recording,
- delete or tombstone recording,
- event/log recording,
- touched account recording,
- nested checkpoints,
- rollback,
- checkpoint merge,
- final commit.

The journal is mandatory for correctness because revert behavior, nested invocation semantics, and partial execution failure handling all depend on it.

---

### 6.3.10 Host Boundary Kernel

The machine must interact with the outside world only through an explicit host interface.

The host boundary must expose only what is necessary, including:

- account lookup,
- balance lookup,
- storage read access,
- storage write requests through journaling paths,
- code/object lookup,
- block metadata queries,
- policy queries,
- feature activation queries,
- event or log sinks,
- precompile dispatch,
- receipt builder hooks.

This interface must be minimal and deterministic.

---

### 6.3.11 Syscall Kernel

If the machine supports syscalls, they must be governed and metered.

Phase 1 syscalls must be:

- explicitly enumerated,
- versioned,
- permission-checked,
- metered,
- deterministic,
- host-mediated.

No open-ended or implicitly privileged syscall surface may exist in Phase 1.

---

### 6.3.12 Outcome and Receipt Kernel

Every execution must end in a canonical result structure.

This structure must include, at minimum:

- status class,
- success/revert/fault distinction,
- gas used,
- gas refunded,
- return data,
- revert data,
- emitted logs,
- created entities,
- deleted or tombstoned entities,
- touched set summary,
- active authorization profile,
- execution specification identifier,
- optional trace root,
- receipt hash.

Receipt construction must be deterministic and schema-versioned.

---

### 6.3.13 Versioned Rule Kernel

The machine must support version-aware execution.

This includes versioning for:

- instruction availability,
- gas schedule,
- object format,
- authorization policy,
- receipt schema,
- syscall surface,
- feature flags,
- migration mode.

This allows protocol evolution without historical ambiguity.

---

## 6.4 Phase 1 Invariants

The following invariants must hold for Phase 1:

- no execution without admitted input,
- no execution without version selection,
- no execution without explicit context,
- no persistent state mutation outside the journal,
- no hidden host interaction,
- no successful receipt without deterministic result classification,
- no object execution without canonical admission,
- no authorization downgrade by accident,
- no resource overrun without defined failure semantics,
- no consensus-visible nondeterminism.

---

## 6.5 Phase 1 Security Model

Phase 1 is the trust foundation of the VM.

The security model must explicitly defend against:

- malformed invocation payloads,
- unauthorized execution,
- replay attacks,
- object integrity violations,
- invalid instruction streams,
- memory boundary violations,
- gas exhaustion denial-of-service attempts,
- journal inconsistency,
- host boundary abuse,
- version confusion,
- receipt ambiguity.

---

## 6.6 Phase 1 Testing Expectations

Phase 1 is incomplete until it is extensively tested.

Required test classes include:

- deterministic replay tests,
- malformed admission tests,
- invalid authorization tests,
- replay resistance tests,
- object verification negative tests,
- invalid opcode tests,
- memory bounds tests,
- out-of-gas tests,
- nested rollback tests,
- receipt consistency tests,
- versioned rule tests,
- touched-set consistency tests,
- adversarial syscall tests.

---

## 6.7 Phase 1 Exit Criteria

Phase 1 is complete only when the machine can be described as:

> a deterministic, versioned, journaled, authorization-aware execution kernel capable of producing canonical AOXC state transitions and receipts under explicit trust boundaries.

Phase 1 must not be marked complete merely because it can "run code".

---

# 7. Phase 2 — Runtime Expansion

## 7.1 Phase 2 Objective

Phase 2 expands AOXC-VMachine-QX1 from a kernel-complete execution core into a richer runtime platform for object deployment, package lifecycle management, enhanced developer ergonomics, and controlled execution capability growth.

Phase 2 must preserve all Phase 1 invariants.

Phase 2 must never weaken:

- determinism,
- host boundary clarity,
- gas safety,
- journal discipline,
- version discipline,
- authorization abstraction.

---

## 7.2 Phase 2 Scope

Phase 2 introduces higher-level execution features around the trusted kernel, including:

- richer object packaging,
- module linking rules,
- package manifests,
- capability descriptors,
- expanded syscall surface,
- precompile and host-call governance,
- richer ABI and interface metadata,
- developer tooling alignment,
- optional multi-entrypoint packaging,
- execution profile differentiation,
- introspection hooks that remain deterministic.

---

## 7.3 Phase 2 Runtime Capabilities

### 7.3.1 Package Model Expansion

Phase 2 should define a richer package abstraction that may include:

- multiple code sections,
- manifest metadata,
- capability declarations,
- object dependencies,
- declared entrypoints,
- package version descriptors,
- upgrade compatibility metadata.

Package expansion must remain canonical and verifier-enforced.

---

### 7.3.2 Module and Link Model

Phase 2 may introduce controlled module linking or import resolution.

If introduced, linking must be:

- explicit,
- deterministic,
- version-aware,
- verifier-checked,
- bounded in complexity,
- safe under replay.

---

### 7.3.3 Enhanced Syscall Surface

Phase 2 may expand syscalls to support richer chain-native execution models.

This may include:

- controlled event classes,
- object registry interactions,
- capability queries,
- metadata access,
- chain-native identity lookups,
- governance-aware feature discovery.

All expanded syscalls must remain permissioned and metered.

---

### 7.3.4 Precompile and Native Extension Governance

Phase 2 should define a governance framework for precompiles or native extensions.

This framework must define:

- activation rules,
- versioning,
- permission scope,
- metering rules,
- determinism requirements,
- deprecation rules.

No native extension may bypass kernel invariants.

---

### 7.3.5 Tooling Alignment

Phase 2 should improve the developer and node operator experience through:

- stable ABI metadata,
- improved diagnostics,
- deterministic traces for debugging,
- better static verification reports,
- object/package linting,
- clearer failure reporting.

These features must remain informational unless explicitly protocol-visible.

---

## 7.4 Phase 2 Security Focus

The main security risk in Phase 2 is surface-area growth.

Therefore, Phase 2 must be governed by:

- capability minimization,
- syscall allowlisting,
- versioned package formats,
- verifier-first admission,
- native extension discipline,
- bounded complexity,
- strict metering,
- backward-compatible safety checks.

---

## 7.5 Phase 2 Testing Expectations

Phase 2 must add:

- package integrity tests,
- module linking determinism tests,
- capability misuse tests,
- native extension boundary tests,
- upgraded manifest compatibility tests,
- expanded syscall adversarial tests,
- trace consistency tests,
- runtime migration tests.

---

## 7.6 Phase 2 Exit Criteria

Phase 2 is complete when AOXC-VMachine-QX1 can support a richer execution ecosystem without weakening the trust guarantees established in Phase 1.

The machine must remain a kernel-first architecture even after runtime expansion.

---

# 8. Phase 3 — Quantum Hardening and Proof Ecosystem

## 8.1 Phase 3 Objective

Phase 3 transforms AOXC-VMachine-QX1 from a crypto-agile, post-quantum-ready machine into a quantum-hardened and proof-oriented execution platform.

The purpose of Phase 3 is not merely to "support post-quantum cryptography" as a marketing statement.

The purpose is to establish:

- activation-ready post-quantum authorization policies,
- controlled hybrid-to-post-quantum migration,
- cryptographic agility under governance,
- receipt and trace support for stronger proof systems,
- long-horizon execution survivability.

---

## 8.2 Phase 3 Scope

Phase 3 focuses on the following areas:

- hybrid authorization activation,
- post-quantum authorization scheme onboarding,
- account migration pathways,
- cryptographic deprecation policies,
- witness and proof-friendly execution artifacts,
- stronger trace commitments,
- optional stateless execution preparation,
- optional ZK/fraud-proof-aligned execution exports.

---

## 8.3 Quantum-Hardening Components

### 8.3.1 Hybrid Authorization Activation

The machine should support protocol-governed hybrid authorization modes in which a transaction or account policy may require more than one authorization profile.

This may include:

- classical + classical hybrid,
- classical + post-quantum hybrid,
- policy-bound multi-domain authorization,
- staged migration requirements by epoch.

---

### 8.3.2 Post-Quantum Authorization Profiles

Phase 3 should activate post-quantum-capable authorization profiles through governed scheme identifiers and verifier implementations.

The machine must not assume one final algorithm forever.

Instead, it must support:

- scheme registry extension,
- verifier plug-in through governed interfaces,
- account auth descriptor evolution,
- backward-compatible migration phases where required.

---

### 8.3.3 Account Migration Framework

Accounts must be able to evolve from legacy authorization descriptors into hybrid and, eventually, post-quantum-first descriptors.

Migration must include:

- descriptor versioning,
- safe rotation rules,
- downgrade resistance,
- replay resistance,
- epoch-based activation policies,
- explicit failure semantics for legacy-incompatible environments.

---

### 8.3.4 Cryptographic Policy Governance

Phase 3 must define the governance rules by which cryptographic primitives are:

- introduced,
- activated,
- preferred,
- deprecated,
- prohibited.

This includes policy for:

- signature schemes,
- hash families where applicable,
- domain separation rules,
- hybrid requirements,
- emergency deactivation or downgrade locks.

---

### 8.3.5 Witness and Proof Exports

Phase 3 should support deterministic witness-oriented artifacts suitable for stronger verification systems.

These may include:

- trace commitments,
- journal commitments,
- execution witnesses,
- state-access witnesses,
- canonical replay packages.

These exports must remain deterministic and version-aware.

---

### 8.3.6 Proof-Oriented Execution Interfaces

Where AOXC pursues fraud proofs, validity proofs, or stateless execution models, Phase 3 should provide interfaces that enable:

- reproducible execution traces,
- witness extraction,
- canonical execution commitments,
- version-stable proof inputs.

These interfaces must not compromise the trusted kernel design.

---

## 8.4 Phase 3 Security Focus

The central Phase 3 security concerns are:

- unsafe cryptographic migration,
- downgrade attacks,
- policy confusion,
- malformed hybrid envelopes,
- incompatible account state migration,
- verifier ambiguity,
- proof artifact inconsistency.

Therefore, Phase 3 must require:

- explicit policy versioning,
- explicit migration modes,
- strong replay and domain separation,
- deterministic witness schemas,
- clear scheme activation and deactivation rules.

---

## 8.5 Phase 3 Testing Expectations

Phase 3 must add:

- hybrid authorization tests,
- post-quantum descriptor parsing tests,
- migration-path tests,
- downgrade-resistance tests,
- verifier compatibility tests,
- witness determinism tests,
- trace-commitment consistency tests,
- proof-input stability tests.

---

## 8.6 Phase 3 Exit Criteria

Phase 3 is complete when AOXC-VMachine-QX1 can support governed hybrid and post-quantum authorization evolution, deterministic witness and proof artifacts, and long-horizon cryptographic adaptability without undermining the security and determinism established in Phases 1 and 2.

---

# 9. Cross-Phase Guarantees

The following guarantees must remain true in all phases:

- the machine remains deterministic,
- the host boundary remains explicit,
- all protocol-visible effects remain canonical,
- journal discipline remains mandatory,
- versioning remains explicit,
- authorization remains policy-bound,
- execution remains resource-bounded,
- receipt generation remains canonical,
- governance cannot silently mutate historical execution semantics.

---

# 10. Recommended Crate-Level Evolution

A practical crate organization for this roadmap may evolve around the following domains:

- kernel,
- context,
- auth,
- object,
- bytecode,
- engine,
- memory,
- gas,
- state,
- storage adapter,
- host,
- syscall,
- verifier,
- receipts,
- version,
- policy.

The crate may remain unified initially, but boundaries should be clear enough that future decomposition remains possible without architectural confusion.

---

# 11. Governance Interaction Model

Governance may influence:

- feature activation,
- execution spec upgrades,
- syscall availability,
- native extension activation,
- authorization scheme policy,
- post-quantum migration mode,
- deprecation windows.

Governance must not directly bypass:

- verifier requirements,
- kernel invariants,
- determinism constraints,
- journal discipline,
- receipt canonicalization.

Governance may choose rules. It must not nullify machine integrity.

---

# 12. Out-of-Scope Items by Default

Unless explicitly activated by protocol design, the following should be treated as out of scope or deferred:

- arbitrary unrestricted native plugin execution,
- direct host filesystem access,
- direct network I/O from VM code,
- nondeterministic entropy primitives,
- runtime-defined unsafe instruction injection,
- implicit cross-package privilege inheritance,
- cryptographic algorithm hardcoding without governance escape hatch.

---

# 13. Final Architectural Statement

AOXC-VMachine-QX1 is not intended to be a narrow interpreter. It is intended to be the canonical execution machine of AOXC: deterministic, journaled, versioned, policy-aware, crypto-agile, and capable of evolving toward hybrid and post-quantum security without sacrificing correctness.

The phases are therefore intentionally ordered:

- **Phase 1 builds trust**
- **Phase 2 builds capability**
- **Phase 3 builds long-horizon resilience**

Any implementation strategy that reverses this order risks producing a machine that is impressive in features but weak in protocol integrity.

AOXC-VMachine-QX1 must instead be built as a machine the chain can trust first, extend second, and harden for the future third.

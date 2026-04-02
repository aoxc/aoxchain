# QUANTUM_ROADMAP.md

Quantum-resilience transformation roadmap for AOXChain.

## Objective

Define a phased engineering program that raises AOXChain from classical cryptographic assumptions to a crypto-agile, post-quantum-capable, operator-verifiable architecture.

This roadmap avoids non-falsifiable claims such as "unbreakable." The target posture is:
- post-quantum cryptographic readiness,
- deterministic upgradeability,
- operationally auditable controls,
- bounded blast radius under cryptographic transition failures.

## Program Principles

1. **Determinism first:** no probabilistic runtime behavior introduced by cryptographic migration.
2. **Crypto-agility by design:** every consensus-critical cryptographic primitive must be versioned and swappable.
3. **Fail-closed security posture:** unsupported or malformed cryptographic inputs are rejected before state transition.
4. **Evidence-governed rollout:** each phase must produce reproducible artifacts in `artifacts/`.
5. **Hybrid transition safety:** classical + PQ paths coexist until objective deprecation criteria are met.

## Phase 0 — Threat Model and Baseline Inventory (Foundation)

### Goals
- Formalize adversary classes (classical, near-term quantum-assisted, long-horizon archive attacker).
- Inventory all cryptographic surfaces: signatures, key exchange, hashing, randomness, commitments, proofs.
- Define performance and storage budgets for post-quantum migration.

### Deliverables
- `docs/security/quantum-threat-model.md`
- `artifacts/crypto-surface-inventory.json`
- `artifacts/pq-budget-baseline.json`

### Exit criteria
- All protocol-facing primitives mapped to owners and upgrade paths.
- Consensus-impacting primitives classified as hard/soft migration surfaces.

## Phase 1 — Block Structure Modernization (First protocol target)

### Goals
- Introduce versioned cryptographic envelopes in block headers and transaction witness sections.
- Add explicit algorithm identifiers for signature and key encapsulation modes.
- Preserve deterministic block validation under mixed classical/PQ payloads.

### Scope
- Header schema versioning (`header_version`, `crypto_profile_id`, `witness_schema_id`).
- Witness container for hybrid signature bundles.
- Canonical serialization updates with strict byte-level test vectors.

### Implementation guidance
- Keep legacy decoding for compatibility windows, but enforce fail-closed validation for unknown IDs.
- Separate consensus hash domain tags by profile to avoid cross-profile replay ambiguity.
- Add hard limits for PQ signature sizes to defend validator memory pressure.

### Exit criteria
- Deterministic consensus on mixed block streams in simulation.
- Replay tests demonstrate profile/domain separation.
- Storage growth and propagation latency remain within declared bounds.

## Phase 2 — Special-Purpose Deterministic VM for PQ Era

### Goals
- Harden VM execution model against resource abuse from larger cryptographic artifacts.
- Provide deterministic cryptographic syscall layer with profile gating.
- Add static verification pipeline for bytecode safety and metering compliance.

### Scope
- Deterministic cryptographic syscall interface (`verify_sig`, `kem_decaps`, `hash_profile`).
- Gas/meter schedule revision for PQ verification costs.
- Bytecode admission rules: opcode allowlist, memory ceilings, recursion constraints.

### Implementation guidance
- Keep cryptographic syscalls side-effect free and deterministic.
- Bind VM cryptographic capabilities to on-chain profile governance, not local node flags.
- Add fixed-cost upper bounds for worst-case verification paths.

### Exit criteria
- VM determinism tests pass across heterogeneous hardware classes.
- DoS simulation confirms bounded verifier resource usage.
- Contract compatibility guide published for profile upgrades.

## Phase 3 — Network and Handshake Cryptography

### Goals
- Migrate peer transport and control-plane key establishment to hybrid/PQ modes.
- Protect long-lived control traffic from harvest-now-decrypt-later exposure.
- Ensure node identity continuity through migration.

### Scope
- Hybrid handshake (classical + PQ KEM).
- Session key rotation policy with profile pinning.
- Compatibility matrix for node software versions.

### Exit criteria
- Mixed-version network remains stable during staged rollout.
- Handshake downgrade attempts are detected and rejected.

## Phase 4 — Consensus and Governance Controls

### Goals
- Introduce explicit on-chain cryptographic profile governance.
- Enforce staged deprecation of legacy algorithms with objective thresholds.
- Bind validator admission to supported secure profile sets.

### Scope
- Governance objects: `CryptoProfile`, `DeprecationWindow`, `EmergencyRollback`.
- Activation rules based on finalized governance state.
- Validator conformance proofs in operator evidence bundles.

### Exit criteria
- Profile upgrade can be executed with deterministic rollback plan.
- Legacy deprecation path validated on testnet rehearsals.

## Phase 5 — AI-Assisted Security and Runtime Risk Controls

### Goals
- Add AI-assisted anomaly detection for mempool and network behavior.
- Keep consensus decisions deterministic and non-ML-dependent.
- Produce operator-facing risk signals with explainable evidence.

### Scope
- Non-consensus AI pipeline for anomaly scoring.
- Alert taxonomy for spam, replay anomalies, handshake downgrade patterns, and signature flood patterns.
- Audit trail for model version, dataset lineage, and threshold policy.

### Exit criteria
- AI outputs influence operations only, never state-transition validity.
- False-positive/false-negative metrics documented and bounded.

## Phase 6 — Verification, Audit, and Production Closure

### Goals
- Validate cryptographic migration via adversarial testing and independent audits.
- Produce production-closure evidence for quantum-resilience posture claims.
- Establish periodic re-evaluation process for cryptographic assumptions.

### Scope
- Fuzzing for serialization/parsing of profile-tagged artifacts.
- Differential consensus tests across profile combinations.
- External cryptography and protocol audits.

### Exit criteria
- Release bundle contains PQ-readiness evidence set.
- Security posture statement updated with explicit residual risks.

## Cross-Phase Risk Register (Top items)

1. **Performance regression:** larger signatures and verification costs degrade throughput.
2. **State bloat:** witness growth increases storage and sync time.
3. **Downgrade risk:** hybrid mode misconfiguration permits weaker-path fallback.
4. **Interoperability breakage:** inconsistent profile handling across versions.
5. **Governance deadlock:** inability to activate/deprecate profiles safely.

## Program KPIs

- Median and p99 block validation latency under each crypto profile.
- Bandwidth per block and witness growth ratio.
- Consensus divergence incidents in mixed-profile simulation.
- Rate of rejected downgrade attempts.
- Mean-time-to-detect and mean-time-to-mitigate crypto-surface anomalies.

## Proposed sequencing summary

1. Block structure migration.
2. Deterministic VM cryptographic syscall layer.
3. Network handshake migration.
4. On-chain profile governance and deprecation controls.
5. AI-assisted operational detection.
6. Audit and production closure.

## Status markers

- ⏳ Planned
- 🚧 In progress
- ✅ Completed
- 🛑 Blocked

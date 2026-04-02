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


## Concrete Work Package Map (Implementation-first)

### WP-1: Block Crypto Envelope

**Primary repositories/surfaces**
- `crates/*` consensus block types and serializers.
- `tests/` deterministic vector and mixed-profile replay suites.
- `artifacts/` compatibility and performance reports.

**Mandatory outputs**
- Header/witness schema version structs.
- Canonical encoding test vectors (golden fixtures).
- Mixed-profile chain simulation report.

**Gate commands (example target set)**
- `make test`
- `make quality`
- `make audit`

### WP-2: VM Crypto Syscall Layer

**Primary repositories/surfaces**
- VM crate syscall dispatcher and gas table definitions.
- Contract ABI and developer docs.
- Determinism and resource-bound adversarial tests.

**Mandatory outputs**
- Profile-gated deterministic syscall API.
- PQ cost model and gas schedule diff report.
- Stress-test evidence for worst-case verifier paths.

### WP-3: P2P Hybrid Handshake

**Primary repositories/surfaces**
- Networking transport handshake state machine.
- Node capability negotiation and downgrade telemetry.
- Operator runbooks for rollout/rollback.

**Mandatory outputs**
- Hybrid handshake implementation and compatibility matrix.
- Downgrade-attempt rejection evidence.
- Rotation policy verification artifacts.

### WP-4: On-chain Crypto Governance

**Primary repositories/surfaces**
- Governance object schemas and activation state machine.
- Validator conformance checks and policy surfaces.
- Migration and emergency rollback runbooks.

**Mandatory outputs**
- `CryptoProfile` object lifecycle with deterministic transition tests.
- Legacy deprecation schedule rehearsal on staged networks.
- Emergency rollback drill evidence.

### WP-5: AI Security Operations (Non-consensus)

**Primary repositories/surfaces**
- Mempool/network anomaly analytics pipeline.
- Alert routing and operator evidence bundles.
- Model registry and threshold governance.

**Mandatory outputs**
- Explainable alert taxonomy and confidence metrics.
- Dataset lineage + model-version audit trails.
- False-positive/negative benchmark report.

## Milestone and Timing Model

- **M0 (2-4 weeks):** complete Phase 0 inventory + threat model + baseline budgets.
- **M1 (4-8 weeks):** complete WP-1 block envelope migration on devnet.
- **M2 (4-8 weeks):** complete WP-2 VM deterministic syscall and metering hardening.
- **M3 (3-6 weeks):** complete WP-3 network handshake rollout in mixed-version topology.
- **M4 (3-6 weeks):** complete WP-4 governance activation/deprecation/rollback drills.
- **M5 (continuous):** operate WP-5 anomaly detection and iterative tuning.

Timing ranges are planning estimates and must be revised from measured throughput and defect density.

## Non-Negotiable Acceptance Thresholds

1. **Consensus safety:** zero tolerated consensus divergence in repeated mixed-profile simulation runs.
2. **Determinism:** bit-for-bit deterministic outputs for all consensus-critical cryptographic verifications.
3. **Downgrade defense:** every forced downgrade attempt must be rejected and recorded.
4. **Operational auditability:** each completed phase must emit reproducible artifact bundles.
5. **Rollback readiness:** every profile activation must have tested rollback procedures before production use.

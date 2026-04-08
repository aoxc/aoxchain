# PQ Compatibility Gap and Implementation Plan (Dilithium + Falcon)

## Purpose

Define the current AOXC post-quantum (PQ) compatibility posture and the minimum implementation plan required to move from policy-level PQ readiness to runtime-enforced PQ operation.

## Current Implementation Posture

### What is already present

- AOXC defines cryptographic profiles including `HybridEd25519Dilithium3` and `PqDilithium3Preview`.
- Mainnet-oriented quantum policy already lists PQ signature schemes (`ML-DSA-*`, `SLH-DSA-*`) and PQ KEM values (`ML-KEM-*`).
- Consensus policy and topology include governance controls for staged PQ migration.

### What is not yet runtime-complete

- Runtime transaction authorization currently exposes only `Ed25519` in execution payload auth schemes.
- Execution signature verification path currently validates only Ed25519 signatures.
- Transaction canonical signing semantics and transaction signature storage are Ed25519-shaped only.
- Base consensus policy template keeps `hybrid_signatures_enabled = false`.
- No Falcon algorithm surface is currently present in implementation-level auth scheme routing.

## Compatibility Statement

### Dilithium

Status: **Partially compatible at policy/profile level; not fully enforced at execution/runtime level.**

Interpretation:

- AOXC has profile and policy naming for Dilithium-oriented operation.
- AOXC still requires runtime authorization and verification path expansion before claiming complete Dilithium execution support.

### Falcon

Status: **Not currently integrated in runtime authorization/verification surfaces.**

Interpretation:

- Falcon is not exposed as a first-class execution auth scheme.
- Falcon support requires explicit signature type plumbing, canonical signing policy, verification backend selection, and governance-gated activation.

## Required Gaps to Close

### Gap 1: Auth scheme model is single-algorithm

Current state:

- `AuthScheme` is Ed25519-only in execution payload model.

Required change:

- Extend `AuthScheme` to include PQ and hybrid modes (example: `MlDsa87`, `Falcon1024`, `HybridEd25519MlDsa87`, `HybridEd25519Falcon1024`).

### Gap 2: Signature envelope is single-signature

Current state:

- Payload carries a single signature field without explicit multi-algorithm envelope semantics.

Required change:

- Introduce versioned signature envelope type carrying algorithm identifiers, signature bytes, and deterministic ordering constraints.

### Gap 3: Verification pipeline is Ed25519-only

Current state:

- Verification branch matches only Ed25519.

Required change:

- Add deterministic verification routing by auth scheme.
- Define strict fail-closed behavior for unknown algorithms and mismatched envelope/profile combinations.

### Gap 4: Transaction canonical signing format is algorithm-specific

Current state:

- Canonical message/signature behavior is framed around Ed25519 assumptions.

Required change:

- Define algorithm-agnostic signing preimage with domain separation fields:
  - tx format version,
  - auth scheme ID,
  - profile ID,
  - replay domain,
  - payload digest.

### Gap 5: Profile activation and runtime policy enforcement are not fully coupled

Current state:

- Governance/policy declares staged migration controls, but runtime acceptance matrix is not fully profile-coupled.

Required change:

- Enforce profile-to-auth-scheme admission matrix in mempool ingress and block validation.
- Reject downgrade attempts even when signatures are structurally valid.

### Gap 6: Falcon cryptographic backend and evidence are absent

Current state:

- No implementation evidence that Falcon verification is wired into runtime path.

Required change:

- Select vetted Falcon verification backend.
- Add deterministic test vectors and negative corpus tests.
- Add release evidence artifact indicating enabled algorithms and verification provenance.

## Phased Implementation Plan

## Phase A — Type and Policy Foundations

Deliverables:

- Versioned `AuthScheme` expansion.
- Versioned signature envelope type.
- Profile-to-scheme admission matrix in config and runtime policy object.

Exit criteria:

- Serialization compatibility tests pass.
- Unknown scheme and malformed envelope tests fail closed.

## Phase B — Runtime Verification Integration

Deliverables:

- Verification dispatch for Ed25519, Dilithium-targeted mode(s), and Falcon-targeted mode(s).
- Hybrid verification rule set (`both required`, deterministic ordering, deterministic error coding).

Exit criteria:

- Deterministic verification pass/fail matrix recorded in CI.
- Replay and downgrade rejection tests pass for all enabled schemes.

## Phase C — Governance-Coupled Activation

Deliverables:

- On-chain or policy-bound activation gate linking active profile to accepted auth schemes.
- Emergency rollback constraints with evidence requirements.

Exit criteria:

- Activation simulation tests show no consensus ambiguity.
- Profile transitions are audit-evidenced and reproducible.

## Phase D — Release and Operator Surfaces

Deliverables:

- CLI and API endpoints expose active auth scheme matrix.
- Readiness/reporting surfaces include PQ scheme status and verification evidence.

Exit criteria:

- `describe` and readiness reports clearly show scheme posture and transition state.
- Release evidence includes algorithm enablement and verification-status artifacts.

## Determinism and Safety Constraints

Any Dilithium/Falcon integration must preserve:

- deterministic signature verification outcomes across nodes,
- fail-closed rejection for unsupported/malformed schemes,
- canonical domain separation,
- replay protection invariants,
- compatibility-safe profile transitions with explicit governance evidence.

## Recommended Immediate Next Step

Implement **Phase A** first, without enabling PQ acceptance in production profiles by default. This allows full schema/pipeline hardening and deterministic test coverage before operational activation.


## Beyond "Quantum-Resistant": Practical Hardening Targets

Absolute "quantum-proof" claims are not realistic for a production chain. The engineering target should be **crypto-agile, compromise-contained, and rapidly updatable** under new cryptanalytic results.

### Target 1: Harvest-now/decrypt-later containment

Required controls:

- Forward-secrecy-only transport profiles for all validator and privileged control channels.
- Aggressive key/epoch rotation with enforced maximum key lifetime.
- Mandatory re-encryption and re-signing windows for long-lived sensitive artifacts.

### Target 2: Algorithm monoculture reduction

Required controls:

- Multi-family admission policy (for example: ML-DSA + SLH-DSA families) under governance.
- Hybrid-by-default critical path during migration windows.
- Explicit anti-downgrade policy requiring stronger profile continuity across epochs.

### Target 3: Evidence-backed activation

Required controls:

- Reproducible algorithm test-vector bundles and negative corpus checks in CI.
- Per-release cryptographic provenance report (algorithms, versions, validation status).
- Independent implementation differential tests for consensus-critical verification outcomes.

### Target 4: Rapid cryptographic emergency response

Required controls:

- Pre-approved emergency profile transition runbook with bounded rollback semantics.
- On-chain/profile-level kill-switch for compromised schemes (fail closed).
- Operator drills proving cluster-wide profile transitions within defined SLOs.

### Target 5: State and signature survivability

Required controls:

- Versioned signature envelopes and long-horizon re-signing strategy for archival objects.
- Domain-separated commitment strategy that can be upgraded without ambiguous interpretation.
- Explicit compatibility rules for historic block verification during profile evolution.

## Practical Interpretation for AOXC

AOXC should not optimize for a static "quantum-proof" endpoint. It should optimize for:

- deterministic multi-algorithm verification,
- governance-coupled and evidence-backed migration,
- strict anti-downgrade invariants,
- and operational ability to rotate or retire cryptographic schemes quickly.

This posture provides stronger real-world resilience than any single immutable algorithm choice.


## Quantum-First Pre-Feature Plan (Where to Start)

This sequence is intended for a chain that is already running, but wants to move to a quantum-hardened baseline **before** major feature expansion.

### Step 0 (Week 0): Freeze cryptographic surface churn

Objectives:

- Declare a temporary policy freeze: no new signature- or transaction-format variants without PQ review.
- Register all cryptographic touchpoints (mempool admission, block validation, p2p handshake, key custody, RPC auth).

Exit criteria:

- A signed crypto-inventory artifact exists and is referenced by release CI.

### Step 1 (Weeks 1-2): Implement type-level crypto agility first

Objectives:

- Complete `AuthScheme` expansion and versioned signature envelope design.
- Add deterministic algorithm identifiers and canonical ordering rules.
- Keep runtime acceptance conservative (do not enable PQ-only admission yet).

Exit criteria:

- Serialization/backward-compatibility tests pass.
- Unknown algorithm and malformed envelope paths fail closed.

### Step 2 (Weeks 2-4): Wire deterministic verification dispatch

Objectives:

- Implement verification routing for classical, hybrid, and PQ-only scheme variants.
- Enforce strict profile-to-scheme admission matrix at mempool ingress and block validation.
- Add anti-downgrade checks across epoch/profile transitions.

Exit criteria:

- Deterministic pass/fail matrix is green in CI for all supported schemes.
- Replay and downgrade tests pass.

### Step 3 (Weeks 4-6): Secure transport and key lifecycle

Objectives:

- Enforce PQ/hybrid KEM posture on validator/privileged channels.
- Introduce mandatory key rotation cadence and max key age policy.
- Add emergency cryptographic profile transition runbook and drills.

Exit criteria:

- Cluster simulation confirms profile transition within target SLO.
- Transport downgrade attempts are rejected and observable.

### Step 4 (Weeks 6-8): Operator evidence and release gating

Objectives:

- Expose active crypto profile + admission matrix through CLI/RPC.
- Require release evidence bundle (algorithms, backend provenance, vector corpus status).
- Promote channels only if `quantum-readiness-gate` and `quantum-full` succeed.

Exit criteria:

- Release artifact includes machine-readable crypto posture evidence.
- Promotion checklist is reproducible by an independent operator.

## Recommended Immediate Starting Point

If you must start today, prioritize in this order:

1. **AuthScheme + signature envelope versioning (Step 1)**
2. **Deterministic verification dispatch with fail-closed routing (Step 2)**
3. **Profile-coupled admission + anti-downgrade enforcement (Step 2)**

This order minimizes later refactor cost and allows feature development to proceed on top of stable cryptographic interfaces.


## Kernel-First Quantum Hardening Track

Yes—starting from the kernel is the correct priority for a running chain.

Reasoning:

- Kernel verification and block semantics define consensus truth; mistakes here cannot be safely compensated in outer services.
- RPC, CLI, and automation layers should only expose/operate policy that kernel validation already enforces.

### Kernel-first order of work

1. **Consensus-critical auth and verification types**
   - Finalize versioned auth scheme identifiers.
   - Finalize versioned signature envelope and deterministic ordering rules.
2. **Block/vote validation path**
   - Route verification deterministically by active profile.
   - Enforce fail-closed unknown-scheme behavior and anti-downgrade rules.
3. **Mempool and transaction admission**
   - Enforce profile-to-scheme admission before execution.
   - Reject structurally valid but policy-invalid signature bundles.
4. **Replay and canonicality invariants**
   - Strengthen domain separation and replay binding for all scheme families.
   - Preserve deterministic canonical signing preimage across transitions.
5. **Then expose operator surfaces**
   - After kernel enforcement is complete, reflect the same matrix in RPC/CLI and readiness gates.

### Kernel readiness gates (must pass before expansion)

- Deterministic verification matrix is reproducible across nodes.
- Downgrade and replay regression suite is green.
- Profile transition simulation has no consensus ambiguity.
- Release evidence records enabled algorithms and verification provenance.

### Practical rule

Do not expand product features on top of an unstable cryptographic kernel. Stabilize kernel crypto policy and verification first, then scale execution/API features.

## Complete Kernel Implementation Blueprint (Full Scope)

This section defines a complete kernel-focused execution blueprint for moving AOXC from policy-level PQ intent to runtime-enforced, consensus-safe cryptographic agility.

### A. Scope Boundary (Kernel only)

In scope:

- consensus-critical type system,
- transaction and vote admission semantics,
- block and vote verification semantics,
- replay/canonicality invariants,
- profile transition safety and failover rules.

Out of scope (until kernel is complete):

- non-critical API cosmetics,
- optional UX improvements,
- feature expansion unrelated to cryptographic enforcement.

### B. Kernel Architecture Targets

#### B1. Auth and signature model

Required end state:

- Versioned `AuthScheme` registry with stable numeric IDs.
- Versioned signature envelope (`SignatureEnvelopeV1`) supporting single/hybrid/PQ-only bundles.
- Deterministic canonical ordering for multi-signature sets.
- Strict envelope validation before cryptographic verification.

Hard requirements:

- Unknown scheme ID => reject (fail closed).
- Duplicate signer/algorithm tuple => reject.
- Non-canonical ordering => reject.

#### B2. Canonical signing preimage

Required end state:

Canonical preimage must include at minimum:

- domain tag,
- transaction format version,
- auth scheme ID,
- active crypto profile ID,
- network replay domain,
- payload digest.

Hard requirements:

- No algorithm-specific message shape.
- No optional field ordering ambiguity.
- Canonical encoding must be byte-for-byte reproducible.

#### B3. Verification dispatch and admission matrix

Required end state:

- Deterministic dispatch by scheme/profile pair.
- Profile-coupled admission matrix enforced in both mempool ingress and block validation.
- Hybrid profile requires both declared components and deterministic error coding.

Hard requirements:

- Structurally valid but policy-invalid signatures are rejected.
- Profile downgrade attempts are rejected even with cryptographically valid signatures.
- Mempool/block validation behavior is identical for same inputs.

#### B4. Vote and block semantics

Required end state:

- Vote authentication context includes explicit scheme/profile binding.
- Block PQ section semantics are policy-checked against active crypto epoch.
- Consensus outcome is independent of node-local library preferences.

Hard requirements:

- Any mismatch between vote metadata and signature envelope => reject.
- Any missing mandatory PQ/hybrid field for active profile => reject.

#### B5. Replay and anti-downgrade invariants

Required end state:

- Domain-separated replay binding across tx and vote surfaces.
- Durable commitment tracking for consumed replay commitments.
- Epoch/profile transition rules prohibit cryptographic weakening.

Hard requirements:

- Replay commitment reuse => reject.
- Epoch transition without required profile continuity evidence => reject.

### C. Implementation Work Packages

#### WP-1: Type system completion

Deliverables:

- Final enum/table for scheme IDs.
- Versioned signature envelope structs.
- Canonical encoding/decoding with explicit error taxonomy.

Acceptance criteria:

- All type codecs deterministic across platforms.
- Fuzz corpus shows no parser acceptance divergence.

#### WP-2: Kernel verification engine

Deliverables:

- Scheme/profile verification dispatcher.
- Hybrid verifier policy (`both required`, deterministic ordering).
- Fail-closed unknown/unsupported path.

Acceptance criteria:

- Golden vector suite passes for each enabled scheme.
- Negative corpus suite proves strict rejection behavior.

#### WP-3: Admission and consensus coupling

Deliverables:

- Mempool admission matrix integration.
- Block/vote validation matrix integration.
- Transition guard logic for profile epochs.

Acceptance criteria:

- Same payload yields identical decision in ingress and block replay.
- Downgrade attempts rejected in simulated transitions.

#### WP-4: Transport and key lifecycle coupling

Deliverables:

- PQ/hybrid KEM enforcement for privileged channels.
- Key max-age policy and rotation hooks at kernel policy boundary.
- Emergency profile switch constraints.

Acceptance criteria:

- Transport downgrade simulation fails closed.
- Rotation cadence violations are observable and block promotion.

#### WP-5: Evidence and release artifacts

Deliverables:

- Machine-readable algorithm enablement manifest.
- Verification backend provenance manifest.
- Determinism + downgrade + replay evidence bundle.

Acceptance criteria:

- Independent operator can reproduce artifacts from tagged release.
- Promotion blocked if evidence bundle is incomplete.

### D. Required Test Matrix (Kernel)

1. Determinism matrix
   - cross-node identical pass/fail outcome for same vectors,
   - cross-architecture reproducibility checks.
2. Negative corpus
   - malformed envelopes,
   - unknown algorithms,
   - ordering violations,
   - mixed-profile violations.
3. Replay and nonce regression
   - duplicate commitment rejection,
   - nonce regression rejection,
   - cross-domain replay rejection.
4. Transition safety
   - profile upgrade path,
   - attempted downgrade path,
   - emergency rollback constraints.
5. Stress and DoS posture
   - oversized PQ payload rejection,
   - bounded verifier resource behavior,
   - deterministic timeout/error mapping.

### E. Rollout Policy

- Devnet: schema and verifier hardening only.
- Testnet: hybrid enforcement + full evidence generation mandatory.
- Mainnet: PQ/hybrid profile activation only after all kernel gates are green and independently reproduced.

No channel may skip deterministic evidence requirements.

### F. Definition of Done (Kernel PQ Program)

Kernel PQ program is complete only when all conditions are true:

- Scheme/profile model is versioned and enforced at runtime.
- Mempool, block, and vote paths share one deterministic admission truth.
- Anti-replay and anti-downgrade invariants are continuously tested.
- Release artifacts include cryptographic provenance and reproducible verification evidence.
- Profile transitions are operationally drill-tested and auditable.

Until these are complete, product feature expansion should remain secondary to kernel cryptographic closure.

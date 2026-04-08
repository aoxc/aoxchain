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

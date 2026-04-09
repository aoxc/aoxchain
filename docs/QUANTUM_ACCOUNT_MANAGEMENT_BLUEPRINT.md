# Quantum-Grade Authority and Account Management Blueprint

## Purpose

Define a production-oriented, post-quantum-native authority architecture for AOXChain that is migration-safe by design.

This blueprint covers:
- cryptographic agility and scheme lifecycle governance,
- account, validator, and governance authority modeling,
- replay protection and recovery boundaries,
- transaction format and verification kernel constraints,
- phased implementation and evidence-backed acceptance gates.

## Net Reality and Design Objective

"Absolute" or "permanent" quantum-proof security cannot be guaranteed.

The correct engineering objective is:

> **Classical secure today, post-quantum primary tomorrow, migration-safe by design.**

AOXChain therefore treats post-quantum migration as a protocol and state-model concern, not only a cryptographic library substitution.

## Forward-Looking Architecture Recommendation

For the next architecture horizon, AOXChain should prioritize **authority-kernel completion before feature expansion**:

1. finalize canonical authority/state semantics,
2. make verification dispatch and replay domains deterministic in all consensus-visible paths,
3. complete validator/governance migration controls before broad wallet ecosystem expansion,
4. activate PQ-first policy by explicit governance windows only after drill evidence is retained.

This ordering minimizes expensive rework and avoids a late-stage redesign where transaction format, replay protection, and authority migration become incompatible across components.

## Scope and Assumptions

### In Scope

- Authority lifecycle for `AccountObject`, `ValidatorObject`, and `GovernanceAuthorityObject`.
- Signature and key-establishment policy surfaces for user, validator, and governance flows.
- On-chain rotation and recovery state transitions.
- Verification dispatch, cost accounting, and activation controls.
- Transaction envelope evolution for large post-quantum keys/signatures.

### Out of Scope

- Unverifiable claims of cryptographic permanence.
- Instant one-shot migration without dual-path operational controls.
- Implicit compatibility guarantees for undocumented legacy signing paths.

### Security Assumptions

- Adversaries can archive traffic for long-horizon cryptanalysis.
- Classical-only signatures are treated as transitional risk surfaces.
- Governance can activate, deprecate, and retire schemes only through explicit policy and retained evidence.

## Core Architecture Requirements

## 1) Cryptographic Agility Is Mandatory

AOXChain must never bind consensus-critical authority to a single algorithm family.

### Required model

- `scheme_id` is a native field in authority state and proof payloads.
- Accounts, validators, and governance authorities may operate under different active schemes concurrently.
- Scheme activation/deprecation windows are governance-controlled and version-bounded.

### Design implication

Post-quantum transition is modeled as controlled protocol evolution with explicit compatibility windows.

## 2) Signature and KEM Baseline

### Signature baseline

- Primary target: **ML-DSA**.
- Secondary fallback line: **SLH-DSA**.
- Classical-only signatures are transitional and must be policy-scoped.

### Node/session confidentiality baseline

- P2P/session/control-plane key establishment should use **ML-KEM** or controlled hybrid mode during migration windows.

## 3) Hybrid Transition Model

Migration requires explicit **classical + PQ hybrid** validation support before hard deprecation.

### Required controls

- Dual-acceptance windows with explicit start/end policy epochs.
- Deterministic rejection rules for unsupported combinations.
- Telemetry for downgrade attempts and profile mismatches.

## 4) Hash and Root Commitment Posture

Commitment surfaces (`block_id`, `state_root`, `policy_root`, `replay_root`) must keep conservative security margins and remain versioned where policy requires.

## 5) Policy-Based Authority Model (Not Signer-Only)

Authority validation is evaluated through canonical policy roots and proof bundles:

```text
actor -> scheme_id -> policy_root -> proof_bundle -> replay_check -> execute
```

This model prevents fragile "replace one signer algorithm" designs by making authorization semantics first-class protocol state.

## 6) Native Rotation and Recovery Kernel

Rotation and recovery are mandatory chain-level transitions, not ad-hoc off-chain events.

### Required transitions

- key rotation,
- scheme migration,
- policy rotation,
- recovery invocation.

### Root separation

- `policy_root` and `recovery_root` must be distinct.
- Recovery authority can rebind active scheme/policy under emergency controls.

## 7) Replay Protection by Domain/Intent

Global nonce-only models are insufficient for complex migration and governance workflows.

### Required model

- domain-scoped replay states (e.g., transfer, governance execution, validator vote, recovery intent),
- deterministic replay rejection keyed by actor, domain, and bounded sequence semantics.

## 8) Transaction Envelope for PQ Payload Sizes

Envelope and witness format must support larger keys and signatures from day zero.

### Mandatory capabilities

- variable-length public keys,
- variable-length signature payloads,
- proof bundle container,
- multi-proof container for hybrid validation.

## 9) Extensible Verification and Costing

Post-quantum verification can be materially more expensive than legacy paths.

### Required mechanism

- verifier dispatch registry,
- scheme-specific deterministic cost accounting,
- governance-controlled algorithm activation and deactivation.

## 10) Validator and Software Trust Surfaces

Wallet migration alone is insufficient.

Post-quantum planning must include:
- validator identity keys,
- consensus signing path,
- node authentication,
- software/firmware signing workflow.

## 11) Threshold and Multisig PQ Roadmap

Initial releases may ship with single-authority + hybrid policy controls.

Threshold/multi-party PQ controls should be delivered as a second-phase native surface with explicit evidence gates.

## Canonical State Objects

The minimum authority-state set is:

- `AccountObject`
- `ValidatorObject`
- `GovernanceAuthorityObject`
- `ReplayState`
- `RotationIntent`
- `RecoveryIntent`

These objects are consensus-visible and versioned under policy governance.

## Phased Delivery Plan

## Phase 0 — Canonical PQ Authority Spec

Deliver:
- actor types,
- `scheme_id` model,
- key commitment and root definitions,
- proof bundle format,
- replay model,
- migration rules.

Exit criteria:
- specification approved,
- policy and recovery boundaries explicit,
- no implicit classical-only assumptions.

## Phase 1 — State Object Implementation

Deliver typed objects and persistence semantics for authority, replay, rotation, and recovery.

Exit criteria:
- deterministic serialization/versioning,
- migration-safe state transitions defined.

## Phase 2 — Validation Kernel

Deliver:
- ML-DSA verifier,
- SLH-DSA fallback verifier,
- hybrid verifier interface,
- policy evaluation engine.

Exit criteria:
- deterministic acceptance/rejection matrix,
- bounded verification cost model.

## Phase 3 — Execution Coupling

Enforce strict ordering:

1. validate,
2. then execute.

Exit criteria:
- no execution path bypasses authority validation.

## Phase 4 — Validator and Governance Integration

Deliver:
- consensus authentication integration,
- governance execution authentication,
- emergency rotation/recovery playbooks.

Exit criteria:
- rehearsal artifacts retained,
- rollback and recovery are reproducible.

## Acceptance Gates

A release is not authority-grade post-quantum ready unless all gates pass:

1. **Agility Gate**: multiple `scheme_id` flows coexist deterministically.
2. **Hybrid Gate**: classical+PQ windows are policy-bounded and tested.
3. **Recovery Gate**: `recovery_root` path is exercised and auditable.
4. **Replay Gate**: domain-intent replay rejection is deterministic.
5. **Envelope Gate**: large key/signature and multi-proof payloads are accepted/rejected predictably.
6. **Validator Gate**: validator/network/software signing surfaces follow the same migration policy posture.
7. **Evidence Gate**: commands, artifacts, and risk statement are retained and linked to commit identity.

## Evidence Artifacts

Each candidate should retain:

- authority scheme distribution report,
- hybrid validation matrix and mismatch rejection evidence,
- rotation/recovery drill records,
- verifier cost-accounting outputs,
- validator auth migration rehearsal logs,
- residual risk statement with current assumptions.

## Maintenance Rule

Any change affecting authority model, scheme governance, replay semantics, recovery boundaries, verifier dispatch, or transaction witness format must update this blueprint in the same change set.

## Tracking Rule

Execution status for this blueprint is tracked in:

- `docs/PQ_AUTHORITY_IMPLEMENTATION_CHECKLIST.md`

Checklist updates should be committed in the same change stream as the implementation or policy change that modifies status.

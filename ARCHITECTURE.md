# AOXChain Architecture Baseline

This document defines the repository-level architectural baseline after the planning reset.

## 1. Design Intent


AOXChain is designed as a deterministic Layer-1 system in which:

- authority is policy-defined,
- cryptography is profile-governed,
- migration and recovery are protocol-native transitions.

Target posture:

- classical-secure operation in current windows,
- post-quantum-primary operation through governed activation,
- migration safety without hidden trust bypasses.

## 2. Layer Topology

### 2.1 Kernel Layer

Kernel components own canonical protocol truth:

- authority object semantics,
- consensus admission and finality-critical validation,
- profile policy enforcement,
- replay and migration transition integrity.

### 2.2 Execution Layer

Execution components:

- execute accepted transactions deterministically,
- enforce scheme-aware metering and verification cost policy,
- do not redefine kernel trust or profile policy.

### 2.3 Service Layer

Service components:

- provide P2P, RPC, and storage transport,
- enforce fail-closed ingress preconditions,
- expose observability without policy override.

### 2.4 Operations Layer

Operations components:

- orchestrate lifecycle and readiness gates,
- generate auditable artifacts,
- run migration, rotation, and recovery drills under policy constraints.

## 3. Authority-Centric Control Flow

Normative control flow:

1. Parse canonical envelope and actor identity.
2. Resolve `scheme_id` and applicable policy.
3. Verify proof bundle and policy constraints.
4. Apply replay-domain controls.
5. Execute deterministic state transition.
6. Persist deterministic results and evidence metadata.

No execution path may bypass steps 2–4.

## 4. Normative State Families

- `AccountObject`
- `ValidatorObject`
- `GovernanceAuthorityObject`
- `ReplayState`
- `RotationIntent`
- `RecoveryIntent`

All families are versioned, policy-aware, and migration-compatible through explicit rules.

## 5. Trust and Validation Boundaries

### Kernel Boundary

- validates consensus-visible cryptographic and authority state,
- rejects unknown scheme/profile combinations,
- enforces downgrade rejection.

### Service Boundary

- all external ingress is untrusted until kernel preconditions pass,
- availability concerns cannot override admission controls.

### Operations Boundary

- operations may trigger governed transitions,
- operations may not mutate canonical truth outside validated transaction flow.

## 6. Cryptographic Agility Requirements

Architecture requires:

- first-class `scheme_id` support,
- explicit activation/deprecation states,
- bounded hybrid migration windows,
- deterministic migration and rollback semantics,
- independent policy and recovery roots.

## 7. Deterministic Failure Model

Deterministic rejection behavior is required for:

- malformed proof bundles,
- replay violations,
- profile mismatch and downgrade attempts,
- migration and recovery authorization failures.

Each class must map to stable multi-node convergence semantics.

## 8. Key-Domain Architecture

Required separation domains:

- wallet transaction authorization,
- validator consensus signing,
- governance authority control,
- recovery authority control,
- node transport/session identity.

Rules:

- no cross-domain key reuse,
- all keys are profile-tagged (`scheme_id`) and policy-bound,
- wallet and node lifecycles are governed transitions,
- recovery authority remains independent from policy authority.

## 9. Evidence and Operability

Architecture claims are valid only with retained evidence:

- gate command outputs,
- deterministic test artifacts,
- migration/recovery drill artifacts,
- environment and profile consistency records.

No architectural claim is authoritative without reproducible artifacts.

# AOXChain Architecture (Reset Baseline)

This document defines the architecture after the repository-wide planning reset.

## 1) Design Intent

AOXChain is designed as a deterministic L1 where authority is policy-based, cryptography is profile-governed, and migration is protocol-native.

Primary architectural objective:

- classical-secure operation now,
- post-quantum-primary operation by governed transition,
- migration-safe operation without hidden trust bypasses.

## 2) Layer Topology

### 2.1 Kernel Layer

Kernel components own canonical protocol truth:

- authority object semantics,
- consensus admission and finality-critical validation,
- profile selection and policy enforcement,
- replay-domain integrity and migration transitions.

### 2.2 Execution Layer

Execution components:

- execute accepted transactions deterministically,
- enforce scheme-aware metering and verification costs,
- do not redefine kernel trust or profile policy.

### 2.3 Service Layer

Service components:

- provide P2P, RPC, and storage transport,
- enforce fail-closed ingress preconditions,
- expose profile/validation telemetry without policy override.

### 2.4 Operations Layer

Operations components:

- run lifecycle orchestration and readiness gates,
- generate audit evidence artifacts,
- run rotation/migration/recovery drills under policy controls.

## 3) Authority-Centric Control Flow

Canonical control flow:

1. Parse canonical envelope and actor identity.
2. Resolve `scheme_id` and applicable policy.
3. Verify proof bundle and policy constraints.
4. Apply replay-domain checks.
5. If accepted, execute deterministic state transition.
6. Persist deterministic result and evidence metadata.

No execution path may bypass steps 2–4.

## 4) State Objects (Normative Families)

- `AccountObject`
- `ValidatorObject`
- `GovernanceAuthorityObject`
- `ReplayState`
- `RotationIntent`
- `RecoveryIntent`

All families are versioned, policy-aware, and migration-compatible by explicit rules.

## 5) Trust and Validation Boundaries

### Kernel Boundary

- kernel validates consensus-visible cryptographic and authority state,
- kernel rejects unknown scheme/profile combinations,
- kernel owns downgrade rejection behavior.

### Service Boundary

- all external ingress is untrusted until kernel acceptance preconditions pass,
- transport availability concerns cannot override acceptance rules.

### Operations Boundary

- operations can trigger governed transitions,
- operations cannot mutate canonical truth outside validated transaction flow.

## 6) Cryptographic Agility and Migration

Architecture requires:

- first-class `scheme_id` support,
- explicit activation/deprecation states,
- bounded hybrid windows,
- deterministic migration and rollback behavior,
- independent policy and recovery roots.

## 7) Failure Model Requirements

System behavior must remain deterministic for:

- malformed proof bundles,
- replay-domain violations,
- profile mismatch and downgrade attempts,
- migration and recovery authorization failures.

Each class must map to stable rejection semantics suitable for multi-node convergence.

## 8) Advanced Key Architecture (Wallet + Node)

Architecture requires explicit key-domain separation across:

- wallet transaction authorization,
- validator consensus signing,
- governance authority control,
- recovery authority control,
- node transport/session identity.

Mandatory rules:

- no cross-domain key reuse,
- all keys are profile-tagged (`scheme_id`) and policy-bound,
- wallet and node key lifecycles are governed state transitions,
- recovery authority remains logically independent from policy authority.

Cryptographic profile usage is policy-driven: ML-DSA primary, SLH-DSA hybrid/secondary where explicitly authorized.

## 9) Evidence and Operability

Architecture validity is demonstrated by retained evidence:

- gate command outputs,
- deterministic test artifacts,
- migration/recovery drill artifacts,
- environment identity and profile consistency checks.

No architecture claim is accepted without reproducible artifacts.
